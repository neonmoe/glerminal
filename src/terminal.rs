//! This module acts as the window and "canvas" of the terminal, handling most behind-the-sceneries
//!
//! The [`Terminal`](struct.Terminal.html) is used to create the window and canvas for the [`TextBuffer`](text_buffer/struct.TextBuffer.html)
//! which can then draw it, close the window, reset the title of the window or handle input.
//!
//! **Note** when building with debug-mode, you are able to press `F3` to toggle between debug and non-debug. see ([`Terminal`](struct.Terminal.html#method.set_debug)) for more information.
//!
//! ### Terminal example:
//! ```no_run
//! use glerminal::terminal::TerminalBuilder;
//!
//! let terminal = TerminalBuilder::new()
//!     .with_title("Hello GLerminal!")
//!     .with_dimensions((1280, 720))
//!     .build();
//! ```
//!
//! ### `let mut terminal` vs `let terminal`
//! In most cases you might just want to initialize the terminal as immutable, but in some, you will need to initialize it as mutable,
//! allowing it to run some additional methods, such as `.show()` and `.set_title("title")`
//!
//! #### Example of a mutable terminal:
//! ```no_run
//! use glerminal::terminal::TerminalBuilder;
//!
//! let mut terminal = TerminalBuilder::new()
//!     .with_title("Hello GLerminal!")
//!     .with_dimensions((1280, 720))
//!     .with_visibility(false)
//!     .build();
//!
//! terminal.set_title("Changed title!");
//! terminal.show();
//! ```

#[allow(unused_imports)]
use glutin::VirtualKeyCode;
use std::cell::{Cell, RefCell};
use std::time::{Duration, SystemTime};

use display::Display;
use font::Font;
use input::Input;
use renderer;
use text_buffer::TextBuffer;

static IOSEVKA_SFL: &'static str = include_str!("../fonts/iosevka.sfl");
static IOSEVKA_PNG: &'static [u8] = include_bytes!("../fonts/iosevka.png");

/// A builder for the `Terminal`. Includes some settings that can be set before building.
///
/// See [terminal mod](index.html) for examples and more detailed documentation.
pub struct TerminalBuilder {
    title: String,
    dimensions: (u32, u32),
    clear_color: (f32, f32, f32, f32),
    font: Font,
    visibility: bool,
    headless: bool,
    text_buffer_aspect_ratio: bool,
}

#[allow(dead_code)]
impl TerminalBuilder {
    /// Creates a new terminal builder with default settings.
    pub fn new() -> TerminalBuilder {
        TerminalBuilder {
            title: "Hello, World ! ".to_owned(),
            dimensions: (1280, 720),
            clear_color: (0.14, 0.19, 0.28, 1.0),
            font: Font::load_raw(IOSEVKA_SFL, IOSEVKA_PNG),
            visibility: true,
            headless: false,
            text_buffer_aspect_ratio: true,
        }
    }

    /// Sets the title for the `Terminal`.
    pub fn with_title<T: Into<String>>(mut self, title: T) -> TerminalBuilder {
        self.title = title.into();
        self
    }

    /// Sets the dimensions the `Terminal` is to be opened with.
    pub fn with_dimensions(mut self, dimensions: (u32, u32)) -> TerminalBuilder {
        self.dimensions = dimensions;
        self
    }

    /// Sets the clear color of the terminal.
    pub fn with_clear_color(mut self, clear_color: (f32, f32, f32, f32)) -> TerminalBuilder {
        self.clear_color = clear_color;
        self
    }

    /// Changes the font that the terminal uses.
    pub fn with_font(mut self, font: Font) -> TerminalBuilder {
        self.font = font;
        self
    }

    /// Changes the visibility that the terminal will be opened with. If headless, visibility will not matter.
    pub fn with_visibility(mut self, visibility: bool) -> TerminalBuilder {
        self.visibility = visibility;
        self
    }

    /// Changes the visibility that the terminal will be opened with
    pub fn with_headless(mut self, headless: bool) -> TerminalBuilder {
        self.headless = headless;
        self
    }

