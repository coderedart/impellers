#![doc = include_str!("../README.md")]
//! ## API
//!
//! In a normal application, you usually have the phases of initialization, event loop and cleanup.
//!
//! ### Initialization
//! The initialization phase is usually the one where you create singletons. For example:
//!
//! 1. Create a [Context] (opengl/vk/metal) with the respective `new` functions. This is used to:
//!     1. create [Surface]/[VkSwapChain] (usually wrapping the default framebuffer).
//!     2. create a [Texture] from raw pixel data
//! 2. Create a [TypographyContext] (and register any custom fonts you may want to use).
//!
//!
//! ### Event Loop
//!
//! Impeller has two core objects: [Paint] and [DisplayListBuilder]. You need to understand both of them, to use this library.
//!
//! [DisplayListBuilder] acts like a canvas on to which we "draw" using methods like [DisplayListBuilder::draw_rect].
//! Internally, it builds up a list of draw commands, so that we can execute those commands later.
//!
//! The draw command is modified by three things before being added to the list:
//! 1. **Transformation**: decides the final position/size/shape of the draw command.
//! 2. **clipping**: how much of the draw command is effectively visible.
//! 3. **[Paint]**: further details of the draw command like color to use or other effects like blur.
//!
//! #### Transformation and Clip
//! The builder maintains an internal "stack" of transformation matrices and clip rects, manipulated by [DisplayListBuilder::save] and [DisplayListBuilder::restore].
//!
//! You can modify the current transformation with [DisplayListBuilder::scale], [DisplayListBuilder::translate] and
//! [DisplayListBuilder::rotate]. You can directly set a transformation matrix with
//! [DisplayListBuilder::set_transform] too or append additional matrix with [DisplayListBuilder::transform].
//! These will be applied to the draw commands that follow, until you use [DisplayListBuilder::restore] (to pop off the stack)
//! or [DisplayListBuilder::reset_transform] to just set the current matrix to identity.
//!
//! You can modify the current clip with [clip](DisplayListBuilder::clip_rect) and [DisplayListBuilder::clip_path].
//! The clip_op argument [ClipOperation] controls whether you want the draw commands to only
//! be visible *inside* or *outside* the clip shape.
//!
//! #### Paint
//! The [Paint] is used to configure the details of draw commands like:
//!
//! * Color, Transparency, Blend mode
//! * Draw style (filled rect or a stroked rect), width of the stroke
//! * Filters for blurs, color tinting a texture draw etc..
//!
//! Finally, once you have added all the draw commands, you build a [DisplayList] with
//! [DisplayListBuilder::build].
//!
//! [DisplayList] is an immutable, self-contained and reusable copy of draw commands that you can
//! execute/replay on to a surface (or add it to another [DisplayListBuilder]).
//!
//! You can use it as a reusable piece of drawing logic. And as you can append
//! it to [DisplayListBuilder], it will be affected by the transformation/clip
//! of the builder.
//!
//! For example, you can buid a [DisplayList] for a button background, and all button
//! widgets can reuse that for drawing different sized/rotated/positioned buttons.
//!
//! After building up this mega displaylist that contains your entire render tree,
//! you draw it to a [Surface]/[VkSwapChain] with [Surface::draw_display_list]
//! and preset it using [Surface::present].
//!
//! ### Cleanup
//! During the cleanup phase, you drop the all the objects including singletons.
//! And then,you destroy the window.
//!
//! ## Shapes
//!
//! Drawing most shapes like rect/circle is pretty simple, but there's a few
//! that are a bit more complex.
//!
//! ### Paths
//! You use [PathBuilder] to build a
//! list of path commands (start here, move to A, move to B etc..) and
//! eventually call [PathBuilder::copy_path_new] or [PathBuilder::take_path_new] to create a [Path].
//!
//! A [Path] is an immutable and reusable list of lines, curves and other path commands.
//! Each path contains zero or more sub-paths. So, a path can contain two independent circles or lines.
//! You draw it using [DisplayListBuilder::draw_path].
//!
//! ### Paragraphs
//!
//! A [ParagraphBuilder] is used to record a bunch of text sections with
//! different styles and then, build an immutable and reusable [Paragraph].
//!
//! 1. [TypographyContext] holds your fonts and provides them to [ParagraphBuilder].
//! 2. [ParagraphBuilder] to record text (with different pieces of text using different styles like colors/fonts/sizes etc..)
//! 3. You layout text with [ParagraphBuilder::build] using some max width and build a [Paragraph]
//! 4. You draw [Paragraph] with [DisplayListBuilder::draw_paragraph].
//!
//! [ParagraphBuilder] also maintains an internal stack of text styles.
//! So, you can push a style, add some text which will be rendered using that style and then,
//! pop the style to go back to previous style.
//!
//! ### Textures
//! You can create a [Texture] from raw pixel data or adopt an opengl texture using a [Context].
//!
//! While you can just draw a texture (image) directly using [DisplayListBuilder::draw_texture],
//! you can also use [ImageFilter] or [ColorFilter] of the [Paint] object to apply fancy effects
//! when sampling a texture.
//!
//! ### Filters/Sources etc..
//! The other objects are there mostly to add fancy effects like blur, color tint, color gradients etc..
//! Read the respective docs for more details.
//!
//! NOTE: The crate currently has *very* little API for some structs like matrices or rects. Contributions welcome :)

mod color;
#[cfg(all(feature = "sys", not(target_os = "windows")))]
pub mod sys;
#[cfg(all(not(feature = "sys"), not(target_os = "windows")))]
mod sys;

#[cfg(target_os = "windows")]
mod win_sys;
#[cfg(all(not(feature = "sys"), target_os = "windows"))]
use win_sys as sys;
#[cfg(all(feature = "sys", target_os = "windows"))]
pub mod sys {
    pub use crate::win_sys::*;
}

use std::borrow::Cow;

use bytemuck::cast;
use bytemuck::cast_ref;
// enums
/// <https://api.flutter.dev/flutter/dart-ui/BlendMode.html>
///
/// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/effects/blend-modes/>
pub use sys::BlendMode;
/// <https://api.flutter.dev/flutter/dart-ui/BlurStyle.html>
///
/// <https://shopify.github.io/react-native-skia/docs/mask-filters#example>
pub use sys::BlurStyle;
/// Layout of color components of the pixels
pub use sys::PixelFormat;

/// <https://api.flutter.dev/flutter/dart-ui/ClipOp.html>
pub use sys::ClipOperation;

/// <https://api.flutter.dev/flutter/dart-ui/ColorSpace.html>
pub use sys::ColorSpace;

/// <https://api.flutter.dev/flutter/dart-ui/PaintingStyle.html>
pub use sys::DrawStyle;

/// <https://api.flutter.dev/flutter/dart-ui/PathFillType.html>
///
/// <https://shopify.github.io/react-native-skia/docs/shapes/path#fill-type>
///
/// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/paths/fill-types>
pub use sys::FillType;

/// <https://api.flutter.dev/flutter/dart-ui/FontStyle.html>
pub use sys::FontStyle;

/// <https://api.flutter.dev/flutter/dart-ui/FontWeight-class.html>
pub use sys::FontWeight;

/// <https://api.flutter.dev/flutter/dart-ui/StrokeCap.html>
pub use sys::StrokeCap;

/// <https://api.flutter.dev/flutter/dart-ui/StrokeJoin.html>
pub use sys::StrokeJoin;

/// <https://api.flutter.dev/flutter/dart-ui/TextAlign.html>
pub use sys::TextAlignment;

/// <https://api.flutter.dev/flutter/dart-ui/TextDirection.html>
pub use sys::TextDirection;

/// The sampling mode to use when drawing a texture.
pub use sys::TextureSampling;

/// <https://api.flutter.dev/flutter/dart-ui/TileMode.html>
pub use sys::TileMode;
pub use sys::{
    ImpellerColor as Color, ImpellerColorMatrix as ColorMatrix, ImpellerRange as Range,
    ImpellerRoundingRadii as RoundingRadii,
};
pub type Rect = euclid::Rect<f32, euclid::UnknownUnit>;
pub type Point = euclid::Point2D<f32, euclid::UnknownUnit>;
pub type ISize = euclid::Size2D<i64, euclid::UnknownUnit>;
pub type Size = euclid::Size2D<f32, euclid::UnknownUnit>;
pub type Matrix = euclid::Transform3D<f32, euclid::UnknownUnit, euclid::UnknownUnit>;
//------------------------------------------------------------------------------
/// The current Impeller API version.
///
/// Rust bindings will automatically pass the version behind the scenes, so this is mostly useless for users.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ImpellerVersion(u32);

impl ImpellerVersion {
    /// The header version from which we parsed the bindings
    pub fn get_header_version() -> Self {
        Self(
            (sys::IMPELLER_VERSION_VARIANT << 29)
                | (sys::IMPELLER_VERSION_MAJOR << 22)
                | (sys::IMPELLER_VERSION_MINOR << 12)
                | sys::IMPELLER_VERSION_PATCH,
        )
    }
    /// Extracts the version variant

    pub fn get_variant(self) -> u32 {
        self.0 >> 29
    }
    /// Extracts the major version
    pub fn get_major(self) -> u32 {
        // zero the first 3 bits (variant) and then shift by 22
        (self.0 & (!0 >> 3)) >> 22
    }
    /// Extracts the minor version
    pub fn get_minor(self) -> u32 {
        (self.0 & (!0 >> 12)) >> 12
    }
    /// Extracts the patch version
    pub fn get_patch(self) -> u32 {
        self.0 & (!0 >> 20)
    }
    /// Get the version of *linked* Impeller library. This is the API that
    /// will be accepted for validity checks when provided to the
    /// context creation methods.
    ///
    /// NOTE: Rust bindings do the version validity checks behind the scenes, so the following doesn't really apply to users.
    ///
    /// The current version of the API generated from `impeller.h` is denoted by the
    /// given by [Self::get_header_version]. This version must be passed to APIs
    /// that create top-level objects like graphics contexts.
    /// Construction of the context may fail if the API version expected
    /// by the caller is not supported by the library.
    ///
    /// Since there are no API stability guarantees today, passing a
    /// version that is different to the one returned by
    /// [Self::get_linked_version] will always fail.
    ///
    /// see [Context::new_opengl_es]
    ///
    ///
    /// @return     The version of the standalone API. None if the version of the bindings is not compatible with the version of the Impeller library that was linked into the application.
    pub fn get_linked_version() -> Self {
        Self(unsafe { sys::ImpellerGetVersion() })
    }
    /// Checks that the [Self::get_header_version] is the same as the [Self::get_linked_version].
    /// ```
    /// impellers::ImpellerVersion::sanity_check();
    /// ```
    pub fn sanity_check() -> bool {
        Self::get_header_version() == Self::get_linked_version()
    }
}
/// The primary form of WSI when using a Vulkan context, these swapchains use
/// the `VK_KHR_surface` Vulkan extension.
///
/// Creating a swapchain is extremely expensive. One must be created at
/// application startup and re-used throughout the application lifecycle.
///
/// Swapchains are resilient to the underlying surfaces being resized. The
/// swapchain images will be re-created as necessary on-demand.
#[derive(Debug)]
#[repr(transparent)]
pub struct VkSwapChain(sys::ImpellerVulkanSwapchain);
// impl Clone for VkSwapChain {
//     fn clone(&self) -> Self {
//         unsafe {
//             sys::ImpellerVulkanSwapchainRetain(self.0);
//         }
//         Self(self.0)
//     }
// }

impl Drop for VkSwapChain {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerVulkanSwapchainRelease(self.0);
        }
    }
}
impl VkSwapChain {
    //------------------------------------------------------------------------------
    /// A potentially blocking operation, acquires the next surface to
    /// render to. Since this may block, surface acquisition must be
    /// delayed for as long as possible to avoid an idle wait on the
    /// CPU.
    ///
    ///
    /// @return     The surface if one could be obtained, NULL otherwise.
    ///
    pub fn acquire_next_surface_new(&mut self) -> Option<Surface> {
        let surface = unsafe { sys::ImpellerVulkanSwapchainAcquireNextSurfaceNew(self.0) };
        if surface.is_null() {
            None
        } else {
            Some(Surface(surface))
        }
    }
}
/// An Impeller graphics context. Contexts are platform and client-rendering-API
/// specific.
///
/// Contexts are thread-safe objects (not thread-safe with openGL) that are expensive to create. Most
/// applications will only ever create a single context during their lifetimes.
/// Once setup, Impeller is ready to render frames as performantly as possible.
///
/// During setup, context create the underlying graphics pipelines, allocators,
/// worker threads, etc...
///
/// The general guidance is to create as few contexts as possible (typically
/// just one) and share them as much as possible.
///
#[derive(Debug)]
pub struct Context(sys::ImpellerContext, ContextType);
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum ContextType {
    Gl,
    Vk,
    Mtl,
}
impl Context {}
impl Clone for Context {
    fn clone(&self) -> Self {
        unsafe {
            sys::ImpellerContextRetain(self.0);
        }
        Self(self.0, self.1)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerContextRelease(self.0);
        }
    }
}
unsafe extern "C" fn wrap_gl_proc_address<F: FnMut(&str) -> *mut std::os::raw::c_void>(
    name: *const std::os::raw::c_char,
    user_data: *mut std::os::raw::c_void,
) -> *mut std::os::raw::c_void {
    let name = if name.is_null() {
        c""
    } else {
        unsafe { std::ffi::CStr::from_ptr(name) }
    };
    let Ok(name) = name.to_str() else {
        panic!("Invalid GL function name: {}", name.to_string_lossy());
    };
    (*(user_data as *mut F))(name)
}
unsafe extern "C" fn wrap_vk_proc_address<
    F: FnMut(*mut std::os::raw::c_void, *const std::os::raw::c_char) -> *mut std::os::raw::c_void,
>(
    vulkan_instance: *mut std::os::raw::c_void,
    vulkan_proc_name: *const std::os::raw::c_char,
    user_data: *mut std::os::raw::c_void,
) -> *mut std::os::raw::c_void {
    (*(user_data as *mut F))(vulkan_instance, vulkan_proc_name)
}

