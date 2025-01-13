import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:lens_attr/model/state.dart';

class LensProfileEditor extends HookConsumerWidget {
  const LensProfileEditor({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final profiles = ref.read(configProvider.select((config) => config.lensProfiles));
    final selectedProfile = useState("");
    final makeEditController = useTextEditingController();
    final modelEditController = useTextEditingController();

    return Column(
      children: [
        element("Lens profile", const DropdownMenu(dropdownMenuEntries: [])),
        element("Make", TextField(controller: makeEditController, decoration: const InputDecoration(border: OutlineInputBorder()),)),
        element("Model", TextField(controller: modelEditController, decoration: const InputDecoration(border: OutlineInputBorder()),))
      ],
    );
  }

  Widget element(String title, Widget editor) {
    return Row(children: [
      SizedBox(width: 200, child: Text(title)), Expanded(child: editor)]);
  }
}
