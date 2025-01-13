
import 'package:flutter_test/flutter_test.dart';
import 'package:lens_attr/exif/exiftool.dart';

void main() {
  test("Test reading with exiftool", () async {
    const exifTool = const ExifTool();
    final images = await exifTool.listImages("testdata/images");
    expect(images, hasLength(3));
  });
}