impl Context {
    /// Create an OpenGL(ES) Impeller context.
    ///
    /// @param
    /// - gl_proc_address: A closure that returns the address of GL fn pointers
    ///
    /// @return The context or error if the context could not be created.
    ///
    /// ```
    /// fn create_impeller_ctx(window: &mut glfw::PWindow) {
    ///     // as easy as it gets
    ///     let impeller_ctx = unsafe {
    ///         // safety:  drop all objects created from this context
    ///         //          before `window` is dropped
    ///         impellers::Context::new_opengl_es( |name| {
    ///             window.get_proc_address(name) as _
    ///         })
    ///     };
    ///
    /// }
    /// ```
    ///
    /// # Safety
    /// * The context must be dropped before the window is dropped.
    /// * The context may only be used only when opengl context is current on the thread.
    /// * Any object (like texture or surface) that you create using this context
    ///   must be dropped before the context is dropped.
    /// * Unlike other context types, the OpenGL ES context can only be
    ///   created, used, and collected on the calling thread. This
    ///   restriction may be lifted in the future once reactor workers are
    ///   exposed in the API. No other context types have threading
    ///   restrictions. Till reactor workers can be used, using the
    ///   context on a background thread will cause a stall of OpenGL
    ///   operations.
    #[must_use = "opengl context has scary lifetime requirements. So, prefer dropping it explicitly with `std::mem:drop`"]
    pub unsafe fn new_opengl_es<F: FnMut(&str) -> *mut std::os::raw::c_void>(
        mut gl_proc_address: F,
    ) -> Result<Context, &'static str> {
        if !ImpellerVersion::sanity_check() {
            return Err("Impeller version mismatch when creating opengl context");
        }
        let ctx = unsafe {
            sys::ImpellerContextCreateOpenGLESNew(
                ImpellerVersion::get_linked_version().0,
                Some(wrap_gl_proc_address::<F>),
                &raw mut gl_proc_address as *mut _,
            )
        };
        if ctx.is_null() {
            Err("ImpellerContextCreateOpenGLESNew returned null :(")
        } else {
            Ok(Self(ctx, ContextType::Gl))
        }
    }
    /// Create a new surface by wrapping an existing framebuffer object.
    /// The surface is just a cheap use-and-throw object.
    /// Create it, draw to it (once) and drop it .
    ///
    ///
    /// - fbo      The framebuffer object handle.
    /// - format   The format of the framebuffer.
    /// - size     The size of the framebuffer is texels.
    ///
    /// @return    The surface if once can be created, NULL otherwise.
    ///
    /// # Safety
    /// * The surface must be properly configured (eg: no pending resizes)
    /// * must be drawn to only once and then dropped (presented if vulkan).
    /// * must be dropped before the context is dropped
    /// * The framebuffer must be complete as determined by
    ///   `glCheckFramebufferStatus`. The framebuffer is still owned by
    ///   the caller and it must be collected once the surface is
    ///   collected.
    pub unsafe fn wrap_fbo(
        &mut self,
        fbo: u64,
        format: PixelFormat,
        size: ISize,
    ) -> Option<Surface> {
        assert_eq!(self.1, ContextType::Gl);
        let surface = unsafe {
            sys::ImpellerSurfaceCreateWrappedFBONew(self.0, fbo, format, cast_ref(&size))
        };
        if surface.is_null() {
            None
        } else {
            Some(Surface(surface))
        }
    }

    /// Create a texture with decompressed bytes.
    ///
    /// @warning    Do **not** supply compressed image data directly (PNG, JPEG,
    ///             etc...). This function only works with tightly packed
    ///             decompressed data.
    /// @param
    /// - contents  texture bytes. contiguously laid out as RGBA8888
    /// - width     width of texture
    /// - height    height of texture
    ///
    /// @return     The texture if one can be created using the provided data, NULL
    ///             otherwise.
    ///
    /// # Safety
    ///
    /// * The texture must be dropped before the context is dropped
    ///
    #[doc(alias = "sys::ImpellerTextureCreateWithContentsNew")]
    pub unsafe fn create_texture_with_rgba8(
        &self,
        contents: Cow<'static, [u8]>,
        width: u32,
        height: u32,
    ) -> Result<Texture, &'static str> {
        if width == 0 || height == 0 {
            return Err("width and height must be greater than zero");
        }

        // we know this is 4 byte per pixel
        let total_bytes = width as usize * height as usize * 4;
        if contents.len() != total_bytes {
            return Err("provided buffer size does not match expected size");
        }
        let mip_count = flutter_mip_count(width as f32, height as f32);

        let t = unsafe {
            // SAFETY: pass the mapping with the right user_data returned from the function.
            let (mapping, user_data) = sys::ImpellerMapping::from_cow(contents);
            sys::ImpellerTextureCreateWithContentsNew(
                self.0,
                &sys::ImpellerTextureDescriptor {
                    size: cast(ISize::new(width.into(), height.into())),
                    pixel_format: PixelFormat::RGBA8888,
                    mip_count,
                },
                &mapping,
                user_data,
            )
        };
        if t.is_null() {
            Err("ImpellerTextureCreateWithContentsNew returned null")
        } else {
            Ok(Texture(t))
        }
    }

    /// Create a texture with an externally created OpenGL texture handle.
    ///
    /// - width     width of texture
    /// - height    height of texture
    /// - mip_count mipcount of texture
    /// - handle      The handle
    ///
    /// @return     The texture if one could be created by adopting the supplied
    ///             texture handle, NULL otherwise.
    /// # Safety
    ///
    /// * The texture must be dropped before the context is dropped
    /// * Ownership of the handle is transferred over to Impeller after a
    ///   successful call to this method. Impeller is responsible for
    ///   calling glDeleteTextures on this handle. Do **not** collect this
    ///   handle yourself as this will lead to a double-free.
    ///
    /// * The handle must be created in the same context as the one used
    ///   by Impeller. If a different context is used, that context must
    ///   be in the same sharegroup as Impellers OpenGL context and all
    ///   synchronization of texture contents must already be complete.
    ///
    /// If the context is not an OpenGL context, this call will always fail.
    ///
    #[doc(alias = "sys::ImpellerTextureCreateWithOpenGLTextureHandleNew")]
    pub unsafe fn adopt_opengl_texture(
        &self,
        width: u32,
        height: u32,
        mip_count: u32,
        handle: u64,
    ) -> Option<Texture> {
        assert_eq!(self.1, ContextType::Gl);
        let size = sys::ImpellerISize {
            width: width.into(),
            height: height.into(),
        };
        let t = sys::ImpellerTextureCreateWithOpenGLTextureHandleNew(
            self.0,
            &sys::ImpellerTextureDescriptor {
                pixel_format: PixelFormat::RGBA8888,
                size,
                mip_count,
            },
            handle,
        );
        if t.is_null() {
            None
        } else {
            Some(Texture(t))
        }
    }
    //------------------------------------------------------------------------------
    /// Create a Metal context using the system default Metal device.
    ///
    /// # Safety
    /// I don't know much about Metal, so I will
    /// leave the work of figuring out the safety to users. good luck :)
    ///
    /// @return     The Metal context or NULL if one cannot be created.
    #[doc(alias = "ImpellerContextCreateMetalNew")]
    #[must_use = "don't just drop a context like that. They usually have scary lifetimes, so prefer dropping them with an explicit `std::mem::drop`"]
    pub unsafe fn new_metal() -> Result<Context, &'static str> {
        if !ImpellerVersion::sanity_check() {
            return Err("ImpellerVersion::sanity_check failed");
        }
        let ctx = sys::ImpellerContextCreateMetalNew(ImpellerVersion::get_linked_version().0);
        if ctx.is_null() {
            Err("ImpellerContextCreateMetalNew returned null")
        } else {
            Ok(Self(ctx, ContextType::Mtl))
        }
    }
    /// Create a Vulkan context using the provided Vulkan Settings.
    ///
    /// - enable_validation  Enable Vulkan validation layers
    /// - proc_address_callback  A callback to query the address of Vulkan function
    ///                          pointers. The first argument is a pointer to vulkan instance.
    ///                          The second argument is a pointer to the function name.
    ///
    /// @return     The Vulkan context or NULL if one cannot be created.
    ///
    /// # Safety
    ///
    /// Don't know much vulkan either, so users will have to figure out the safety invariants.
    ///
    /// Just look at vulkan docs for how your proc_address_callback should work.
    /// Don't hold on to any pointers given to your closure (instance pointer or char pointer).
    #[must_use = "don't just drop vulkan context like that :( It's lifetimes are scary, so prefer dropping it explicitly using `std::mem::drop`"]
    #[doc(alias = "ImpellerContextCreateVulkanNew")]
    pub unsafe fn new_vulkan<
        F: FnMut(*mut std::os::raw::c_void, *const std::os::raw::c_char) -> *mut std::os::raw::c_void,
    >(
        enable_validation: bool,
        mut proc_address_callback: F,
    ) -> Result<Context, &'static str> {
        if !ImpellerVersion::sanity_check() {
            return Err("ImpellerVersion::sanity_check failed");
        }
        let settings = sys::ImpellerContextVulkanSettings {
            user_data: &raw mut proc_address_callback as *mut _,
            proc_address_callback: Some(wrap_vk_proc_address::<F>),
            enable_vulkan_validation: enable_validation,
        };
        let ctx =
            sys::ImpellerContextCreateVulkanNew(ImpellerVersion::get_linked_version().0, &settings);
        if ctx.is_null() {
            Err("ImpellerContextCreateVulkanNew returned null")
        } else {
            Ok(Self(ctx, ContextType::Vk))
        }
    }
    /// Get internal Vulkan handles managed by the given Vulkan context.
    /// Ownership of the handles is still maintained by Impeller. This
    /// accessor is just available so embedders can create resources
    /// using the same device and instance as Impeller for interop.
    ///
    /// @warning    If the context is not a Vulkan context, this will return Err.
    ///
    #[doc(alias = "ImpellerContextGetVulkanInfo")]
    pub fn get_vulkan_info(&self) -> Result<sys::ImpellerContextVulkanInfo, &'static str> {
        assert_eq!(self.1, ContextType::Vk);
        let mut vulkan_info = sys::ImpellerContextVulkanInfo::default();
        if unsafe { sys::ImpellerContextGetVulkanInfo(self.0, &mut vulkan_info) } {
            Ok(vulkan_info)
        } else {
            Err("ImpellerContextGetVulkanInfo returned false. Is the context a Vulkan context?")
        }
    }

    //------------------------------------------------------------------------------
    /// Create a new Vulkan swapchain using a VkSurfaceKHR instance.
    /// Ownership of the surface is transferred over to Impeller.
    ///
    /// - vulkan_surface_khr  The vulkan surface.
    ///
    /// @return     The vulkan swapchain.
    ///
    /// # Safety
    ///
    /// The Vulkan instance the surface is created from must the same as the
    /// context provided.
    ///
    /// The context must be a Vulkan context whose
    ///          instance is the same used to create the
    ///          surface passed into the next argument.
    ///
    /// The surface pointer must be valid (and kept alive until this swapchain is dropped).
    pub unsafe fn create_new_vulkan_swapchain(
        &self,
        vulkan_surface_khr: *mut std::os::raw::c_void,
    ) -> Option<VkSwapChain> {
        assert_eq!(self.1, ContextType::Vk);
        let swapchain = sys::ImpellerVulkanSwapchainCreateNew(self.0, vulkan_surface_khr);
        if swapchain.is_null() {
            None
        } else {
            Some(VkSwapChain(swapchain))
        }
    }
    //------------------------------------------------------------------------------
    /// Create a surface by wrapping a Metal drawable. This is useful
    /// during WSI when the drawable is the backing store of the Metal
    /// layer being drawn to.
    ///
    /// # Safety
    ///
    /// The Metal layer must be using the same device managed by the
    /// underlying context.
    ///
    /// The Metal device managed by this
    /// context must be the same used to create the
    /// drawable that is being wrapped.
    ///
    /// - metal_drawable  The drawable to wrap as a surface.
    ///
    /// @return     The surface if one could be wrapped, NULL otherwise.
    pub unsafe fn wrap_metal_drawable(
        &self,
        metal_drawable: *mut std::os::raw::c_void,
    ) -> Option<Surface> {
        assert_eq!(self.1, ContextType::Mtl);
        let surface = sys::ImpellerSurfaceCreateWrappedMetalDrawableNew(self.0, metal_drawable);
        if surface.is_null() {
            None
        } else {
            Some(Surface(surface))
        }
    }
    /// Create a color source whose pixels are shaded by a fragment program.
    ///
    /// <https://docs.flutter.dev/ui/design/graphics/fragment-shaders>
    ///
    ///
    /// # Safety
    /// Make sure the uniform data is laid out according to the fragment program's requirements
    ///
    /// TODO: add an example and better docs
    pub unsafe fn new_color_source_from_fragment_program(
        &self,
        frag_program: &FragmentProgram,
        samplers: &[Texture],
        uniform_data: &[u8],
    ) -> ColorSource {
        let samplers_len = samplers.len();
        let mut samplers: Vec<sys::ImpellerTexture> = samplers.iter().map(|t| t.0).collect();
        let cs = unsafe {
            sys::ImpellerColorSourceCreateFragmentProgramNew(
                self.0,
                frag_program.0,
                if samplers_len == 0 {
                    std::ptr::null_mut()
                } else {
                    samplers.as_mut_ptr()
                },
                samplers_len.try_into().unwrap(),
                uniform_data.as_ptr(),
                uniform_data.len().try_into().unwrap(),
            )
        };
        assert!(!cs.is_null());
        ColorSource(cs)
    }
    /// Create an image filter where each pixel is shaded by a fragment program.
    ///
    /// <https://docs.flutter.dev/ui/design/graphics/fragment-shaders>
    ///
    /// # Safety
    /// Make sure the uniform data is laid out according to the fragment program's requirements
    ///
    /// TODO: add an example and better docs
    pub unsafe fn new_image_filter_from_fragment_program(
        &self,
        frag_program: &FragmentProgram,
        samplers: &[Texture],
        uniform_data: &[u8],
    ) -> ImageFilter {
        let samplers_len = samplers.len();
        let mut samplers: Vec<sys::ImpellerTexture> = samplers.iter().map(|t| t.0).collect();
        let cs = unsafe {
            sys::ImpellerImageFilterCreateFragmentProgramNew(
                self.0,
                frag_program.0,
                if samplers_len == 0 {
                    std::ptr::null_mut()
                } else {
                    samplers.as_mut_ptr()
                },
                samplers_len.try_into().unwrap(),
                uniform_data.as_ptr(),
                uniform_data.len().try_into().unwrap(),
            )
        };
        assert!(!cs.is_null());
        ImageFilter(cs)
    }
}

/// Display lists represent encoded rendering intent (draw commands). These objects are
/// immutable, reusable, thread-safe, and context-agnostic.
///
/// While it is perfectly fine to create new display lists per frame, there may
/// be opportunities for optimization when display lists are reused multiple
/// times.
#[derive(Debug)]
#[repr(transparent)]
pub struct DisplayList(sys::ImpellerDisplayList);

impl Clone for DisplayList {
    fn clone(&self) -> Self {
        unsafe {
            sys::ImpellerDisplayListRetain(self.0);
        }
        Self(self.0)
    }
}

