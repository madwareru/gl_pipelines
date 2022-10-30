
use std::num::NonZeroU32;
use glow::{HasContext};
use crate::cache_impl::GlCache;
use crate::glow_context::GlowContext;

mod glow_context;
mod texture;
mod shader_impl;
mod types_impl;
mod query_impl;
mod buffer_impl;
mod cache_impl;

pub mod window;
pub mod egui_integration;

pub use texture::{FilterMode, Texture, TextureAccess, TextureFormat, TextureParams, TextureWrap, TextureKind};
pub use shader_impl::{Shader, ShaderMeta, ShaderImage, ShaderUniform, ShaderType, ShaderError};
pub use types_impl::{
    UniformType, UniformDesc, UniformBlockLayout, VertexFormat, VertexStep, BufferLayout,
    VertexAttribute, PipelineLayout, BlendState, StencilState, StencilFaceState, StencilOp, CompareFunc,
    Equation, BlendValue, BlendFactor, CullFace, FrontFaceOrder, Comparison, PrimitiveType, IndexType,
    BufferType, Usage
};
pub use query_impl::*;
pub use buffer_impl::*;
pub use buffer_impl::Buffer;
use crate::shader_impl::ShaderInternal;

pub const MAX_VERTEX_ATTRIBUTES: usize = 16;
pub const MAX_SHADERSTAGE_IMAGES: usize = 12;

pub struct Context {
    window_size: (i32, i32),
    dpi: (f32, f32),
    shaders: Vec<ShaderInternal>,
    pipelines: Vec<PipelineInternal>,
    passes: Vec<RenderPassInternal>,
    default_framebuffer: glow::Framebuffer,
    cache: GlCache,
    glow_ctx: GlowContext
}

impl Context {
    pub fn new_from_sdl2(video: &sdl2::VideoSubsystem, default_w: i32, default_h: i32) -> Self {
        Self::new_impl(&GlowContext::new_from_sdl2_video(video), default_w, default_h)
    }

