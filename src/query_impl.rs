use glow::{HasContext};
use crate::{Context, GlowContext};

#[derive(Clone)]
pub struct ElapsedQuery {
    glow_ctx: GlowContext,
    gl_query: Option<glow::Query>,
}

impl ElapsedQuery {
    pub fn new(ctx: &mut Context) -> Self {
        Self {
            glow_ctx: ctx.glow_ctx.clone(),
            gl_query: None
        }
    }

    pub fn begin_query(&mut self) {
        let query = match self.gl_query {
            None => unsafe {
                let query = self.glow_ctx.0.gl.create_query().unwrap();
                self.gl_query = Some(query);
                query
            }
            Some(query) => query
        };

        unsafe {
            self.glow_ctx.0.gl.begin_query(glow::TIME_ELAPSED, query);
        }
    }

    pub fn end_query(&mut self) {
        unsafe {
            self.glow_ctx.0.gl.end_query(glow::TIME_ELAPSED);
        };
    }

    pub fn get_result(&self) -> Option<u64> {
        self.gl_query.map(|query| unsafe {
            self.glow_ctx.0.gl.get_query_parameter_u64(query, glow::QUERY_RESULT)
        })
    }

    /// Reports whenever result of submitted query is available for retrieval with
    /// [`ElapsedQuery::get_result()`].
    ///
    /// Note that the result may be ready only couple frames later due to asynchrnous nature of GPU
    /// command submission.
    ///
    /// Use [`ElapsedQuery::is_supported()`] to check if functionality is available and the method can be called.
    pub fn is_available(&self) -> bool {
        match self.gl_query {
            None => false,
            Some(query) => unsafe {
                let available = self.glow_ctx.0.gl.get_query_parameter_u32(
                    query,
                    glow::QUERY_RESULT_AVAILABLE
                );
                available != 0
            }
        }
    }

    /// Delete query.
    ///
    /// Note that the query is not deleted automatically when dropped.
    ///
    /// Implemented as `glDeleteQueries(...)` on OpenGL/WebGL platforms.
    pub fn delete(&mut self) {
        match self.gl_query {
            None => {}
            Some(query) => unsafe {
                self.glow_ctx.0.gl.delete_query(query);
                self.gl_query = None;
            }
        }
    }
}