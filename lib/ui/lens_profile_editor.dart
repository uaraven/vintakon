import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:lens_attr/model/state.dart';

class LensProfileEditor extends HookConsumerWidget {
  const LensProfileEditor({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final config = ref.watch(configProvider);

    return Row(children: [
      SizedBox(
        width: 300,
        child: ListView.builder(
            itemBuilder: (context, index) {
              return Text(config.lensProfiles[index].name);
            },
            itemCount: config.lensProfiles.length),
      ),
    ]);
  }
}