impl Drop for DisplayList {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerDisplayListRelease(self.0);
        }
    }
}
/// Display list builders allow for the incremental creation of display lists.
///
/// Display list builders are context-agnostic.
///
/// ### Recorder Semantics
///
/// This is a mix of skia's Canvas and PictureRecorder. You call functions
/// to draw things, but technically, you are just queuing draw commands.
///
/// And when you are done recording, you use [Self::build] to create a
/// [DisplayList].
///
/// Finally, you "execute" all the queued draw commands
/// on an actual [Surface] with [Surface::draw_display_list].
///
/// [Self::draw_display_list] can be used to push a copy of draw commands from
/// a [DisplayList] into a [DisplayListBuilder].
///
/// <https://api.flutter.dev/flutter/dart-ui/Canvas-class.html>
///
/// <https://learn.microsoft.com/en-us/dotnet/api/skiasharp.skcanvas?view=skiasharp-2.88>
///
/// ### Transformation And Clip Stack
///
/// Internally, this maintains a stack of (transformation matrices + clip rects).
///
/// You push a new transformation or clip on to the stack using [Self::save]
/// and pop them off [Self::restore].
///
/// You can check the current size of the stack with [Self::get_save_count].
/// You can use this to pop off all elements above that that point using [Self::restore_to_count].
///
/// <https://learn.microsoft.com/en-us/dotnet/api/skiasharp.skcanvas?view=skiasharp-2.88#clipping-and-state>
///
/// ### Save Layer
///
/// [Self::save_layer] creates an offscreen layer and redirects
/// all the subsequent draw commands to this layer. This offscreen
/// layer is blended back onto the parent layer using [Self::restore].
///
/// This is expensive, but is useful to apply fancy effects for a whole
/// "group" of draw commands (a whole layer).
///
/// ### Transforms
///
/// You can apply transforms to the canvas using [Self::translate], [Self::scale], [Self::rotate] and [Self::transform].
/// This allows you to affect the size/positions of the subsequent draw commands (until you pop off the transform from stack).
///
/// On hidpi screens, you might want to scale your canvas by scale factor of the screen.
/// This will ensure that the rest of the application can just draw normally and still be the right size.
///
/// Remember that the transforms compose. So, if you scale by 2.0, save stack, scale by 2.0, you
/// are now scaling by 4.0 (2.0 from first transform and 2.0 from current transform).
///
/// <https://learn.microsoft.com/en-us/dotnet/api/skiasharp.skcanvas?view=skiasharp-2.88#transformations>
///
/// ### Clipping
///
/// Clipping simply ignores all the draws outside of its shape. While [Self::clip_rect]
/// is used often, you can use [Self::clip_path] to do arbitrary shaped clipping.
///
/// ### Paint
///
/// [Paint] is the most commonly used object and stores the configuration
/// for draw commands.
///
/// For example, [Self::draw_rect] draws a rectangle. But whether it is a filled rect or just
/// a stroked (bordered) rect is decided by the paint's [Paint::set_draw_style].
///
#[derive(Debug)]
#[repr(transparent)]
pub struct DisplayListBuilder(sys::ImpellerDisplayListBuilder);

// impl Clone for DisplayListBuilder {
//     fn clone(&self) -> Self {
//         unsafe {
//             sys::ImpellerDisplayListBuilderRetain(self.0);
//         }
//         Self(self.0)
//     }
// }
unsafe impl Send for DisplayListBuilder {}
unsafe impl Sync for DisplayListBuilder {}
impl Drop for DisplayListBuilder {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerDisplayListBuilderRelease(self.0);
        }
    }
}
impl DisplayListBuilder {
    /// Create a new display list builder.
    ///
    /// An optional cull rectangle may be specified. Impeller is allowed
    /// to treat the contents outside this rectangle as being undefined.
    /// This may aid performance optimizations.
    ///
    /// @param
    /// - cull_rect:    The cull rectangle or NULL.
    ///
    /// @return         The display list builder.
    #[doc(alias = "sys::ImpellerDisplayListBuilderNew")]
    pub fn new(cull_rect: Option<&Rect>) -> Self {
        let result = unsafe {
            sys::ImpellerDisplayListBuilderNew(cull_rect.map_or(std::ptr::null(), |r| cast_ref(r)))
        };
        assert!(!result.is_null(), "Failed to create display list builder");
        Self(result)
    }
    //------------------------------------------------------------------------------
    /// Create a new display list using the rendering intent already
    /// encoded in the builder. The builder is reset after this call.
    ///
    /// @return     The display list.
    #[must_use]
    #[doc(alias = "sys::ImpellerDisplayListBuilderCreateDisplayListNew")]
    pub fn build(&mut self) -> Option<DisplayList> {
        let d = unsafe { sys::ImpellerDisplayListBuilderCreateDisplayListNew(self.0) };
        if d.is_null() {
            None
        } else {
            Some(DisplayList(d))
        }
    }
    //------------------------------------------------------------------------------
    // Display List Builder: Managing the transformation stack.
    //------------------------------------------------------------------------------

    //------------------------------------------------------------------------------
    /// Stashes the current transformation and clip state onto a save
    /// stack.
    #[doc(alias = "sys::ImpellerDisplayListBuilderSave")]
    pub fn save(&mut self) {
        unsafe {
            sys::ImpellerDisplayListBuilderSave(self.0);
        }
    }
    //------------------------------------------------------------------------------
    /// Stashes the current transformation and clip state onto a save
    /// stack and creates and creates an offscreen layer onto which
    /// subsequent rendering intent will be directed to.
    ///
    /// On the balancing call to restore, the supplied paints filters
    /// and blend modes will be used to composite the offscreen contents
    /// back onto the display display list.
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/Canvas/saveLayer.html>
    ///
    /// <https://shopify.github.io/react-native-skia/docs/group#layer-effects>
    ///
    /// - bounds    The bounds.
    /// - paint     The paint.
    /// - backdrop  The backdrop.
    ///
    #[doc(alias = "sys::ImpellerDisplayListBuilderSaveLayer")]
    pub fn save_layer(
        &mut self,
        bounds: &Rect,
        paint: Option<&Paint>,
        backdrop: Option<&ImageFilter>,
    ) {
        unsafe {
            sys::ImpellerDisplayListBuilderSaveLayer(
                self.0,
                cast_ref(bounds),
                paint.map_or(std::ptr::null_mut(), |p| p.0),
                backdrop.map_or(std::ptr::null_mut(), |b| b.0),
            );
        }
    }

    //------------------------------------------------------------------------------
    /// Pops the last entry pushed onto the save stack using a call to
    /// [Self::save] or [Self::save_layer].
    #[doc(alias = "sys::ImpellerDisplayListBuilderRestore")]
    pub fn restore(&mut self) {
        unsafe {
            sys::ImpellerDisplayListBuilderRestore(self.0);
        }
    }
    //------------------------------------------------------------------------------
    /// Apply a scale to the transformation matrix currently on top of
    /// the save stack.
    ///
    /// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/transforms/scale>
    ///
    /// - x_scale  The x scale.
    /// - y_scale  The y scale.
    #[doc(alias = "sys::ImpellerDisplayListBuilderScale")]
    pub fn scale(&mut self, x_scale: f32, y_scale: f32) {
        unsafe {
            sys::ImpellerDisplayListBuilderScale(self.0, x_scale, y_scale);
        }
    }
    //------------------------------------------------------------------------------
    /// Apply a clockwise rotation to the transformation matrix
    /// currently on top of the save stack.
    ///
    /// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/transforms/rotate>
    ///
    /// - angle_degrees  The angle in degrees.
    #[doc(alias = "sys::ImpellerDisplayListBuilderRotate")]
    pub fn rotate(&mut self, angle_degrees: f32) {
        unsafe {
            sys::ImpellerDisplayListBuilderRotate(self.0, angle_degrees);
        }
    }
    /// Apply a translation to the transformation matrix currently on
    /// top of the save stack.
    ///
    /// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/transforms/translate>
    ///
    /// - x_translation  The x translation.
    /// - y_translation  The y translation.
    #[doc(alias = "sys::ImpellerDisplayListBuilderTranslate")]
    pub fn translate(&mut self, x_translation: f32, y_translation: f32) {
        unsafe {
            sys::ImpellerDisplayListBuilderTranslate(self.0, x_translation, y_translation);
        }
    }
    //------------------------------------------------------------------------------
    /// Appends the the provided transformation to the transformation
    /// already on the save stack.
    ///
    /// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/transforms/matrix>
    ///
    /// - transform  The transform to append.
    #[doc(alias = "sys::ImpellerDisplayListBuilderTransform")]
    pub fn transform(&mut self, transform: &Matrix) {
        unsafe {
            sys::ImpellerDisplayListBuilderTransform(self.0, cast_ref(transform));
        }
    }

    //------------------------------------------------------------------------------
    /// Clear the transformation on top of the save stack and replace it
    /// with a new value.
    ///
    /// <https://shopify.github.io/react-native-skia/docs/group/#transformations>
    ///
    /// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/transforms/matrix>
    ///
    /// - transform  The new transform.
    #[doc(alias = "sys::ImpellerDisplayListBuilderSetTransform")]
    pub fn set_transform(&mut self, transform: &Matrix) {
        unsafe {
            sys::ImpellerDisplayListBuilderSetTransform(self.0, cast_ref(transform));
        }
    }
    //------------------------------------------------------------------------------
    /// Get the transformation currently built up on the top of the
    /// transformation stack.
    ///
    /// @see [Self::set_transform] and [Self::transform]
    ///
    /// @return The transform.
    #[doc(alias = "sys::ImpellerDisplayListBuilderGetTransform")]
    pub fn get_transform(&self) -> sys::ImpellerMatrix {
        let mut out_transform = sys::ImpellerMatrix::default();
        unsafe {
            sys::ImpellerDisplayListBuilderGetTransform(self.0, &mut out_transform);
        }
        out_transform
    }

    //------------------------------------------------------------------------------
    /// Reset the transformation on top of the transformation stack to
    /// identity.
    ///
    /// @see [Self::set_transform], [Self::transform] and [Self::get_transform]
    #[doc(alias = "sys::ImpellerDisplayListBuilderResetTransform")]
    pub fn reset_transform(&mut self) {
        unsafe {
            sys::ImpellerDisplayListBuilderResetTransform(self.0);
        }
    }
    //------------------------------------------------------------------------------
    /// Get the current size of the save stack.
    ///
    /// @see [Self::save], [Self::save_layer], [Self::restore] and [Self::restore_to_count]
    ///
    /// @return     The save stack size.
    #[doc(alias = "sys::ImpellerDisplayListBuilderGetSaveCount")]
    pub fn get_save_count(&mut self) -> u32 {
        unsafe { sys::ImpellerDisplayListBuilderGetSaveCount(self.0) }
    }
    //------------------------------------------------------------------------------
    /// Effectively calls ImpellerDisplayListBuilderRestore till the
    /// size of the save stack becomes a specified count.
    ///
    /// @see [Self::save], [Self::save_layer], [Self::restore] and [Self::get_save_count]
    ///
    /// - count    The count.
    #[doc(alias = "sys::ImpellerDisplayListBuilderRestoreToCount")]
    pub fn restore_to_count(&mut self, count: u32) {
        unsafe {
            sys::ImpellerDisplayListBuilderRestoreToCount(self.0, count);
        }
    }
    //------------------------------------------------------------------------------
    // Display List Builder: Clipping
    //------------------------------------------------------------------------------

    //------------------------------------------------------------------------------
    /// Reduces the clip region to the intersection of the current clip
    /// and the given rectangle taking into account the clip operation.
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/Canvas/clipRect.html>
    ///
    /// <https://shopify.github.io/react-native-skia/docs/group/#clip-rectangle>
    ///
    /// - rect     The rectangle.
    /// - op       The operation.
    #[doc(alias = "sys::ImpellerDisplayListBuilderClipRect")]
    pub fn clip_rect(&mut self, rect: &Rect, op: ClipOperation) {
        unsafe {
            sys::ImpellerDisplayListBuilderClipRect(self.0, cast_ref(rect), op);
        }
    }

    //------------------------------------------------------------------------------
    /// Reduces the clip region to the intersection of the current clip
    /// and the given oval taking into account the clip operation.
    ///
    /// - oval_bounds  The oval bounds.
    /// - op           The operation.
    #[doc(alias = "sys::ImpellerDisplayListBuilderClipOval")]
    pub fn clip_oval(&mut self, oval_bounds: &Rect, op: ClipOperation) {
        unsafe {
            sys::ImpellerDisplayListBuilderClipOval(self.0, cast_ref(oval_bounds), op);
        }
    }
    //------------------------------------------------------------------------------
    /// Reduces the clip region to the intersection of the current clip
    /// and the given rounded rectangle taking into account the clip
    /// operation.
    ///
    /// <https://shopify.github.io/react-native-skia/docs/group/#clip-rounded-rectangle>
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/Canvas/clipRRect.html>
    ///
    /// - rect     The rectangle.
    /// - radii    The radii.
    /// - op       The operation.
    #[doc(alias = "sys::ImpellerDisplayListBuilderClipRoundedRect")]
    pub fn clip_rounded_rect(&mut self, rect: &Rect, radii: &RoundingRadii, op: ClipOperation) {
        unsafe {
            sys::ImpellerDisplayListBuilderClipRoundedRect(self.0, cast_ref(rect), radii, op);
        }
    }
    //------------------------------------------------------------------------------
    /// Reduces the clip region to the intersection of the current clip
    /// and the given path taking into account the clip operation.
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/Canvas/clipPath.html>
    ///
    /// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/curves/clipping>
    ///
    /// <https://shopify.github.io/react-native-skia/docs/group#clip-path>
    ///
    /// - path     The path.
    /// - op       The operation.
    #[doc(alias = "sys::ImpellerDisplayListBuilderClipPath")]
    pub fn clip_path(&mut self, path: &Path, op: ClipOperation) {
        unsafe {
            sys::ImpellerDisplayListBuilderClipPath(self.0, path.0, op);
        }
    }
    //------------------------------------------------------------------------------
    // Display List Builder: Drawing Shapes
    //------------------------------------------------------------------------------

    //------------------------------------------------------------------------------
    /// Fills the current clip with the specified paint.
    ///
    /// - paint    The paint.
    #[doc(alias = "sys::ImpellerDisplayListBuilderDrawPaint")]
    pub fn draw_paint(&mut self, paint: &Paint) {
        unsafe {
            sys::ImpellerDisplayListBuilderDrawPaint(self.0, paint.0);
        }
    }
    //------------------------------------------------------------------------------
    /// Draws a line segment.
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/Canvas/drawLine.html>
    ///
    /// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/paths/lines>
    ///
    /// - from     The starting point of the line.
    /// - to       The end point of the line.
    /// - paint    The paint.
    #[doc(alias = "sys::ImpellerDisplayListBuilderDrawLine")]
    pub fn draw_line(&mut self, from: Point, to: Point, paint: &Paint) {
        unsafe {
            sys::ImpellerDisplayListBuilderDrawLine(
                self.0,
                cast_ref(&from),
                cast_ref(&to),
                paint.0,
            );
        }
    }

