
# Static Builds
The flutter submodule is mainly intended for building static libraries. If you want to play with that, just remember these few things.
* Look at `.github/workflows/statics.yaml` and `build_impeller.sh` files
    for how we build things.
* Look at wiki pages [setting up environment](https://github.com/flutter/flutter/blob/master/engine/src/flutter/docs/contributing/Setting-up-the-Engine-development-environment.md) and [compiling engine](https://github.com/flutter/flutter/blob/master/engine/src/flutter/docs/contributing/Compiling-the-engine.md) and walk through them *step* by *step* to see that you are not missing anything.
* Just add lots of print statements in gn files or src/flutter/tools/gn to debug any issues.

### Problems with Static linking
I am just documenting this here, in case anyone familiar with gn build system can help fix this.

On linux, libcxx sources are not getting pulled into static library. We can verify this by just generating the gn files and looking at the diff between `library_static.ninja` vs `library.ninja` in the `out/{profile}/obj/flutter/impeller/toolkit/interop` directory.

This causes undefined reference errors for libcxx symbols like strings/share_ptrs when you try to link `libimpeller.a` with your app.

If we explicitly add libcxx as a dependency to `library_static` target, it works on windows, but causes issues with windows builds. So, for now, we only add libcxx as the dependency for `library_static` if the target os is not windows.

