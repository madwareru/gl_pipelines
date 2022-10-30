use std::num::NonZeroU32;
use glow::{HasContext, PixelPackData, PixelUnpackData};
use crate::{Context, GlowContext};

#[derive(Clone)]
pub struct Texture {
    glow_ctx: GlowContext,
    pub(crate) texture: Option<glow::Texture>,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub format: TextureFormat,
    pub kind: TextureKind
}

impl Texture {
    pub fn empty(ctx: &mut Context) -> Texture {
        Texture {
            glow_ctx: ctx.glow_ctx.clone(),
            texture: None,
            width: 0,
            height: 0,
            depth: 1,
            format: TextureFormat::RGBA8,
            kind: TextureKind::Texture2D
        }
    }

    pub fn gl_internal_id(&self) -> u32 {
        match self.texture {
            None => 0,
            Some(tex) => {
                unsafe { std::mem::transmute(tex) }
            }
        }
    }

    pub unsafe fn from_raw_id(ctx: &mut Context, texture: u32) -> Self {
        Self {
            glow_ctx: ctx.glow_ctx.clone(),
            texture: unsafe { std::mem::transmute(NonZeroU32::new(texture)) },
            width: 0,
            height: 0,
            depth: 1,
            format: TextureFormat::RGBA8, // assumed for now
            kind: TextureKind::Texture2D // assumed for now
        }
    }

    pub fn delete(&self) {
        unsafe {
            match self.texture {
                None => {}
                Some(tex) => {
                    self.glow_ctx.0.gl.delete_texture(tex);
                }
            }
        }
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureKind {
    Texture2D,
    Texture3D,
    Texture2DArray,
}

/// List of all the possible formats of input data when uploading to texture.
/// The list is built by intersection of texture formats supported by 3.3 core profile and webgl1.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureFormat {
    RGB8,
    RGBA8,
    Depth,
    Alpha,
}

/// Converts from TextureFormat to (internal_format, format, pixel_type)
impl From<TextureFormat> for (u32, u32, u32) {
    fn from(format: TextureFormat) -> Self {
        match format {
            TextureFormat::RGB8 => (glow::RGB, glow::RGB, glow::UNSIGNED_BYTE),
            TextureFormat::RGBA8 => (glow::RGBA, glow::RGBA, glow::UNSIGNED_BYTE),
            TextureFormat::Depth => (glow::DEPTH_COMPONENT, glow::DEPTH_COMPONENT, glow::UNSIGNED_SHORT),
            TextureFormat::Alpha => (glow::R8, glow::RED, glow::UNSIGNED_BYTE), // texture updates will swizzle Red -> Alpha
        }
    }
}

impl TextureFormat {
    /// Returns the size in bytes of texture with `dimensions`.
    pub fn size(self, width: u32, height: u32) -> u32 {
        let square = width * height;
        match self {
            TextureFormat::RGB8 => 3 * square,
            TextureFormat::RGBA8 => 4 * square,
            TextureFormat::Depth => 2 * square,
            TextureFormat::Alpha => 1 * square,
        }
    }