    //------------------------------------------------------------------------------
    /// Draws a dash line segment.
    ///
    /// - from        The starting point of the line.
    /// - to          The end point of the line.
    /// - on_length   On length.
    /// - off_length  Off length.
    /// - paint       The paint.
    #[doc(alias = "sys::ImpellerDisplayListBuilderDrawDashedLine")]
    pub fn draw_dashed_line(
        &mut self,
        from: Point,
        to: Point,
        on_length: f32,
        off_length: f32,
        paint: &Paint,
    ) {
        unsafe {
            sys::ImpellerDisplayListBuilderDrawDashedLine(
                self.0,
                cast_ref(&from),
                cast_ref(&to),
                on_length,
                off_length,
                paint.0,
            );
        }
    }

    //------------------------------------------------------------------------------
    /// Draws a rectangle.
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/Canvas/drawRect.html>
    ///
    /// - rect     The rectangle.
    /// - paint    The paint.
    #[doc(alias = "sys::ImpellerDisplayListBuilderDrawRect")]
    pub fn draw_rect(&mut self, rect: &Rect, paint: &Paint) {
        unsafe {
            sys::ImpellerDisplayListBuilderDrawRect(self.0, cast_ref(rect), paint.0);
        }
    }
    //------------------------------------------------------------------------------
    /// Draws an oval.
    ///
    /// <https://shopify.github.io/react-native-skia/docs/shapes/ellipses#oval>
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/Canvas/drawOval.html>
    ///
    /// - oval_bounds  The oval bounds.
    /// - paint        The paint.
    #[doc(alias = "sys::ImpellerDisplayListBuilderDrawOval")]
    pub fn draw_oval(&mut self, oval_bounds: &Rect, paint: &Paint) {
        unsafe {
            sys::ImpellerDisplayListBuilderDrawOval(self.0, cast_ref(oval_bounds), paint.0);
        }
    }
    //------------------------------------------------------------------------------
    /// Draws a rounded rect.
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/Canvas/drawRRect.html>
    ///
    /// <https://shopify.github.io/react-native-skia/docs/shapes/polygons#using-custom-radii>
    ///
    /// - rect     The rectangle.
    /// - radii    The radii.
    /// - paint    The paint.
    #[doc(alias = "sys::ImpellerDisplayListBuilderDrawRoundedRect")]
    pub fn draw_rounded_rect(&mut self, rect: &Rect, radii: &RoundingRadii, paint: &Paint) {
        unsafe {
            sys::ImpellerDisplayListBuilderDrawRoundedRect(self.0, cast_ref(rect), radii, paint.0);
        }
    }
    //------------------------------------------------------------------------------
    /// Draws a shape that is the different between the specified
    /// rectangles (each with configurable corner radii).
    ///
    /// <https://shopify.github.io/react-native-skia/docs/shapes/polygons#diffrect>
    ///
    /// - outer_rect   The outer rectangle.
    /// - outer_radii  The outer radii.
    /// - inner_rect   The inner rectangle.
    /// - inner_radii  The inner radii.
    /// - paint        The paint.
    #[doc(alias = "sys::ImpellerDisplayListBuilderDrawRoundedRectDifference")]
    pub fn draw_rounded_rect_difference(
        &mut self,
        outer_rect: &Rect,
        outer_radii: &RoundingRadii,
        inner_rect: &Rect,
        inner_radii: &RoundingRadii,
        paint: &Paint,
    ) {
        unsafe {
            sys::ImpellerDisplayListBuilderDrawRoundedRectDifference(
                self.0,
                cast_ref(outer_rect),
                outer_radii,
                cast_ref(inner_rect),
                inner_radii,
                paint.0,
            );
        }
    }
    //------------------------------------------------------------------------------
    /// Draws the specified path shape.
    ///
    /// @see [Path] and [PathBuilder]
    ///
    /// - path     The path.
    /// - paint    The paint.
    #[doc(alias = "sys::ImpellerDisplayListBuilderDrawPath")]
    pub fn draw_path(&mut self, path: &Path, paint: &Paint) {
        unsafe {
            sys::ImpellerDisplayListBuilderDrawPath(self.0, path.0, paint.0);
        }
    }
    //------------------------------------------------------------------------------
    /// Flattens the contents of another display list into the one
    /// currently being built.
    ///
    /// In skia, display lists are often called `Pictures`.
    ///
    /// <https://shopify.github.io/react-native-skia/docs/shapes/pictures>
    ///
    /// - display_list  The display list.
    /// - opacity       The opacity.
    #[doc(alias = "sys::ImpellerDisplayListBuilderDrawDisplayList")]
    pub fn draw_display_list(&mut self, display_list: &DisplayList, opacity: f32) {
        unsafe {
            sys::ImpellerDisplayListBuilderDrawDisplayList(self.0, display_list.0, opacity);
        }
    }
    //------------------------------------------------------------------------------
    /// Draw a paragraph at the specified point.
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/Canvas/drawParagraph.html>
    ///
    /// @see [Paragraph], [ParagraphBuilder] and [ParagraphStyle]
    ///
    /// - paragraph  The paragraph.
    /// - point      The point where to draw the paragraph. (offset)
    #[doc(alias = "sys::ImpellerDisplayListBuilderDrawParagraph")]
    pub fn draw_paragraph(&mut self, paragraph: &Paragraph, point: Point) {
        unsafe {
            sys::ImpellerDisplayListBuilderDrawParagraph(self.0, paragraph.0, cast_ref(&point));
        }
    }

    //------------------------------------------------------------------------------
    /// Draw a shadow for a Path given a material elevation. If the
    /// occluding object is not opaque, additional hints (via the
    /// `occluder_is_transparent` argument) must be provided to render
    /// the shadow correctly.
    ///
    /// * path       The shadow path.
    /// * color      The shadow color.
    /// * elevation  The material elevation.
    /// * occluder_is_transparent If the object casting the shadow is transparent.
    /// *  device_pixel_ratio The device pixel ratio.
    pub fn draw_shadow(
        &mut self,
        path: &Path,
        color: &Color,
        elevation: f32,
        occluder_is_transparent: bool,
        device_pixel_ratio: f32,
    ) {
        unsafe {
            sys::ImpellerDisplayListBuilderDrawShadow(
                self.0,
                path.0,
                cast_ref(color),
                elevation,
                occluder_is_transparent,
                device_pixel_ratio,
            );
        }
    }
    //------------------------------------------------------------------------------
    // Display List Builder: Drawing Textures
    //------------------------------------------------------------------------------

    //------------------------------------------------------------------------------
    /// Draw a texture at the specified point.
    ///
    /// When you draw a texture, you draw it in its full size.
    /// To adjust the size, you can use [DisplayListBuilder::scale].
    /// eg: if you wanted to draw a 500x500 texture as 250x250, just scale by 0.5
    ///     Make sure to use the same scale for both x and y, or you will stretch/compress the image.
    ///
    /// Another way to draw a texture is to use [DisplayListBuilder::draw_texture_rect],
    /// which allows you to choose a source rect (part of image) and draw it to any rect on canvas.
    ///
    /// - texture   The texture.
    /// - point     The point.
    /// - sampling  The sampling.
    /// - paint     The paint.
    #[doc(alias = "sys::ImpellerDisplayListBuilderDrawTexture")]
    pub fn draw_texture(
        &mut self,
        texture: &Texture,
        point: Point,
        sampling: TextureSampling,
        paint: &Paint,
    ) {
        unsafe {
            sys::ImpellerDisplayListBuilderDrawTexture(
                self.0,
                texture.0,
                cast_ref(&point),
                sampling,
                paint.0,
            );
        }
    }
    //------------------------------------------------------------------------------
    /// Draw a portion of texture at the specified location.
    ///
    /// This function takes a portion (src_rect) of the texture
    /// and draws it at dst_rect on canvas. It will do the necessary
    /// scaling to make sure that the src_rect will fit onto dst_rect exactly.
    ///
    /// Look at [Paint] struct for how you can customize the drawing.
    /// eg: blurring the image or adding a color tint.
    ///
    /// - texture   The texture.
    /// - src_rect  The source rectangle.
    /// - dst_rect  The destination rectangle.
    /// - sampling  The sampling.
    /// - paint     The paint.
    #[doc(alias = "sys::ImpellerDisplayListBuilderDrawTextureRect")]
    pub fn draw_texture_rect(
        &mut self,
        texture: &Texture,
        src_rect: &Rect,
        dst_rect: &Rect,
        sampling: TextureSampling,
        paint: Option<&Paint>,
    ) {
        unsafe {
            sys::ImpellerDisplayListBuilderDrawTextureRect(
                self.0,
                texture.0,
                cast_ref(src_rect),
                cast_ref(dst_rect),
                sampling,
                paint.map_or(std::ptr::null_mut(), |p| p.0),
            );
        }
    }
}
/// Paints control the behavior of draw calls encoded in a display list.
///
/// Like display lists, paints are context-agnostic.
///
/// NOTE: If you understand this struct, then you understand Impeller.
///
/// <https://api.flutter.dev/flutter/dart-ui/Paint-class.html>
///
/// <https://shopify.github.io/react-native-skia/docs/paint/overview>
///
/// <https://shopify.github.io/react-native-skia/docs/paint/properties>
///
/// <https://learn.microsoft.com/en-us/dotnet/api/skiasharp.skpaint?view=skiasharp-2.88>
#[derive(Debug)]
#[repr(transparent)]
pub struct Paint(sys::ImpellerPaint);

// impl Clone for Paint {
//     fn clone(&self) -> Self {
//         unsafe {
//             sys::ImpellerPaintRetain(self.0);
//         }
//         Self(self.0)
//     }
// }
unsafe impl Send for Paint {}
unsafe impl Sync for Paint {}

impl Drop for Paint {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerPaintRelease(self.0);
        }
    }
}
impl Default for Paint {
    fn default() -> Self {
        let p = unsafe { sys::ImpellerPaintNew() };
        assert!(!p.is_null());
        Self(p)
    }
}
impl Paint {
    /// Set the paint color for stroking or filling.
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/Paint/color.html>
    ///
    /// - color     The color.
    ///
    pub fn set_color(&mut self, color: Color) {
        unsafe {
            sys::ImpellerPaintSetColor(self.0, &color);
        }
    }

    /// Set the paint blend mode. The blend mode controls how the new
    /// paints contents are mixed with the values already drawn using
    /// previous draw calls.
    ///
    /// - mode      The mode.
    ///
    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        unsafe {
            sys::ImpellerPaintSetBlendMode(self.0, mode);
        }
    }

    /// Set the paint draw style. The style controls if the closed
    /// shapes are filled and/or stroked.
    ///
    /// - style     The style.
    ///
    pub fn set_draw_style(&mut self, style: DrawStyle) {
        unsafe {
            sys::ImpellerPaintSetDrawStyle(self.0, style);
        }
    }

    /// Sets how strokes rendered using this paint are capped.
    ///
    /// - cap       The stroke cap style.
    ///
    pub fn set_stroke_cap(&mut self, cap: StrokeCap) {
        unsafe {
            sys::ImpellerPaintSetStrokeCap(self.0, cap);
        }
    }

    /// Sets how strokes rendered using this paint are joined.
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/Paint/strokeJoin.html>
    ///
    /// - join      The join.
    ///
    pub fn set_stroke_join(&mut self, join: StrokeJoin) {
        unsafe {
            sys::ImpellerPaintSetStrokeJoin(self.0, join);
        }
    }

    /// Set the width of the strokes rendered using this paint.
    ///
    /// - width     The width.
    ///
    pub fn set_stroke_width(&mut self, width: f32) {
        unsafe {
            sys::ImpellerPaintSetStrokeWidth(self.0, width);
        }
    }

    /// Set the miter limit of the strokes rendered using this paint.
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/Paint/strokeMiterLimit.html>
    ///
    /// - miter     The miter limit.
    ///
    pub fn set_stroke_miter(&mut self, miter: f32) {
        unsafe {
            sys::ImpellerPaintSetStrokeMiter(self.0, miter);
        }
    }

    /// Set the color filter of the paint.
    ///
    /// Color filters are functions that take two colors and mix them to
    /// produce a single color. This color is then usually merged with
    /// the destination during blending.
    ///
    /// - color_filter  The color filter.
    pub fn set_color_filter(&mut self, color_filter: &ColorFilter) {
        unsafe {
            sys::ImpellerPaintSetColorFilter(self.0, color_filter.0);
        }
    }

    /// Set the image filter of a paint.
    ///
    /// Image filters are functions that are applied to regions of a
    /// texture to produce a single color.
    ///
    /// - image_filter  The image filter.
    pub fn set_image_filter(&mut self, image_filter: &ImageFilter) {
        unsafe {
            sys::ImpellerPaintSetImageFilter(self.0, image_filter.0);
        }
    }
    /// Set the color source of the paint.
    ///
    /// Color sources are functions that generate colors for each
    /// texture element covered by a draw call.
    ///
    /// - color_source  The color source.
    pub fn set_color_source(&mut self, color_source: &ColorSource) {
        unsafe {
            sys::ImpellerPaintSetColorSource(self.0, color_source.0);
        }
    }
    /// Set the mask filter of a paint.
    ///
    /// Mask filters are functions that are applied over a shape after it
    /// has been drawn but before it has been blended into the final
    /// image.
    ///
    /// - mask_filter  The mask filter.
    pub fn set_mask_filter(&mut self, mask_filter: &MaskFilter) {
        unsafe {
            sys::ImpellerPaintSetMaskFilter(self.0, mask_filter.0);
        }
    }
}
/// Color filters are functions that take two colors and mix them to produce a
/// single color. This color is then merged with the destination during
/// blending.
///
/// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/effects/color-filters>
///
/// <https://api.flutter.dev/flutter/dart-ui/ColorFilter-class.html>
///
/// <https://shopify.github.io/react-native-skia/docs/color-filters>
#[derive(Debug)]
#[repr(transparent)]
pub struct ColorFilter(sys::ImpellerColorFilter);
unsafe impl Send for ColorFilter {}
unsafe impl Sync for ColorFilter {}
impl Clone for ColorFilter {
    fn clone(&self) -> Self {
        unsafe {
            sys::ImpellerColorFilterRetain(self.0);
        }
        Self(self.0)
    }
}

impl Drop for ColorFilter {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerColorFilterRelease(self.0);
        }
    }
}
impl ColorFilter {
    /// Create a color filter that performs blending of pixel values
    /// independently.
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/ColorFilter/ColorFilter.mode.html>
    ///
    /// <https://shopify.github.io/react-native-skia/docs/color-filters#blendcolor>
    ///
    ///
    /// - color       The color.
    /// - blend_mode  The blend mode.
    ///
    /// @return     The color filter.
    #[must_use]
    pub fn new_blend(color: Color, blend_mode: BlendMode) -> Self {
        unsafe { Self(sys::ImpellerColorFilterCreateBlendNew(&color, blend_mode)) }
    }

