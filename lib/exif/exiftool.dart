import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:lens_attr/exif/common.dart';
import 'package:lens_attr/exif/tags.dart';
import 'package:lens_attr/model/file_info.dart';

final exifToolDateFormat = DateFormat('yyyy:MM:dd HH:mm:ssZ');

class ExifToolFileInfo {
  final String filePath;
  final DateTime modTime;
  final String make;
  final String model;
  final String lensMake;
  final String lensModel;
  final String lensInfo;
  final String exposureCompensation;
  final String iso;
  final String aperture;
  final String shutterSpeed;
  final String focalLength;
  final String focalLength35mm;
  final String? previewBase64;

  const ExifToolFileInfo(
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
      this.previewBase64);

  factory ExifToolFileInfo.fromJson(Map<String, dynamic> json) {
    var modTime = json[tagFileModifyDate] ?? "";
    var preview = json[tagPreviewImage]?.toString() ?? json[tagJpgFromRaw]?.toString() ?? json[tagThumbnailImage] ?? "";
    preview = preview.startsWith("base64:") ? preview.substring(7) : preview;
    return ExifToolFileInfo(
        json[tagSourceFile],
        exifToolDateFormat.parse(modTime),
        json[tagMake] ?? "",
        json[tagModel] ?? "",
        json[tagLensMake] ?? "",
        json[tagLensModel] ?? "",
        json[tagLensInfo] ?? "",
        json[tagExposureCompensation] ?? "",
        json[tagIso] ?? "",
        json[tagAperture] ?? "",
        json[tagShutterSpeed] ?? "",
        json[tagFocalLength] ?? "",
        json[tagFocalLengthIn35mmFormat] ?? "",
        preview);
  }

  static List<ExifToolFileInfo> fromJsonList(List<dynamic> json) {
    return json.map((elem) => ExifToolFileInfo.fromJson(elem)).toList(growable: false);
  }
}

class ExifToolError implements Exception {
  final String errorMessage;

  const ExifToolError(this.errorMessage);
}

class ExifTool {
  final String exifToolPath;

  final List<String> commonParams = const [
    "-j",
    "-b",
    "-q",
    "-api",
    "structformat=jsonq",
  ];

  // These are tags queried for reading.
  // For example Aperture tag is a composite which is combined from FNumber and/or ApertureValue
  // When writing actual tags will be used. Below is the list of combined tags and their corresponding actual tags:
  // - Aperture = FNumber,ApertureValue
  // - ShutterSpeed = ExposureTime, ShutterSpeedValue
  final queryTags = const [
    "-FileModifyDate",
    "-Make",
    "-Model",
    "-LensMake",
    "-LensModel",
    "-ExposureCompensation",
    "-LensInfo#",
    "-ISO",
    "-Aperture",
    "-ShutterSpeed",
    "-FocalLength",
    "-FocalLengthIn35mmFormat",
    "-previewimage",
    "-JpgFromRaw",
    "-ThumbnailImage"
  ];

  const ExifTool({this.exifToolPath = ""});

  Future<List<ExifToolFileInfo>> listImages(String folderName) async {
    final List<String> args = List.from(commonParams);
    args.addAll(queryTags);
    for (var ext in supportedFormats) {
      args.add("-ext");
      args.add(ext);
    }
    args.add(folderName);
    final exifTool = exifToolPath == "" ? "exiftool" : exifToolPath;
    var execResult = await Process.run(exifTool, args,
        stderrEncoding: Encoding.getByName("utf-8"), stdoutEncoding: Encoding.getByName("utf-8"));
    if (execResult.exitCode != 0) {
      throw ExifToolError('exiftool error: ${execResult.stderr}');
    }
    try {
      final fileList = jsonDecode(execResult.stdout);
      return ExifToolFileInfo.fromJsonList(fileList);
    } catch (e) {
      debugPrint(e.toString());
      return [];
    }
  }
}
