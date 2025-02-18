# Impeller

Impeller is a 2D vector graphics renderer used in [Flutter](https://flutter.dev). Impeller can also be used standalone (without flutter) with its C API. This crate provides a safe rust wrapper around the C API (and also the raw bindings).

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

For example, pretty much NONE of the enums are documented. We will slowly improved the situation, but until then, the below resources should help cover the gaps.

#### [Dart:Ui Docs](https://api.flutter.dev/flutter/dart-ui/)
Impeller is actually designed for dart/flutter, so, it is the best place to find documentation.
Most of the object and function names are same in rust/dart, so you can easily translate the docs from dart to rust. eg: [StrokeCap Enum](https://api.flutter.dev/flutter/dart-ui/StrokeCap.html).

It has the best explanation for [BlendMode](https://api.flutter.dev/flutter/dart-ui/BlendMode.html) docs with lots of pictures.

`DisplayListBuilder` is the combination of `PictureRecorder` and `Canvas` in dart. `DisplayList` is `Picture` in dart.

#### [React Native Skia Docs](https://shopify.github.io/react-native-skia/docs/canvas/overview/)
Technically, this is documenting Skia. But both Impeller and Skia have such a large overlap in terminology, design and functionality that a lot of learning transfers over seamlessly.

It also provides a lot of images showing the different options (eg: [MaskFilters Blur](https://shopify.github.io/react-native-skia/docs/mask-filters)), and a jpeg screenshot is worth a thousand lines of text documentation.

#### [SkiaSharp docs](https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/effects/blend-modes/porter-duff)
Again, skia. But lots of guide-level docs for people who are starting out with Vector graphics.


### Why Impeller?
* **Blazingly? Fast** - It is used in Flutter, so, you know it will be maintained and improved continuously. The focus is also on *consistency* to keep everything running smooth.
* **Great text rendering AND layout** - The rust ecosystem is severely lacking
    when it comes to text. Impeller is basically production grade when it comes to text.
* **Simple Object Model**: The object model is very simple and takes like 5 minutes to learn.
* **Easy to Embed** - Any (opengl/vk/mtl)-based app/game can embed Impeller in less than 10 lines.
* **Fast compilation** -  The only bottleneck is network speed to download the prebuilt libs.
    And even that can be avoided with the `cache_libs` feature.
* **Easy Bindings** - The C API is really easy and allows us to "auto-generate" bindings.
    So, if we are trying to generate lua or wasm bindings, this is a huge QoL feature.

### Why not Impeller?
* Impeller is written in C++. Unfortunately, there's no pure rust library that can do everything that Impeller/Skia can. 
* no support for building from source. We use pre-built static/shared libraries for fast compile times.
* We only support opengl/vk/metal. no d3d and no fallback software renderer.
* Impeller is a light-weight Skia. If you need more powerful features like custom shaders, use [Skia-rs](https://github.com/rust-skia/rust-skia).
* Impeller is not thread-safe (It can be in future, but not today).

### How to use the crate
For libraries who are just "using" the API, all you need to do is just use the crate with no features.

The final binary/executable should enable the `prebuilt_libs` feature to download the prebuilt libraries from github releases and link them.

> **NOTE**: We use curl to download and tar (or unzip on linux) to extract the archives. linux, mac and windows (10+) will have these by default.

## Features
* `prebuilt_libs` - Downloads the prebuilt libraries from github releases and links them to your project.
* `static_link` - If enabled, we will link static libraries. only available on linux/windows. All other platforms will need to use shared libraries.
* `debug_static_link` - If enabled, we will use the unstripped static libs with debug info  (useful for debugging).
* `cache_libs` - If enabled, we will cache the libs in `.impeller_cache` directory inside your project directory (parent of `target`). Add `/.impeller_cache` to `.gitignore` for convenience.
    * caching avoids redownloading after `cargo clean` saving bandwidth and this in turns also makes the builds faster.
    * You also get to inspect the downloaded archives in the cache to debug any errors.

## Object Model
We have roughly four kinds of objects:
#### **Immutable Handles** : like Arc
* thread-safe - `Sync` and `Send`
* shallow ref-counted - `Clone`
* immutable

eg: `Paragraph`, `DisplayList`, `Texture`, `ImageFilter` etc..

#### **Mutable Handles** : like Rc\<Cell\<T\>\> 
* NOT thread-safe - `!Sync` and `!Send`. You create, use and destroy these in the same thread 
* shallow ref-counted - `Clone`. the cloned handle still refers (mutates) the same object.
* mutable - via interior mutability (can be mutated via any cloned handle)

eg: `Paint`, `DisplayListBuilder`, `PathBuilder`, `TypographyContext` etc..
> A special case here is `Context` that is thread-safe with vk/metal, but not opengl.

#### **Linear Handles** 
* NOT thread-safe - `!Sync` and `!Send`
* NOT ref-counted - `!Clone`
* consumed by move (`self`)

eg: `Surface` is the only one at this moment.

#### **Value Types**
These are simply your normal plain-old-data c structs and enums. These implement `Copy` and have no special semantics. 

eg: `Rect`, `ISize`, `Color`, `BlendMode` etc..

I try to keep the bindings sound, but graphics programming is just really really unsafe. Especially around the handling of context/surface lifetimes.

## Safety

Objects like textures/contexts are inherently linked to platform resources (like openGL context or vulkan device). So, they must ALL be destroyed before you destroy the underlying window or opengl context. 