    /// Create a color filter that transforms pixel color values
    /// independently.
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/ColorFilter/ColorFilter.matrix.html>
    ///
    /// <https://shopify.github.io/react-native-skia/docs/color-filters#color-matrix>
    ///
    /// playground to play with matrices: <https://fecolormatrix.com/>
    ///
    /// read more in struct docs [ColorFilter]
    #[must_use]
    pub fn new_matrix(color_matrix: ColorMatrix) -> Self {
        unsafe { Self(sys::ImpellerColorFilterCreateColorMatrixNew(&color_matrix)) }
    }
}
/// Color sources are functions that generate colors for each texture element
/// covered by a draw call. The colors for each element can be generated using a
/// mathematical function (to produce gradients for example) or sampled from a
/// texture.
///
/// <https://api.flutter.dev/flutter/dart-ui/Gradient-class.html>
///
/// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/effects/shaders/>
#[derive(Debug)]
#[repr(transparent)]
///
pub struct ColorSource(sys::ImpellerColorSource);
unsafe impl Send for ColorSource {}
unsafe impl Sync for ColorSource {}
impl Clone for ColorSource {
    fn clone(&self) -> Self {
        unsafe {
            sys::ImpellerColorSourceRetain(self.0);
        }
        Self(self.0)
    }
}

impl Drop for ColorSource {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerColorSourceRelease(self.0);
        }
    }
}
impl ColorSource {
    //------------------------------------------------------------------------------
    /// Create a color source that forms a linear gradient.
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/Gradient/Gradient.linear.html>
    ///
    /// <https://api.flutter.dev/flutter/painting/LinearGradient-class.html>
    ///
    /// <https://shopify.github.io/react-native-skia/docs/shaders/gradients#linear-gradient>
    ///
    /// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/effects/shaders/linear-gradient>
    ///
    /// - start_point     The start point.
    /// - end_point       The end point.
    /// - colors          The colors.
    /// - stops           The stops.
    /// - tile_mode       The tile mode.
    /// - transformation  The transformation.
    ///
    /// @return     The color source.
    #[must_use]
    pub fn new_linear_gradient(
        start: Point,
        end: Point,
        colors: &[Color],
        stops: &[f32],
        tile_mode: TileMode,
        transformation: Option<&Matrix>,
    ) -> Self {
        assert_eq!(colors.len(), stops.len());
        assert!(!colors.is_empty());
        let result = unsafe {
            sys::ImpellerColorSourceCreateLinearGradientNew(
                cast_ref(&start),
                cast_ref(&end),
                stops.len() as _,
                colors.as_ptr(),
                stops.as_ptr(),
                tile_mode,
                transformation.map_or(std::ptr::null(), |m| cast_ref(m)),
            )
        };
        assert!(!result.is_null());
        Self(result)
    }

    //------------------------------------------------------------------------------
    /// Create a color source that forms a radial gradient.
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/Gradient/Gradient.radial.html>
    ///
    /// <https://api.flutter.dev/flutter/painting/RadialGradient-class.html>
    ///
    /// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/effects/shaders/circular-gradients#the-radial-gradient>
    ///
    /// <https://shopify.github.io/react-native-skia/docs/shaders/gradients#radial-gradient>
    ///
    /// - center          The center.
    /// - radius          The radius.
    /// - stop_count      The stop count.
    /// - colors          The colors.
    /// - stops           The stops.
    /// - tile_mode       The tile mode.
    /// - transformation  The transformation.
    ///
    /// @return     The color source.
    #[must_use]
    pub fn new_radial_gradient(
        center: Point,
        radius: f32,
        colors: &[Color],
        stops: &[f32],
        tile_mode: TileMode,
        transformation: Option<&Matrix>,
    ) -> Self {
        assert_eq!(colors.len(), stops.len());
        assert!(!colors.is_empty());
        let result = unsafe {
            sys::ImpellerColorSourceCreateRadialGradientNew(
                cast_ref(&center),
                radius,
                stops.len() as _,
                colors.as_ptr(),
                stops.as_ptr(),
                tile_mode,
                transformation.map_or(std::ptr::null(), |m| cast_ref(m)),
            )
        };
        assert!(!result.is_null());
        Self(result)
    }

    //------------------------------------------------------------------------------
    /// Create a color source that forms a conical gradient.
    ///
    /// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/effects/shaders/circular-gradients#the-two-point-conical-gradient>
    ///
    /// <https://shopify.github.io/react-native-skia/docs/shaders/gradients#two-point-conical-gradient>
    ///
    /// - start_center    The start center.
    /// - start_radius    The start radius.
    /// - end_center      The end center.
    /// - end_radius      The end radius.
    /// - stop_count      The stop count.
    /// - colors          The colors.
    /// - stops           The stops.
    /// - tile_mode       The tile mode.
    /// - transformation  The transformation.
    ///
    /// @return     The color source.
    #[allow(clippy::too_many_arguments)]
    pub fn new_conical_gradient(
        start_center: Point,
        start_radius: f32,
        end_center: Point,
        end_radius: f32,
        colors: &[Color],
        stops: &[f32],
        tile_mode: TileMode,
        transformation: Option<&Matrix>,
    ) -> Self {
        assert_eq!(colors.len(), stops.len());
        assert!(!colors.is_empty());
        let result = unsafe {
            sys::ImpellerColorSourceCreateConicalGradientNew(
                cast_ref(&start_center),
                start_radius,
                cast_ref(&end_center),
                end_radius,
                stops.len() as _,
                colors.as_ptr(),
                stops.as_ptr(),
                tile_mode,
                transformation.map_or(std::ptr::null(), |m| cast_ref(m)),
            )
        };
        assert!(!result.is_null());
        Self(result)
    }

    //------------------------------------------------------------------------------
    /// Create a color source that forms a sweep gradient.
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/Gradient/Gradient.sweep.html>
    ///
    /// <https://api.flutter.dev/flutter/painting/SweepGradient-class.html>
    ///
    /// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/effects/shaders/circular-gradients#the-sweep-gradient>
    ///
    /// <https://shopify.github.io/react-native-skia/docs/shaders/gradients#sweep-gradient>
    ///
    /// - center          The center.
    /// - start           The start.
    /// - end             The end.
    /// - stop_count      The stop count.
    /// - colors          The colors.
    /// - stops           The stops.
    /// - tile_mode       The tile mode.
    /// - transformation  The transformation.
    ///
    /// @return     The color source.
    #[must_use]
    pub fn new_sweep_gradient(
        center: Point,
        start: f32,
        end: f32,
        colors: &[Color],
        stops: &[f32],
        tile_mode: TileMode,
        transformation: Option<&Matrix>,
    ) -> Self {
        assert_eq!(colors.len(), stops.len());
        assert!(!colors.is_empty());
        let result = unsafe {
            sys::ImpellerColorSourceCreateSweepGradientNew(
                cast_ref(&center),
                start,
                end,
                stops.len() as _,
                colors.as_ptr(),
                stops.as_ptr(),
                tile_mode,
                transformation.map_or(std::ptr::null(), |m| cast_ref(m)),
            )
        };
        assert!(!result.is_null());
        Self(result)
    }
    /// Create a color source that samples from an image.
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/ImageShader/ImageShader.html>
    ///
    /// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/effects/shaders/bitmap-tiling>
    ///
    /// <https://shopify.github.io/react-native-skia/docs/shaders/images>
    ///
    /// - image                 The image.
    /// - horizontal_tile_mode  The horizontal tile mode.
    /// - vertical_tile_mode    The vertical tile mode.
    /// - sampling              The sampling.
    /// - transformation        The transformation.
    ///
    /// @return     The color source.
    #[must_use]
    pub fn new_image(
        image: &Texture,
        horizontal_tile_mode: TileMode,
        vertical_tile_mode: TileMode,
        sampling: TextureSampling,
        transformation: Option<&sys::ImpellerMatrix>,
    ) -> Self {
        let result = unsafe {
            sys::ImpellerColorSourceCreateImageNew(
                image.0,
                horizontal_tile_mode,
                vertical_tile_mode,
                sampling,
                transformation.map_or(std::ptr::null(), |m| m),
            )
        };
        assert!(!result.is_null());
        Self(result)
    }
}
/// Image filters are functions that are applied regions of a texture to produce
/// a single color. Contrast this with color filters that operate independently
/// on a per-pixel basis. The generated color is then merged with the
/// destination during blending.
///
/// <https://api.flutter.dev/flutter/dart-ui/ImageFilter-class.html>
///
/// <https://shopify.github.io/react-native-skia/docs/image-filters/overview>
///
/// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/effects/image-filters>
#[derive(Debug)]
#[repr(transparent)]
pub struct ImageFilter(sys::ImpellerImageFilter);
unsafe impl Send for ImageFilter {}
unsafe impl Sync for ImageFilter {}
impl Clone for ImageFilter {
    fn clone(&self) -> Self {
        unsafe {
            sys::ImpellerImageFilterRetain(self.0);
        }
        Self(self.0)
    }
}
impl Drop for ImageFilter {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerImageFilterRelease(self.0);
        }
    }
}
impl ImageFilter {
    /// Creates an image filter that applies a Gaussian blur.
    ///
    /// The Gaussian blur applied may be an approximation for
    /// performance.
    ///
    /// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/effects/image-filters#blurring-vector-graphics-and-bitmaps>
    ///
    /// <https://shopify.github.io/react-native-skia/docs/image-filters/blur>
    ///
    /// - x_sigma    The x sigma.
    /// - y_sigma    The y sigma.
    /// - tile_mode  The tile mode.
    ///
    /// @return     The image filter.
    #[must_use]
    pub fn new_blur(x_sigma: f32, y_sigma: f32, tile_mode: TileMode) -> Self {
        let result = unsafe { sys::ImpellerImageFilterCreateBlurNew(x_sigma, y_sigma, tile_mode) };
        assert!(!result.is_null());
        Self(result)
    }
    /// Creates an image filter that enhances the per-channel pixel
    /// values to the maximum value in a circle around the pixel.
    ///
    /// <https://shopify.github.io/react-native-skia/docs/image-filters/morphology>
    ///
    /// - x_radius  The x radius.
    /// - y_radius  The y radius.
    ///
    /// @return     The image filter.
    #[must_use]
    pub fn new_dilate(x_radius: f32, y_radius: f32) -> Self {
        let result = unsafe { sys::ImpellerImageFilterCreateDilateNew(x_radius, y_radius) };
        assert!(!result.is_null());
        Self(result)
    }
    /// Creates an image filter that dampens the per-channel pixel
    /// values to the minimum value in a circle around the pixel.
    ///
    /// <https://shopify.github.io/react-native-skia/docs/image-filters/morphology>
    ///
    /// - x_radius  The x radius.
    /// - y_radius  The y radius.
    ///
    /// @return     The image filter.
    #[must_use]
    pub fn new_erode(x_radius: f32, y_radius: f32) -> Self {
        let result = unsafe { sys::ImpellerImageFilterCreateErodeNew(x_radius, y_radius) };
        assert!(!result.is_null());
        Self(result)
    }
    /// Creates an image filter that applies a transformation matrix to
    /// the underlying image.
    ///
    /// - matrix    The transformation matrix.
    /// - sampling  The image sampling mode.
    ///
    /// @return     The image filter.
    #[must_use]
    pub fn new_matrix(matrix: &Matrix, sampling: TextureSampling) -> Self {
        let result = unsafe { sys::ImpellerImageFilterCreateMatrixNew(cast_ref(matrix), sampling) };
        assert!(!result.is_null());
        Self(result)
    }

    //------------------------------------------------------------------------------
    /// Creates a composed filter that when applied is identical to
    /// subsequently applying the inner and then the outer filters.
    ///
    /// ```cpp
    /// destination = outer_filter(inner_filter(source))
    /// ```
    ///
    /// <https://shopify.github.io/react-native-skia/docs/image-filters/overview#composing-filters>
    ///
    /// - outer  The outer image filter.
    /// - inner  The inner image filter.
    ///
    /// @return     The combined image filter.
    #[must_use]
    pub fn new_compose(outer: &Self, inner: &Self) -> Self {
        let result = unsafe { sys::ImpellerImageFilterCreateComposeNew(outer.0, inner.0) };
        assert!(!result.is_null());
        Self(result)
    }
}
/// Mask filters are functions that are applied over a shape after it has been
/// drawn but before it has been blended into the final image.
///
/// <https://api.flutter.dev/flutter/dart-ui/MaskFilter-class.html>
///
/// <https://shopify.github.io/react-native-skia/docs/mask-filters>
///
/// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/effects/mask-filters>
#[derive(Debug)]
#[repr(transparent)]
pub struct MaskFilter(sys::ImpellerMaskFilter);
unsafe impl Send for MaskFilter {}
unsafe impl Sync for MaskFilter {}
impl Clone for MaskFilter {
    fn clone(&self) -> Self {
        unsafe {
            sys::ImpellerMaskFilterRetain(self.0);
        }
        Self(self.0)
    }
}
impl Drop for MaskFilter {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerMaskFilterRelease(self.0);
        }
    }
}
impl MaskFilter {
    //------------------------------------------------------------------------------
    /// Create a mask filter that blurs contents in the masked shape.
    ///
    /// <https://api.flutter.dev/flutter/dart-ui/MaskFilter/MaskFilter.blur.html>
    ///
    /// @see doc of struct [MaskFilter]
    ///
    /// - style  The style.
    /// - sigma  The sigma.
    ///
    /// @return     The mask filter.
    #[must_use]
    pub fn new_blur(style: BlurStyle, sigma: f32) -> Self {
        let result = unsafe { sys::ImpellerMaskFilterCreateBlurNew(style, sigma) };
        assert!(!result.is_null());
        Self(result)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct FragmentProgram(sys::ImpellerFragmentProgram);
unsafe impl Sync for FragmentProgram {}
unsafe impl Send for FragmentProgram {}
impl Clone for FragmentProgram {
    fn clone(&self) -> Self {
        unsafe {
            sys::ImpellerFragmentProgramRetain(self.0);
        }
        Self(self.0)
    }
}
impl Drop for FragmentProgram {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerFragmentProgramRelease(self.0);
        }
    }
}
impl FragmentProgram {
    /// Create a new fragment program using data obtained by compiling a GLSL shader with impellerc.
    /// # Safety
    /// The data provided MUST be compiled by impellerc.
    /// Providing raw GLSL strings is not supported.
    /// Impeller does not compile shaders at runtime.
    pub unsafe fn new(glsl_shader_compiled_by_impellerc: Cow<'static, [u8]>) -> Option<Self> {
        let f = unsafe {
            let (mapping, userdata) =
                sys::ImpellerMapping::from_cow(glsl_shader_compiled_by_impellerc);
            sys::ImpellerFragmentProgramNew(&mapping, userdata)
        };
        if f.is_null() {
            None
        } else {
            Some(Self(f))
        }
    }
}
/// Typography contexts allow for the layout and rendering of text.
///
/// These are typically expensive to create and applications will only ever need
/// to create a single one of these during their lifetimes.
///
/// These hold the "font data" for building paragraphs.
/// You can optionally register custom fonts or just use the fonts
/// available on user's system.
///
/// Unlike graphics context, typograhy contexts are not thread-safe. These must
/// be created, used, and collected on a single thread.
///
/// @see [ParagraphStyle]
#[derive(Debug)]
#[repr(transparent)]
pub struct TypographyContext(sys::ImpellerTypographyContext);

