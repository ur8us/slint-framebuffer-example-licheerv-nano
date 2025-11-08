use linuxfb::{double::Buffer, /*set_terminal_mode,*/ Framebuffer /* , TerminalMode*/};
use slint::{
    platform::{
        software_renderer::{
            MinimalSoftwareWindow, PremultipliedRgbaColor, RepaintBufferType, /*Rgb565Pixel,*/
        },
        Platform,
    },
    PhysicalSize,
};

use std::rc::Rc;
use std::time::Duration;

use evdevil::{
    event::{EventKind, Key, KeyState},
    Evdev,
};

use std::cell::RefCell;

slint::include_modules!();

struct FramebufferPlatform {
    window: Rc<MinimalSoftwareWindow>,
    // fb: Framebuffer,
    buffer: RefCell<Buffer>,
    ev: Evdev,
    ui: slint::Weak<AppWindow>,
    stride: usize,
}

impl FramebufferPlatform {
    // fn new(fb: Framebuffer, ev: Evdev, ui: slint::Weak<AppWindow>) -> Self {
    fn new(buffer: Buffer, ev: Evdev, ui: slint::Weak<AppWindow>) -> Self {
        //        let size = fb.get_size();
        let size = (buffer.width, buffer.height);
        let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
        window.set_size(PhysicalSize::new(size.0, size.1));
        Self {
            window,
            // fb,
            buffer: RefCell::new(buffer),
            ev,
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
        loop {
            slint::platform::update_timers_and_animations();

            // Draw the screen with double buffering

            let mut buffer = self.buffer.borrow_mut(); // Get the mutable buffer from the RefCell (example of interior mutability)

            self.window.draw_if_needed(|renderer| {
                let frame = buffer.as_mut_slice() as &mut [u8];

                let (_, pixels, _) = unsafe {
                    frame.align_to_mut::</*Rgb565Pixel*/PremultipliedRgbaColor>()
                };
                renderer.render(pixels, self.stride);

                buffer.flip().unwrap(); // Flip the display so the new screen becomes visible
            });

            // Handle button event, emulate button click

            if self.ev.is_readable().unwrap_or(false) {
                for event in self.ev.raw_events() {
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

    let /*mut*/ fb = Framebuffer::new(fb_path).expect("open framebuffer");
    let /*mut*/ buffer = linuxfb::double::Buffer::new(fb).unwrap();

    // let ly = fb.get_pixel_layout();
    // println!("ly: {:?}", ly); // returns ABGR, need RGBA

    // let bytes_pp = fb.get_bytes_per_pixel();
    // println!("bytes_pp: {}", bytes_pp);

    // _ = fb.set_bytes_per_pixel(2);

    let ev = Evdev::open("/dev/input/event0").expect("open event");
    _ = ev.set_nonblocking(true);

    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();

    // Instruct slint to use the FramebufferPlatform
    //    slint::platform::set_platform(Box::new(FramebufferPlatform::new(fb, ev, ui_handle)))
    slint::platform::set_platform(Box::new(FramebufferPlatform::new(buffer, ev, ui_handle)))
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
        }
    });

    ui.run()
}
