#!/bin/sh

# CWD=$(pwd)
# trap 'cd "$CWD"' EXIT

cargo build --release

mkdir -p target/MacOs/
cp -r dist/MacOs/Vintakon.app target/MacOs/
mkdir -p target/MacOs/Vintakon.app/Contents/MacOS
cp target/release/vintakon target/MacOs/Vintakon.app/Contents/MacOS/vintakon
