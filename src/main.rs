use linuxfb::{double::Buffer, /*set_terminal_mode,*/ Framebuffer /* , TerminalMode*/};
use slint::{
    platform::{
        software_renderer::{
            MinimalSoftwareWindow, PremultipliedRgbaColor, RepaintBufferType, Rgb565Pixel,
        },
        Platform, PointerEventButton, WindowEvent,
    },
    PhysicalSize,
};

use std::rc::Rc;
use std::time::Duration;

use evdevil::{
    event::{Abs, AbsEvent, EventKind, Key, KeyState},
    Evdev,
};

use std::cell::RefCell;

#[cfg(feature = "use_double_buffering")]
type BufferType = Buffer;
#[cfg(not(feature = "use_double_buffering"))]
type BufferType = Framebuffer;

#[cfg(feature = "color_32bit")]
type ColorType = PremultipliedRgbaColor;
#[cfg(not(feature = "color_32bit"))]
type ColorType = Rgb565Pixel;

slint::include_modules!();

struct FramebufferPlatform {
    window: Rc<MinimalSoftwareWindow>,
    // fb: Framebuffer,
    buffer: RefCell<BufferType>,
    ev_keys: Evdev,
    ev_touch: Option<Evdev>,
    ui: slint::Weak<AppWindow>,
    stride: usize,
}

impl FramebufferPlatform {
    // fn new(fb: Framebuffer, ev: Evdev, ui: slint::Weak<AppWindow>) -> Self {
    fn new(
        buffer: BufferType,
        ev_keys: Evdev,
        ev_touch: Option<Evdev>,
        ui: slint::Weak<AppWindow>,
    ) -> Self {
        #[cfg(feature = "use_double_buffering")]
        let size = (buffer.width, buffer.height);

        #[cfg(not(feature = "use_double_buffering"))]
        let size = buffer.get_size();

        let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
        window.set_size(PhysicalSize::new(size.0, size.1));
        Self {
            window,
            buffer: RefCell::new(buffer),
            ev_keys,
            ev_touch,
            ui,
            stride: size.0 as usize,
        }
    }
}

