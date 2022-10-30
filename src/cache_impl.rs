use glow::HasContext;
use crate::{CachedAttribute, ColorMask, MAX_SHADERSTAGE_IMAGES, MAX_VERTEX_ATTRIBUTES, Pipeline};
use crate::glow_context::GlowContext;
use crate::types_impl::{BlendState, CullFace, IndexType, StencilState};

pub(crate) struct GlCache {
    pub(crate) glow_ctx: GlowContext,
    pub(crate) stored_index_buffer: Option<glow::Buffer>,
    pub(crate) stored_index_type: Option<IndexType>,
    pub(crate) stored_vertex_buffer: Option<glow::Buffer>,
    pub(crate) stored_texture: Option<glow::Texture>,
    pub(crate) index_buffer: Option<glow::Buffer>,
    pub(crate) index_type: Option<IndexType>,
    pub(crate) vertex_buffer: Option<glow::Buffer>,
    pub(crate) textures: [Option<glow::Texture>; MAX_SHADERSTAGE_IMAGES],
    pub(crate) cur_pipeline: Option<Pipeline>,
    pub(crate) color_blend: Option<BlendState>,
    pub(crate) alpha_blend: Option<BlendState>,
    pub(crate) stencil: Option<StencilState>,
    pub(crate) color_write: ColorMask,
    pub(crate) cull_face: CullFace,
    pub(crate) attributes: [Option<CachedAttribute>; MAX_VERTEX_ATTRIBUTES],
}

impl GlCache {
    pub(crate) fn bind_buffer(&mut self, target: u32, buffer: Option<glow::Buffer>, index_type: Option<IndexType>) {
        let gl = &self.glow_ctx.0.gl;

        if target == glow::ARRAY_BUFFER {
            if self.vertex_buffer != buffer {
                self.vertex_buffer = buffer;
                unsafe {
                    gl.bind_buffer(target, buffer);
                }
            }
        } else {
            if self.index_buffer != buffer {
                self.index_buffer = buffer;
                unsafe {
                    gl.bind_buffer(target, buffer);
                }
            }
            self.index_type = index_type;
        }
    }

    pub(crate) fn store_buffer_binding(&mut self, target: u32) {
        if target == glow::ARRAY_BUFFER {
            self.stored_vertex_buffer = self.vertex_buffer;
        } else {
            self.stored_index_buffer = self.index_buffer;
            self.stored_index_type = self.index_type;
        }
    }

    pub(crate) fn restore_buffer_binding(&mut self, target: u32) {
        if target == glow::ARRAY_BUFFER {
            match self.stored_vertex_buffer {
                Some(vb) => {
                    self.bind_buffer(target, Some(vb), None);
                    self.stored_vertex_buffer = None;
                },
                _ => ()
            }
        } else {
            match self.stored_index_buffer {
                Some(ib) => {
                    self.bind_buffer(target, Some(ib), None);
                    self.stored_index_buffer = None;
                },
                _ => ()
            }
        }
    }

    pub(crate) fn bind_texture(&mut self, slot_index: usize, texture: Option<glow::Texture>) {
        let gl = &self.glow_ctx.0.gl;
        unsafe {
            gl.active_texture(glow::TEXTURE0 + slot_index as u32);
            if self.textures[slot_index] != texture {
                gl.bind_texture(glow::TEXTURE_2D, texture);
                self.textures[slot_index] = texture;
            }
        }
    }

    pub(crate) fn store_texture_binding(&mut self, slot_index: usize) {
        self.stored_texture = self.textures[slot_index];
    }

    pub(crate) fn restore_texture_binding(&mut self, slot_index: usize) {
        self.bind_texture(slot_index, self.stored_texture);
    }

    pub(crate) fn clear_buffer_bindings(&mut self) {
        self.bind_buffer(glow::ARRAY_BUFFER, None, None);
        self.vertex_buffer = None;

        self.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None, None);
        self.index_buffer = None;
    }

    pub(crate) fn clear_texture_bindings(&mut self) {
        for ix in 0..MAX_SHADERSTAGE_IMAGES {
            if self.textures[ix].is_some() {
                self.bind_texture(ix, None);
                self.textures[ix] = None;
            }
        }
    }
}