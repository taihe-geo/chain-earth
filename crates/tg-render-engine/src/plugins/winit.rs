use std::time::Instant;

use crate::{
    events::{event, Events, ManualEventReader},
    winit::{
        event::{
            CreateWindow, CursorMoved, RequestRedraw, WindowCloseRequested, WindowCreated,
            WindowResized,
        },
        input::MouseMotion,
        window::{WindowDescriptor, WindowId},
        windows::{Windows},
        winit_config::{UpdateMode,WinitSettings},
        winit_windows::{WinitWindows}
    },
    App, Plugin,
};
use nalgebra_glm::{DVec2, Vec2};
use specs::{Read, ReadStorage, System, WorldExt, Write, WriteStorage};
use winit::{
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

/// An event that indicates the app should exit. This will fully exit the app process.
#[derive(Debug, Clone, Default)]
pub struct AppExit;
pub struct ExitOnWindowCloseSystem;
impl<'a> System<'a> for ExitOnWindowCloseSystem {
    type SystemData = (
        Write<'a, Events<AppExit>>,
        Read<'a, Events<WindowCloseRequested>>,
    );

    fn run(&mut self, (mut app_exit_events, window_close_requested_events): Self::SystemData) {
        let mut reader = ManualEventReader::<WindowCloseRequested>::default();
        if reader.iter(&window_close_requested_events).next().is_some() {
            app_exit_events.send(AppExit);
        }
    }
}

pub struct WinitPlugin {
    pub add_primary_window: bool,
    pub exit_on_close: bool,
}
impl Default for WinitPlugin {
    fn default() -> Self {
        WinitPlugin {
            add_primary_window: true,
            exit_on_close: true,
        }
    }
}

impl Plugin for WinitPlugin {
    fn build(&self, app: &mut App) {
        app.world.insert(WinitSettings::default());
        app.world.insert(WinitWindows::default());
        app.world.insert(Windows::default());
        app.world.insert(Events::<KeyboardInput>::default());
        app.world.insert(Events::<CursorMoved>::default());
        app.world.insert(Events::<WindowResized>::default());
        app.world.insert(Events::<MouseMotion>::default());
        app.world.insert(Events::<CreateWindow>::default());
        app.world.insert(Events::<WindowCreated>::default());
        app.world.insert(Events::<RequestRedraw>::default());
        app.world.insert(Events::<AppExit>::default());
        app.world.insert(Events::<WindowCloseRequested>::default());
        app.world.insert(WindowDescriptor::default());
        if self.add_primary_window {
            let mut window_descriptor = app.world.write_resource::<WindowDescriptor>();
            let mut create_window_event = app.world.write_resource::<Events<CreateWindow>>();
            create_window_event.send(CreateWindow {
                id: WindowId::primary(),
                descriptor: (*window_descriptor).clone(),
            });
        }

        if self.exit_on_close {
            // app.add_system(exit_on_window_close_system);
            app.add_add_systems(|dispaptch_builder| {
                dispaptch_builder.add(ExitOnWindowCloseSystem, "ExitOnWindowCloseSystem", &[]);
            });
        }
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
    handle_initial_window_events(&mut app, &event_loop);
    let mut create_window_event_reader = ManualEventReader::<CreateWindow>::default();
    let mut app_exit_event_reader = ManualEventReader::<AppExit>::default();
    let mut redraw_event_reader = ManualEventReader::<RequestRedraw>::default();
    let mut winit_state = WinitPersistentState::default();
    let event_handler = move |event: Event<()>,
                              event_loop: &EventLoopWindowTarget<()>,
                              control_flow: &mut ControlFlow| {
        match event {
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
                    // WindowEvent::CloseRequested
                    // | WindowEvent::KeyboardInput {
                    //     input:
                    //         KeyboardInput {
                    //             state: ElementState::Pressed,
                    //             virtual_keycode: Some(VirtualKeyCode::Escape),
                    //             ..
                    //         },
                    //     ..
                    // } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(size) => {
                        window.update_actual_size_from_backend(size.width, size.height);
                        let mut r = app.world.write_resource::<Events<WindowResized>>();
                        r.send(WindowResized {
                            id: window_id,
                            width: window.width(),
                            height: window.height(),
                        });
                    }
                    WindowEvent::CloseRequested => {
                        let mut window_close_requested_events =
                            app.world.write_resource::<Events<WindowCloseRequested>>();
                        window_close_requested_events.send(WindowCloseRequested { id: window_id });
                        // *control_flow = ControlFlow::Exit;
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
            Event::RedrawEventsCleared => {
                {
                    let winit_config = app.world.write_resource::<WinitSettings>();
                    let windows = app.world.write_resource::<Windows>();
                    let focused = windows.iter().any(|w| w.is_focused());
                    let now = Instant::now();
                    use UpdateMode::*;
                    *control_flow = match winit_config.update_mode(focused) {
                        Continuous => ControlFlow::Poll,
                        Reactive { max_wait } | ReactiveLowPower { max_wait } => {
                            ControlFlow::WaitUntil(now + *max_wait)
                        }
                    };
                }
                // This block needs to run after `app.update()` in `MainEventsCleared`. Otherwise,
                // we won't be able to see redraw requests until the next event, defeating the
                // purpose of a redraw request!
                let mut redraw = false;
                let app_redraw_events = app.world.write_resource::<Events<RequestRedraw>>();
                if redraw_event_reader
                    .iter(&app_redraw_events)
                    .last()
                    .is_some()
                {
                    *control_flow = ControlFlow::Poll;
                    redraw = true;
                }
                let app_exit_events = app.world.write_resource::<Events<AppExit>>();
                if app_exit_event_reader
                    .iter(&app_exit_events)
                    .last()
                    .is_some()
                {
                    *control_flow = ControlFlow::Exit;
                }
                winit_state.redraw_request_sent = redraw;
            }
            Event::MainEventsCleared => {
                handle_create_window_events(&mut app, event_loop, &mut create_window_event_reader);
                let update = if winit_state.active {
                    let winit_config = app.world.write_resource::<WinitSettings>();
                    let windows = app.world.write_resource::<Windows>();
                    let focused = windows.iter().any(|w| w.is_focused());
                    match winit_config.update_mode(focused) {
                        UpdateMode::Continuous | UpdateMode::Reactive { .. } => true,
                        UpdateMode::ReactiveLowPower { .. } => {
                            winit_state.low_power_event
                                || winit_state.redraw_request_sent
                                || winit_state.timeout_reached
                        }
                    }
                } else {
                    false
                };
                if update {
                    winit_state.last_update = Instant::now();
                    app.update();
                }
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
fn handle_create_window_events(
    app: &mut App,
    event_loop: &EventLoopWindowTarget<()>,
    create_window_event_reader: &mut ManualEventReader<CreateWindow>,
) {
    let mut winit_windows = app.world.write_resource::<WinitWindows>();
    let mut windows = app.world.write_resource::<Windows>();
    let create_window_events = app.world.write_resource::<Events<CreateWindow>>();
    let mut window_created_events = app.world.write_resource::<Events<WindowCreated>>();
    for create_window_event in create_window_event_reader.iter(&create_window_events) {
        let window = winit_windows.create_window(
            event_loop,
            create_window_event.id,
            &create_window_event.descriptor,
        );
        windows.add(window);
        window_created_events.send(WindowCreated {
            id: create_window_event.id,
        });
    }
}

fn handle_initial_window_events(app: &mut App, event_loop: &EventLoop<()>) {
    let mut winit_windows = app.world.write_resource::<WinitWindows>();
    let mut create_window_events = app.world.write_resource::<Events<CreateWindow>>();
    let mut windows = app.world.write_resource::<Windows>();
    let mut window_created_events = app.world.write_resource::<Events<WindowCreated>>();

    for create_window_event in create_window_events.drain() {
        let window = winit_windows.create_window(
            event_loop,
            create_window_event.id,
            &create_window_event.descriptor,
        );
        windows.add(window);
        window_created_events.send(WindowCreated {
            id: create_window_event.id,
        });
    }
}