impl Clone for TypographyContext {
    fn clone(&self) -> Self {
        unsafe {
            sys::ImpellerTypographyContextRetain(self.0);
        }
        Self(self.0)
    }
}
impl Drop for TypographyContext {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerTypographyContextRelease(self.0);
        }
    }
}
impl Default for TypographyContext {
    fn default() -> Self {
        let result = unsafe { sys::ImpellerTypographyContextNew() };
        assert!(!result.is_null());
        Self(result)
    }
}
impl TypographyContext {
    /// Register a custom font.
    ///
    /// The following font formats are supported:
    /// * OpenType font collections (.ttc extension)
    /// * TrueType fonts: (.ttf extension)
    /// * OpenType fonts: (.otf extension)
    ///
    /// @warning: Web Open Font Formats (.woff and .woff2 extensions) are **not**
    /// supported.
    ///
    /// The family alias name can be NULL. In such cases, the font
    /// family specified in paragraph styles must match the family that
    /// is specified in the font data.
    ///
    /// If the family name alias is not NULL, that family name must be
    /// used in the paragraph style to reference glyphs from this font
    /// instead of the one encoded in the font itself.
    ///
    /// Multiple fonts (with glyphs for different styles) can be
    /// specified with the same family.
    ///
    /// @see        [ParagraphStyle::set_font_family]
    ///
    /// - font_data: The contents.
    /// - family_name_alias: The family name alias or NULL if the one specified in the font
    ///                      data is to be used.
    ///
    /// @return     If the font could be successfully registered.
    #[doc(alias = "ImpellerTypographyContextRegisterFont")]
    pub fn register_font(
        &mut self,
        font_data: Cow<'static, [u8]>,
        family_name_alias: Option<&str>,
    ) -> Result<(), &'static str> {
        let family_name_alias = if let Some(s) = family_name_alias {
            Some(std::ffi::CString::new(s).map_err(|_| "the family name alias has a null byte")?)
        } else {
            None
        };

        let result = unsafe {
            // SAFETY: pass the correct userdata with the correct mapping. Here, we only have one pair, so, we are good.
            let (mapping, userdata) = sys::ImpellerMapping::from_cow(font_data);
            sys::ImpellerTypographyContextRegisterFont(
                self.0,
                &mapping,
                userdata,
                family_name_alias
                    .as_ref()
                    .map_or(std::ptr::null(), |s| s.as_ptr()),
            )
        };
        // explicit drop to ensure that it's not dropped before this point.
        // When I first wrote this function, I used family_name_alias.map(|s|s.as_ptr()) in the previous line, which would have been UB :/
        std::mem::drop(family_name_alias);
        result.then_some(()).ok_or("Failed to register font")
    }
}

/// An immutable, fully laid out paragraph.
///
///
/// <https://shopify.github.io/react-native-skia/docs/text/paragraph>
///
/// @see [ParagraphStyle] and [ParagraphBuilder]
///
#[derive(Debug)]
#[repr(transparent)]
pub struct Paragraph(sys::ImpellerParagraph);
unsafe impl Send for Paragraph {}
unsafe impl Sync for Paragraph {}
impl Clone for Paragraph {
    fn clone(&self) -> Self {
        unsafe {
            sys::ImpellerParagraphRetain(self.0);
        }
        Self(self.0)
    }
}
impl Drop for Paragraph {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerParagraphRelease(self.0);
        }
    }
}
impl Paragraph {
    //------------------------------------------------------------------------------
    /// @see        [Self::get_min_intrinsic_width]
    ///
    /// The width provided to the paragraph builder during the call to
    /// layout. This is the maximum width any line in the laid out
    /// paragraph can occupy. But, it is not necessarily the actual
    ///             width of the paragraph after layout.
    #[doc(alias = "ImpellerParagraphGetMaxWidth")]
    pub fn get_max_width(&self) -> f32 {
        unsafe { sys::ImpellerParagraphGetMaxWidth(self.0) }
    }
    //------------------------------------------------------------------------------
    /// The height of the laid out paragraph. This is **not** a tight
    /// bounding box and some glyphs may not reach the minimum location
    /// they are allowed to reach.
    #[doc(alias = "ImpellerParagraphGetHeight")]
    pub fn get_height(&self) -> f32 {
        unsafe { sys::ImpellerParagraphGetHeight(self.0) }
    }
    //------------------------------------------------------------------------------
    /// The length of the longest line in the paragraph. This is the
    /// horizontal distance between the left edge of the leftmost glyph
    /// and the right edge of the rightmost glyph, in the longest line
    /// in the paragraph.
    #[doc(alias = "ImpellerParagraphGetLongestLineWidth")]
    pub fn get_longest_line_width(&self) -> f32 {
        unsafe { sys::ImpellerParagraphGetLongestLineWidth(self.0) }
    }
    //------------------------------------------------------------------------------
    /// @see        [Self::get_max_width]
    ///
    /// The actual width of the longest line in the paragraph after
    /// layout. This is expected to be less than or equal to
    /// [Self::get_max_width].
    #[doc(alias = "ImpellerParagraphGetMinIntrinsicWidth")]
    pub fn get_min_intrinsic_width(&self) -> f32 {
        unsafe { sys::ImpellerParagraphGetMinIntrinsicWidth(self.0) }
    }
    //------------------------------------------------------------------------------
    /// The width of the paragraph without line breaking.
    #[doc(alias = "ImpellerParagraphGetMaxIntrinsicWidth")]
    pub fn get_max_intrinsic_width(&self) -> f32 {
        unsafe { sys::ImpellerParagraphGetMaxIntrinsicWidth(self.0) }
    }
    //------------------------------------------------------------------------------
    /// The distance from the top of the paragraph to the ideographic
    /// baseline of the first line when using ideographic fonts
    /// (Japanese, Korean, etc...).
    #[doc(alias = "ImpellerParagraphGetIdeographicBaseline")]
    pub fn get_ideographic_baseline(&self) -> f32 {
        unsafe { sys::ImpellerParagraphGetIdeographicBaseline(self.0) }
    }
    //------------------------------------------------------------------------------
    /// The distance from the top of the paragraph to the alphabetic
    /// baseline of the first line when using alphabetic fonts (A-Z,
    /// a-z, Greek, etc...).
    #[doc(alias = "ImpellerParagraphGetAlphabeticBaseline")]
    pub fn get_alphabetic_baseline(&self) -> f32 {
        unsafe { sys::ImpellerParagraphGetAlphabeticBaseline(self.0) }
    }
    //------------------------------------------------------------------------------
    /// The number of lines visible in the paragraph after line
    /// breaking.
    #[doc(alias = "ImpellerParagraphGetLineCount")]
    pub fn get_line_count(&self) -> u32 {
        unsafe { sys::ImpellerParagraphGetLineCount(self.0) }
    }
    /// Get the range into the UTF-16 code unit buffer that represents
    /// the word at the specified caret location in the same buffer.
    ///
    /// Word boundaries are defined more precisely in [Unicode Standard
    /// Annex #29](http://www.unicode.org/reports/tr29/#Word_Boundaries)
    ///
    /// * code_unit_index The code unit index
    ///
    /// * return The impeller range.
    ///
    pub fn get_word_boundary_utf16(&self, code_unit_index: usize) -> Range {
        let mut range = Range::default();
        unsafe { sys::ImpellerParagraphGetWordBoundary(self.0, code_unit_index, &raw mut range) };
        range
    }

    //------------------------------------------------------------------------------
    /// Get the line metrics of this laid out paragraph. Calculating the
    /// line metrics is expensive. The first time line metrics are
    /// requested, they will be cached along with the paragraph (which
    /// is immutable).
    ///
    /// * return The line metrics.
    pub fn get_line_metrics(&self) -> Option<LineMetrics> {
        let ptr = unsafe { sys::ImpellerParagraphGetLineMetrics(self.0) };
        if ptr.is_null() {
            None
        } else {
            Some(LineMetrics(ptr))
        }
    }
    //------------------------------------------------------------------------------
    /// Create a new instance of glyph info that can be queried for
    /// information about the glyph at the given UTF-16 code unit index.
    /// The instance must be freed using `ImpellerGlyphInfoRelease`.
    ///
    /// * code_unit_index  The UTF-16 code unit index.
    ///
    /// * return     The glyph information.
    pub fn create_glyph_info_at_code_unit_index_utf16(
        &self,
        code_unit_index: usize,
    ) -> Option<GlyphInfo> {
        let ptr = unsafe {
            sys::ImpellerParagraphCreateGlyphInfoAtCodeUnitIndexNew(self.0, code_unit_index)
        };
        if ptr.is_null() {
            None
        } else {
            Some(GlyphInfo(ptr))
        }
    }

    //------------------------------------------------------------------------------
    /// Create a new instance of glyph info that can be queried for
    /// information about the glyph closest to the specified coordinates
    /// relative to the origin of the paragraph. The instance must be
    /// freed using `ImpellerGlyphInfoRelease`.
    ///
    /// * x          The x coordinate relative to paragraph origin.
    /// * y          The x coordinate relative to paragraph origin.
    ///
    /// * return     The glyph information.
    ///
    pub fn create_glyph_info_at_paragraph_coordinates(&self, x: f64, y: f64) -> Option<GlyphInfo> {
        let ptr =
            unsafe { sys::ImpellerParagraphCreateGlyphInfoAtParagraphCoordinatesNew(self.0, x, y) };
        if ptr.is_null() {
            None
        } else {
            Some(GlyphInfo(ptr))
        }
    }
}
#[derive(Debug)]
#[repr(transparent)]
pub struct LineMetrics(sys::ImpellerLineMetrics);
unsafe impl Send for LineMetrics {}
unsafe impl Sync for LineMetrics {}
impl Clone for LineMetrics {
    fn clone(&self) -> Self {
        unsafe {
            sys::ImpellerLineMetricsRetain(self.0);
        }
        Self(self.0)
    }
}
impl Drop for LineMetrics {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerLineMetricsRelease(self.0);
        }
    }
}
impl LineMetrics {
    //------------------------------------------------------------------------------
    /// The rise from the baseline as calculated from the font and style
    /// for this line ignoring the height from the text style.
    ///
    /// * line     The line index (zero based).
    ///
    /// @return     The unscaled ascent.
    pub fn get_unscaled_ascent(&self, line: usize) -> f64 {
        unsafe { sys::ImpellerLineMetricsGetUnscaledAscent(self.0, line) }
    }
    //------------------------------------------------------------------------------
    /// The rise from the baseline as calculated from the font and style
    /// for this line.
    ///
    /// * line     The line index (zero based).
    ///
    /// * return     The ascent.
    ///
    pub fn get_ascent(&self, line: usize) -> f64 {
        unsafe { sys::ImpellerLineMetricsGetAscent(self.0, line) }
    }
    //------------------------------------------------------------------------------
    /// The drop from the baseline as calculated from the font and style
    /// for this line.
    ///
    /// * line     The line index (zero based).
    ///
    /// * return     The descent.
    pub fn get_descent(&self, line: usize) -> f64 {
        unsafe { sys::ImpellerLineMetricsGetDescent(self.0, line) }
    }
    //------------------------------------------------------------------------------
    /// The y coordinate of the baseline for this line from the top of
    /// the paragraph.
    ///
    /// * line     The line index (zero based).
    ///
    /// * return     The baseline.
    ///
    pub fn get_baseline(&self, line: usize) -> f64 {
        unsafe { sys::ImpellerLineMetricsGetBaseline(self.0, line) }
    }
    //------------------------------------------------------------------------------
    /// Used to determine if this line ends with an explicit line break
    /// (e.g. '\n') or is the end of the paragraph.
    ///
    /// * line     The line index (zero based).
    ///
    /// * return     True if the line is a hard break.
    ///
    pub fn is_hardbreak(&self, line: usize) -> bool {
        unsafe { sys::ImpellerLineMetricsIsHardbreak(self.0, line) }
    }

    //------------------------------------------------------------------------------
    /// Width of the line from the left edge of the leftmost glyph to
    /// the right edge of the rightmost glyph.
    ///
    /// * line     The line index (zero based).
    ///
    /// * return     The width.
    ///
    pub fn get_width(&self, line: usize) -> f64 {
        unsafe { sys::ImpellerLineMetricsGetWidth(self.0, line) }
    }
    //------------------------------------------------------------------------------
    /// Total height of the line from the top edge to the bottom edge.
    ///
    /// * line     The line index (zero based).
    ///
    /// * return     The height.
    ///
    pub fn get_height(&self, line: usize) -> f64 {
        unsafe { sys::ImpellerLineMetricsGetHeight(self.0, line) }
    }
    //------------------------------------------------------------------------------
    /// @brief      The x coordinate of left edge of the line.
    ///
    /// * line     The line index (zero based).
    ///
    /// * return     The left edge coordinate.
    ///
    pub fn get_left(&self, line: usize) -> f64 {
        unsafe { sys::ImpellerLineMetricsGetLeft(self.0, line) }
    }
    //------------------------------------------------------------------------------
    /// Fetch the start index in the buffer of UTF-16 code units used to
    /// represent the paragraph line.
    ///
    /// * line     The line index (zero based).
    ///
    /// * return     The UTF-16 code units start index.
    ///
    pub fn get_code_unit_start_index_utf16(&self, line: usize) -> usize {
        unsafe { sys::ImpellerLineMetricsGetCodeUnitStartIndex(self.0, line) }
    }