    pub fn size_3d(self, width: u32, height: u32, depth: u32) -> usize {
        let square = width as usize * height as usize * depth as usize;
        match self {
            TextureFormat::RGB8 => 3 * square,
            TextureFormat::RGBA8 => 4 * square,
            TextureFormat::Depth => 2 * square,
            TextureFormat::Alpha => 1 * square,
        }
    }
}

impl Default for TextureParams {
    fn default() -> Self {
        TextureParams {
            format: TextureFormat::RGBA8,
            wrap: TextureWrap::Clamp,
            filter: FilterMode::Linear,
            width: 0,
            height: 0,
            depth: 1
        }
    }
}

/// Sets the wrap parameter for texture.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureWrap {
    /// Samples at coord x + 1 map to coord x.
    Repeat = glow::REPEAT as isize,
    /// Samples at coord x + 1 map to coord 1 - x.
    Mirror = glow::MIRRORED_REPEAT as isize,
    /// Samples at coord x + 1 map to coord 1.
    Clamp = glow::CLAMP_TO_EDGE as isize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterMode {
    Linear = glow::LINEAR as isize,
    Nearest = glow::NEAREST as isize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureAccess {
    /// Used as read-only from GPU
    Static,
    /// Can be written to from GPU
    RenderTarget,
}

#[derive(Debug, Copy, Clone)]
pub struct TextureParams {
    pub format: TextureFormat,
    pub wrap: TextureWrap,
    pub filter: FilterMode,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl Texture {
    /// Shorthand for `new(ctx, TextureAccess::RenderTarget, params)`
    pub fn new_render_texture(ctx: &mut Context, params: TextureParams) -> Texture {
        Self::new(ctx, TextureAccess::RenderTarget, None, params, TextureKind::Texture2D)
    }

    pub fn new(
        ctx: &mut Context,
        _access: TextureAccess,
        bytes: Option<&[u8]>,
        params: TextureParams,
        kind: TextureKind,
    ) -> Texture {
        if let Some(bytes_data) = bytes {
            assert_eq!(
                params.format.size(params.width, params.height) as usize,
                bytes_data.len()
            );
        }

        let (internal_format, format, pixel_type) = params.format.into();

        ctx.cache.store_texture_binding(0);

        let gl = &ctx.glow_ctx.0.gl;

        let texture;
        unsafe {
            texture = gl.create_texture().unwrap();

            ctx.cache.bind_texture(0, Some(texture));
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);

            if params.format == TextureFormat::Alpha {
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_SWIZZLE_A,
                    glow::RED as _
                );
            } else {
                // keep alpha -> alpha
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_SWIZZLE_A,
                    glow::ALPHA as _
                );
            }

            match kind {
                TextureKind::Texture2D => {
                    gl.tex_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        internal_format as i32,
                        params.width as i32,
                        params.height as i32,
                        0,
                        format,
                        pixel_type,
                        bytes
                    );

                    gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_WRAP_S,
                        params.wrap as i32
                    );
                    gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_WRAP_T,
                        params.wrap as i32
                    );
                    gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_MIN_FILTER,
                        params.filter as i32
                    );
                    gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_MAG_FILTER,
                        params.filter as i32
                    );
                },
                TextureKind::Texture3D => {
                    gl.tex_image_3d(
                        glow::TEXTURE_3D,
                        0,
                        internal_format as i32,
                        params.width as i32,
                        params.height as i32,
                        params.depth as i32,
                        0,
                        format,
                        pixel_type,
                        bytes,
                    );

                    gl.tex_parameter_i32(
                        glow::TEXTURE_3D,
                        glow::TEXTURE_WRAP_S,
                        params.wrap as i32
                    );
                    gl.tex_parameter_i32(
                        glow::TEXTURE_3D,
                        glow::TEXTURE_WRAP_T,
                        params.wrap as i32
                    );
                    gl.tex_parameter_i32(
                        glow::TEXTURE_3D,
                        glow::TEXTURE_WRAP_R,
                        params.wrap as i32
                    );
                    gl.tex_parameter_i32(
                        glow::TEXTURE_3D,
                        glow::TEXTURE_MIN_FILTER,
                        params.filter as i32
                    );
                    gl.tex_parameter_i32(
                        glow::TEXTURE_3D,
                        glow::TEXTURE_MAG_FILTER,
                        params.filter as i32
                    );
                },
                TextureKind::Texture2DArray => {
                    gl.tex_image_3d(
                        glow::TEXTURE_2D_ARRAY,
                        0,
                        internal_format as i32,
                        params.width as i32,
                        params.height as i32,
                        params.depth as i32,
                        0,
                        format,
                        pixel_type,
                        bytes
                    );

                    gl.tex_parameter_i32(
                        glow::TEXTURE_3D,
                        glow::TEXTURE_WRAP_S,
                        params.wrap as i32
                    );
                    gl.tex_parameter_i32(
                        glow::TEXTURE_3D,
                        glow::TEXTURE_WRAP_T,
                        params.wrap as i32
                    );
                    gl.tex_parameter_i32(
                        glow::TEXTURE_3D,
                        glow::TEXTURE_WRAP_R,
                        params.wrap as i32
                    );
                    gl.tex_parameter_i32(
                        glow::TEXTURE_3D,
                        glow::TEXTURE_MIN_FILTER,
                        params.filter as i32
                    );
                    gl.tex_parameter_i32(
                        glow::TEXTURE_3D,
                        glow::TEXTURE_MAG_FILTER,
                        params.filter as i32
                    );
                }
            }
        }
        ctx.cache.restore_texture_binding(0);

        Texture {
            glow_ctx: ctx.glow_ctx.clone(),
            texture: Some(texture),
            width: params.width,
            height: params.height,
            depth: params.depth,
            format: params.format,
            kind
        }
    }

    /// Upload texture to GPU with given TextureParams
    pub fn from_data_and_format(ctx: &mut Context, bytes: &[u8], params: TextureParams, kind: TextureKind) -> Texture {
        Self::new(ctx, TextureAccess::Static, Some(bytes), params, kind)
    }

    /// Upload RGBA8 texture to GPU
    pub fn from_rgba8(ctx: &mut Context, width: u16, height: u16, depth: u16, bytes: &[u8], kind: TextureKind) -> Texture {
        assert_eq!(width as usize * height as usize * 4, bytes.len());

        Self::from_data_and_format(
            ctx,
            bytes,
            TextureParams {
                width: width as _,
                height: height as _,
                depth: depth as _,
                format: TextureFormat::RGBA8,
                wrap: TextureWrap::Clamp,
                filter: FilterMode::Linear,
            },
            kind
        )
    }

    pub fn set_filter(&self, ctx: &mut Context, filter: FilterMode) {
        ctx.cache.store_texture_binding(0);
        ctx.cache.bind_texture(0, self.texture);
        unsafe {
            let target = match self.kind {
                TextureKind::Texture2D => glow::TEXTURE_2D,
                TextureKind::Texture3D => glow::TEXTURE_3D,
                TextureKind::Texture2DArray => glow::TEXTURE_2D_ARRAY
            };
            ctx.glow_ctx.0.gl.tex_parameter_i32(
                target,
                glow::TEXTURE_MIN_FILTER,
                filter as i32
            );
            ctx.glow_ctx.0.gl.tex_parameter_i32(
                target,
                glow::TEXTURE_MAG_FILTER,
                filter as i32
            );
        }
        ctx.cache.restore_texture_binding(0);
    }

    pub fn resize(&mut self, ctx: &mut Context, width: u32, height: u32, bytes: Option<&[u8]>) {
        ctx.cache.store_texture_binding(0);

        let (internal_format, format, pixel_type) = self.format.into();

        self.width = width;
        self.height = height;

        unsafe {
            ctx.glow_ctx.0.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                internal_format as i32,
                self.width as i32,
                self.height as i32,
                0,
                format,
                pixel_type,
                bytes,
            );
        }

        ctx.cache.restore_texture_binding(0);
    }

    /// Update whole texture content
    /// bytes should be width * height * 4 size - non rgba8 textures are not supported yet anyway
    pub fn update(&self, ctx: &mut Context, bytes: &[u8]) {
        assert_eq!(self.size(self.width, self.height, self.depth), bytes.len());

        self.update_texture_part(
            ctx,
            0 as _,
            0 as _,
            0 as _,
            self.width as _,
            self.height as _,
            self.depth as _,
            bytes,
        )
    }

    pub fn update_texture_part(
        &self,
        ctx: &mut Context,
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        width: i32,
        height: i32,
        depth: i32,
        bytes: &[u8],
    ) {
        assert_eq!(self.size(width as _, height as _, depth as _), bytes.len());
        assert!(x_offset + width <= self.width as _);
        assert!(y_offset + height <= self.height as _);

        ctx.cache.store_texture_binding(0);
        ctx.cache.bind_texture(0, self.texture);

        let gl = &ctx.glow_ctx.0.gl;

        let (_, format, pixel_type) = self.format.into();

        unsafe {
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);

            match self.kind {
                TextureKind::Texture2D => {
                    if self.format == TextureFormat::Alpha {
                        // if alpha miniquad texture, the value is stored in red channel
                        // swizzle red -> alpha
                        gl.tex_parameter_i32(
                            glow::TEXTURE_2D,
                            glow::TEXTURE_SWIZZLE_A,
                            glow::RED as _
                        );
                    } else {
                        // keep alpha -> alpha
                        gl.tex_parameter_i32(
                            glow::TEXTURE_2D,
                            glow::TEXTURE_SWIZZLE_A,
                            glow::ALPHA as _
                        );
                    }

                    gl.tex_sub_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        x_offset as _,
                        y_offset as _,
                        width as _,
                        height as _,
                        format,
                        pixel_type,
                        PixelUnpackData::Slice(bytes)
                    );
                }
                TextureKind::Texture3D => {
                    if self.format == TextureFormat::Alpha {
                        // if alpha miniquad texture, the value is stored in red channel
                        // swizzle red -> alpha
                        gl.tex_parameter_i32(
                            glow::TEXTURE_3D,
                            glow::TEXTURE_SWIZZLE_A,
                            glow::RED as _
                        );
                    } else {
                        // keep alpha -> alpha
                        gl.tex_parameter_i32(
                            glow::TEXTURE_3D,
                            glow::TEXTURE_SWIZZLE_A,
                            glow::ALPHA as _
                        );
                    }

                    gl.tex_sub_image_3d(
                        glow::TEXTURE_3D,
                        0,
                        x_offset as _,
                        y_offset as _,
                        z_offset as _,
                        width as _,
                        height as _,
                        depth as _,
                        format,
                        pixel_type,
                        PixelUnpackData::Slice(bytes),
                    );
                }
                TextureKind::Texture2DArray => {
                    if self.format == TextureFormat::Alpha {
                        // if alpha miniquad texture, the value is stored in red channel
                        // swizzle red -> alpha
                        gl.tex_parameter_i32(
                            glow::TEXTURE_3D,
                            glow::TEXTURE_SWIZZLE_A,
                            glow::RED as _
                        );
                    } else {
                        // keep alpha -> alpha
                        gl.tex_parameter_i32(
                            glow::TEXTURE_3D,
                            glow::TEXTURE_SWIZZLE_A,
                            glow::ALPHA as _
                        );
                    }

                    gl.tex_sub_image_3d(
                        glow::TEXTURE_2D_ARRAY,
                        0,
                        x_offset as _,
                        y_offset as _,
                        z_offset as _,
                        width as _,
                        height as _,
                        depth as _,
                        format,
                        pixel_type,
                        PixelUnpackData::Slice(bytes),
                    );
                }
            }
        }

        ctx.cache.restore_texture_binding(0);
    }

    /// Read texture data into CPU memory
    pub fn read_pixels(&self, bytes: &mut [u8]) {
        assert_eq!(self.kind, TextureKind::Texture2D);

        let (_, format, pixel_type) = self.format.into();

        let gl = &self.glow_ctx.0.gl;

        unsafe {
            let current_fb = {
                let fb = NonZeroU32::new(gl.get_parameter_i32(glow::FRAMEBUFFER_BINDING) as _);
                std::mem::transmute(fb)
            };

            let new_fb = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(new_fb));

            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                self.texture,
                0
            );

            gl.read_pixels(
                0,
                0,
                self.width as _,
                self.height as _,
                format,
                pixel_type,
                PixelPackData::Slice(bytes)
            );

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(current_fb));
            gl.delete_framebuffer(new_fb);
        }
    }

    #[inline]
    fn size(&self, width: u32, height: u32, depth: u32) -> usize {
        match self.kind {
            TextureKind::Texture2D => self.format.size(width, height) as usize,
            TextureKind::Texture3D => self.format.size_3d(width, height, depth) as usize,
            TextureKind::Texture2DArray => self.format.size_3d(width, height, depth) as usize
        }
    }
}