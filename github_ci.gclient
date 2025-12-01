# Copied from flutter/engine/scripts/standard.gclient
# As of 1 Dec, 2025 - flutter by default from standard.gclient downloads about 23 GB on a simple:
# `gclient sync --delete_unversioned_trees --no-history --shallow`

# Especially on linux, this includes irrelevant stuff like fuchsia-sdk (9GB+), android tools (3GB+) etc.

# So, we use the custom_vars to skip those downloads and save space.

solutions = [
  {
    "custom_deps": {},
    "deps_file": "DEPS",
    "managed": False,
    "name": ".",
    "safesync_url": "",

    # If you are using SSH to connect to GitHub, change the URL to:
    # git@github.com:flutter/flutter.git
    "url": "https://github.com/flutter/flutter.git",

    "custom_vars": {
      "download_emsdk": False,
      "download_android_deps": False,
      "download_jdk": False,
      "download_fuchsia_deps": False,
    },
  },
]