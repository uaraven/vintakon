import 'dart:convert';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lens_attr/exif/tags.dart';
import 'package:lens_attr/model/config.dart';
import 'package:path/path.dart' as p;
import 'package:lens_attr/exif/exiftool.dart';
import 'package:flutter/widgets.dart';

final lensInfoPattern = RegExp(r'([\d.]+) ([\d.]+) ([\d.]+) ([\d.]+)');

class ExifTag {
  final String name;
  final String value;

  const ExifTag(this.name, this.value);
}

class LensInfo {
  final double minF;
  final double maxF;
  final double minFocalLength;
  final double maxFocalLength;

  LensInfo(this.minF, this.maxF, this.minFocalLength, this.maxFocalLength);

  factory LensInfo.empty() {
    return LensInfo(0, 0, 0, 0);
  }

  factory LensInfo.fromString(String lensInfo) {
    final match = lensInfoPattern.firstMatch(lensInfo);
    if (match != null) {
      return LensInfo(
        double.parse(match.group(3) ?? "0"),
        double.parse(match.group(4) ?? "0"),
        double.parse(match.group(1) ?? "0"),
        double.parse(match.group(2) ?? "0"),
      );
    } else {
      return LensInfo(0, 0, 0, 0);
    }
  }

  factory LensInfo.fromJson(Map<String, dynamic> json) {
    if (!json.containsKey("minF") ||
        !json.containsKey("maxF") ||
        !json.containsKey("minFocalLength") ||
        !json.containsKey("maxFocalLength")) {
      return LensInfo(0, 0, 0, 0);
    } else {
      return LensInfo(json["minF"], json["maxF"], json["minFocalLength"], json["maxFocalLength"]);
    }
  }

  Map<String, dynamic> toJson() {
    return {
      "minF": minF,
      "maxF": maxF,
      "minFocalLength": minFocalLength,
      "maxFocalLength": maxFocalLength,
    };
  }

  String toExif() {
    return "${minFocalLength.toStringAsFixed(1)} ${maxFocalLength.toStringAsFixed(1)} "
        "${minF.toStringAsFixed(1)} ${maxF.toStringAsFixed(1)}";
  }

  bool isEmpty() {
    return minF == 0 && maxF == 0;
  }

  @override
  String toString() {
    final focal = minFocalLength == maxFocalLength
        ? "${minFocalLength.toStringAsFixed(1)}mm"
        : "${minFocalLength.toStringAsFixed(1)}-${maxFocalLength.toStringAsFixed(1)}mm";
    final aperture =
        minF == maxF ? "f/${minF.toStringAsFixed(1)}" : "f/${minF.toStringAsFixed(1)}-${maxF.toStringAsFixed(1)}";
    return "$focal $aperture";
  }
}

class ImageFileInfo {
  final String filePath;
  final DateTime modTime;
  final String make;
  final String model;
  final String lensMake;
  final String lensModel;
  final LensInfo lensInfo;
  final String exposureCompensation;
  final String iso;
  final String aperture;
  final String shutterSpeed;
  final String focalLength;
  final String focalLength35mm;
  final Image? preview;

  ImageFileInfo(
      this.filePath,
      this.modTime,
      this.make,
      this.model,
      this.lensMake,
      this.lensModel,
      this.lensInfo,
      this.exposureCompensation,
      this.iso,
      this.aperture,
      this.shutterSpeed,
      this.focalLength,
      this.focalLength35mm,
      this.preview);

  factory ImageFileInfo.fromExifInfo(ExifToolFileInfo fileInfo) {
    final lensInfo = LensInfo.fromString(fileInfo.lensInfo);
    Image? image;
    if (fileInfo.previewBase64 != null && fileInfo.previewBase64 != "") {
      image = Image.memory(base64Decode(fileInfo.previewBase64!));
    }
    return ImageFileInfo(
        fileInfo.filePath,
        fileInfo.modTime,
        fileInfo.make,
        fileInfo.model,
        fileInfo.lensMake,
        fileInfo.lensModel,
        lensInfo,
        fileInfo.exposureCompensation,
        fileInfo.iso,
        fileInfo.aperture,
        fileInfo.shutterSpeed,
        fileInfo.focalLength,
        fileInfo.focalLength35mm,
        image);
  }

  String fileName() {
    return p.basename(filePath);
  }

  List<ExifTag> forUI() {
    return [
      ExifTag(tagNames[tagMake]!, make),
      ExifTag(tagNames[tagModel]!, model),
      ExifTag(tagNames[tagLensMake]!, lensMake),
      ExifTag(tagNames[tagLensModel]!, lensModel),
      ExifTag(tagNames[tagLensInfo]!, lensInfo.toString()),
      ExifTag(tagNames[tagAperture]!, aperture),
      ExifTag(tagNames[tagShutterSpeed]!, shutterSpeed),
      ExifTag(tagNames[tagIso]!, iso),
      ExifTag(tagNames[tagExposureCompensation]!, exposureCompensation),
      ExifTag(tagNames[tagFocalLength]!, focalLength),
      ExifTag(tagNames[tagFocalLengthIn35mmFormat]!, focalLength35mm),
    ];
  }
}

class ExifChanges {
  final String? make;
  final String? model;
  final String? aperture;
  final String? focalLength;
  final String? focalLength35mm;
  final LensInfo? lensInfo;

  const ExifChanges({this.make, this.model, this.aperture, this.focalLength, this.focalLength35mm, this.lensInfo});

  factory ExifChanges.of(LensProfile? lensProfile, String? aperture) {
    return ExifChanges(
        make: lensProfile?.make,
        model: lensProfile?.model,
        focalLength: lensProfile?.focalLength,
        focalLength35mm: lensProfile?.focalLength35mm,
        lensInfo: lensProfile?.lensInfo,
        aperture: aperture);
  }

  List<String> toExifTool() {
    final sb = <String>[];
    String? lensTag;
    if (make != null) {
      sb.add("-$tagLensMake=$make");
      lensTag = make!;
    }
    if (model != null) {
      sb.add("-$tagLensModel=$model");
      lensTag = (lensTag ?? "") + model!;
    }
    if (lensTag != null) {
      sb.add("-$tagLens=$lensTag");
    }
    if (focalLength != null) {
      sb.add("-$tagFocalLength=$focalLength");
    }
    if (focalLength35mm != null) {
      sb.add("-$tagFocalLengthIn35mmFormat=$focalLength35mm");
    }
    if (lensInfo != null) {
      sb.add("-$tagLensInfo=${lensInfo!.toExif()}");
    }
    if (aperture != null) {
      sb.add("-$tagAperture=$aperture");
      sb.add("-$tagFNumber=$aperture");
    }
    return sb;
  }
}

class FolderImages {
  final List<ImageFileInfo> data;
  final String folder;

  FolderImages(this.data, this.folder);
}

class SelectedImage {
  final String filePath;
  final ImageFileInfo imageInfo;

  const SelectedImage(this.filePath, this.imageInfo);
}

class CheckedImages extends StateNotifier<Set<String>> {
  CheckedImages() : super({});

  void addCheckedImage(String path) {
    state = {...state, path};
  }

  void removeCheckedImage(String path) {
    state = state.where((it) => it != path).toSet();
  }

  void clear() {
    state = {};
  }
}