    //------------------------------------------------------------------------------
    /// Fetch the end index in the buffer of UTF-16 code units used to
    /// represent the paragraph line.
    ///
    /// * line     The line index (zero based).
    ///
    /// * return     The UTF-16 code units end index.
    ///
    pub fn get_code_unit_end_index_utf16(&self, line: usize) -> usize {
        unsafe { sys::ImpellerLineMetricsGetCodeUnitEndIndex(self.0, line) }
    }
    //------------------------------------------------------------------------------
    /// Fetch the end index (excluding whitespace) in the buffer of
    /// UTF-16 code units used to represent the paragraph line.
    ///
    /// * line     The line index (zero based).
    ///
    /// * return     The UTF-16 code units end index excluding whitespace.
    ///
    pub fn get_code_unit_end_index_excluding_whitespace_utf16(&self, line: usize) -> usize {
        unsafe { sys::ImpellerLineMetricsGetCodeUnitEndIndexExcludingWhitespace(self.0, line) }
    }
    //------------------------------------------------------------------------------
    /// Fetch the end index (including newlines) in the buffer of
    /// UTF-16 code units used to represent the paragraph line.
    ///
    /// * line     The line index (zero based).
    ///
    /// * return     The UTF-16 code units end index including newlines.
    ///
    pub fn get_code_unit_end_index_including_newline_utf16(&self, line: usize) -> usize {
        unsafe { sys::ImpellerLineMetricsGetCodeUnitEndIndexIncludingNewline(self.0, line) }
    }
}
#[derive(Debug)]
#[repr(transparent)]
pub struct GlyphInfo(sys::ImpellerGlyphInfo);
impl Clone for GlyphInfo {
    fn clone(&self) -> Self {
        unsafe { sys::ImpellerGlyphInfoRetain(self.0) };
        GlyphInfo(self.0)
    }
}
unsafe impl Send for GlyphInfo {}
unsafe impl Sync for GlyphInfo {}
impl Drop for GlyphInfo {
    fn drop(&mut self) {
        unsafe { sys::ImpellerGlyphInfoRelease(self.0) };
    }
}
impl GlyphInfo {
    /// Fetch the start index in the buffer of UTF-16 code units used to
    /// represent the grapheme cluster for a glyph.
    ///
    /// * return     The UTF-16 code units start index.
    pub fn get_grapheme_cluster_code_unit_range_begin_utf16(&self) -> usize {
        unsafe { sys::ImpellerGlyphInfoGetGraphemeClusterCodeUnitRangeBegin(self.0) }
    }
    /// Fetch the end index in the buffer of UTF-16 code units used to
    /// represent the grapheme cluster for a glyph.
    ///
    /// * return     The UTF-16 code units end index.
    pub fn get_grapheme_cluster_code_unit_range_end_utf16(&self) -> usize {
        unsafe { sys::ImpellerGlyphInfoGetGraphemeClusterCodeUnitRangeEnd(self.0) }
    }
    /// Fetch the bounds of the grapheme cluster for the glyph in the
    /// coordinate space of the paragraph.
    ///
    /// * return     The grapheme cluster bounds.
    pub fn get_grapheme_cluster_bounds(&self) -> Rect {
        let mut rect = crate::sys::ImpellerRect::default();
        unsafe { sys::ImpellerGlyphInfoGetGraphemeClusterBounds(self.0, &raw mut rect) };
        cast(rect)
    }
    /// * return True if the glyph represents an ellipsis. False otherwise.
    pub fn is_ellipsis(&self) -> bool {
        unsafe { sys::ImpellerGlyphInfoIsEllipsis(self.0) }
    }
    /// * return The direction of the run that contains the glyph.
    pub fn get_text_direction(&self) -> TextDirection {
        unsafe { sys::ImpellerGlyphInfoGetTextDirection(self.0) }
    }
}
/// Paragraph builders allow for the creation of fully laid out paragraphs
/// (which themselves are immutable).
///
/// This is not thread-safe, as TypoGraphy context is not thread-safe. But
/// [Paragraph] is thread-safe.
///
/// <https://api.flutter.dev/flutter/dart-ui/ParagraphBuilder-class.html>
///
/// To build a paragraph, users push/pop paragraph styles onto a stack then add
/// UTF-8 encoded text. The properties on the top of paragraph style stack when
/// the text is added are used to layout and shape that subset of the paragraph.
///
/// @see      [ParagraphStyle]
///
/// ```
/// # use impellers::{TypographyContext, ParagraphStyle, ParagraphBuilder};
/// // this contains the fonts from user's system (or you can add custom fonts)
/// let fonts = TypographyContext::default();
/// // style decides the appearance of the text
/// let mut style = ParagraphStyle::default();
/// style.set_font_family("Arial");
/// style.set_font_size(12.0);
/// let mut builder = ParagraphBuilder::new(&fonts).expect("failed to create para builder");
/// builder.push_style(&style); // DON'T forget to set the style before adding text
/// builder.add_text("Hello, world!\n");
/// style.set_font_size(24.0);
/// builder.push_style(&style);
/// builder.add_text("Big World!\n"); // 24.0 font size
/// builder.pop_style(); // the 24.0 style is popped off. the previous 12.0 style is used
/// builder.add_text("Small World!\n"); // 12.0 font size
/// let paragraph = builder.build(100.0).expect("building paragraph failed");
/// ```
#[derive(Debug)]
#[repr(transparent)]
pub struct ParagraphBuilder(sys::ImpellerParagraphBuilder);

// impl Clone for ParagraphBuilder {
//     fn clone(&self) -> Self {
//         unsafe {
//             sys::ImpellerParagraphBuilderRetain(self.0);
//         }
//         Self(self.0)
//     }
// }
impl Drop for ParagraphBuilder {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerParagraphBuilderRelease(self.0);
        }
    }
}

impl ParagraphBuilder {
    //------------------------------------------------------------------------------
    /// Create a new paragraph builder.
    ///
    /// @return     The paragraph builder.
    #[doc(alias = "ImpellerParagraphBuilderNew")]
    pub fn new(context: &TypographyContext) -> Option<ParagraphBuilder> {
        let result = unsafe { sys::ImpellerParagraphBuilderNew(context.0) };
        (!result.is_null()).then_some(ParagraphBuilder(result))
    }
    //------------------------------------------------------------------------------
    /// Push a new paragraph style onto the paragraph style stack
    /// managed by the paragraph builder.
    ///
    /// Not all paragraph styles can be combined. For instance, it does
    /// not make sense to mix text alignment for different text runs
    /// within a paragraph. In such cases, the preference of the the
    /// first paragraph style on the style stack will take hold.
    ///
    /// If text is pushed onto the paragraph builder without a style
    /// previously pushed onto the stack, a default paragraph text style
    /// will be used. This may not always be desirable because some
    /// style element cannot be overridden. It is recommended that a
    /// default paragraph style always be pushed onto the stack before
    /// the addition of any text.
    ///
    /// - style              The style.
    #[doc(alias = "ImpellerParagraphBuilderPushStyle")]
    pub fn push_style(&mut self, style: &ParagraphStyle) {
        unsafe {
            sys::ImpellerParagraphBuilderPushStyle(self.0, style.0);
        }
    }
    //------------------------------------------------------------------------------
    /// Pop a previously pushed paragraph style from the paragraph style
    /// stack.
    ///
    #[doc(alias = "ImpellerParagraphBuilderPopStyle")]
    pub fn pop_style(&mut self) {
        unsafe {
            sys::ImpellerParagraphBuilderPopStyle(self.0);
        }
    }
    //------------------------------------------------------------------------------
    /// Add UTF-8 encoded text to the paragraph. The text will be styled
    /// according to the paragraph style already on top of the paragraph
    /// style stack.
    #[doc(alias = "ImpellerParagraphBuilderAddText")]
    pub fn add_text(&mut self, text: &str) {
        unsafe {
            sys::ImpellerParagraphBuilderAddText(self.0, text.as_ptr(), text.len() as u32);
        }
    }

    //------------------------------------------------------------------------------
    /// Layout and build a new paragraph using the specified width. The
    /// resulting paragraph is immutable. The paragraph builder must be
    /// discarded and a new one created to build more paragraphs.
    ///
    /// - width              The paragraph width.
    ///
    /// @return     The paragraph if one can be created, NULL otherwise.
    #[must_use]
    #[doc(alias = "ImpellerParagraphBuilderBuildParagraphNew")]
    pub fn build(&mut self, width: f32) -> Option<Paragraph> {
        let result = unsafe { sys::ImpellerParagraphBuilderBuildParagraphNew(self.0, width) };
        (!result.is_null()).then_some(Paragraph(result))
    }
}

/// Specified when building a paragraph, paragraph styles are managed in a stack
/// with specify text properties to apply to text that is added to the paragraph
/// builder.
///
/// The below link should be considered a full reference of what's possible with text.
///
/// <https://api.flutter.dev/flutter/painting/TextStyle-class.html>
///
/// @see [Paragraph] and [ParagraphBuilder]
///
#[derive(Debug)]
#[repr(transparent)]
pub struct ParagraphStyle(sys::ImpellerParagraphStyle);
unsafe impl Send for ParagraphStyle {}
unsafe impl Sync for ParagraphStyle {}
// impl Clone for ParagraphStyle {
//     fn clone(&self) -> Self {
//         unsafe {
//             sys::ImpellerParagraphStyleRetain(self.0);
//         }
//         Self(self.0)
//     }
// }
impl Drop for ParagraphStyle {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerParagraphStyleRelease(self.0);
        }
    }
}
impl Default for ParagraphStyle {
    fn default() -> Self {
        let result = unsafe { sys::ImpellerParagraphStyleNew() };
        assert!(!result.is_null());
        Self(result)
    }
}
impl ParagraphStyle {
    /// Set the paint used to render the text glyph contents.
    ///
    /// - paint            The paint.
    #[doc(alias = "ImpellerParagraphStyleSetForeground")]
    pub fn set_foreground(&mut self, paint: &Paint) {
        unsafe {
            sys::ImpellerParagraphStyleSetForeground(self.0, paint.0);
        }
    }
    //------------------------------------------------------------------------------
    /// Set the paint used to render the background of the text glyphs.
    ///
    /// - paint            The paint.
    #[doc(alias = "ImpellerParagraphStyleSetBackground")]
    pub fn set_background(&mut self, paint: &Paint) {
        unsafe {
            sys::ImpellerParagraphStyleSetBackground(self.0, paint.0);
        }
    }
    /// Set the weight of the font to select when rendering glyphs.
    ///
    /// - weight           The weight.
    #[doc(alias = "ImpellerParagraphStyleSetFontWeight")]
    pub fn set_font_weight(&mut self, weight: FontWeight) {
        unsafe {
            sys::ImpellerParagraphStyleSetFontWeight(self.0, weight);
        }
    }
    /// Set whether the glyphs should be bolded or italicized.
    ///
    /// - style            The style.
    #[doc(alias = "ImpellerParagraphStyleSetFontStyle")]
    pub fn set_font_style(&mut self, style: FontStyle) {
        unsafe {
            sys::ImpellerParagraphStyleSetFontStyle(self.0, style);
        }
    }
    /// Set the font family.
    ///
    /// <https://api.flutter.dev/flutter/painting/TextStyle/fontFamily.html>
    ///
    /// - family_name      The family name.
    #[doc(alias = "ImpellerParagraphStyleSetFontFamily")]
    pub fn set_font_family(&mut self, family_name: &str) {
        let family_name =
            std::ffi::CString::new(family_name).expect("failed to create Cstring from family name");
        unsafe {
            sys::ImpellerParagraphStyleSetFontFamily(self.0, family_name.as_ptr());
        }
        std::mem::drop(family_name);
    }
    /// Set the font size.
    ///
    /// <https://api.flutter.dev/flutter/painting/TextStyle/fontSize.html>
    ///
    /// - size             The size.
    #[doc(alias = "ImpellerParagraphStyleSetFontSize")]
    pub fn set_font_size(&mut self, size: f32) {
        unsafe {
            sys::ImpellerParagraphStyleSetFontSize(self.0, size);
        }
    }
    /// The height of the text as a multiple of text size.
    ///
    /// <https://api.flutter.dev/flutter/painting/TextStyle/height.html>
    ///
    /// When height is 0.0, the line height will be determined by the
    /// font's metrics directly, which may differ from the font size.
    /// Otherwise the line height of the text will be a multiple of font
    /// size, and be exactly fontSize * height logical pixels tall.
    ///
    /// - height           The height.
    #[doc(alias = "ImpellerParagraphStyleSetHeight")]
    pub fn set_height(&mut self, height: f32) {
        unsafe {
            sys::ImpellerParagraphStyleSetHeight(self.0, height);
        }
    }
    //------------------------------------------------------------------------------
    /// Set the alignment of text within the paragraph.
    ///
    /// - align            The align.
    #[doc(alias = "ImpellerParagraphStyleSetTextAlignment")]
    pub fn set_text_alignment(&mut self, align: TextAlignment) {
        unsafe {
            sys::ImpellerParagraphStyleSetTextAlignment(self.0, align);
        }
    }
    //------------------------------------------------------------------------------
    /// Set the directionality of the text within the paragraph.
    ///
    /// - direction        The direction.
    #[doc(alias = "ImpellerParagraphStyleSetTextDirection")]
    pub fn set_text_direction(&mut self, direction: TextDirection) {
        unsafe {
            sys::ImpellerParagraphStyleSetTextDirection(self.0, direction);
        }
    }
    //------------------------------------------------------------------------------
    /// Set the maximum line count within the paragraph.
    ///
    /// - max_lines        The maximum lines.
    #[doc(alias = "ImpellerParagraphStyleSetMaxLines")]
    pub fn set_max_lines(&mut self, max_lines: u32) {
        unsafe {
            sys::ImpellerParagraphStyleSetMaxLines(self.0, max_lines);
        }
    }
    //------------------------------------------------------------------------------
    /// Set the paragraph locale.
    ///
    /// <https://api.flutter.dev/flutter/painting/TextStyle/locale.html>
    ///
    /// - locale           The locale.
    #[doc(alias = "ImpellerParagraphStyleSetLocale")]
    pub fn set_locale(&mut self, locale: &str) {
        let locale = std::ffi::CString::new(locale).expect("failed to create Cstring from locale");
        unsafe {
            sys::ImpellerParagraphStyleSetLocale(self.0, locale.as_ptr());
        }
        std::mem::drop(locale);
    }
    pub fn set_ellipsis(&mut self, ellipsis: &str) {
        let ellipsis =
            std::ffi::CString::new(ellipsis).expect("failed to create cstr from ellipsis str");
        unsafe {
            sys::ImpellerParagraphStyleSetEllipsis(self.0, ellipsis.as_ptr());
        }
        std::mem::drop(ellipsis);
    }
}
/// Represents a two-dimensional path that is immutable and graphics context
/// agnostic.
///
/// <https://api.flutter.dev/flutter/dart-ui/Path-class.html>
///
/// <https://learn.microsoft.com/en-us/previous-versions/xamarin/xamarin-forms/user-interface/graphics/skiasharp/paths/paths>
///
/// <https://shopify.github.io/react-native-skia/docs/shapes/path>
///
/// Paths in Impeller consist of linear, cubic Bézier curve, and quadratic
/// Bézier curve segments. All other shapes are approximations using these
/// building blocks.
///
/// Paths are created using path builder that allow for the configuration of the
/// path segments, how they are filled, and/or stroked.
#[derive(Debug)]
#[repr(transparent)]
pub struct Path(sys::ImpellerPath);
unsafe impl Send for Path {}
unsafe impl Sync for Path {}
impl Clone for Path {
    fn clone(&self) -> Self {
        unsafe {
            sys::ImpellerPathRetain(self.0);
        }
        Self(self.0)
    }
}
impl Drop for Path {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerPathRelease(self.0);
        }
    }
}
impl Path {
    /// Get the bounds of the path.
    ///
    /// The bounds are conservative.
    /// That is, they may be larger than the actual shape of the path and could include the control points and isolated calls to move the cursor.
    pub fn get_bounds(&self) -> Rect {
        let mut rect = sys::ImpellerRect::default();
        unsafe {
            sys::ImpellerPathGetBounds(self.0, &raw mut rect);
        }
        cast(rect)
    }
}
/// Path builders allow for the incremental building up of paths.
///
/// @see docs of [Path]
#[derive(Debug)]
#[repr(transparent)]
pub struct PathBuilder(sys::ImpellerPathBuilder);

