mod input;
mod painter;

// ----------------------------------------------------------------------------

use egui::CursorIcon;
use crate as gl_p;

/// egui bindings for orom_miniquad.
pub struct EguiMq {
    egui_ctx: egui::CtxRef,
    egui_input: egui::RawInput,
    painter: painter::Painter,
    shapes: Option<Vec<egui::epaint::ClippedShape>>,
}

impl EguiMq {
    pub fn new(gl_p_ctx: &mut gl_p::Context) -> Self {
        Self {
            egui_ctx: egui::CtxRef::default(),
            painter: painter::Painter::new(gl_p_ctx),
            egui_input: Default::default(),
            shapes: None,
        }
    }

    /// Use this to open egui windows, panels etc.
    /// Can only be used between [`Self::begin_frame`] and [`Self::end_frame`].
    pub fn egui_ctx(&self) -> &egui::CtxRef { &self.egui_ctx }

    /// Call this at the start of each `draw` call.
    pub fn begin_frame(&mut self, gl_p_ctx: &mut gl_p::Context) {
        input::on_frame_start(&mut self.egui_input, gl_p_ctx);
        self.egui_ctx.begin_frame(self.egui_input.take());
    }

    /// Call this at the end of each `draw` call.
    /// This will draw the `egui` interface.
    pub fn end_frame(&mut self, _gl_p_ctx: &mut gl_p::Context, win_ctx: &mut gl_p::window::WindowContext) {
        let (output, shapes) = self.egui_ctx.end_frame();
        if self.shapes.is_some() {
            eprintln!(
                "Egui contents not drawed. You need to call `draw` after calling `end_frame`"
            );
        }
        self.shapes = Some(shapes);

        let egui::Output { cursor_icon, open_url, copied_text, .. } = output;

        if let Some(url) = open_url {
            webbrowser::open(&url.url).unwrap();
        }

        if cursor_icon == CursorIcon::None {
            win_ctx.hide_cursor();
        } else {
            let gl_p_cursor_icon = to_gl_p_cursor_icon(cursor_icon);
            let gl_p_cursor_icon = gl_p_cursor_icon.unwrap_or(gl_p::window::CursorIcon::Default);
            win_ctx.set_system_cursor(gl_p_cursor_icon);
        }

        if !copied_text.is_empty() {
            win_ctx.set_clipboard_content(copied_text);
        }
    }

    /// Call this when you need to draw egui.
    /// Must be called after `end_frame`.
    pub fn draw(&mut self, gl_p_ctx: &mut gl_p::Context) {
        if let Some(shapes) = self.shapes.take() {
            let paint_jobs = self.egui_ctx.tessellate(shapes);
            self.painter
                .paint(gl_p_ctx, paint_jobs, &self.egui_ctx.texture());
        } else {
            eprintln!("Failed to draw egui. You need to call `end_frame` before calling `draw`");
        }
    }

    /// Call from your [`orom_miniquad::EventHandler`].
    pub fn mouse_motion_event(&mut self, ctx: &mut gl_p::Context, x: f32, y: f32) {
        let dpi = ctx.get_dpi();
        let pos = egui::pos2(x as f32 / dpi.0, y as f32 / dpi.1);
        self.egui_input.events.push(egui::Event::PointerMoved(pos))
    }

    /// Call from your [`orom_miniquad::EventHandler`].
    pub fn mouse_wheel_event(&mut self, _ctx: &mut gl_p::Context, dx: f32, dy: f32) {
        let delta = egui::vec2(dx, dy); // Correct for web, but too slow for mac native :/

        if self.egui_input.modifiers.ctrl {
            // Treat as zoom instead:
            self.egui_input.zoom_delta *= (delta.y / 200.0).exp();
        } else {
            self.egui_input.scroll_delta += delta;
        }
    }

    /// Call from your [`orom_miniquad::EventHandler`].
    pub fn mouse_button_down_event(
        &mut self,
        ctx: &mut gl_p::Context,
        mb: gl_p::window::MouseButton,
        x: f32,
        y: f32,
    ) {
        let dpi = ctx.get_dpi();
        let pos = egui::pos2(x as f32 / dpi.0, y as f32 / dpi.1);
        let button = to_egui_button(mb);
        self.egui_input.events.push(egui::Event::PointerButton {
            pos,
            button,
            pressed: true,
            modifiers: self.egui_input.modifiers,
        })
    }

    /// Call from your [`orom_miniquad::EventHandler`].
    pub fn mouse_button_up_event(
        &mut self,
        ctx: &mut gl_p::Context,
        mb: gl_p::window::MouseButton,
        x: f32,
        y: f32,
    ) {
        let dpi = ctx.get_dpi();
        let pos = egui::pos2(x as f32 / dpi.0, y as f32 / dpi.1);
        let button = to_egui_button(mb);

        self.egui_input.events.push(egui::Event::PointerButton {
            pos,
            button,
            pressed: false,
            modifiers: self.egui_input.modifiers,
        })
    }

