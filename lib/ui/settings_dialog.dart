import 'dart:io';

import 'package:filesystem_picker/filesystem_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:lens_attr/model/state.dart';
import 'package:lens_attr/ui/lens_profile_editor.dart';

class SettingsWindow extends StatelessWidget {
  const SettingsWindow({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        appBar: AppBar(backgroundColor: Theme.of(context).colorScheme.inversePrimary, title: const Text("Settings")),
        body: const SettingsDialog());
  }
}

class SettingsDialog extends HookConsumerWidget {
  const SettingsDialog({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final config = ref.watch(configProvider);
    final home = Platform.environment['HOME'] ?? Platform.environment['USER'] ?? "/";

    final controller = useTextEditingController();
    controller.text = config.exifToolPath;

    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        children: [
          Row(
            children: [
              const Text("exiftool path"),
              const SizedBox(width: 8),
              Expanded(child: TextField(controller: controller, enabled: false,)),
              ElevatedButton(
                  onPressed: () {
                    FilesystemPicker.open(
                            context: context,
                            rootDirectory: Directory(home),
                            directory: Directory(home),
                            showGoUp: true,
                            title: "Select exiftool executable",
                            fsType: FilesystemType.file,
                            itemFilter: (fsEntity, path, name) => !name.startsWith("."),
                            pickText: "Ok")
                        .then((file) {
                      if (file != null) {
                        ref.read(configProvider.notifier).update((cf) => cf.updateExifToolPath(file));
                      }
                    });
                  },
                  child: const Icon(Icons.folder_open)),
            ],
          ),
          LensProfileEditor(),
        ],
      ),
    );
  }
}
