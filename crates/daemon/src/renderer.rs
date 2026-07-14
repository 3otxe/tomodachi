

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, watch};
use tracing::{error, info};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId, WindowLevel};

use tomodachi_shared::{CreatureEvent, CreatureState, Mood};

use crate::sprite::{self, WINDOW_SIZE};
use crate::tray;

const IDLE_FPS: u32 = 4;

const ACTIVE_FPS: u32 = 10;

pub fn run_renderer(
    state_rx: watch::Receiver<CreatureState>,
    _event_tx: mpsc::UnboundedSender<CreatureEvent>,
) {
    let event_loop = EventLoop::new().expect("failed to create event loop");

    let mut app = TomodachiApp {
        state_rx,
        window: None,
        surface: None,
        context: None,
        tray: None,
        tray_ids: None,
        tick: 0,
        last_mood: Mood::Idle,
        last_frame: Instant::now(),
        movable: false,
        opacity: 1.0,
    };

    info!("starting winit event loop");
    event_loop.run_app(&mut app).expect("event loop error");
}

struct TomodachiApp {
    state_rx: watch::Receiver<CreatureState>,
    window: Option<Arc<Window>>,
    surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
    context: Option<softbuffer::Context<Arc<Window>>>,
    tray: Option<tray_icon::TrayIcon>,
    tray_ids: Option<tray::TrayMenuIds>,
    tick: u32,
    last_mood: Mood,
    last_frame: Instant,
    movable: bool,
    opacity: f32,
}

impl ApplicationHandler for TomodachiApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return; 
        }

        let monitor = event_loop
            .primary_monitor()
            .or_else(|| event_loop.available_monitors().next());

        let window_size = LogicalSize::new(WINDOW_SIZE, WINDOW_SIZE);

        let window_attrs = Window::default_attributes()
            .with_title("tomodachi")
            .with_inner_size(window_size)
            .with_decorations(false)
            .with_transparent(true)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_resizable(false);

        let window = Arc::new(
            event_loop
                .create_window(window_attrs)
                .expect("failed to create window"),
        );

        if let Some(monitor) = monitor {
            let monitor_size = monitor.size();
            let pos_x = monitor_size.width.saturating_sub(WINDOW_SIZE + 20);
            let pos_y = monitor_size.height.saturating_sub(WINDOW_SIZE + 60); 
            window.set_outer_position(PhysicalPosition::new(pos_x as i32, pos_y as i32));
        }

        let context = softbuffer::Context::new(window.clone()).expect("failed to create context");
        let surface =
            softbuffer::Surface::new(&context, window.clone()).expect("failed to create surface");

        match tray::create_tray() {
            Ok((tray_icon, ids)) => {
                self.tray = Some(tray_icon);
                self.tray_ids = Some(ids);
            }
            Err(e) => {
                error!("failed to create system tray: {}", e);
            }
        }

        self.window = Some(window);
        self.surface = Some(surface);
        self.context = Some(context);

        info!("popup window created");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                self.render_frame();
            }

            WindowEvent::MouseInput { state: winit::event::ElementState::Pressed, button: winit::event::MouseButton::Left, .. } => {
                if self.movable {
                    if let Some(ref window) = self.window {
                        let _ = window.drag_window();
                    }
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        
        if let Some(ref ids) = self.tray_ids {
            if let Some(menu_id) = tray::poll_tray_event() {
                if menu_id == ids.quit_id {
                    info!("quit requested from tray");
                    std::process::exit(0);
                }
                if menu_id == ids.status_id {
                    let state = self.state_rx.borrow().clone();
                    let msg = format!(
                        "Mood:\t{}\nLevel:\t{}\nXP:\t{}\nHP:\t{}/100\nStreak:\t{}\0",
                        state.mood, state.level, state.xp, state.hp, state.streak
                    );
                    
                    std::thread::spawn(move || {
                        unsafe {
                            windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxA(
                                std::ptr::null_mut(),
                                msg.as_ptr() as *const u8,
                                b"\xF0\x9F\x90\xBE Tomodachi Status\0".as_ptr(),
                                windows_sys::Win32::UI::WindowsAndMessaging::MB_OK | windows_sys::Win32::UI::WindowsAndMessaging::MB_ICONINFORMATION,
                            );
                        }
                    });
                }
                
                if menu_id == ids.movable_id {
                    self.movable = !self.movable;
                }
                if menu_id == ids.op_25_id { self.opacity = 0.25; }
                if menu_id == ids.op_50_id { self.opacity = 0.50; }
                if menu_id == ids.op_75_id { self.opacity = 0.75; }
                if menu_id == ids.op_100_id { self.opacity = 1.0; }
            }
        }

        let current_mood = self.state_rx.borrow().mood;
        let fps = if current_mood != self.last_mood {
            self.last_mood = current_mood;
            ACTIVE_FPS
        } else {
            IDLE_FPS
        };

        let frame_duration = Duration::from_millis(1000 / fps as u64);
        let now = Instant::now();

        let state_changed = self.state_rx.has_changed().unwrap_or(false);
        if state_changed {
            let _ = self.state_rx.borrow_and_update(); 
        }

        if state_changed || now.duration_since(self.last_frame) >= frame_duration {
            self.last_frame = now;
            self.tick = self.tick.wrapping_add(1);

            if let Some(ref window) = self.window {
                window.request_redraw();
            }
        }

        event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(self.last_frame + frame_duration));
    }
}

impl TomodachiApp {
    fn render_frame(&mut self) {
        let Some(ref mut surface) = self.surface else {
            return;
        };
        let Some(ref window) = self.window else {
            return;
        };

        let size = window.inner_size();
        let width = match NonZeroU32::new(size.width) {
            Some(w) => w,
            None => return,
        };
        let height = match NonZeroU32::new(size.height) {
            Some(h) => h,
            None => return,
        };

        if surface.resize(width, height).is_err() {
            return;
        }

        let mut buffer = match surface.buffer_mut() {
            Ok(b) => b,
            Err(_) => return,
        };

        let state = self.state_rx.borrow().clone();

        let pixels = sprite::render_sprite(state.mood, self.tick);

        let buf_width = width.get() as usize;
        let buf_height = height.get() as usize;
        let sprite_width = WINDOW_SIZE as usize;

        for y in 0..buf_height.min(sprite_width) {
            for x in 0..buf_width.min(sprite_width) {
                let mut src_pixel = pixels[y * sprite_width + x];
                if self.opacity < 1.0 {
                    let a = (((src_pixel >> 24) & 0xFF) as f32 * self.opacity) as u32;
                    let r = (((src_pixel >> 16) & 0xFF) as f32 * self.opacity) as u32;
                    let g = (((src_pixel >> 8) & 0xFF) as f32 * self.opacity) as u32;
                    let b = ((src_pixel & 0xFF) as f32 * self.opacity) as u32;
                    src_pixel = (a << 24) | (r << 16) | (g << 8) | b;
                }
                
                buffer[y * buf_width + x] = src_pixel;
            }
        }

        let _ = buffer.present();
    }
}
