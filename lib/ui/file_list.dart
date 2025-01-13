import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lens_attr/model/file_info.dart';
import 'package:lens_attr/model/state.dart';

class FileList extends ConsumerWidget {
  final List<ImageFileInfo> imageData;

  const FileList(this.imageData, {super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final currentImage = ref.watch(selectedImageProvider);
    final checked = ref.watch(checkedImagesProvider);

    return ListView.separated(
      itemBuilder: (BuildContext context, int index) {
        return fromImageData(context,
            ref,
            checked,
            imageData[index],
            imageData[index].filePath == currentImage?.filePath);
      },
      separatorBuilder: (BuildContext context, int index) {
        return const Divider();
      },
      itemCount: imageData.length,
    );
  }

  Widget fromImageData(BuildContext context,
      WidgetRef ref,
      Set<String> checked,
      ImageFileInfo image,
      bool isSelected) {
    final txtStyle = isSelected
        ? const TextStyle(fontSize: 16, fontWeight: FontWeight.bold)
        : const TextStyle(fontSize: 16);

    return InkWell(
      child: Padding(
        padding: const EdgeInsets.all(8),
        child: Wrap(
          children: [
            Checkbox(
                value: checked.contains(image.filePath), onChanged: (value) {
                  if (value ?? false) {
                    ref.read(checkedImagesProvider.notifier).addCheckedImage(image.filePath);
                  } else {
                    ref.read(checkedImagesProvider.notifier).removeCheckedImage(image.filePath);
                  }
            }),
            const SizedBox(width: 8,),
            Text(
              image.fileName(),
              style: txtStyle,
            ),
          ],
        ),
      ),
      onTap: () {
        ref
            .read(selectedImageProvider.notifier)
            .state =
            SelectedImage(image.filePath, image);
      },
    );
  }
}
