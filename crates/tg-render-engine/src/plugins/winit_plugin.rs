use std::time::Instant;

use crate::{
    events::Events,
    winit::{
        event::{CursorMoved, WindowResized},
        input::MouseMotion,
        winit_config::UpdateMode,
    },
    App, Plugin,
};
use nalgebra_glm::{DVec2, Vec2};
use specs::WorldExt;
use winit::{
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

use crate::winit::{windows::Windows, winit_config::WinitSettings, winit_windows::WinitWindows};

#[derive(Default)]
pub struct WinitPlugin;

impl Plugin for WinitPlugin {
    fn build(&self, app: &mut App) {
        app.world.insert(WinitSettings::default());
        app.set_runner(winit_runner);
    }
}
pub fn winit_runner(app: App) {
    winit_runner_with(app);
}

/// Stores state that must persist between frames.
struct WinitPersistentState {
    /// Tracks whether or not the application is active or suspended.
    active: bool,
    /// Tracks whether or not an event has occurred this frame that would trigger an update in low
    /// power mode. Should be reset at the end of every frame.
    low_power_event: bool,
    /// Tracks whether the event loop was started this frame because of a redraw request.
    redraw_request_sent: bool,
    /// Tracks if the event loop was started this frame because of a `WaitUntil` timeout.
    timeout_reached: bool,
    last_update: Instant,
}
impl Default for WinitPersistentState {
    fn default() -> Self {
        Self {
            active: true,
            low_power_event: false,
            redraw_request_sent: false,
            timeout_reached: false,
            last_update: Instant::now(),
        }
    }
}
pub fn winit_runner_with(mut app: App) {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("title")
        .build(&event_loop)
        .unwrap();
    let mut winit_state = WinitPersistentState::default();

    let event_handler = move |event: Event<()>,
                              event_loop: &EventLoopWindowTarget<()>,
                              control_flow: &mut ControlFlow| {
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            // NEW!
            Event::NewEvents(start) => {
                let winit_config = app.world.read_resource::<WinitSettings>();
                let windows = app.world.read_resource::<Windows>();
                let focused = windows.iter().any(|w| w.is_focused());
                // Check if either the `WaitUntil` timeout was triggered by winit, or that same
                // amount of time has elapsed since the last app update. This manual check is needed
                // because we don't know if the criteria for an app update were met until the end of
                // the frame.
                let auto_timeout_reached = matches!(start, StartCause::ResumeTimeReached { .. });
                let now = Instant::now();
                let manual_timeout_reached = match winit_config.update_mode(focused) {
                    UpdateMode::Continuous => false,
                    UpdateMode::Reactive { max_wait }
                    | UpdateMode::ReactiveLowPower { max_wait } => {
                        now.duration_since(winit_state.last_update) >= *max_wait
                    }
                };
                // The low_power_event state and timeout must be reset at the start of every frame.
                winit_state.low_power_event = false;
                winit_state.timeout_reached = auto_timeout_reached || manual_timeout_reached;
            }
            // UPDATED!
            Event::WindowEvent {
                ref event,
                window_id: winit_window_id,
            } => {
                let winit_windows = app.world.write_resource::<WinitWindows>();
                let mut windows = app.world.write_resource::<Windows>();
                let window_id =
                    if let Some(window_id) = winit_windows.get_window_id(winit_window_id) {
                        window_id
                    } else {
                        return;
                    };

                let window = if let Some(window) = windows.get_mut(window_id) {
                    window
                } else {
                    return;
                };
                winit_state.low_power_event = true;

                match event {
                    WindowEvent::Resized(size) => {
                        window.update_actual_size_from_backend(size.width, size.height);
                        let mut r = app.world.write_resource::<Events<WindowResized>>();
                        r.send(WindowResized {
                            id: window_id,
                            width: window.width(),
                            height: window.height(),
                        });
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        let mut keyboard_input_events =
                            app.world.write_resource::<Events<KeyboardInput>>();
                        keyboard_input_events.send(*input);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let mut cursor_moved_events =
                            app.world.write_resource::<Events<CursorMoved>>();
                        let winit_window = winit_windows.get_window(window_id).unwrap();
                        let inner_size = winit_window.inner_size();

                        // move origin to bottom left
                        let y_position = inner_size.height as f64 - position.y;

                        let physical_position = DVec2::new(position.x, y_position);
                        window
                            .update_cursor_physical_position_from_backend(Some(physical_position));
                        let factor = window.scale_factor();
                        let new_position = Vec2::new(
                            (physical_position.x / factor) as f32,
                            (physical_position.y / factor) as f32,
                        );
                        cursor_moved_events.send(CursorMoved {
                            id: window_id,
                            position: new_position,
                        });
                    }
                    _ => {}
                }
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                let mut mouse_motion_events = app.world.write_resource::<Events<MouseMotion>>();
                mouse_motion_events.send(MouseMotion {
                    delta: Vec2::new(delta.0 as f32, delta.1 as f32),
                });
            }
            Event::Suspended => {
                winit_state.active = false;
            }
            Event::Resumed => {
                winit_state.active = true;
            }
            _ => {}
        }
    };
    run(event_loop, event_handler);
}
fn run<F>(event_loop: EventLoop<()>, event_handler: F) -> !
where
    F: 'static + FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow),
{
    event_loop.run(event_handler)
}
