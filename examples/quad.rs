use bytemuck::Zeroable;
use gl_pipelines::{
    Bindings, Buffer, BufferLayout, BufferType, Context, Pipeline, Shader, Texture, TextureKind,
    VertexAttribute, VertexFormat
};
use gl_pipelines::window::{Conf, EventHandler, SimpleEventHandler, WindowContext};

#[repr(C)]
#[derive(Copy, Clone)]
struct Vec2 {
    x: f32,
    y: f32,
}
unsafe impl Zeroable for Vec2 {}
unsafe impl bytemuck::Pod for Vec2{}

#[repr(C)]
#[derive(Copy, Clone)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
}
unsafe impl Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex{}

struct Stage {
    pipeline: Pipeline,
    bindings: Bindings,
}

impl EventHandler for Stage {
    fn update(&mut self, _ctx: &mut Context, _win_ctx: &mut WindowContext) {}

    fn draw(&mut self, ctx: &mut Context, _win_ctx: &mut WindowContext) {
        ctx.begin_default_pass(Default::default());

        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&self.bindings);

        let t = {
            use std::time::SystemTime;

            let time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_else(|e| panic!("{}", e));
            time.as_secs_f64()
        };

        for i in 0..10 {
            let t = t + i as f64 * 0.3;

            ctx.apply_uniforms(&shader::Uniforms {
                offset: (t.sin() as f32 * 0.5, (t * 3.).cos() as f32 * 0.5),
            });
            ctx.draw(0, 6, 1);
        }
        ctx.end_render_pass();

        ctx.commit_frame();
    }

    fn char_event(&mut self, _ctx: &mut Context, _win_ctx: &mut WindowContext, character: char) {
        println!("{}", character);
    }
}

impl SimpleEventHandler for Stage {
    fn make(ctx: &mut Context, _win_ctx: &mut WindowContext) -> Stage {
        let vertices: [Vertex; 4] = [
            Vertex { pos : Vec2 { x: -0.5, y: -0.5 }, uv: Vec2 { x: 0., y: 0. } },
            Vertex { pos : Vec2 { x:  0.5, y: -0.5 }, uv: Vec2 { x: 1., y: 0. } },
            Vertex { pos : Vec2 { x:  0.5, y:  0.5 }, uv: Vec2 { x: 1., y: 1. } },
            Vertex { pos : Vec2 { x: -0.5, y:  0.5 }, uv: Vec2 { x: 0., y: 1. } },
        ];
        let vertex_buffer = Buffer::immutable(
            ctx,
            BufferType::VertexBuffer,
            &vertices
        );

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = Buffer::immutable(
            ctx,
            BufferType::IndexBuffer,
            &indices
        );

        let pixels : [u32; 4*4] = [
            0xFFFFFFFF, 0xFF0000FF, 0xFFFFFFFF, 0xFF0000FF,
            0xFF0000FF, 0xFFFFFFFF, 0xFF0000FF, 0xFFFFFFFF,
            0xFFFFFFFF, 0xFF0000FF, 0xFFFFFFFF, 0xFF0000FF,
            0xFF0000FF, 0xFFFFFFFF, 0xFF0000FF, 0xFFFFFFFF,
        ];

        let texture = Texture::from_rgba8(
            ctx,
            4, 4, 1,
            bytemuck::cast_slice(&pixels[..]),
            TextureKind::Texture2D
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![texture],
        };

        let shader = Shader::new(
            ctx,
            shader::VERTEX,
            shader::FRAGMENT,
            shader::meta()
        ).unwrap();

        let pipeline = Pipeline::new(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            shader,
        );

        Stage { pipeline, bindings }
    }
}

fn main() {
    gl_pipelines::window::start::<Stage>(
        Conf {
            high_dpi: true,
            .. Default::default()
        }
    );
}

mod shader {
    use gl_pipelines::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 pos;
    attribute vec2 uv;

    uniform vec2 offset;

    varying lowp vec2 texcoord;

    void main() {
        gl_Position = vec4(pos + offset, 0, 1);
        texcoord = uv;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec2 texcoord;

    uniform sampler2D tex;

    void main() {
        gl_FragColor = texture2D(tex, texcoord);
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![UniformDesc::new("offset", UniformType::Float2)],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub offset: (f32, f32),
    }
}