// impl Clone for PathBuilder {
//     fn clone(&self) -> Self {
//         unsafe {
//             sys::ImpellerPathBuilderRetain(self.0);
//         }
//         Self(self.0)
//     }
// }
unsafe impl Send for PathBuilder {}
unsafe impl Sync for PathBuilder {}
impl Drop for PathBuilder {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerPathBuilderRelease(self.0);
        }
    }
}
impl Default for PathBuilder {
    /// Create a new path builder. Paths themselves are immutable.
    /// A builder builds these immutable paths.
    fn default() -> Self {
        let p = unsafe { sys::ImpellerPathBuilderNew() };
        assert!(!p.is_null());
        Self(p)
    }
}
impl PathBuilder {
    /// Move the cursor to the specified location.
    ///
    /// -  location  The location.
    pub fn move_to(&mut self, location: Point) {
        unsafe {
            sys::ImpellerPathBuilderMoveTo(self.0, cast_ref(&location));
        }
    }
    /// Add a line segment from the current cursor location to the given
    /// location. The cursor location is updated to be at the endpoint.
    ///
    /// - location  The location.
    pub fn line_to(&mut self, location: Point) {
        unsafe {
            sys::ImpellerPathBuilderLineTo(self.0, cast_ref(&location));
        }
    }

    /// Add a quadratic curve from whose start point is the cursor to
    /// the specified end point using the a single control point.
    ///
    /// The new location of the cursor after this call is the end point.
    ///
    /// - control_point  The control point.
    /// - end_point      The end point.
    pub fn quadratic_curve_to(&mut self, control_point: Point, end_point: Point) {
        unsafe {
            sys::ImpellerPathBuilderQuadraticCurveTo(
                self.0,
                cast_ref(&control_point),
                cast_ref(&end_point),
            );
        }
    }
    /// Add a cubic curve whose start point is current cursor location
    /// to the specified end point using the two specified control
    /// points.
    ///
    /// The new location of the cursor after this call is the end point
    /// supplied.
    ///
    /// - control_point_1  The control point 1
    /// - control_point_2  The control point 2
    /// - end_point        The end point
    pub fn cubic_curve_to(
        &mut self,
        control_point_1: Point,
        control_point_2: Point,
        end_point: Point,
    ) {
        unsafe {
            sys::ImpellerPathBuilderCubicCurveTo(
                self.0,
                cast_ref(&control_point_1),
                cast_ref(&control_point_2),
                cast_ref(&end_point),
            );
        }
    }
    /// Adds a rectangle to the path.
    ///
    /// - rect     The rectangle.
    pub fn add_rect(&mut self, rect: &Rect) {
        unsafe {
            sys::ImpellerPathBuilderAddRect(self.0, cast_ref(rect));
        }
    }
    /// Add an arc to the path.
    ///
    /// - oval_bounds          The oval bounds.
    /// - start_angle_degrees  The start angle in degrees.
    /// - end_angle_degrees    The end angle in degrees.
    pub fn add_arc(
        &mut self,
        oval_bounds: &Rect,
        start_angle_degrees: f32,
        end_angle_degrees: f32,
    ) {
        unsafe {
            sys::ImpellerPathBuilderAddArc(
                self.0,
                cast_ref(oval_bounds),
                start_angle_degrees,
                end_angle_degrees,
            );
        }
    }

    /// Add an oval to the path.
    ///
    /// - oval_bounds  The oval bounds.
    pub fn add_oval(&mut self, oval_bounds: &Rect) {
        unsafe {
            sys::ImpellerPathBuilderAddOval(self.0, cast_ref(oval_bounds));
        }
    }
    /// Add a rounded rect with potentially non-uniform radii to the path.
    ///
    /// - oval_bounds     The oval bounds.
    /// - rounding_radii  The rounding radii.
    pub fn add_rounded_rect(&mut self, oval_bounds: &Rect, rounding_radii: &RoundingRadii) {
        unsafe {
            sys::ImpellerPathBuilderAddRoundedRect(self.0, cast_ref(oval_bounds), rounding_radii);
        }
    }
    /// Close the path.
    pub fn close(&mut self) {
        unsafe {
            sys::ImpellerPathBuilderClose(self.0);
        }
    }

    /// Create a new path by copying the existing built-up path. The
    /// existing path can continue being added to.
    ///
    /// - fill  The fill.
    ///
    /// @return     The impeller path.
    pub fn copy_path_new(&mut self, fill: FillType) -> Path {
        let p = unsafe { sys::ImpellerPathBuilderCopyPathNew(self.0, fill) };
        assert!(!p.is_null());
        Path(p)
    }
    /// Create a new path using the existing built-up path. The existing
    /// path builder now contains an empty path.
    ///
    /// - fill  The fill.
    ///
    /// @return     The impeller path.
    pub fn take_path_new(&mut self, fill: FillType) -> Path {
        let p = unsafe { sys::ImpellerPathBuilderTakePathNew(self.0, fill) };
        assert!(!p.is_null());
        Path(p)
    }
}
/// A surface represents a render target for Impeller to direct the rendering
/// intent specified the form of display lists to.
///
/// Render targets are how Impeller API users perform Window System Integration
/// (WSI). Users wrap swapchain images as surfaces and draw display lists onto
/// these surfaces to present content.
///
/// Creating surfaces is typically platform and client-rendering-API specific.
///
/// This is an inherently "temporary" object. Just create one every frame and
/// destroy it after presenting.
#[derive(Debug)]
#[repr(transparent)]
pub struct Surface(sys::ImpellerSurface);

// impl Clone for Surface {
//     fn clone(&self) -> Self {
//         unsafe {
//             sys::ImpellerSurfaceRetain(self.0);
//         }
//         Self(self.0)
//     }
// }
impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerSurfaceRelease(self.0);
        }
    }
}

impl Surface {
    /// Draw a display list onto the surface. The same display list can
    /// be drawn multiple times to different surfaces. BUT, you cannot
    /// draw multiple display lists to the same surface.
    ///
    /// To be specific, each call to [Surface::draw_display_list] will clear
    /// the contents of the surface. So, any previous drawing will be
    /// lost.
    ///
    /// @warning    In the OpenGL backend, Impeller will not make an effort to
    ///             preserve the OpenGL state that is current in the context.
    ///             Embedders that perform additional OpenGL operations in the
    ///             context should expect the reset state after control transitions
    ///             back to them. Key state to watch out for would be the viewports,
    ///             stencil rects, test toggles, resource (texture, framebuffer,
    ///             buffer) bindings, etc...
    ///
    /// - display_list  The display list to draw onto the surface.
    ///
    /// @return     If the display list could be drawn onto the surface.
    pub fn draw_display_list(&mut self, display_list: &DisplayList) -> Result<(), &'static str> {
        unsafe { sys::ImpellerSurfaceDrawDisplayList(self.0, display_list.0) }
            .then_some(())
            .ok_or("failed to draw to surface")
    }
    /// Present the surface to the underlying window system.
    ///
    /// This is for platforms like Vulkan which acquire a a surface from [VkSwapChain].
    ///
    /// For OpenGL, use your windowing library's `SwapBuffers`-like function.
    ///
    /// @return     Ok if the surface could be presented.
    pub fn present(self) -> Result<(), &'static str> {
        unsafe { sys::ImpellerSurfacePresent(self.0) }
            .then_some(())
            .ok_or("failed to present surface")
    }
}
/// A reference to a texture whose data is resident on the GPU. These can be
/// referenced in draw calls and paints.
///
/// Creating textures is extremely expensive. Creating a single one can
/// typically comfortably blow the frame budget of an application. Textures
/// should be created on background threads.
///
///
/// @warning    While textures themselves are thread safe, some context types
///             (like OpenGL) may need extra configuration to be able to operate
///             from multiple threads.
#[derive(Debug)]
#[repr(transparent)]
pub struct Texture(sys::ImpellerTexture);
unsafe impl Sync for Texture {}
unsafe impl Send for Texture {}
impl Clone for Texture {
    fn clone(&self) -> Self {
        unsafe {
            sys::ImpellerTextureRetain(self.0);
        }
        Self(self.0)
    }
}
impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            sys::ImpellerTextureRelease(self.0);
        }
    }
}
impl Texture {
    /// Get the OpenGL handle associated with this texture. If this is
    /// not an OpenGL texture, this method will always return 0.
    ///
    /// OpenGL handles are lazily created, this method will return
    /// GL_NONE is no OpenGL handle is available. To ensure that this
    /// call eagerly creates an OpenGL texture, call this on a thread
    /// where Impeller knows there is an OpenGL context available.
    ///
    /// @return     The OpenGL handle if one is available, GL_NONE otherwise.
    ///
    /// # Safety
    /// READ the docs that it may return GL_NONE (which may not be zero).
    /// use opengl constants to compare the return value properly.
    ///
    pub fn get_opengl_handle(&self) -> u64 {
        unsafe { sys::ImpellerTextureGetOpenGLHandle(self.0) }
    }
}

/// based on the size, it will calculate a suitable mipcount.
/// This function skips 1x1 mip levels because that's what flutter does.
pub fn flutter_mip_count(width: f32, height: f32) -> u32 {
    // https://github.com/flutter/engine/blob/main/impeller/geometry/size.h#L134

    let mut result = width.log2().ceil().max(height.log2().ceil()) as u32;
    // This check avoids creating 1x1 mip levels, which are both pointless
    // and cause rendering problems on some Adreno GPUs.
    // See:
    //      * https://github.com/flutter/flutter/issues/160441
    //      * https://github.com/flutter/flutter/issues/159876
    //      * https://github.com/flutter/flutter/issues/160587
    if result > 1 {
        result -= 1;
    }
    std::cmp::max(result, 1)
}

impl Color {
    pub const TRANSPARENT: Self = Self::new_srgba(0.0, 0.0, 0.0, 0.0);
    pub const fn new_srgb(red: f32, green: f32, blue: f32) -> Self {
        Self::new_srgba(red, green, blue, 1.0)
    }
    pub const fn new_srgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
            color_space: ColorSpace::SRGB,
        }
    }
    pub const fn with_alpha(self, alpha: f32) -> Self {
        Self { alpha, ..self }
    }
}
bitflags::bitflags! {
    pub struct TextDecorationType: std::ffi::c_int {
        const NONE = sys::TextDecorationType::None as _;
        const UNDERLINE = sys::TextDecorationType::Underline as _;
        const OVERLINE = sys::TextDecorationType::Overline as _;
        const LINETHROUGH = sys::TextDecorationType::LineThrough as _;
    }
}
impl sys::ImpellerMapping {
    /// A helper function to create a mapping from a boxed slice.
    ///
    /// This wraps `Box<[u8]>` in a `Box<Box<[u8]>>`, and leaks it.
    /// The leak is dropped inside the on_release callback. The userdata pointer returned
    /// can be used to access the original boxed slice.
    ///
    /// # Safety
    /// - The returned Self's on_release callback MUST be called with only the returned userdata pointer.
    /// - The allocator must be global, as we never know when or where the release callbacks can be called
    ///
    /// NOTE: We can probably simplify this using https://doc.rust-lang.org/std/boxed/struct.ThinBox.html on stabilization
    pub unsafe fn from_cow(contents: Cow<'static, [u8]>) -> (Self, *mut std::ffi::c_void) {
        let contents: Box<Cow<'static, [u8]>> = Box::new(contents);
        let data: *const u8 = contents.as_ptr();
        let length = contents.len() as u64;
        let user_data: *mut Cow<'static, [u8]> = Box::leak(contents);
        extern "C" fn boxed_cow_slice_dropper(on_release_user_data: *mut std::ffi::c_void) {
            let contents: Box<Cow<'static, [u8]>> =
                unsafe { Box::from_raw(on_release_user_data as *mut _) };
            drop(contents);
        }
        (
            sys::ImpellerMapping {
                data,
                length,
                on_release: Some(boxed_cow_slice_dropper),
            },
            user_data.cast(),
        )
    }
}
unsafe impl bytemuck::Zeroable for sys::ImpellerISize {}
unsafe impl bytemuck::Pod for sys::ImpellerISize {}
unsafe impl bytemuck::Zeroable for sys::ImpellerPoint {}
unsafe impl bytemuck::Pod for sys::ImpellerPoint {}
unsafe impl bytemuck::Zeroable for sys::ImpellerSize {}
unsafe impl bytemuck::Pod for sys::ImpellerSize {}
unsafe impl bytemuck::Zeroable for sys::ImpellerMatrix {}
unsafe impl bytemuck::Pod for sys::ImpellerMatrix {}
unsafe impl bytemuck::Zeroable for sys::ImpellerRect {}
unsafe impl bytemuck::Pod for sys::ImpellerRect {}
unsafe impl bytemuck::Zeroable for sys::ImpellerColor {}
unsafe impl bytemuck::Pod for sys::ImpellerColor {}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_version() {
        assert_eq!(
            ImpellerVersion::get_header_version().get_variant(),
            sys::IMPELLER_VERSION_VARIANT
        );
        assert_eq!(
            ImpellerVersion::get_linked_version().get_variant(),
            sys::IMPELLER_VERSION_VARIANT
        );
        assert_eq!(
            ImpellerVersion::get_header_version().get_major(),
            sys::IMPELLER_VERSION_MAJOR
        );
        assert_eq!(
            ImpellerVersion::get_linked_version().get_major(),
            sys::IMPELLER_VERSION_MAJOR
        );
        assert_eq!(
            ImpellerVersion::get_header_version().get_minor(),
            sys::IMPELLER_VERSION_MINOR
        );
        assert_eq!(
            ImpellerVersion::get_linked_version().get_minor(),
            sys::IMPELLER_VERSION_MINOR
        );
        assert_eq!(
            ImpellerVersion::get_header_version().get_patch(),
            sys::IMPELLER_VERSION_PATCH
        );
        assert_eq!(
            ImpellerVersion::get_linked_version().get_patch(),
            sys::IMPELLER_VERSION_PATCH
        );
    }
}
