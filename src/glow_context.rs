use std::rc::Rc;

/// The context required to interact with the GPU
#[derive(Clone)]
pub struct GlowContext(pub(crate) Rc<ContextContents>);

pub(crate) struct ContextContents {
    pub(crate) gl: glow::Context
}

impl GlowContext {
    pub(crate) fn new_from_sdl3_video(video: &sdl3::VideoSubsystem) -> Self {
        GlowContext(Rc::new(ContextContents {
            gl: unsafe {
                let gl = glow::
                Context::
                from_loader_function(|s| {
                    let proc_address = video.gl_get_proc_address(s);
                    match proc_address {
                        None => std::ptr::null() as *const _,
                        Some(proc_address) => proc_address as *const _
                    }
                });
                gl
            }
        }))
    }
}