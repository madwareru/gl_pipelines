use sdl2::event::WindowEvent;
use sdl2::EventPump;
use sdl2::keyboard::{Mod};
use crate::Context;

pub trait SimpleEventHandler : EventHandler {
    fn make(_gfx_ctx: &mut Context, _win_ctx: &mut WindowContext) -> Self;
}

pub trait ParametrizedEventHandler<TParameter> : EventHandler {
    fn make(_gfx_ctx: &mut Context, _win_ctx: &mut WindowContext, extra_param: TParameter) -> Self;
}

pub trait EventHandler {
    // +
    fn update(&mut self, _gfx_ctx: &mut Context, _win_ctx: &mut WindowContext);

    // +
    fn draw(&mut self, _gfx_ctx: &mut Context, _win_ctx: &mut WindowContext);

    // +
    fn resize_event(&mut self, _gfx_ctx: &mut Context, _win_ctx: &mut WindowContext, _width: i32, _height: i32) {}

    // +
    fn mouse_motion_event(&mut self, _gfx_ctx: &mut Context, _win_ctx: &mut WindowContext, _x: i32, _y: i32, _x_rel: i32, _y_rel: i32) {}

    // +
    fn mouse_wheel_event(&mut self, _gfx_ctx: &mut Context, _win_ctx: &mut WindowContext, _x: i32, _y: i32, _direction: MouseWheelDirection) {}

    // +
    fn mouse_button_down_event(
        &mut self,
        _gfx_ctx: &mut Context,
        _win_ctx: &mut WindowContext,
        _button: MouseButton,
        _x: i32,
        _y: i32,
        _clicks: u8
    ) {}

    // +
    fn mouse_button_up_event(
        &mut self,
        _gfx_ctx: &mut Context,
        _win_ctx: &mut WindowContext,
        _button: MouseButton,
        _x: i32,
        _y: i32,
        _clicks: u8
    ) {}

    // +
    fn char_event(&mut self, _gfx_ctx: &mut Context, _win_ctx: &mut WindowContext, _character: char) {}

    // +
    fn key_down_event(
        &mut self,
        _gfx_ctx: &mut Context,
        _win_ctx: &mut WindowContext,
        _keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
    }

    fn key_up_event(
        &mut self,
        _gfx_ctx: &mut Context,
        _win_ctx: &mut WindowContext,
        _keycode: KeyCode,
        _keymods: KeyMods
    ) {}

    fn window_minimized_event(&mut self, _gfx_ctx: &mut Context, _win_ctx: &mut WindowContext) {}

    // +
    fn window_restored_event(&mut self, _gfx_ctx: &mut Context, _win_ctx: &mut WindowContext) {}

    // +
    fn window_lost_focus_event(&mut self, _gfx_ctx: &mut Context, _win_ctx: &mut WindowContext) {}

    // +
    fn window_gained_focus_event(&mut self, _gfx_ctx: &mut Context, _win_ctx: &mut WindowContext) {}

    // +
    fn window_take_focus_event(&mut self, _gfx_ctx: &mut Context, _win_ctx: &mut WindowContext) {}

    // +
    fn quit_requested_event(&mut self, _gfx_ctx: &mut Context, _win_ctx: &mut WindowContext) {}
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct KeyMods {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool,
    pub caps_lock: bool
}

impl From<Mod> for KeyMods {
    fn from(mods: Mod) -> Self {
        Self {
            shift: mods.contains(Mod::LSHIFTMOD) || mods.contains(Mod::RSHIFTMOD),
            ctrl: mods.contains(Mod::LCTRLMOD) || mods.contains(Mod::RCTRLMOD),
            alt: mods.contains(Mod::LALTMOD) || mods.contains(Mod::RALTMOD),
            logo: mods.contains(Mod::LGUIMOD) || mods.contains(Mod::RGUIMOD),
            caps_lock: mods.contains(Mod::CAPSMOD)
        }
    }
}

pub use sdl2::keyboard::Keycode as KeyCode;
use sdl2::mouse::{Cursor, SystemCursor};
use sdl2::video::GLContext;

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum MouseButton {
    Right,
    Left,
    Middle,
    Unknown,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MouseWheelDirection {
    Normal,
    Flipped,
    Unknown(u32),
}

#[derive(Debug, Copy, Clone)]
pub struct Touch {
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum CursorIcon { Default, Help, Pointer, Wait, Crosshair, Text, Move, NotAllowed, EWResize, NSResize, NESWResize, NWSEResize }

#[derive(Debug)]
pub struct Conf {
    /// Title of the window, defaults to an empty string.
    pub window_title: String,
    /// The preferred width of the window
    ///
    /// Default: 800
    pub window_width: i32,
    /// The preferred height of the window
    ///
    /// Default: 600
    pub window_height: i32,
    /// Whether the rendering canvas is full-resolution on HighDPI displays.
    ///
    /// Default: false
    pub high_dpi: bool,
    /// Whether the window should be created in fullscreen mode
    ///
    /// Default: false
    pub fullscreen: bool,
    /// MSAA sample count
    ///
    /// Default: 1
    pub sample_count: u8,
    /// MSAA sample buffers
    ///
    /// Default: 1
    pub sample_buffers: u8,
    /// Determines if the application user can resize the window
    pub window_resizable: bool,
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            window_title: "".to_string(),
            window_width: 800,
            window_height: 600,
            high_dpi: false,
            fullscreen: false,
            sample_count: 1,
            sample_buffers: 1,
            window_resizable: true,
        }
    }
}

fn make_ctx_and_other_goodies(conf: &Conf) -> (Context, WindowContext, EventPump, GLContext) {
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 2);
    gl_attr.set_multisample_buffers(conf.sample_buffers);
    gl_attr.set_multisample_samples(conf.sample_count);

