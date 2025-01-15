import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:lens_attr/model/state.dart';
import 'package:lens_attr/ui/exif_tag_list.dart';
import 'package:lens_attr/ui/file_list.dart';
import 'package:lens_attr/ui/folder_selector.dart';
import 'package:lens_attr/ui/loading_spinner.dart';
import 'package:lens_attr/ui/settings_dialog.dart';

class MainWindow extends ConsumerWidget {
  const MainWindow({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.watch(configProvider);

    return mainWindowContents(context, ref);
  }

  Scaffold mainWindowContents(BuildContext context, WidgetRef ref) {
    final files = ref.watch(imageDataProvider);
    final selectedFile = ref.watch(selectedImageProvider);
    return Scaffold(
        appBar: AppBar(
            backgroundColor: Theme.of(context).colorScheme.inversePrimary,
            leading: Padding(padding: const EdgeInsets.all(4), child: Image.asset("images/app_icon.png")),
            actions: [
              ElevatedButton(
                  onPressed: () {
                    Navigator.push(context, MaterialPageRoute(builder: (context) => const SettingsWindow()));
                  },
                  child: const Icon(Icons.settings))
            ],
            title: const FolderSelector()),
        body: files.when(
            data: (data) => Row(
                  children: [
                    SizedBox(width: 400, child: Padding(padding: const EdgeInsets.all(8), child: FileList(data.data))),
                    Expanded(child: ExifTagList(imageInfo: selectedFile?.imageInfo))
                  ],
                ),
            error: (err, _) => Text("Error $err"),
            loading: () => const LoadingSpinner()));
  }
}
