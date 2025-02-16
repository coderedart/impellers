#!/bin/bash
set -eoux pipefail

PROFILE="$1"    # first argument: debug, release
UPLOAD_DIR="$2" # second argument: upload directory where the tarballs will be stored
OUT_DIR="$(pwd)/engine/src/out/out" # absolute path to build dir
LIB_NAME="libimpeller.a" # the name of the library
HOST_OS=$(echo "$RUNNER_OS" | tr '[:upper:]' '[:lower:]') # will be windows or linux
STATIC_ZIP_NAME="${HOST_OS}_x64_static_$PROFILE.zip" # the names of the final zip files
if [[ "$HOST_OS" == "windows" ]]; then
    LIB_NAME="impeller.lib" # update the content of the variable for windows
fi
# sanity checks and start build
rm -rf "$OUT_DIR" # clean out any old build files
ls -lh "$UPLOAD_DIR" # list for sanity
./engine/src/flutter/tools/gn ${USE_CCACHE:+"$USE_CCACHE"} --runtime-mode="$PROFILE"  --prebuilt-dart-sdk --no-goma --no-build-engine-artifacts --no-enable-unittests --no-lto --target-dir=out # lto makes static libs huge and unstrippable
cat "$OUT_DIR"/args.gn ||: # displays the gn args
ninja -C "$OUT_DIR" flutter/impeller/toolkit/interop:library_static
# prepare for archive
rm -rf tmp && mkdir tmp && mkdir tmp/lib
cp "$OUT_DIR"/obj/flutter/impeller/toolkit/interop/"$LIB_NAME" tmp/lib
ls -lh tmp/lib
if [[ "$PROFILE" == "release" ]]; then
    llvm-strip --strip-unneeded tmp/lib/* && ls -lh tmp/lib 
fi
cd tmp
7z a ../"$STATIC_ZIP_NAME" *
cd ..
mv "$STATIC_ZIP_NAME" "$UPLOAD_DIR"
rm -rf tmp "$OUT_DIR"

