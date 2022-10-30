use crate as gl_p;

pub fn on_frame_start(
    egui_input: &mut egui::RawInput,
    _equi_ctx: &egui::Context,
    gl_p_ctx: &gl_p::Context
) {
    let screen_size_in_points = gl_p_ctx.get_window_size();
    let pixels_per_point = gl_p_ctx.get_dpi();

    let screen_size_in_points = egui::vec2(screen_size_in_points.0 as f32, screen_size_in_points.1 as f32);
    egui_input.screen_rect = Some(egui::Rect::from_min_size(
        Default::default(),
        screen_size_in_points,
    ));
    egui_input.pixels_per_point = Some(pixels_per_point.0);
    egui_input.time = Some({
        use std::time::SystemTime;

        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|e| panic!("{}", e));
        time.as_secs_f64()
    });
}

/// orom_miniquad sends special keys (backspace, delete, F1, ...) as characters.
/// Ignore those.
/// We also ignore '\r', '\n', '\t'.
/// Newlines are handled by the `Key::Enter` event.
pub fn is_printable_char(chr: char) -> bool {
    #![allow(clippy::manual_range_contains)]

    let is_in_private_use_area = '\u{e000}' <= chr && chr <= '\u{f8ff}'
        || '\u{f0000}' <= chr && chr <= '\u{ffffd}'
        || '\u{100000}' <= chr && chr <= '\u{10fffd}';

    !is_in_private_use_area && !chr.is_ascii_control()
}

pub fn egui_modifiers_from_gl_p_modifiers(keymods: gl_p::window::KeyMods) -> egui::Modifiers {
    egui::Modifiers {
        alt: keymods.alt,
        ctrl: keymods.ctrl,
        shift: keymods.shift,
        mac_cmd: keymods.logo && cfg!(target_os = "macos"),
        command: if cfg!(target_os = "macos") {
            keymods.logo
        } else {
            keymods.ctrl
        },
    }
}

pub fn egui_key_from_gl_p_key(key: gl_p::window::KeyCode) -> Option<egui::Key> {
    Some(match key {
        gl_p::window::KeyCode::Down => egui::Key::ArrowDown,
        gl_p::window::KeyCode::Left => egui::Key::ArrowLeft,
        gl_p::window::KeyCode::Right => egui::Key::ArrowRight,
        gl_p::window::KeyCode::Up => egui::Key::ArrowUp,

        gl_p::window::KeyCode::Escape => egui::Key::Escape,
        gl_p::window::KeyCode::Tab => egui::Key::Tab,
        gl_p::window::KeyCode::Backspace => egui::Key::Backspace,
        gl_p::window::KeyCode::Return => egui::Key::Enter,
        gl_p::window::KeyCode::Space => egui::Key::Space,

        gl_p::window::KeyCode::Insert => egui::Key::Insert,
        gl_p::window::KeyCode::Delete => egui::Key::Delete,
        gl_p::window::KeyCode::Home => egui::Key::Home,
        gl_p::window::KeyCode::End => egui::Key::End,
        gl_p::window::KeyCode::PageUp => egui::Key::PageUp,
        gl_p::window::KeyCode::PageDown => egui::Key::PageDown,

        gl_p::window::KeyCode::Num0 => egui::Key::Num0,
        gl_p::window::KeyCode::Num1 => egui::Key::Num1,
        gl_p::window::KeyCode::Num2 => egui::Key::Num2,
        gl_p::window::KeyCode::Num3 => egui::Key::Num3,
        gl_p::window::KeyCode::Num4 => egui::Key::Num4,
        gl_p::window::KeyCode::Num5 => egui::Key::Num5,
        gl_p::window::KeyCode::Num6 => egui::Key::Num6,
        gl_p::window::KeyCode::Num7 => egui::Key::Num7,
        gl_p::window::KeyCode::Num8 => egui::Key::Num8,
        gl_p::window::KeyCode::Num9 => egui::Key::Num9,

        gl_p::window::KeyCode::A => egui::Key::A,
        gl_p::window::KeyCode::B => egui::Key::B,
        gl_p::window::KeyCode::C => egui::Key::C,
        gl_p::window::KeyCode::D => egui::Key::D,
        gl_p::window::KeyCode::E => egui::Key::E,
        gl_p::window::KeyCode::F => egui::Key::F,
        gl_p::window::KeyCode::G => egui::Key::G,
        gl_p::window::KeyCode::H => egui::Key::H,
        gl_p::window::KeyCode::I => egui::Key::I,
        gl_p::window::KeyCode::J => egui::Key::J,
        gl_p::window::KeyCode::K => egui::Key::K,
        gl_p::window::KeyCode::L => egui::Key::L,
        gl_p::window::KeyCode::M => egui::Key::M,
        gl_p::window::KeyCode::N => egui::Key::N,
        gl_p::window::KeyCode::O => egui::Key::O,
        gl_p::window::KeyCode::P => egui::Key::P,
        gl_p::window::KeyCode::Q => egui::Key::Q,
        gl_p::window::KeyCode::R => egui::Key::R,
        gl_p::window::KeyCode::S => egui::Key::S,
        gl_p::window::KeyCode::T => egui::Key::T,
        gl_p::window::KeyCode::U => egui::Key::U,
        gl_p::window::KeyCode::V => egui::Key::V,
        gl_p::window::KeyCode::W => egui::Key::W,
        gl_p::window::KeyCode::X => egui::Key::X,
        gl_p::window::KeyCode::Y => egui::Key::Y,
        gl_p::window::KeyCode::Z => egui::Key::Z,

        _ => return None,
    })
}