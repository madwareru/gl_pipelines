use std::rc::Rc;

/// The context required to interact with the GPU
#[derive(Clone)]
pub struct GlowContext(pub(crate) Rc<ContextContents>);

pub(crate) struct ContextContents {
    pub(crate) gl: glow::Context
}

impl GlowContext {
    pub(crate) fn new_from_sdl2_video(video: &sdl2::VideoSubsystem) -> Self {
        GlowContext(Rc::new(ContextContents {
            gl: unsafe {
                let gl = glow::
                Context::
                from_loader_function(|s| video.gl_get_proc_address(s) as *const _);
                gl
            }
        }))
    }
}