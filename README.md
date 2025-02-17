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

### Why Impeller?
* **Blazingly? Fast** - It is used in Flutter, so, you know it will be maintained and improved continuously. The focus is also on *consistency* to keep everything running smooth.
* **Great text rendering AND layout** - The rust ecosystem is severely lacking
    when it comes to text. Impeller is basically production grade when it comes to text.
* The object model is very simple and takes like 15 minutes to learn.
* **Easy to Embed** - Any (opengl/vk/mtl)-based app/game can embed Impeller in less than 10 lines.
* **Fast compilation** -  The only bottleneck is network speed to download the prebuilt libs.
    And even that can be avoided with the `cache_libs` feature.
* **Easy Bindings** - The C API is really easy and allows us to "auto-generate" bindings.
    So, if we are trying to generate lua or wasm bindings, this is a huge QoL feature.

### Why not Impeller?
* This is not a pure rust library. Impeller is written in C++. The rust wrapper
    only supports mainstream platforms like macos/linux/windows/android by using pre-built shared libraries. static libs are available for linux/windows. m 
* cannot build from source. We use pre-built static/shared libraries.
    Impeller lives in flutter's monorepo, so building it requires like 15+ GB of space, which makes building from source difficult. you are always welcome to contribute to support source builds.
* We only support opengl/vk/metal (no d3d and no fallback software renderer).
* Impeller supports a limited subset of skia's features. If you need more power, use [Skia-rs](https://github.com/rust-skia/rust-skia).
* Impeller is not thread-safe (It could be, but I have yet to figure out the soundness of different operations in a multithreaded context).

### How to use the crate
For libraries who are just "using" the API (eg: building structs/calling fns), all you need to do is just use the crate with no features. By default, we don't do *anything* (no download or linking at all).

The final bin/executable user can enable the `prebuilt_libs` feature to download the prebuilt artefacts (static or dynamic libraries) from flutter (or github) releases.

> **NOTE**: We use curl to download and tar (or unzip on linux) to extract the archives. linux, mac and windows (10+) will have these by default, but if you don't, please install curl + tar (or unzip for linux).

## Features
* `prebuilt_libs` - Downloads the prebuilt libraries from github releases and links them to your project.
* `static_link` - If disabled, we will use dynamic linking. If enabled, we will link static libraries. only available on linux/windows.
* `debug_static_link` - If disabled, we will use the release (debug info stripped) libs. If enabled, we will use the debug libs (useful for debugging).
* `cache_libs` - If you enable tehis feature, we will cache the binaries in `.impeller_cache` of your project dir (parent of `target` dir). Add it to .gitignore to avoid committing the cache. caching is good, as it avoids redownloading after `cargo clean` and this also makes the builds faster. You also get to inspect the downloaded archives in the cache to debug any errors.

## Object Model
We have roughly three kinds of objects:
#### **Immutable Handles**
These are thread-safe opaque reference-counted objects. Think of them as `Arc<T>` - cheap to clone and you can only read from them or drop them.
eg: `Paragraph`, `DisplayList`, `Texture`, `ImageFilter` etc..
They generally implement Send and Sync.

#### **Mutable Handles**
These are like `Rc<UnsafeCell<T>>`.
But as they are opaque and don't let us "borrow" their contents (thus, hiding the `UnsafeCell` part from us), they are safe to clone/mutate via their methods.
Most of the API uses value semantics (structs are generally "owned" and have no lifetimes).
eg: `Paint`, `DisplayListBuilder`, `PathBuilder`, `Context` (only opengl ctx, vulkan/metal are thread-safe) etc..
They *usually* don't implement Send OR Sync. You create them and destroy them in the same thread (like `Rc` or `RefCell`). They do implement `Clone` though and their methods take `&self` due to interior mutability.
### **Linear Handles**
These are basically linear types that are "consumed" by their APIs. They don't implemlent `Clone` or `Send` or `Sync` and are not thread-safe.
eg: `Surface`

Finally, there's just raw pointers. **Value Types**. These are simply your normal plain-old-data c structs and enums. These are "value" types that implement copy and follows the normal rules of borrow checking and thread safety. eg: `Rect`, `ISize`, `Color`, `BlendMode` etc..

I try my best to keep the bindings sound, but graphics programming is just really really unsafe. Especially around the handling of context/surface lifetimes. Most users should just stick to objects like displaylists.