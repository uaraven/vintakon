import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:lens_attr/model/config.dart';
import 'package:lens_attr/model/state.dart';

class LensProfileEditor extends HookConsumerWidget {
  final int index;

  const LensProfileEditor(this.index, {super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final config = ref.watch(configProvider);

    final nameController = useTextEditingController(text: config.lensProfiles[index].name);
    final makeController = useTextEditingController(text: config.lensProfiles[index].make ?? "");
    final modelController = useTextEditingController(text: config.lensProfiles[index].model ?? "");
    final focalController = useTextEditingController(text: config.lensProfiles[index].focalLength ?? "");
    final focal35Controller = useTextEditingController(text: config.lensProfiles[index].focalLength35mm ?? "");

    return Padding(
      padding: const EdgeInsets.all(8),
      child: Column(mainAxisSize: MainAxisSize.min, children: [
        ListTile(
          contentPadding: const EdgeInsets.all(8),
          title: const Text("Profile name"),
          subtitle: TextField(
            controller: nameController,
          ),
        ),
        ListTile(
          contentPadding: const EdgeInsets.all(8),
          title: const Text("Lens make"),
          subtitle: TextField(controller: makeController),
        ),
        ListTile(
          contentPadding: const EdgeInsets.all(8),
          title: const Text("Lens model"),
          subtitle: TextField(controller: modelController),
        ),
        ListTile(
          contentPadding: const EdgeInsets.all(8),
          title: const Text("Focal length"),
          subtitle: numericTextField(focalController),
        ),
        ListTile(
            contentPadding: const EdgeInsets.all(8),
            title: const Text("Focal length in 35mm"),
            subtitle: numericTextField(focal35Controller)),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            ElevatedButton(
                onPressed: () {
                  ref.read(configProvider.notifier).update((cb) {
                    return cb.updateProfile(
                        index,
                        LensProfile(nameController.text, makeController.text, modelController.text,
                            config.lensProfiles[index].lensInfo, focalController.text, focal35Controller.text));
                  });
                  Navigator.pop(context);
                },
                child: const Icon(Icons.check_circle_rounded)),
            const SizedBox(width: 8),
            ElevatedButton(onPressed: () {}, child: const Icon(Icons.cancel_rounded))
          ],
        )
      ]),
    );
  }

  TextField numericTextField(TextEditingController controller) {
    return TextField(
      keyboardType: const TextInputType.numberWithOptions(signed: false, decimal: true),
      inputFormatters: [FilteringTextInputFormatter.allow(RegExp(r'^[\d.]*'))],
      controller: controller,
    );
  }
}

class LensProfileViewItem extends ConsumerWidget {
  final LensProfile profile;
  final int index;

  const LensProfileViewItem(this.profile, this.index, {super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(8),
        child: ListTile(
          title: Text(
            profile.name,
            style: const TextStyle(fontSize: 17),
          ),
          subtitle: Text(
            profile.asString(),
            style: const TextStyle(fontSize: 15),
          ),
          trailing: Wrap(children: [
            IconButton(
                onPressed: () {
                  editProfile(context);
                },
                icon: const Icon(Icons.edit)),
            IconButton(
                onPressed: () {
                  ref.read(configProvider.notifier).update((cb) => cb.deleteProfile(index));
                },
                icon: const Icon(Icons.delete_outline))
          ]),
        ),
      ),
    );
  }

  void editProfile(BuildContext context) {
    showDialog(
        context: context,
        builder: (context) {
          return Dialog(child: LensProfileEditor(index));
        });
  }
}

class LensProfileListEditor extends HookConsumerWidget {
  const LensProfileListEditor({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final config = ref.watch(configProvider);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(
            child: config.lensProfiles.isEmpty
                ? const Center(child: Text("Add a lens profile ⬇"))
                : ListView.builder(
                    itemBuilder: (context, index) {
                      return LensProfileViewItem(config.lensProfiles[index], index);
                    },
                    itemCount: config.lensProfiles.length)),
        Tooltip(
          message: "Add new lens profile",
          child: IconButton(
              onPressed: () {
                ref.read(configProvider.notifier).update((config) {
                  // selectedIndex.value = config.lensProfiles.length;
                  return config.addEmptyProfile();
                });
              },
              icon: const Icon(Icons.add)),
        ),
      ],
    );
  }
}