    /// Call from your [`orom_miniquad::EventHandler`].
    pub fn char_event(&mut self, chr: char) {
        if input::is_printable_char(chr)
            && !self.egui_input.modifiers.ctrl
            && !self.egui_input.modifiers.mac_cmd
        {
            self.egui_input
                .events
                .push(egui::Event::Text(chr.to_string()));
        }
    }

    /// Call from your [`orom_miniquad::EventHandler`].
    pub fn key_down_event(
        &mut self,
        _gl_p_ctx: &mut gl_p::Context,
        win_ctx: &mut gl_p::window::WindowContext,
        keycode: gl_p::window::KeyCode,
        keymods: gl_p::window::KeyMods,
    ) {
        let modifiers = input::egui_modifiers_from_gl_p_modifiers(keymods);
        self.egui_input.modifiers = modifiers;

        if modifiers.command && keycode == gl_p::window::KeyCode::X {
            self.egui_input.events.push(egui::Event::Cut);
        } else if modifiers.command && keycode == gl_p::window::KeyCode::C {
            self.egui_input.events.push(egui::Event::Copy);
        } else if modifiers.command && keycode == gl_p::window::KeyCode::V {
            if let Some(text) = win_ctx.get_clipboard_content() {
                self.egui_input.events.push(egui::Event::Text(text));
            }
        } else if let Some(key) = input::egui_key_from_gl_p_key(keycode) {
            self.egui_input.events.push(egui::Event::Key {
                key,
                pressed: true,
                modifiers,
            })
        }
    }

    /// Call from your [`orom_miniquad::EventHandler`].
    pub fn key_up_event(&mut self, keycode: gl_p::window::KeyCode, keymods: gl_p::window::KeyMods) {
        let modifiers = input::egui_modifiers_from_gl_p_modifiers(keymods);
        self.egui_input.modifiers = modifiers;
        if let Some(key) = input::egui_key_from_gl_p_key(keycode) {
            self.egui_input.events.push(egui::Event::Key {
                key,
                pressed: false,
                modifiers,
            })
        }
    }
}

fn to_egui_button(mb: gl_p::window::MouseButton) -> egui::PointerButton {
    match mb {
        gl_p::window::MouseButton::Left => egui::PointerButton::Primary,
        gl_p::window::MouseButton::Right => egui::PointerButton::Secondary,
        gl_p::window::MouseButton::Middle => egui::PointerButton::Middle,
        gl_p::window::MouseButton::Unknown => egui::PointerButton::Primary,
    }
}

fn to_gl_p_cursor_icon(cursor_icon: egui::CursorIcon) -> Option<gl_p::window::CursorIcon> {
    match cursor_icon {
        // Handled outside this function
        CursorIcon::None => None,

        egui::CursorIcon::Default => Some(gl_p::window::CursorIcon::Default),
        egui::CursorIcon::PointingHand => Some(gl_p::window::CursorIcon::Pointer),
        egui::CursorIcon::Text => Some(gl_p::window::CursorIcon::Text),
        egui::CursorIcon::ResizeHorizontal => Some(gl_p::window::CursorIcon::EWResize),
        egui::CursorIcon::ResizeVertical => Some(gl_p::window::CursorIcon::NSResize),
        egui::CursorIcon::ResizeNeSw => Some(gl_p::window::CursorIcon::NESWResize),
        egui::CursorIcon::ResizeNwSe => Some(gl_p::window::CursorIcon::NWSEResize),
        egui::CursorIcon::Help => Some(gl_p::window::CursorIcon::Help),
        egui::CursorIcon::Wait => Some(gl_p::window::CursorIcon::Wait),
        egui::CursorIcon::Crosshair => Some(gl_p::window::CursorIcon::Crosshair),
        egui::CursorIcon::Move => Some(gl_p::window::CursorIcon::Move),
        egui::CursorIcon::NotAllowed => Some(gl_p::window::CursorIcon::NotAllowed),

        // Similar enough
        egui::CursorIcon::AllScroll => Some(gl_p::window::CursorIcon::Move),
        egui::CursorIcon::Progress => Some(gl_p::window::CursorIcon::Wait),

        // Not implemented
        egui::CursorIcon::Grab | egui::CursorIcon::Grabbing => None,

        // Also not implemented:
        egui::CursorIcon::Alias
        | egui::CursorIcon::Cell
        | egui::CursorIcon::ContextMenu
        | egui::CursorIcon::Copy
        | egui::CursorIcon::NoDrop
        | egui::CursorIcon::VerticalText
        | egui::CursorIcon::ZoomIn
        | egui::CursorIcon::ZoomOut => None,
    }
}