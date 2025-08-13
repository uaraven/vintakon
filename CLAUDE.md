# Vintakor

GUI application to update Lens exif information for the pictures taken with vintage lenses.

Vintakor uses `exiftool` for reading and writing exif info.

To read info exiftool is called with following parameters:

```
-j
-b
-q
-api
structformat=jsonq
-FileModifyDate
-Make
-Model
-LensMake
-LensModel
-ExposureCompensation
-LensInfo#
-ISO
-Aperture
-ShutterSpeed
-FocalLength
-FocalLengthIn35mmFormat
-previewimage
-JpgFromRaw
-ThumbnailImage
```

User can define lens profiles that can be applied to files
Lens profile contains following information:

 - lens make
 - lens model;
 - use aperture;
 - lens focal length;
 - lens focal length in 35mm equivalent
 - lensInfo (f/min - f/max focalLengthMin - focalLengthMax);


## Testing

Test data is in `test-data` folder

To run tests execute

```shell
cargo test
```

## UI

UI is built using iced library. 