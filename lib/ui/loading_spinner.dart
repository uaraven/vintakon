import 'package:flutter/material.dart';

class LoadingSpinner extends StatelessWidget {
  const LoadingSpinner({super.key});

  @override
  Widget build(BuildContext context) {
    return const Center(
      child: Wrap(
        children: [
          CircularProgressIndicator(),
          SizedBox(
            width: 10,
          ),
          Text(
            "Reading Exif data",
            style: TextStyle(fontSize: 20),
          )
        ],
      ),
    );
  }
}
