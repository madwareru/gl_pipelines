use std::mem;
use glow::HasContext;
use crate::{Context, GlowContext};
use crate::types_impl::{BufferType, IndexType, Usage};

#[derive(Clone)]
pub struct Buffer {
    pub(crate) glow_ctx: GlowContext,
    pub(crate) gl_buf: glow::Buffer,
    pub(crate) buffer_type: BufferType,
    pub(crate) size: usize,
    pub(crate) index_type: Option<IndexType>,
}

impl Buffer {
    pub fn immutable<T: bytemuck::Pod>(ctx: &mut Context, buffer_type: BufferType, data: &[T]) -> Buffer {
        let index_type = if buffer_type == BufferType::IndexBuffer {
            Some(IndexType::for_type::<T>())
        } else {
            None
        };

        let gl_target = gl_buffer_target(&buffer_type);
        let gl_usage = gl_usage(&Usage::Immutable);
        let size = mem::size_of_val(data);

        let gl = &ctx.glow_ctx.0.gl;

        let gl_buf = unsafe {
            let gl_buf = gl.create_buffer().unwrap();
            ctx.cache.store_buffer_binding(gl_target);
            ctx.cache.bind_buffer(gl_target, Some(gl_buf), index_type);

            let data_casted: &[u8] = bytemuck::cast_slice(&data[..]);
            gl.buffer_data_u8_slice(gl_target, data_casted, gl_usage);
            ctx.cache.restore_buffer_binding(gl_target);
            gl_buf
        };

        Buffer {
            glow_ctx: ctx.glow_ctx.clone(),
            gl_buf,
            buffer_type,
            size,
            index_type,
        }
    }

    pub fn stream(ctx: &mut Context, buffer_type: BufferType, size: usize) -> Buffer {
        let index_type = if buffer_type == BufferType::IndexBuffer {
            Some(IndexType::Short)
        } else {
            None
        };

        let gl_target = gl_buffer_target(&buffer_type);
        let gl_usage = gl_usage(&Usage::Stream);

        let gl = &ctx.glow_ctx.0.gl;

        let gl_buf = unsafe {
            let gl_buf = gl.create_buffer().unwrap();
            ctx.cache.store_buffer_binding(gl_target);
            ctx.cache.bind_buffer(gl_target, Some(gl_buf), None);
            gl.buffer_data_size(gl_target, size as _, gl_usage);
            ctx.cache.restore_buffer_binding(gl_target);
            gl_buf
        };

        Buffer {
            glow_ctx: ctx.glow_ctx.clone(),
            gl_buf,
            buffer_type,
            size,
            index_type,
        }
    }

    pub fn index_stream(ctx: &mut Context, index_type: IndexType, size: usize) -> Buffer {
        let gl_target = gl_buffer_target(&BufferType::IndexBuffer);
        let gl_usage = gl_usage(&Usage::Stream);

        let gl = &ctx.glow_ctx.0.gl;

        let gl_buf = unsafe {
            let gl_buf = gl.create_buffer().unwrap();
            ctx.cache.store_buffer_binding(gl_target);
            ctx.cache.bind_buffer(gl_target, Some(gl_buf), None);
            gl.buffer_data_size(gl_target, size as _, gl_usage);
            ctx.cache.restore_buffer_binding(gl_target);
            gl_buf
        };

        Buffer {
            glow_ctx: ctx.glow_ctx.clone(),
            gl_buf,
            buffer_type: BufferType::IndexBuffer,
            size,
            index_type: Some(index_type),
        }
    }

    pub fn update<T: bytemuck::Pod>(&self, ctx: &mut Context, data: &[T]) {
        if self.buffer_type == BufferType::IndexBuffer {
            assert!(self.index_type.is_some());
            assert_eq!(self.index_type.unwrap(), IndexType::for_type::<T>());
        };

        let size = mem::size_of_val(data);

        assert!(size <= self.size);

        let gl_target = gl_buffer_target(&self.buffer_type);
        ctx.cache.store_buffer_binding(gl_target);
        ctx.cache.bind_buffer(gl_target, Some(self.gl_buf), self.index_type);
        unsafe {
            let data_casted: &[u8] = bytemuck::cast_slice(&data[..]);
            ctx.glow_ctx.0.gl.buffer_sub_data_u8_slice(gl_target, 0, data_casted);
        };
        ctx.cache.restore_buffer_binding(gl_target);
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn delete(&self) {
        unsafe {
            self.glow_ctx.0.gl.delete_buffer(self.gl_buf);
        }
    }
}

fn gl_buffer_target(buffer_type: &BufferType) -> u32 {
    match buffer_type {
        BufferType::VertexBuffer => glow::ARRAY_BUFFER,
        BufferType::IndexBuffer => glow::ELEMENT_ARRAY_BUFFER,
    }
}

fn gl_usage(usage: &Usage) -> u32 {
    match usage {
        Usage::Immutable => glow::STATIC_DRAW,
        Usage::Dynamic => glow::DYNAMIC_DRAW,
        Usage::Stream => glow::STREAM_DRAW,
    }
}