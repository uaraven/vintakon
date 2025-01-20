import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:lens_attr/model/config.dart';
import 'package:lens_attr/model/file_info.dart';
import 'package:lens_attr/model/state.dart';

const headerSize = 200.0;

class ApplyProfile extends HookConsumerWidget {
  const ApplyProfile({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final config = ref.watch(configProvider);
    final checkedImagesExist = ref.watch(checkedImagesProvider).isNotEmpty;

    final selectedLensProfile = useState("");
    if (!config.lensProfiles.any((p) => p.name == selectedLensProfile.value)) {
        selectedLensProfile.value = "";
    }
    final apertureEditController = useTextEditingController();
    final lensProfileChecked = useState(false);
    final apertureChecked = useState(false);
    return SizedBox(
      height: 100,
      child: Row(children: [
        Expanded(
          child: Column(children: [
            Row(children: [
              Checkbox(
                  value: lensProfileChecked.value,
                  onChanged: !checkedImagesExist
                      ? null
                      : (value) {
                          lensProfileChecked.value = value ?? false;
                        }),
              const SizedBox(width: headerSize, child: Text("Lens profile")),
              Expanded(
                  child:
                  DropdownButtonHideUnderline(
                      child: DropdownButton<String>(
                          value: selectedLensProfile.value,
                          onChanged: lensProfileChecked.value ? (value) {
                            selectedLensProfile.value = value.toString();
                            }
                            : null,
                          items: listProfiles(config))),),
            ]),
            Row(
              children: [
                Checkbox(
                    value: apertureChecked.value,
                    onChanged: !checkedImagesExist
                        ? null
                        : (value) {
                            apertureChecked.value = value ?? false;
                          }),
                const SizedBox(width: headerSize, child: Text("Aperture")),
                Expanded(
                    child: TextField(
                  controller: apertureEditController,
                  keyboardType: const TextInputType.numberWithOptions(decimal: true, signed: false),
                  inputFormatters: [FilteringTextInputFormatter.allow(RegExp(r'[\d.]*'))],
                  enabled: checkedImagesExist && apertureChecked.value,
                )),
              ],
            ),
          ]),
        ),
        const SizedBox(
          width: 10,
        ),
        SizedBox(
          width: 200,
          child: Column(children: [
            ElevatedButton(onPressed: checkedImagesExist ? applyChanges : null, child: const Text("Apply"))
          ]),
        ),
      ]),
    );
  }

  List<DropdownMenuItem<String>> listProfiles(Config config) {
    final elems = config.lensProfiles.map((elem) {
      return DropdownMenuItem(
        value: elem.name,
        child: Padding(padding: const EdgeInsets.all(8), child: Text(elem.name)),
      );
    }).toList();
    return [const DropdownMenuItem(value: "", child: Padding(padding: EdgeInsets.all(8), child: Text(""))), ...elems];
  }

  applyChanges() {}
}