    /// Changes whether the aspect ratio should be retrieved from TextBuffer instead of the original resolution of the screen.
    ///
    /// If set to false, the aspect ratio used to make black bars for the screen will be fetched from the original resolution of the screen;
    /// This will cause the fonts to strech a bit though.
    ///
    /// If set to true (default), the aspect ratio will be fetched from the TextBuffer, causing almost any resolution
    /// to have black bars to make up for the missing spaces.
    pub fn with_text_buffer_aspect_ratio(mut self, tbar: bool) -> TerminalBuilder {
        self.text_buffer_aspect_ratio = tbar;
        self
    }

    /// Builds the actual terminal and opens the window
    pub fn build(self) -> Terminal {
        Terminal::new(
            self.title,
            self.dimensions,
            self.clear_color,
            self.font,
            self.visibility,
            self.headless,
            self.text_buffer_aspect_ratio,
        )
    }
}

/// Represents the Terminal itself.
///
/// See [terminal mod](index.html) for examples and more detailed documentation.
pub struct Terminal {
    display: Option<Display>,
    program: renderer::Program,
    background_program: renderer::Program,
    debug_program: renderer::Program,
    debug: Cell<bool>,
    running: Cell<bool>,
    pub(crate) headless: bool,
    since_start: SystemTime,
    pub(crate) font: Font,
    frame_counter: RefCell<FrameCounter>,
    text_buffer_aspect_ratio: bool,
}

impl Terminal {
    fn new<T: Into<String>>(
        title: T,
        window_dimensions: (u32, u32),
        clear_color: (f32, f32, f32, f32),
        font: Font,
        visibility: bool,
        headless: bool,
        text_buffer_aspect_ratio: bool,
    ) -> Terminal {
        let display;
        let program;
        let background_program;
        let debug_program;
        if headless {
            display = None;
            program = 0;
            background_program = 0;
            debug_program = 0;
        } else {
            display = Some(Display::new(
                title,
                window_dimensions,
                clear_color,
                visibility,
            ));
            program = renderer::create_program(renderer::VERT_SHADER, renderer::FRAG_SHADER);
            background_program =
                renderer::create_program(renderer::VERT_SHADER, renderer::BG_FRAG_SHADER);
            debug_program =
                renderer::create_program(renderer::VERT_SHADER, renderer::DEBUG_FRAG_SHADER);
        }
        let font = font;
        Terminal {
            display,
            program,
            background_program,
            debug_program,
            debug: Cell::new(false),
            running: Cell::new(true),
            headless,
            since_start: SystemTime::now(),
            font,
            frame_counter: RefCell::new(FrameCounter::new()),
            text_buffer_aspect_ratio,
        }
    }

    /// Sets debug mode (changes characters and backgrounds into wireframe)
    pub fn set_debug(&self, debug: bool) {
        if !self.headless {
            renderer::set_debug(debug);
            self.debug.set(debug);
        }
    }

    /// Refreshes the screen and returns weather the while-loop should continue (is the program running)
    #[cfg(debug_assertions)]
    pub fn refresh(&self) -> bool {
        let mut frame_counter = self.frame_counter.borrow_mut();
        frame_counter.update();
        drop(frame_counter);

        if let Some(ref display) = self.display {
            let input = self.get_current_input();
            if input.was_just_pressed(VirtualKeyCode::F3) {
                self.set_debug(!self.debug.get());
            }
            display.refresh() && self.running.get()
        } else {
            self.running.get()
        }
    }

    /// Refreshes the screen and returns weather the while-loop should continue (is the program running)
    #[cfg(not(debug_assertions))]
    pub fn refresh(&self) -> bool {
        let mut frame_counter = self.frame_counter.borrow_mut();
        frame_counter.update();
        drop(frame_counter);

        if let Some(ref display) = self.display {
            display.refresh() && self.running.get()
        } else {
            self.running.get()
        }
    }

    /// Flushes `TextBuffer`, taking it's character-grid and making it show for the next draw.
    ///
    /// This is quite a heavy function and it's calling should be avoided when unnecessary.
    pub fn flush(&self, text_buffer: &mut TextBuffer) {
        text_buffer.swap_buffers(&self.font);
    }

