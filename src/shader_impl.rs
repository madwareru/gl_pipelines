use std::error::Error;
use std::fmt::Display;
use glow::HasContext;
use crate::{Context};
use crate::types_impl::{UniformBlockLayout, UniformType};

pub struct ShaderMeta {
    pub uniforms: UniformBlockLayout,
    pub images: Vec<String>,
}

#[derive(Clone, Debug, Copy)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

#[derive(Clone, Debug)]
pub enum ShaderError {
    CompilationError {
        shader_type: ShaderType,
        error_message: String,
    },
    LinkError(String),
    /// Shader strings should never contains \00 in the middle
    FFINulError(std::ffi::NulError),
}

impl From<std::ffi::NulError> for ShaderError {
    fn from(e: std::ffi::NulError) -> ShaderError {
        ShaderError::FFINulError(e)
    }
}

impl Display for ShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self) // Display the same way as Debug
    }
}

impl Error for ShaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Shader(pub(crate) usize);

impl Shader {
    pub fn new(
        ctx: &mut Context,
        vertex_shader: &str,
        fragment_shader: &str,
        meta: ShaderMeta,
    ) -> Result<Self, ShaderError> {
        let shader = load_shader_internal(ctx, vertex_shader, fragment_shader, meta)?;
        ctx.shaders.push(shader);
        Ok(Self(ctx.shaders.len() - 1))
    }
}

pub struct ShaderImage {
    pub(crate) gl_loc: Option<glow::UniformLocation>,
}

#[derive(Debug)]
pub struct ShaderUniform {
    pub(crate) gl_loc: Option<glow::UniformLocation>,
    pub(crate) uniform_type: UniformType,
    pub(crate) array_count: i32,
}

pub(crate) struct ShaderInternal {
    pub(crate) program: glow::Program,
    pub(crate) images: Vec<ShaderImage>,
    pub(crate) uniforms: Vec<ShaderUniform>,
}

impl ShaderInternal {
    pub(crate) fn delete(&self, ctx: &mut Context) {
        let gl = &ctx.glow_ctx.0.gl;
        unsafe {
            gl.delete_program(self.program);
        }
    }
}

fn load_shader_internal(
    context: &mut Context,
    vertex_shader: &str,
    fragment_shader: &str,
    meta: ShaderMeta,
) -> Result<ShaderInternal, ShaderError> {
    unsafe {
        let vertex_shader = load_shader(context, glow::VERTEX_SHADER, vertex_shader)?;
        let fragment_shader = load_shader(context,glow::FRAGMENT_SHADER, fragment_shader)?;

        let gl = &context.glow_ctx.0.gl;

        let program = gl.create_program().unwrap();
        gl.attach_shader(program, vertex_shader);
        gl.attach_shader(program, fragment_shader);
        gl.link_program(program);

        if !gl.get_program_link_status(program) {
            let error_message = gl.get_program_info_log(program);

            gl.delete_shader(vertex_shader);
            gl.delete_shader(fragment_shader);
            return Err(ShaderError::LinkError(error_message));
        }

        gl.use_program(Some(program));

        let images = meta.images.iter().map(|name| ShaderImage {
            gl_loc: gl.get_uniform_location(program, name),
        }).collect();

        let uniforms = meta.uniforms.uniforms.iter().map( |uniform| {
            ShaderUniform {
                gl_loc: gl.get_uniform_location(program, &uniform.name),
                uniform_type: uniform.uniform_type,
                array_count: uniform.array_count as _,
            }
        }).collect();

        gl.delete_shader(vertex_shader);
        gl.delete_shader(fragment_shader);

        Ok(ShaderInternal {
            program,
            images,
            uniforms,
        })
    }
}

fn load_shader(context: &mut Context, shader_type: u32, source: &str) -> Result<glow::Shader, ShaderError> {
    let gl = &context.glow_ctx.0.gl;
    unsafe {
        let shader = gl.create_shader(shader_type).unwrap();
        gl.shader_source(shader, source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            let error_message = gl.get_shader_info_log(shader);

            return Err(ShaderError::CompilationError {
                shader_type: match shader_type {
                    glow::VERTEX_SHADER => ShaderType::Vertex,
                    glow::FRAGMENT_SHADER => ShaderType::Fragment,
                    _ => unreachable!(),
                },
                error_message,
            });
        }

        Ok(shader)
    }
}