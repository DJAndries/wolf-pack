use cubik::input::{InputListener, process_input_event, center_cursor};
use cubik::container::RenderContainer;
use cubik::glium::glutin::event_loop::{EventLoop, ControlFlow};
use cubik::glium::glutin::event::{Event, WindowEvent, VirtualKeyCode, ElementState, StartCause};
use crate::game_client::{GameClient, GameClientError};
use crate::menu::{MainMenu, MenuResult};
use crate::settings::Settings;

fn new_game(ctr: &mut RenderContainer, menu: &mut MainMenu, host: String, name: String, fps_count_enabled: bool) -> Option<GameClient> {
	match GameClient::init(ctr, host, name, fps_count_enabled) {
		Ok(new_client) => {
			center_cursor(&ctr.display, false);
			return Some(new_client);
		},
		Err(e) => {
			if let GameClientError::NetClientError(_) = e {
				eprintln!("{:?}", e);
				menu.show_info_dialog("Failed to connect to server.".to_string());
			} else {
				panic!("{:?}", e);
			}
		}
	};
	None
}

pub fn start_client(fullscreen: bool, host: Option<String>, username: Option<String>, fps_count_enabled: bool, input_switcher_enabled: bool) {
	let mut settings = Settings::load().unwrap();

	let event_loop = EventLoop::new();
	let mut ctr = RenderContainer::new(&event_loop, settings.resolution[0], settings.resolution[1],
		"Wolf Pack", if !fullscreen { false } else { !settings.windowed });

	let mut input_enabled = true;

	let mut menu = MainMenu::new(&ctr.display, settings).unwrap();
	let mut game_client: Option<GameClient> = if host.is_some() && username.is_some() {
		new_game(&mut ctr, &mut menu, host.unwrap(), username.unwrap(), fps_count_enabled)
	} else {
		None
	};

	event_loop.run(move |ev, _, control_flow| {
		let listeners: Vec<&mut dyn InputListener> = if let Some(game_client) = game_client.as_mut() {
			vec![&mut game_client.player]
		} else {
			vec![&mut menu]
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
					center_cursor(&ctr.display, game_client.is_none());
				},
				StartCause::Poll => (),
				_ => return
			},
			_ => return
		}

		let mut target = ctr.display.draw();

		match game_client.as_mut() {
			None => {
				// Main menu is shown
				if let Some(menu_result) = menu.draw(&mut target, &ctr).unwrap() {
					match menu_result {
						MenuResult::Start { host, name } => {
							game_client = new_game(&mut ctr, &mut menu, host, name, fps_count_enabled);
						},
						MenuResult::SettingsChange(new_settings) => {
							settings = new_settings;
							settings.save().unwrap();
							ctr.update_size_and_mode(settings.resolution[0], settings.resolution[1], !settings.windowed);
						},
						MenuResult::Quit => *control_flow = ControlFlow::Exit
					};
				}
			},
			Some(g_client) => {
				// Game in progress
				if let Err(e) = g_client.update(&mut target, &ctr) {
					if let GameClientError::NetClientError(_) = e {
						center_cursor(&ctr.display, true);
						menu.show_info_dialog("Lost connection to server.".to_string());
						game_client = None;
					} else {
						panic!("{:?}", e);
					}
				}
			}
		};

		target.finish().unwrap();
	});
}
