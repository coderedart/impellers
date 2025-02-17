#!/bin/bash
set -eoux pipefail
UPLOAD_DIR="$1" # The directory where we will store the tarballs
ENGINE_SHA="$(git submodule status | cut -d ' ' -f2)"
function download() {
    NAME="$(echo "$1" | sed 's/-/_/g')" # replace hyphen with underscore for zip names
    # 1: The name of the zip based on https://github.com/flutter/engine/blob/main/impeller/toolkit/interop/README.md#prebuilt-artifacts
    curl -L -o "$UPLOAD_DIR"/"$NAME".zip "https://storage.googleapis.com/flutter_infra_release/flutter/$ENGINE_SHA/$1/impeller_sdk.zip"
}
# The names are based on target_os and target_arch from https://doc.rust-lang.org/reference/conditional-compilation.html#target_arch
download linux-x64
download linux-arm64
download windows-x64
download windows-arm64
download darwin-x64
download darwin-arm64
download android-x64
download android-arm64
download android-x86
download android-arm