impl Platform for FramebufferPlatform {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }

    fn run_event_loop(&self) -> Result<(), slint::PlatformError> {
        let mut touch_last_x = None;
        let mut touch_last_y = None;
        let mut touch_just_pressed = false;

        loop {
            let mut touch_event_processed = false;

            slint::platform::update_timers_and_animations();

            // Draw the screen with double buffering

            // #[cfg(feature = "use_double_buffering")]
            let mut buffer = self.buffer.borrow_mut(); // Get the mutable buffer from the RefCell (example of interior mutability)

            self.window.draw_if_needed(|renderer| {
                #[cfg(feature = "use_double_buffering")]
                let frame = buffer.as_mut_slice();

                #[cfg(not(feature = "use_double_buffering"))]
                let mut frame = buffer.map().unwrap();

                let (_, pixels, _) = unsafe { frame.align_to_mut::<ColorType>() };
                renderer.render(pixels, self.stride);

                #[cfg(feature = "use_double_buffering")]
                buffer.flip().unwrap(); // Flip the display so the new screen becomes visible
            });

            // Handle button event, emulate button click

            if self.ev_keys.is_readable().unwrap_or(false) {
                for event in self.ev_keys.raw_events() {
                    if let Ok(ie) = event {
                        if let EventKind::Key(k) = ie.kind() {
                            if k.key() == Key::KEY_DISPLAYTOGGLE && k.state() == KeyState::PRESSED {
                                if let Some(ui) = self.ui.upgrade() {
                                    ui.set_counter(ui.get_counter() + 1);
                                }
                            }
                        }
                    }
                }
            }

            // Handle touch events

            if let Some(evt) = &self.ev_touch {
                if evt.is_readable().unwrap_or(false) {
                    for event in evt.raw_events() {
                        // let ref z = event.map_err(|_| ());

                        if let Ok(ie) = event {
                            {
                                // println!("{:?}", ie);

                                match ie.kind() {
                                    EventKind::Abs(a) => {
                                        match a.abs() {
                                            Abs::MT_POSITION_X => {
                                                touch_last_x = Some(a.value());
                                            }

                                            Abs::MT_POSITION_Y => {
                                                touch_last_y = Some(a.value());
                                            }

                                            _ => (),
                                        }

                                        if touch_last_x.is_some() && touch_last_y.is_some() {
                                            let position = slint::PhysicalPosition::new(
                                                touch_last_x.unwrap(),
                                                touch_last_y.unwrap(),
                                            )
                                            .to_logical(self.window.scale_factor());

                                            self.window.dispatch_event(WindowEvent::PointerMoved {
                                                position,
                                            });

                                            if touch_just_pressed {
                                                self.window.dispatch_event(
                                                    WindowEvent::PointerPressed {
                                                        position,
                                                        button: PointerEventButton::Left,
                                                    },
                                                );
                                                touch_just_pressed = false;
                                            }

                                            touch_event_processed = true;
                                        }
                                    }

                                    EventKind::Key(k) => {
                                        if k.key() == Key::BTN_TOUCH {
                                            match k.state() {
                                                KeyState::PRESSED => {
                                                    touch_just_pressed = true;
                                                }

                                                KeyState::RELEASED => {
                                                    if touch_last_x.is_some()
                                                        && touch_last_y.is_some()
                                                    {
                                                        let position =
                                                            slint::PhysicalPosition::new(
                                                                touch_last_x.unwrap(),
                                                                touch_last_y.unwrap(),
                                                            )
                                                            .to_logical(self.window.scale_factor());

                                                        self.window.dispatch_event(
                                                            WindowEvent::PointerReleased {
                                                                position,
                                                                button: PointerEventButton::Left,
                                                            },
                                                        );

                                                        touch_event_processed = true;
                                                    }

                                                    self.window
                                                        .dispatch_event(WindowEvent::PointerExited);

                                                    touch_last_x = None;
                                                    touch_last_y = None;
                                                    touch_just_pressed = false;
                                                }

                                                _ => {}
                                            }
                                        }
                                    }

                                    _ => {}
                                }
                                // }
                            }
                        }
                    }
                }
            }

            // if let z  = self.ev_touch;

            if touch_event_processed {
                continue;
            }

            if !self.window.has_active_animations() {
                std::thread::sleep(
                    slint::platform::duration_until_next_timer_update()
                        .unwrap_or(Duration::from_millis(1)),
                );
            }
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    // TODO: adjust these values to match your system:

    // Path of the current TTY. Used to switch the terminal to graphics mode
    // and back to text mode.
    // let tty_path = "/dev/tty1";
    // Path to the framebuffer device. Normally this is fb0.
    // I'm using a `fbtft` based display on the RaspberryPi, which shows up
    // as fb1 (fb0 is raspi's builtin graphics card).
    let fb_path = "/dev/fb0";

    // Switch back to text mode when terminating
    // ctrlc::set_handler(move || {
    //     let tty = std::fs::File::open(tty_path).unwrap();
    //     set_terminal_mode(&tty, TerminalMode::Text).expect("switch to text mode");
    //     std::process::exit(1);
    // })
    // .expect("install signal handlers");

    let /*mut*/ buffer = Framebuffer::new(fb_path).expect("open framebuffer");

    #[cfg(feature = "use_double_buffering")]
    let /*mut*/ buffer = linuxfb::double::Buffer::new(buffer).unwrap();

    // let ly = fb.get_pixel_layout();
    // println!("ly: {:?}", ly); // returns ABGR, need RGBA

    // let bytes_pp = fb.get_bytes_per_pixel();
    // println!("bytes_pp: {}", bytes_pp);

    // _ = fb.set_bytes_per_pixel(2);

    let ev_keys = Evdev::open("/dev/input/event0").expect("open event"); // Keys are always present
    _ = ev_keys.set_nonblocking(true);

    let mut ev_touch = None; // Touch panel may be absent
    if let Ok(evt) = Evdev::open("/dev/input/event1") {
        _ = evt.set_nonblocking(true);
        ev_touch = Some(evt); // Touch panel is present
    }

    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();

    // Instruct slint to use the FramebufferPlatform
    //    slint::platform::set_platform(Box::new(FramebufferPlatform::new(fb, ev, ui_handle)))
    slint::platform::set_platform(Box::new(FramebufferPlatform::new(
        buffer, ev_keys, ev_touch, ui_handle,
    )))
    .expect("set platform");

    // Switch terminal to graphics mode
    // let tty = std::fs::File::open(tty_path).expect("open TTY");
    // set_terminal_mode(&tty, TerminalMode::Graphics).expect("switch to graphics mode");
    // drop(tty);

    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
            print!("+1");
        }
    });

    ui.run()
}