    let mut window_builder = video.window(
        &conf.window_title,
        conf.window_width as _,
        conf.window_height as _
    );

    window_builder.opengl();

    if conf.window_resizable {
        window_builder.resizable();
    }

    if conf.high_dpi {
        window_builder.allow_highdpi();
    }

    if conf.fullscreen {
        window_builder.fullscreen();
    }

    let window = window_builder.build().unwrap();

    let gl_context = window.gl_create_context().unwrap();

    let mut ctx = Context::new_from_sdl2(&video, conf.window_width, conf.window_height);

    let drawable_size = window.drawable_size();

    ctx.set_dpi_info(
        drawable_size.0 as f32 / conf.window_width as f32,
        drawable_size.1 as f32 / conf.window_height as f32
    );

    let event_loop = sdl.event_pump().unwrap();

    (
        ctx,
        WindowContext(
            window,
            video,
            sdl.mouse(),
            sdl.event().unwrap()
        ),
        event_loop,
        gl_context
    )
}

pub fn start<THandler: SimpleEventHandler>(conf: Conf) {
    let (mut ctx, mut window_context, mut events_loop, _gl_context) = {
        make_ctx_and_other_goodies(&conf)
    };

    let mut handler = THandler::make(&mut ctx, &mut window_context);

    start_main_loop(&mut ctx, &mut window_context, &mut events_loop, &mut handler);
}

pub fn start_parametrized<THandler, TParameter>(conf: Conf, extra_parameter: TParameter)
where THandler: ParametrizedEventHandler<TParameter>
{
    let (mut ctx, mut window_context, mut events_loop, _gl_context) = {
        make_ctx_and_other_goodies(&conf)
    };

    let mut handler = THandler::make(&mut ctx, &mut window_context, extra_parameter);

    start_main_loop(&mut ctx, &mut window_context, &mut events_loop, &mut handler);
}

