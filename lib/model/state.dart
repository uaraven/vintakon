import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lens_attr/exif/exiftool.dart';
import 'package:lens_attr/model/config.dart';
import 'package:lens_attr/model/file_info.dart';

final selectedFolderProvider = StateProvider<String?>((ref) {
  return null;
});

final imageDataProvider = FutureProvider.autoDispose<FolderImages>((ref) async {
  final folder = ref.watch(selectedFolderProvider);
  if (folder == null || folder == "") {
    return FolderImages([]);
  } else {
    const exifTool = ExifTool();
    final exifToolList = await exifTool.listImages(folder);
    final imageInfoList = exifToolList.map((exif) => ImageFileInfo.fromExifInfo(exif)).toList(growable: false);
    return FolderImages(imageInfoList);
  }
});

final selectedImageProvider = StateProvider<SelectedImage?>((ref) => null);

final checkedImagesProvider = StateNotifierProvider<CheckedImages, Set<String>>((ref) => CheckedImages());

final configProvider = StateProvider<Config>((ref) {
  return Config.loadSync();
});
