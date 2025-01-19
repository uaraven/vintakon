import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/cupertino.dart';
import 'package:lens_attr/model/file_info.dart';
import 'package:path/path.dart' as p;

/// Contains information about non-electronic lens profile
/// All the fields are optional except for the profile name
class LensProfile {
  /// Profile name
  final String name;

  /// Lens make
  final String? make;

  /// Lens model
  final String? model;

  /// Lens info (min/max aperture and focal length)
  final LensInfo? lensInfo;

  /// Lens focal length
  final String? focalLength;

  /// Lens focal length in full-frame equivalent
  final String? focalLength35mm;

  final bool isEmpty;

  const LensProfile(this.name, this.make, this.model, this.lensInfo, this.focalLength, this.focalLength35mm,
      {this.isEmpty = true});

  LensProfile copyWith(
      {String? name, String? make, String? model, LensInfo? lensInfo, String? focalLength, String? focalLength35mm}) {
    return LensProfile(name ?? this.name, make ?? this.make, model ?? this.model, lensInfo ?? this.lensInfo,
        focalLength ?? this.focalLength, focalLength35mm ?? this.focalLength35mm);
  }

  factory LensProfile.fromJson(Map<String, dynamic> json) {
    final Map<String, dynamic> lensInfo = json["lensInfo"];
    return LensProfile(json["name"], json["make"], json["model"], LensInfo.fromJson(lensInfo), json["focalLength"],
        json["focalLength35mm"]);
  }

  static List<LensProfile> fromJsonList(List<dynamic> json) {
    return json.map((elem) => LensProfile.fromJson(elem)).toList();
  }

  Map<String, dynamic> toJson() {
    return {
      "name": name,
      "make": make,
      "model": model,
      "lensInfo": lensInfo?.toJson() ?? {},
      "focalLength": focalLength,
      "focalLength35mm": focalLength35mm,
    };
  }

  factory LensProfile.empty() {
    return LensProfile("", "", "", LensInfo.empty(), "", "", isEmpty: true);
  }

  String asString() {
    final repr = "${make ?? ''} ${model ?? ''} ${(lensInfo?.isEmpty() ?? true) ? 'No data' : lensInfo.toString()}";
    return repr.isEmpty ? "No data" : repr;
  }
}

class LensProfileEditable {
  String name;
  String make;
  String model;
  LensInfo lensInfo;
  String focalLength;
  String focalLength35mm;

  LensProfileEditable(this.name, this.make, this.model, this.lensInfo, this.focalLength, this.focalLength35mm);

  factory LensProfileEditable.fromImmutable(LensProfile profile) {
    return LensProfileEditable(profile.name, profile.make ?? "",
        profile.model ?? "",
        profile.lensInfo ?? LensInfo.empty(),
        profile.focalLength ?? "",
        profile.focalLength35mm ?? "");
  }

  LensProfile toImmutable() {
    return LensProfile(name, make, model, lensInfo, focalLength, focalLength35mm);
  }
}

class Config {
  final String exifToolPath;
  final List<LensProfile> lensProfiles;

  const Config(this.exifToolPath, this.lensProfiles);

  factory Config.fromJson(Map<String, dynamic> json) {
    final List<dynamic> profiles = json["lensProfiles"];
    return Config(json["exifToolPath"], LensProfile.fromJsonList(profiles));
  }

  Map<String, dynamic> toJson() {
    var map = lensProfiles.map((elem) => elem.toJson()).toList();
    return {"exifToolPath": exifToolPath, "lensProfiles": map};
  }

  static String _configDir() {
    String configDir;
    if (Platform.isWindows) {
      configDir = p.join(
          Platform.environment['LOCALAPPDATA'] ?? p.join(Platform.environment['USER'] ?? "", "config"), "vintaperture");
    } else {
      configDir = p.join(
          Platform.environment['XDG_CONFIG_HOME'] ?? p.join(Platform.environment['HOME'] ?? "", ".config"),
          "vintaperture");
    }
    return configDir;
  }

  static String _configFile() {
    final configFile = p.join(_configDir(), "config.dat");
    return configFile;
  }

  static Future<Config> load() async {
    try {
      final contents = await File(_configFile()).readAsBytes();
      final Config config = Config.fromJson(jsonDecode(utf8.decode(contents)));
      return config;
    } catch (e) {
      return const Config("", []);
    }
  }

  static Config loadSync() {
    try {
      final contents = File(_configFile()).readAsBytesSync();
      final Config config = Config.fromJson(jsonDecode(utf8.decode(contents)));
      return config;
    } catch (e) {
      return const Config("", []);
    }
  }

  Future<void> save() async {
    final dir = _configDir();
    try {
      await Directory(dir).create(recursive: true);
      final json = jsonEncode(toJson());
      await File(_configFile()).writeAsString(json);
    } catch (e) {
      debugPrint("Failed to save config: $e");
    }
  }

  saveSync() {
    final dir = _configDir();
    try {
      Directory(dir).createSync(recursive: true);
      final json = jsonEncode(toJson());
      File(_configFile()).writeAsStringSync(json);
    } catch (e) {
      debugPrint("Failed to save config: $e");
    }
  }

  Config addProfile(LensProfile profile) {
    final profiles = [...lensProfiles, profile];
    final newConfig = Config(exifToolPath, profiles);
    newConfig.save();
    return newConfig;
  }

  Config deleteProfile(int index) {
    final profiles = [...lensProfiles];
    profiles.removeAt(index);
    final newConfig = Config(exifToolPath, profiles);
    newConfig.save();
    return newConfig;
  }

  Config updateProfile(int index, LensProfile profile) {
    final profiles = [...lensProfiles];
    profiles[index] = profile;
    final newConfig = Config(exifToolPath, profiles);
    newConfig.save();
    return newConfig;
  }

  Config updateExifToolPath(String path) {
    final newConfig = Config(path, lensProfiles);
    newConfig.save();
    return newConfig;
  }

  Config addEmptyProfile() {
    var profileIndex = lensProfiles.length + 1;
    while (!isValidName("Lens Profile #$profileIndex")) {
      profileIndex++;
    }
    final newProfile = LensProfile("Lens Profile #$profileIndex", "", "", LensInfo.empty(), "", "", isEmpty: false);
    return addProfile(newProfile);
  }

  bool isValidName(String profileName, {int ignoreIndex = -1}) {
    if (profileName.isEmpty) {
      return false;
    }
    var ignoredProfile = ignoreIndex == -1 ? null : lensProfiles[ignoreIndex];
    return !lensProfiles.any((profile) => profile != ignoredProfile && profile.name == profileName);
  }
}
