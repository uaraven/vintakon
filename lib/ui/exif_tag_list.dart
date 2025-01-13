import 'package:flutter/material.dart';
import 'package:lens_attr/model/file_info.dart';
import 'package:lens_attr/ui/apply_profile.dart';

class ExifTagList extends StatelessWidget {
  final ImageFileInfo? imageInfo;

  const ExifTagList({super.key, required this.imageInfo});

  @override
  Widget build(BuildContext context) {
    final tags = imageInfo?.forUI() ?? [];
    return Padding(
      padding: const EdgeInsets.all(8),
      child: Column(
        children: [
          (imageInfo == null || imageInfo?.preview == null)
              ? Container()
              : SizedBox(height: 300, child: Padding(padding: const EdgeInsets.all(8), child: imageInfo!.preview!)),
          Expanded(
            child: SelectionArea(
                child: ListView.builder(
                    itemBuilder: (context, index) => Row(
                          children: [
                            SizedBox(
                              width: 220,
                              child: Text(
                                tags[index].name,
                                style: const TextStyle(fontSize: 16, fontFeatures: [FontFeature('smcp')]),
                              ),
                            ),
                            Expanded(
                              child: Text(tags[index].value, style: const TextStyle(fontSize: 16)),
                            ),
                          ],
                        ),
                    itemCount: tags.length)),
          ),
          const ApplyProfile(),
        ],
      ),
    );
  }
}