    fn new_impl(glow_ctx: &GlowContext, default_w: i32, default_h: i32) -> Self {
        let glow_ctx = glow_ctx.clone();
        let glow_ctx2 = glow_ctx.clone();

        let gl = &glow_ctx.0.gl;

        let default_framebuffer = unsafe {
            let fb = NonZeroU32::new(gl.get_parameter_i32(glow::FRAMEBUFFER_BINDING) as _);
            std::mem::transmute(fb)
        };

        unsafe {
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));
        }

        Context {
            window_size: (default_w, default_h),
            dpi: (1.0, 1.0),
            default_framebuffer,
            pipelines: Vec::new(),
            passes: Vec::new(),
            shaders: Vec::new(),
            glow_ctx,
            cache: GlCache {
                glow_ctx: glow_ctx2,
                stored_index_buffer: None,
                stored_index_type: None,
                stored_vertex_buffer: None,
                index_buffer: None,
                index_type: None,
                vertex_buffer: None,
                color_blend: None,
                alpha_blend: None,
                stencil: None,
                color_write: (true, true, true, true),
                cull_face: CullFace::Nothing,
                stored_texture: None,
                textures: [None; MAX_SHADERSTAGE_IMAGES],
                cur_pipeline: None,
                attributes: [(); MAX_VERTEX_ATTRIBUTES].map(|_| None)
            },
        }
    }

    pub fn update_window_size(&mut self, w: i32, h: i32) {
        self.window_size = (w, h);
    }

    pub fn get_window_size(&self) -> (i32, i32) {
        self.window_size
    }

    pub fn set_dpi_info(&mut self, horizontal_dpi: f32, vertical_dpi: f32) {
        self.dpi = (horizontal_dpi, vertical_dpi);
    }

    pub(crate) fn get_dpi(&self) -> (f32, f32) {
        self.dpi
    }

    pub fn apply_pipeline(&mut self, pipeline: &Pipeline) {
        self.cache.cur_pipeline = Some(*pipeline);
        let gl = &self.glow_ctx.0.gl;

        {
            let pipeline = &self.pipelines[pipeline.0];
            let shader = &mut self.shaders[pipeline.shader.0];
            unsafe {
                gl.use_program(Some(shader.program));
            }

            unsafe {
                gl.enable(glow::SCISSOR_TEST);
            }

            if pipeline.params.depth_write {
                unsafe {
                    gl.enable(glow::DEPTH_TEST);
                    gl.depth_func(pipeline.params.depth_test.into())
                }
            } else {
                unsafe {
                    gl.disable(glow::DEPTH_TEST);
                }
            }

            match pipeline.params.front_face_order {
                FrontFaceOrder::Clockwise => unsafe {
                    gl.front_face(glow::CW);
                },
                FrontFaceOrder::CounterClockwise => unsafe {
                    gl.front_face(glow::CCW);
                },
            }
        }

        self.set_cull_face(self.pipelines[pipeline.0].params.cull_face);
        self.set_blend(
            self.pipelines[pipeline.0].params.color_blend,
            self.pipelines[pipeline.0].params.alpha_blend,
        );

        self.set_stencil(self.pipelines[pipeline.0].params.stencil_test);
        self.set_color_write(self.pipelines[pipeline.0].params.color_write);
    }

    pub fn set_cull_face(&mut self, cull_face: CullFace) {
        if self.cache.cull_face == cull_face {
            return;
        }

        let gl = &self.glow_ctx.0.gl;

        match cull_face {
            CullFace::Nothing => unsafe {
                gl.disable(glow::CULL_FACE);
            },
            CullFace::Front => unsafe {
                gl.enable(glow::CULL_FACE);
                gl.cull_face(glow::FRONT);
            },
            CullFace::Back => unsafe {
                gl.enable(glow::CULL_FACE);
                gl.cull_face(glow::BACK);
            },
        }
        self.cache.cull_face = cull_face;
    }

    pub fn set_color_write(&mut self, color_write: ColorMask) {
        if self.cache.color_write == color_write {
            return;
        }
        let (r, g, b, a) = color_write;
        unsafe {
            self.glow_ctx.0.gl.color_mask(r, g, b, a);
        }
        self.cache.color_write = color_write;
    }

    pub fn set_blend(&mut self, color_blend: Option<BlendState>, alpha_blend: Option<BlendState>) {
        if color_blend.is_none() && alpha_blend.is_some() {
            panic!("AlphaBlend without ColorBlend");
        }
        if self.cache.color_blend == color_blend && self.cache.alpha_blend == alpha_blend {
            return;
        }

        let gl = &self.glow_ctx.0.gl;

        unsafe {
            if let Some(color_blend) = color_blend {
                if self.cache.color_blend.is_none() {
                    gl.enable(glow::BLEND);
                }

                let BlendState {
                    equation: eq_rgb,
                    sfactor: src_rgb,
                    dfactor: dst_rgb,
                } = color_blend;

                if let Some(BlendState {
                                equation: eq_alpha,
                                sfactor: src_alpha,
                                dfactor: dst_alpha,
                            }) = alpha_blend
                {
                    gl.blend_func_separate(
                        src_rgb.into(),
                        dst_rgb.into(),
                        src_alpha.into(),
                        dst_alpha.into(),
                    );
                    gl.blend_equation_separate(eq_rgb.into(), eq_alpha.into());
                } else {
                    gl.blend_func(src_rgb.into(), dst_rgb.into());
                    gl.blend_equation_separate(eq_rgb.into(), eq_rgb.into());
                }
            } else if self.cache.color_blend.is_some() {
                gl.disable(glow::BLEND);
            }
        }

        self.cache.color_blend = color_blend;
        self.cache.alpha_blend = alpha_blend;
    }

    pub fn set_stencil(&mut self, stencil_test: Option<StencilState>) {
        if self.cache.stencil == stencil_test {
            return;
        }

        let gl = &self.glow_ctx.0.gl;

        unsafe {
            if let Some(stencil) = stencil_test {
                if self.cache.stencil.is_none() {
                    gl.enable(glow::STENCIL_TEST);
                }

                let front = &stencil.front;
                gl.stencil_op_separate(
                    glow::FRONT,
                    front.fail_op.into(),
                    front.depth_fail_op.into(),
                    front.pass_op.into(),
                );
                gl.stencil_func_separate(
                    glow::FRONT,
                    front.test_func.into(),
                    front.test_ref,
                    front.test_mask,
                );
                gl.stencil_mask_separate(glow::FRONT, front.write_mask);

                let back = &stencil.back;
                gl.stencil_op_separate(
                    glow::BACK,
                    back.fail_op.into(),
                    back.depth_fail_op.into(),
                    back.pass_op.into(),
                );
                gl.stencil_func_separate(
                    glow::BACK,
                    back.test_func.into(),
                    back.test_ref.into(),
                    back.test_mask,
                );
                gl.stencil_mask_separate(glow::BACK, back.write_mask);
            } else if self.cache.stencil.is_some() {
                gl.disable(glow::STENCIL_TEST);
            }
        }

        self.cache.stencil = stencil_test;
    }

    pub fn apply_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) {
        unsafe {
            self.glow_ctx.0.gl.viewport(x, y, w, h);
        }
    }

    pub fn apply_scissor_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        unsafe {
            self.glow_ctx.0.gl.scissor(x, y, w, h);
        }
    }

    pub fn apply_bindings(&mut self, bindings: &Bindings) {
        let pip = &self.pipelines[self.cache.cur_pipeline.unwrap().0];
        let shader = &self.shaders[pip.shader.0];

        let gl = &self.glow_ctx.0.gl;

        for (n, shader_image) in shader.images.iter().enumerate() {
            let bindings_image = bindings
                .images
                .get(n)
                .unwrap_or_else(|| panic!("Image count in bindings and shader did not match!"));
            if let Some(gl_loc) = shader_image.gl_loc {
                unsafe {
                    self.cache.bind_texture(n, bindings_image.texture);
                    gl.uniform_1_i32(Some(&gl_loc), n as i32);
                }
            }
        }

        self.cache.bind_buffer(
            glow::ELEMENT_ARRAY_BUFFER,
            Some(bindings.index_buffer.gl_buf),
            bindings.index_buffer.index_type,
        );

        let pip = &self.pipelines[self.cache.cur_pipeline.unwrap().0];

        for attr_index in 0..MAX_VERTEX_ATTRIBUTES {
            let cached_attr = self.cache.attributes[attr_index];

            let pip_attribute = pip.layout.get(attr_index).copied();

            if let Some(Some(attribute)) = pip_attribute {
                let vb = bindings.vertex_buffers[attribute.buffer_index].clone();

                if cached_attr.map_or(true, |cached_attr| {
                    if attribute != cached_attr.attribute {
                        return true;
                    }
                    match cached_attr.gl_vbuf {
                        None => true,
                        Some(gl_vbuf) => gl_vbuf != vb.gl_buf
                    }
                }) {
                    self.cache.bind_buffer(
                        glow::ARRAY_BUFFER,
                        Some(vb.gl_buf),
                        vb.index_type
                    );

                    unsafe {
                        gl.vertex_attrib_pointer_f32(
                            attr_index as _,
                            attribute.size,
                            attribute.type_,
                            false,
                            attribute.stride,
                            attribute.offset as _,
                        );
                        gl.vertex_attrib_divisor(attr_index as _, attribute.divisor as _);
                        gl.enable_vertex_attrib_array(attr_index as _);
                    };

                    let cached_attr = &mut self.cache.attributes[attr_index];
                    *cached_attr = Some(CachedAttribute {
                        attribute,
                        gl_vbuf: Some(vb.gl_buf),
                    });
                }
            } else {
                if cached_attr.is_some() {
                    unsafe {
                        gl.disable_vertex_attrib_array(attr_index as _);
                    }
                    self.cache.attributes[attr_index] = None;
                }
            }
        }
    }

    pub fn apply_uniforms<U>(&mut self, uniforms: &U) {
        self.apply_uniforms_from_bytes(uniforms as *const _ as *const u8, std::mem::size_of::<U>())
    }

    fn apply_uniforms_from_bytes(&mut self, uniform_ptr: *const u8, size: usize) {
        let pip = &self.pipelines[self.cache.cur_pipeline.unwrap().0];
        let shader = &self.shaders[pip.shader.0];

        let mut offset = 0;

        let gl = &self.glow_ctx.0.gl;

        for uniform in shader.uniforms.iter() {
            use types_impl::UniformType::*;

            assert!(
                offset <= size - uniform.uniform_type.size() / 4,
                "Uniforms struct does not match shader uniforms layout"
            );

            unsafe {
                let data = (uniform_ptr as *const f32).offset(offset as isize);
                let data_int = (uniform_ptr as *const i32).offset(offset as isize);

                if let Some(gl_loc) = uniform.gl_loc {
                    match uniform.uniform_type {
                        Float1 => {
                            gl.uniform_1_f32_slice(Some(&gl_loc), std::slice::from_raw_parts(data, uniform.array_count as _));
                        }
                        Float2 => {
                            gl.uniform_2_f32_slice(Some(&gl_loc), std::slice::from_raw_parts(data, (uniform.array_count * 2) as _));
                        }
                        Float3 => {
                            gl.uniform_3_f32_slice(Some(&gl_loc), std::slice::from_raw_parts(data, (uniform.array_count * 3) as _));
                        }
                        Float4 => {
                            gl.uniform_4_f32_slice(Some(&gl_loc), std::slice::from_raw_parts(data, (uniform.array_count * 4) as _));
                        }
                        Int1 => {
                            gl.uniform_1_i32_slice(Some(&gl_loc), std::slice::from_raw_parts(data_int, uniform.array_count as _));
                        }
                        Int2 => {
                            gl.uniform_2_i32_slice(Some(&gl_loc), std::slice::from_raw_parts(data_int, (uniform.array_count * 2) as _));
                        }
                        Int3 => {
                            gl.uniform_3_i32_slice(Some(&gl_loc), std::slice::from_raw_parts(data_int, (uniform.array_count * 3) as _));
                        }
                        Int4 => {
                            gl.uniform_3_i32_slice(Some(&gl_loc), std::slice::from_raw_parts(data_int, (uniform.array_count * 4) as _));
                        }
                        Mat4 => {
                            gl.uniform_matrix_4_f32_slice(
                                Some(&gl_loc),
                                false,
                                std::slice::from_raw_parts(data, (uniform.array_count * 16) as _)
                            );
                        }
                    }
                }
            }
            offset += uniform.uniform_type.size() / 4 * uniform.array_count as usize;
        }
    }

    pub fn clear(
        &self,
        color: Option<(f32, f32, f32, f32)>,
        depth: Option<f32>,
        stencil: Option<i32>,
    ) {
        let gl = &self.glow_ctx.0.gl;

        let mut bits = 0;
        if let Some((r, g, b, a)) = color {
            bits |= glow::COLOR_BUFFER_BIT;
            unsafe {
                gl.clear_color(r, g, b, a);
            }
        }

        if let Some(v) = depth {
            bits |= glow::DEPTH_BUFFER_BIT;
            unsafe {
                gl.clear_depth_f32(v);
            }
        }

        if let Some(v) = stencil {
            bits |= glow::STENCIL_BUFFER_BIT;
            unsafe {
                gl.clear_stencil(v);
            }
        }

        if bits != 0 {
            unsafe {
                gl.clear(bits);
            }
        }
    }

    pub fn begin_default_pass(&mut self, action: PassAction) {
        self.begin_pass(None, action);
    }

    pub fn begin_pass(&mut self, pass: impl Into<Option<RenderPass>>, action: PassAction) {
        let (default_w, default_h) = self.window_size;
        let (h_dpi, v_dpi) = self.dpi;
        let (framebuffer, w, h) = match pass.into() {
            None => (
                self.default_framebuffer,
                (default_w as f32 * h_dpi) as i32,
                (default_h as f32 * v_dpi) as i32,
            ),
            Some(pass) => {
                let pass = &self.passes[pass.0];
                (
                    pass.gl_fb,
                    pass.texture.width as i32,
                    pass.texture.height as i32,
                )
            }
        };

        let gl = &self.glow_ctx.0.gl;

        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
            gl.viewport(0, 0, w, h);
            gl.scissor(0, 0, w, h);
        }
        match action {
            PassAction::Nothing => {}
            PassAction::Clear {
                color,
                depth,
                stencil,
            } => {
                self.clear(color, depth, stencil);
            }
        }
    }

    pub fn end_render_pass(&mut self) {
        unsafe {
            self.glow_ctx.0.gl.bind_framebuffer(
                glow::FRAMEBUFFER,
                Some(self.default_framebuffer)
            );
            self.cache.bind_buffer(glow::ARRAY_BUFFER, None, None);
            self.cache.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None, None);
        }
    }

    pub fn commit_frame(&mut self) {
        self.cache.clear_buffer_bindings();
        self.cache.clear_texture_bindings();
    }

    pub fn draw(&self, base_element: i32, num_elements: i32, num_instances: i32) {
        assert!(
            self.cache.cur_pipeline.is_some(),
            "Drawing without any binded pipeline"
        );

        let pip = &self.pipelines[self.cache.cur_pipeline.unwrap().0];
        let primitive_type = pip.params.primitive_type.into();
        let index_type = self.cache.index_type.expect("Unset index buffer type");

        unsafe {
            self.glow_ctx.0.gl.draw_elements_instanced(
                primitive_type,
                num_elements,
                index_type.into(),
                index_type.size() as i32 * base_element,
                num_instances,
            );
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        let shaders = std::mem::take(&mut self.shaders);

        for shader in shaders.iter() {
            shader.delete(self);
        }

        self.shaders = shaders;
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PipelineParams {
    pub cull_face: CullFace,
    pub front_face_order: FrontFaceOrder,
    pub depth_test: Comparison,
    pub depth_write: bool,
    pub depth_write_offset: Option<(f32, f32)>,
    pub color_blend: Option<BlendState>,
    pub alpha_blend: Option<BlendState>,
    pub stencil_test: Option<StencilState>,
    pub color_write: ColorMask,
    pub primitive_type: PrimitiveType,
}

#[derive(Copy, Clone, Debug)]
pub struct Pipeline(usize);

impl Default for PipelineParams {
    fn default() -> PipelineParams {
        PipelineParams {
            cull_face: CullFace::Nothing,
            front_face_order: FrontFaceOrder::CounterClockwise,
            depth_test: Comparison::Always, // no depth test,
            depth_write: false,             // no depth write,
            depth_write_offset: None,
            color_blend: None,
            alpha_blend: None,
            stencil_test: None,
            color_write: (true, true, true, true),
            primitive_type: PrimitiveType::Triangles,
        }
    }
}

impl Pipeline {
    pub fn new(
        ctx: &mut Context,
        buffer_layout: &[BufferLayout],
        attributes: &[VertexAttribute],
        shader: Shader,
    ) -> Pipeline {
        Self::with_params(ctx, buffer_layout, attributes, shader, Default::default())
    }

    pub fn with_params(
        ctx: &mut Context,
        buffer_layout: &[BufferLayout],
        attributes: &[VertexAttribute],
        shader: Shader,
        params: PipelineParams,
    ) -> Pipeline {
        #[derive(Clone, Copy, Default)]
        struct BufferCacheData {
            stride: i32,
            offset: i64,
        }

        let mut buffer_cache: Vec<BufferCacheData> =
            vec![BufferCacheData::default(); buffer_layout.len()];

        for VertexAttribute {
            format,
            buffer_index,
            ..
        } in attributes
        {
            let layout = buffer_layout.get(*buffer_index).unwrap_or_else(|| panic!());
            let mut cache = buffer_cache
                .get_mut(*buffer_index)
                .unwrap_or_else(|| panic!());

            if layout.stride == 0 {
                cache.stride += format.byte_len();
            } else {
                cache.stride = layout.stride;
            }
            // WebGL 1 limitation
            assert!(cache.stride <= 255);
        }

        let program = ctx.shaders[shader.0].program;

        let attributes_len = attributes
            .iter()
            .map(|layout| match layout.format {
                VertexFormat::Mat4 => 4,
                _ => 1,
            })
            .sum();

        let mut vertex_layout: Vec<Option<VertexAttributeInternal>> = vec![None; attributes_len];

        for VertexAttribute {
            name,
            format,
            buffer_index,
        } in attributes
        {
            let mut buffer_data = &mut buffer_cache
                .get_mut(*buffer_index)
                .unwrap_or_else(|| panic!());
            let layout = buffer_layout.get(*buffer_index).unwrap_or_else(|| panic!());

            let attr_loc = unsafe {
                ctx.glow_ctx.0.gl.get_attrib_location(program, *name)
            };
            let divisor = if layout.step_func == VertexStep::PerVertex {
                0
            } else {
                layout.step_rate
            };

            let mut attributes_count: usize = 1;
            let mut format = *format;

            if format == VertexFormat::Mat4 {
                format = VertexFormat::Float4;
                attributes_count = 4;
            }
            for i in 0..attributes_count {
                if let Some(attr_loc) = attr_loc {
                    let attr_loc = attr_loc as u32 + i as u32;

                    let attr = VertexAttributeInternal {
                        attr_loc,
                        size: format.size(),
                        type_: format.type_(),
                        offset: buffer_data.offset,
                        stride: buffer_data.stride,
                        buffer_index: *buffer_index,
                        divisor,
                    };
                    //println!("{}: {:?}", name, attr);

                    assert!(
                        attr_loc < vertex_layout.len() as u32,
                        "attribute: {} outside of allocated attributes array len: {}",
                        name,
                        vertex_layout.len()
                    );
                    vertex_layout[attr_loc as usize] = Some(attr);
                }
                buffer_data.offset += format.byte_len() as i64
            }
        }

        let pipeline = PipelineInternal {
            layout: vertex_layout,
            shader,
            params,
        };

        ctx.pipelines.push(pipeline);
        Pipeline(ctx.pipelines.len() - 1)
    }

    pub fn set_blend(&self, ctx: &mut Context, color_blend: Option<BlendState>) {
        let mut pipeline = &mut ctx.pipelines[self.0];
        pipeline.params.color_blend = color_blend;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
struct VertexAttributeInternal {
    attr_loc: u32,
    size: i32,
    type_: u32,
    offset: i64,
    stride: i32,
    buffer_index: usize,
    divisor: i32,
}

struct PipelineInternal {
    layout: Vec<Option<VertexAttributeInternal>>,
    shader: Shader,
    params: PipelineParams,
}

#[derive(Clone)]
pub struct Bindings {
    pub vertex_buffers: Vec<Buffer>,
    pub index_buffer: Buffer,
    pub images: Vec<Texture>,
}

impl Drop for Bindings {
    fn drop(&mut self) {
        for buffer in self.vertex_buffers.iter() {
            buffer.delete();
        }
        self.index_buffer.delete();
        for image in self.images.iter() {
            image.delete();
        }
    }
}

type ColorMask = (bool, bool, bool, bool);

#[derive(Default, Copy, Clone)]
struct CachedAttribute {
    attribute: VertexAttributeInternal,
    gl_vbuf: Option<glow::Buffer>,
}

pub enum PassAction {
    Nothing,
    Clear {
        color: Option<(f32, f32, f32, f32)>,
        depth: Option<f32>,
        stencil: Option<i32>,
    },
}

impl PassAction {
    pub fn clear_color(r: f32, g: f32, b: f32, a: f32) -> PassAction {
        PassAction::Clear {
            color: Some((r, g, b, a)),
            depth: Some(1.),
            stencil: None,
        }
    }
}

impl Default for PassAction {
    fn default() -> PassAction {
        PassAction::Clear {
            color: Some((0.0, 0.0, 0.0, 0.0)),
            depth: Some(1.),
            stencil: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderPass(usize);

struct RenderPassInternal {
    gl_fb: glow::Framebuffer,
    texture: Texture,
    depth_texture: Option<Texture>,
}

impl RenderPass {
    pub fn new(
        context: &mut Context,
        color_img: Texture,
        depth_img: impl Into<Option<Texture>>,
    ) -> RenderPass {
        let pass = unsafe {
            let depth_img = depth_img.into();
            let gl = &context.glow_ctx.0.gl;
            let gl_fb = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(gl_fb));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                color_img.texture,
                0,
            );
            if let Some(depth_img) = depth_img.clone() {
                gl.framebuffer_texture_2d(
                    glow::FRAMEBUFFER,
                    glow::DEPTH_ATTACHMENT,
                    glow::TEXTURE_2D,
                    depth_img.texture,
                    0,
                );
            }
            gl.bind_framebuffer(
                glow::FRAMEBUFFER,
                Some(context.default_framebuffer)
            );

            RenderPassInternal {
                gl_fb,
                texture: color_img,
                depth_texture: depth_img,
            }
        };

        context.passes.push(pass);

        RenderPass(context.passes.len() - 1)
    }

    pub fn texture(&self, ctx: &mut Context) -> Texture {
        let render_pass = &mut ctx.passes[self.0];

        render_pass.texture.clone()
    }

    pub fn delete(&self, ctx: &mut Context) {
        let render_pass = &mut ctx.passes[self.0];

        unsafe {
            ctx.glow_ctx.0.gl.delete_framebuffer(render_pass.gl_fb);
        }

        render_pass.texture.delete();
        if let Some(depth_texture) = render_pass.depth_texture.clone() {
            depth_texture.delete();
        }
    }
}