    /// Draws the `TextBuffer`, this should be called every time in the while-loop.
    pub fn draw(&self, text_buffer: &TextBuffer) {
        if let (&Some(ref display), &Some(ref mesh), &Some(ref background_mesh)) = (
            &self.display,
            &text_buffer.mesh,
            &text_buffer.background_mesh,
        ) {
            if self.text_buffer_aspect_ratio
                && text_buffer.aspect_ratio != display.get_aspect_ratio()
            {
                display.set_aspect_ratio(text_buffer.aspect_ratio);
            }
            renderer::clear();
            let duration = SystemTime::now().duration_since(self.since_start).unwrap();

            let time = duration.as_secs() as f32 + duration.subsec_nanos() as f32 / 1_000_000_000.0;

            renderer::draw(
                self.get_background_program(),
                display.proj_matrix.get(),
                time,
                background_mesh,
            );
            renderer::draw(self.get_program(), display.proj_matrix.get(), time, mesh);
        }
    }

    /// Draws the `TextBuffer`s, this should be called every time in
    /// the while-loop. (Use this instead of `draw` if you need to
    /// draw multiple `TextBuffer`s.)
    pub fn draw_multiple(&self, text_buffers: Vec<&TextBuffer>) {
        renderer::clear();
        for text_buffer in text_buffers {
            if let (&Some(ref display), &Some(ref mesh), &Some(ref background_mesh)) = (
                &self.display,
                &text_buffer.mesh,
                &text_buffer.background_mesh,
            ) {
                if self.text_buffer_aspect_ratio
                    && text_buffer.aspect_ratio != display.get_aspect_ratio()
                {
                    display.set_aspect_ratio(text_buffer.aspect_ratio);
                }
                let duration = SystemTime::now().duration_since(self.since_start).unwrap();

                let time =
                    duration.as_secs() as f32 + duration.subsec_nanos() as f32 / 1_000_000_000.0;

                renderer::draw(
                    self.get_background_program(),
                    display.proj_matrix.get(),
                    time,
                    background_mesh,
                );
                renderer::draw(self.get_program(), display.proj_matrix.get(), time, mesh);
            }
        }
    }

    /// Gets the current Input, must be retrieved every time you want new inputs. (ie. every frame)
    pub fn get_current_input(&self) -> Input {
        if let Some(ref display) = self.display {
            display.get_current_input()
        } else {
            Input::new()
        }
    }

    /// Closes the Terminal
    pub fn close(&self) {
        self.running.set(false);
    }

    /// Sets the title for the window.
    ///
    /// **Warning:** This is a nuclear hazard (takes up a lot of performance), it might melt down your computer if called every frame (or so).
    pub fn set_title<T: Into<String>>(&mut self, title: T) {
        if let Some(ref mut display) = self.display {
            display.set_title(&title.into());
        }
    }

    /// Shows the window, if it's hidden
    pub fn show(&mut self) {
        if let Some(ref mut display) = self.display {
            display.show();
        }
    }

    /// Returns the current fps; updates every second
    pub fn get_fps(&self) -> f32 {
        self.frame_counter.borrow().get_fps()
    }

    pub(crate) fn get_program(&self) -> renderer::Program {
        if self.headless {
            panic!("Unable to get program from headless terminal");
        }
        if !self.debug.get() {
            self.program
        } else {
            self.debug_program
        }
    }

    pub(crate) fn get_background_program(&self) -> renderer::Program {
        if self.headless {
            panic!("Unable to get program from headless terminal");
        }
        if !self.debug.get() {
            self.background_program
        } else {
            self.debug_program
        }
    }

    #[cfg(test)]
    pub(crate) fn update_virtual_keycode(&mut self, keycode: VirtualKeyCode, pressed: bool) {
        if let Some(ref mut display) = self.display {
            display.update_virtual_keycode(keycode, pressed);
        }
    }
}

pub(crate) struct FrameCounter {
    frames: u32,
    last_check: SystemTime,
    fps: f32,
}

impl FrameCounter {
    pub fn new() -> FrameCounter {
        FrameCounter {
            frames: 0,
            last_check: SystemTime::now(),
            fps: 0.0,
        }
    }

    pub fn update(&mut self) {
        self.frames += 1;
        let current_time = SystemTime::now();
        if current_time.duration_since(self.last_check).unwrap() > Duration::from_secs(1) {
            self.fps = self.frames as f32;
            self.last_check = current_time;
            self.frames = 0;
        }
    }

    pub fn get_fps(&self) -> f32 {
        self.fps
    }
}
