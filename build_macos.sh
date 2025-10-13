#!/bin/sh

cargo build --release

mkdir -p target/MacOs/
cp -r dist/MacOs/Vintakon.app target/MacOs/
cp target/release/vintakon target/MacOs/Vintakon.app/Contents/MacOS/vintakon
