import 'dart:io';

import 'package:filesystem_picker/filesystem_picker.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:lens_attr/model/state.dart';

class FolderSelector extends HookConsumerWidget {
  const FolderSelector({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final currentFolder = ref.watch(selectedFolderProvider);
    final home = Platform.environment['HOME'] ?? Platform.environment['USER'] ?? "/";
    final filePath = ref.watch(selectedFolderProvider);
    return Row(
      children: [
        Expanded(
            child: Text(
          filePath ?? "Select a folder with images ➜",
          style: filePath == null || filePath == ""
              ? TextStyle(fontSize: 16, color: Theme.of(context).disabledColor)
              : const TextStyle(fontSize: 16),
              textAlign: filePath == null || filePath == "" ? TextAlign.end : TextAlign.start,
        )),
        const SizedBox(width: 8),
        ElevatedButton(
            onPressed: () {
              FilesystemPicker.open(
                      context: context,
                      rootDirectory: Directory(home),
                      directory: Directory(currentFolder ?? home),
                      showGoUp: true,
                      title: "Choose a folder",
                      fsType: FilesystemType.folder,
                      itemFilter: (fsEntity, path, name) => !name.startsWith("."),
                      pickText: "Select this folder")
                  .then((folder) {
                if (folder != null) {
                  ref.read(checkedImagesProvider.notifier).clear();
                  ref.read(selectedFolderProvider.notifier).state = folder;
                  ref.read(selectedImageProvider.notifier).state = null;
                }
              });
            },
            child: const Icon(Icons.folder_open))
      ],
    );
  }
}
