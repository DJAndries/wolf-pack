use cubik::input::{InputListener, process_input_event, center_cursor};
use cubik::container::RenderContainer;
use cubik::glium::glutin::event_loop::{EventLoop, ControlFlow};
use cubik::glium::glutin::event::{Event, WindowEvent, VirtualKeyCode, ElementState, StartCause};
use crate::game_client::GameClient;

pub fn start_client(fullscreen: bool, host: String, username: String, fps_count_enabled: bool, input_switcher_enabled: bool) {
	let event_loop = EventLoop::new();
	let mut ctr = RenderContainer::new(&event_loop, 1280, 720, "Wolf Pack", fullscreen);

	let mut input_enabled = true;

	let mut game_client = Some(GameClient::init(&mut ctr, host, username, fps_count_enabled).unwrap());

	event_loop.run(move |ev, _, control_flow| {
		let listeners: Vec<&mut dyn InputListener> = if let Some(game_client) = game_client.as_mut() {
			vec![&mut game_client.player]
		} else {
			Vec::new()
		};

		*control_flow = ControlFlow::Poll;
		match ev {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::CloseRequested => {
					*control_flow = ControlFlow::Exit;
					return;
				},
				WindowEvent::KeyboardInput { input, .. } => {
					if let Some(keycode) = input.virtual_keycode {
						match keycode {
							VirtualKeyCode::Escape => {
								*control_flow = ControlFlow::Exit;
								return;
							},
							VirtualKeyCode::T => {
								if input_switcher_enabled && input.state == ElementState::Released {
									input_enabled = !input_enabled;
									let gl_window = ctr.display.gl_window();
									let window = gl_window.window();
									window.set_cursor_visible(!input_enabled);
								}
								return;
							}
							_ => ()
						};
					}
					if !input_enabled { return; }
					process_input_event(event, listeners, &ctr.display);
					return;
				},
				_ => {
					if !input_enabled { return; }
					process_input_event(event, listeners, &ctr.display);
					return;
				}
			},
			Event::NewEvents(cause) => match cause {
				StartCause::ResumeTimeReached { .. } => (),
				StartCause::Init => {
					center_cursor(&ctr.display, false);
				},
				StartCause::Poll => (),
				_ => return
			},
			_ => return
		}

		let mut target = ctr.display.draw();

		if let Some(game_client) = game_client.as_mut() {
			game_client.update(&mut target, &ctr).unwrap();
			// if let Err(e) = game_client.update(&mut target, &ctr) {
			// }
		}

		target.finish().unwrap();
	});
}
