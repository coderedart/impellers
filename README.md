# Impeller

`Impeller` is a 2D vector graphics renderer used in [Flutter](https://flutter.dev). Impeller can also be used standalone (without flutter) with its C API. This crate provides a safe rust wrapper around the C API (and also the raw bindings). 

### What can it do?
* draw 2D shapes like paths (lines/curvies), rectangles, circles, etc.
* draw AND layout text.
* draw effects like blurs, shadows, color blending etc.
* clipping using any shape.

### Where do I want to use it?
* UI libraries are the best use-case.
* 2D games? easy to embed Impeller into any opengl/vk/metal app.

### Docs
The docs and examples of this crate are good enough for *really* basic drawing (eg: drawing a rect). But this is nowhere enough for real world usage. They will tell you *how* to use a object, but now *why* or *where* you would use this. 

For example, pretty much NONE of the enums are documented. We will slowly improve the situation, but until then, the below resources should help cover the gaps. Most rust item docs will also contain direct links to their counterparts in dart/skiasharp/react-native-skia.

#### [Dart:Ui Docs](https://api.flutter.dev/flutter/dart-ui/)
Impeller is actually designed for dart/flutter, so, it is the best place to find documentation.
Most of the object and function names are same in rust/dart, so you can easily translate the docs from dart to rust. eg: [StrokeCap Enum](https://api.flutter.dev/flutter/dart-ui/StrokeCap.html).

Some names are different though. `DisplayListBuilder` is the combination of `PictureRecorder` and `Canvas` items in dart. `DisplayList` is `Picture` in dart.

It also has the [best explanation for BlendMode](https://api.flutter.dev/flutter/dart-ui/BlendMode.html) with lots of pictures.

> One jpeg screenshot is worth a thousand words of text documentation. - Albert Einstein

#### [React Native Skia Docs](https://shopify.github.io/react-native-skia/docs/canvas/overview/)
Technically, this is documenting Skia. But both Impeller and Skia have such a large overlap in terminology, design and functionality that a lot of learning transfers over seamlessly.

It also provides a lot of images showing the different options (eg: [MaskFilters Blur](https://shopify.github.io/react-native-skia/docs/mask-filters)).

#### [SkiaSharp docs](https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/effects/blend-modes/porter-duff)
Again, skia. But lots of guide-level docs for people who are starting out with Vector graphics.


### Why Impeller?
* **Blazingly? Fast** - It is used in Flutter, so, you know it will be maintained and improved continuously. The focus is also on *consistency* to keep everything running smooth.
* **Great text rendering AND layout** - The rust ecosystem is severely lacking
    when it comes to text. Impeller should cover *most* of your needs.
* **Simple Object Model**: The object model is very simple and takes like 5 minutes to learn. see [Object Model](#object-model)
* **Easy to Embed** - Any (opengl/vk/mtl)-based app/game can embed Impeller in less than 10 lines.
* **Fast compilation** -  The only bottleneck is network speed to download the prebuilt libs.
    And even that can be amortized with the `cache_libs` feature. see [Features](#features)
* **Easy Bindings** - The C API is really easy and allows us to "auto-generate" bindings.
    So, if we are trying to generate lua or wasm bindings, this is a huge QoL feature.

### Why not Impeller?
* Impeller is written in C++ and we do not support building from source. We use pre-built static/shared libraries instead.
* No support for d3d and no fallback software renderer.
* No powerful features like custom shaders. use [Skia-rs](https://github.com/rust-skia/rust-skia) instead.
* As the bindings are not widely used yet, we may still have some bugs.

### How to use the crate
For libraries who are just "using" the API, all you need to do is just use the crate with no features.

The final binary/executable should enable the `prebuilt_libs` feature to download the prebuilt libraries from github releases and link them.

> **NOTE**: We use curl to download and tar (or unzip on linux) to extract the archives. linux, mac and windows (10+) will have these by default.

## Features
* `prebuilt_libs` - Downloads the prebuilt libraries from github releases and links them to your project.
* `static_link` - If enabled, we will link static libraries. only available on linux/windows. All other platforms will need to use shared libraries or provide their own (see [Custom Linking](#custom-linking)).
* `debug_static_link` - If enabled, we will use the unstripped static libs with debug info (useful for debugging). only available on linux/windows just like `static_link`.
* `cache_libs` - If enabled, we will cache the prebuilt-libs in `.impeller_cache` directory inside your project directory (parent dir of `target`). Add `/.impeller_cache` to `.gitignore`, if you enable this feature. 
    * You can customize cache directory path with `IMPELLER_CACHE_DIR` env variable. And also use this to provide your own custom built libs.
    * caching avoids redownloading after `cargo clean` saving bandwidth and this in turns also makes the builds faster.
    * You also get to inspect the downloaded archives in the cache to debug any errors.


## Safety

I try to keep the bindings sound, but graphics programming is just really really unsafe. Especially around the handling of context/surface lifetimes.

Objects like textures/contexts are inherently linked to platform resources (like openGL context or vulkan device). So, they must ALL be destroyed before you destroy the underlying window or opengl context. That is the sole reason creating them is `unsafe`.

## Custom Linking
When you want to link in your own custom impeller library:
* Enable `cache_libs` feature to make it use libraries from a cached directory.
* set `IMPELLER_CACHE_DIR` environment variable to manually set the location of the cache directory.
* Inside that directory, create `targetos_targetarch` (eg: `linux_x64`) directory for dynamic libs and `targetos_targetarch_static_profile` (eg: `linux_x64_static_release`) directory for static libs.
* depending on the features `static_link` and `debug_static_link` you might need to create `targetos_targetarch_static_debug` directory, build script will search for dynamic libs or release static libs or debug static libs (useful for debugging).