fn start_main_loop<THandler: EventHandler>(
    mut ctx: &mut Context,
    mut window_context: &mut WindowContext,
    events_loop: &mut EventPump,
    handler: &mut THandler
) {
    'main_loop: loop {
        {
            for event in events_loop.poll_iter() {
                match event {
                    sdl2::event::Event::Quit { .. } => {
                        handler.quit_requested_event(&mut ctx, &mut window_context);
                        break 'main_loop;
                    }
                    sdl2::event::Event::MouseMotion { x, y, xrel, yrel, .. } => {
                        handler.mouse_motion_event(&mut ctx, &mut window_context, x, y, xrel, yrel);
                    }
                    sdl2::event::Event::MouseWheel { x, y, direction, .. } => {
                        handler.mouse_wheel_event(
                            &mut ctx,
                            &mut window_context,
                            x,
                            y,
                            match direction {
                                sdl2::mouse::MouseWheelDirection::Normal => MouseWheelDirection::Normal,
                                sdl2::mouse::MouseWheelDirection::Flipped => MouseWheelDirection::Flipped,
                                sdl2::mouse::MouseWheelDirection::Unknown(what) => MouseWheelDirection::Unknown(what)
                            })
                    }
                    sdl2::event::Event::MouseButtonDown { mouse_btn, clicks, x, y, .. } => {
                        handler.mouse_button_down_event(
                            &mut ctx, &mut window_context,
                            match mouse_btn {
                                sdl2::mouse::MouseButton::Left => MouseButton::Left,
                                sdl2::mouse::MouseButton::Middle => MouseButton::Middle,
                                sdl2::mouse::MouseButton::Right => MouseButton::Right,
                                _ => MouseButton::Unknown
                            },
                            x,
                            y,
                            clicks
                        )
                    }
                    sdl2::event::Event::TextInput { text, .. } => {
                        for chr in text.chars() {
                            handler.char_event(&mut ctx, &mut window_context, chr);
                        }
                    }
                    sdl2::event::Event::MouseButtonUp { mouse_btn, clicks, x, y, .. } => {
                        handler.mouse_button_up_event(
                            &mut ctx, &mut window_context,
                            match mouse_btn {
                                sdl2::mouse::MouseButton::Left => MouseButton::Left,
                                sdl2::mouse::MouseButton::Middle => MouseButton::Middle,
                                sdl2::mouse::MouseButton::Right => MouseButton::Right,
                                _ => MouseButton::Unknown
                            },
                            x,
                            y,
                            clicks
                        )
                    }
                    sdl2::event::Event::KeyDown { keycode, keymod, repeat, .. } => {
                        if let Some(key_code) = keycode {
                            handler.key_down_event(
                                &mut ctx, &mut window_context,
                                key_code,
                                keymod.into(),
                                repeat
                            );
                        }
                    }
                    sdl2::event::Event::KeyUp { keycode, keymod, .. } => {
                        if let Some(key_code) = keycode {
                            handler.key_up_event(
                                &mut ctx, &mut window_context,
                                key_code,
                                keymod.into()
                            );
                        }
                    }
                    sdl2::event::Event::Window { win_event: WindowEvent::Resized(new_w, new_h), .. } => {
                        ctx.update_window_size(new_w, new_h);
                        handler.resize_event(&mut ctx, &mut window_context, new_w, new_h);
                    }
                    sdl2::event::Event::Window { win_event: WindowEvent::SizeChanged(new_w, new_h), .. } => {
                        ctx.update_window_size(new_w, new_h);
                        handler.resize_event(&mut ctx, &mut window_context, new_w, new_h);
                    }
                    sdl2::event::Event::Window { win_event: WindowEvent::Minimized, .. } => {
                        handler.window_minimized_event(&mut ctx, &mut window_context);
                    }
                    sdl2::event::Event::Window { win_event: WindowEvent::Restored, .. } => {
                        handler.window_restored_event(&mut ctx, &mut window_context);
                    }
                    sdl2::event::Event::Window { win_event: WindowEvent::FocusLost, .. } => {
                        handler.window_lost_focus_event(&mut ctx, &mut window_context);
                    }
                    sdl2::event::Event::Window { win_event: WindowEvent::FocusGained, .. } => {
                        handler.window_gained_focus_event(&mut ctx, &mut window_context);
                    }
                    sdl2::event::Event::Window { win_event: WindowEvent::TakeFocus, .. } => {
                        handler.window_take_focus_event(&mut ctx, &mut window_context);
                    }
                    _ => {}
                }
            }
        }

        handler.update(&mut ctx, &mut window_context);
        handler.draw(&mut ctx, &mut window_context);
        window_context.0.gl_swap_window();
    }
}

pub struct WindowContext(
    sdl2::video::Window,
    sdl2::VideoSubsystem,
    sdl2::mouse::MouseUtil,
    sdl2::EventSubsystem
);

impl WindowContext {
    pub fn get_clipboard_content(&self) -> Option<String> {
        if !self.1.clipboard().has_clipboard_text() {
            None
        } else {
            self.1.clipboard().clipboard_text().ok()
        }
    }

    pub fn set_clipboard_content(&mut self, content: String) {
        self.1.clipboard().set_clipboard_text(&content).unwrap();
    }

    pub fn show_cursor(&mut self) {
        self.2.show_cursor(true);
    }

    pub fn hide_cursor(&mut self) {
        self.2.show_cursor(false);
    }

    pub fn set_system_cursor(&mut self, icon: CursorIcon) {
        self.show_cursor();
        let cursor = match icon {
            CursorIcon::Default => Cursor::from_system(SystemCursor::Arrow),
            CursorIcon::Help => Cursor::from_system(SystemCursor::Arrow),
            CursorIcon::Pointer => Cursor::from_system(SystemCursor::Arrow),
            CursorIcon::Wait => Cursor::from_system(SystemCursor::Wait),
            CursorIcon::Crosshair => Cursor::from_system(SystemCursor::Crosshair),
            CursorIcon::Text => Cursor::from_system(SystemCursor::Arrow),
            CursorIcon::Move => Cursor::from_system(SystemCursor::Hand),
            CursorIcon::NotAllowed => Cursor::from_system(SystemCursor::No),
            CursorIcon::EWResize => Cursor::from_system(SystemCursor::SizeWE),
            CursorIcon::NSResize => Cursor::from_system(SystemCursor::SizeNS),
            CursorIcon::NESWResize => Cursor::from_system(SystemCursor::SizeNESW),
            CursorIcon::NWSEResize => Cursor::from_system(SystemCursor::SizeNWSE),
        }.unwrap();
        cursor.set();
    }

    pub fn quit(&mut self) {
        self.3
            .event_sender()
            .push_event(sdl2::event::Event::Quit { timestamp: 0 })
            .unwrap();
    }
}