use cubik::glium::{glutin, Surface};
use cubik::draw::{ObjDrawInfo, EnvDrawInfo, basic_render, MAX_LIGHTS, Light};
use cubik::camera::perspective_matrix;
use cubik::input::{InputListener, process_input_event, center_cursor};
use cubik::skybox::Skybox;
use cubik::animation::ObjAnimation;
use cubik::player::{Player, PlayerControlType};
use cubik::peer_player::PeerPlayer;
use crate::constants::{APP_ID, PORT};
use cubik::audio::{buffer_sound, get_sound_stream, SoundStream};
use cubik::container::RenderContainer;
use cubik::math::mult_vector;
use std::collections::HashMap;
use cubik::client::ClientContainer;
use cubik::map::GameMap;
use crate::msg::AppMessage;
use crate::leaderboard::Leaderboard;
use crate::minipack::MiniPacks;
use crate::constants::player_color;

fn net_update(client_container: &mut ClientContainer<AppMessage>, peer_map: &mut HashMap<u8, PeerPlayer>,
	player: &mut Player, sound_stream: &SoundStream, packs: &mut MiniPacks, time_delta: f32) {
	let pids = client_container.pids();
	peer_map.retain(|&k, _| pids.contains(&k));

	client_container.update().unwrap();

	for msg in client_container.get_msgs() {
		match msg {
			AppMessage::PlayerChange { msg, player_id } => {
				if client_container.player_id.unwrap_or(0) == player_id {
					player.update(0., None, Some(sound_stream), Some(msg));
				} else {
					let peer_player = peer_map.entry(player_id)
						.or_insert_with(|| {
							let mut r = PeerPlayer::new();
							r.obj_draw_info.color = mult_vector(player_color(player_id), 10.);
							r
						});

					peer_player.update(Some(msg), time_delta);
				}
			},
			AppMessage::PackUpdate(_) => {
				packs.client_update_msg(msg);
			}
		}
	}

	if let Some(msg) = player.update(time_delta, None, Some(sound_stream), None) {
		client_container.send(AppMessage::PlayerChange {
			player_id: 0,
			msg: msg
		}).unwrap();
	}

	for peer_player in peer_map.values_mut() {
		peer_player.update(None, time_delta);
	}
}

pub fn start_client(fullscreen: bool, host: String, username: String) {
	let event_loop = glutin::event_loop::EventLoop::new();
	let mut ctr = RenderContainer::new(&event_loop, 1280, 720, "Wolf Pack", fullscreen);

	let mut map_info = ObjDrawInfo {
		position: [0.0, 0.0, 0.0f32],
		rotation: [0.0, 0.0, 0.0f32],
		scale: [1.0, 1.0, 1.0],
		color: [1.0, 1.0, 1.0],
		model_mat: None 
	};
	map_info.generate_matrix();

	let sound_stream = get_sound_stream().unwrap();

	let mut peer_map: HashMap<u8, PeerPlayer> = HashMap::new();

	let mut client_container: ClientContainer<AppMessage> = ClientContainer::new(format!("{}:{}", host, PORT).as_str()).unwrap();
	client_container.state_name(username).unwrap();
	let mut player = Player::new([0.0, 1.5, 0.0], PlayerControlType::MultiplayerClient,
		[0.0, 0.275, 0.0], [0.44, 0.275, 0.08]);

	player.walking_sound = Some(buffer_sound("./audio/running.wav", APP_ID).unwrap());

	let mut map = GameMap::load_map("models/map3", APP_ID, Some(&ctr.display), Some(&mut ctr.textures), true).unwrap();

	let mut packs = MiniPacks::create_from_map(&mut map);

	let wolf_standing = cubik::wavefront::load_obj("models/wolf_standing.obj", APP_ID, Some(&ctr.display), Some(&mut ctr.textures),
		&[1., 1., 1.], None, None, None).unwrap();
	let wolf_anim = ObjAnimation::load_wavefront("models/wolfrunning", APP_ID, &ctr.display, &mut ctr.textures, 0.041).unwrap();

	let skybox = Skybox::new(&ctr.display, "skybox1", APP_ID, 512, 50.).unwrap();

	let mut player_pack_counts: HashMap<u8, usize> = HashMap::new();
	let mut leaderboard = Leaderboard::new(&ctr.display).unwrap();

	let mut lights_arr: [Light; MAX_LIGHTS] = Default::default();
	let mut light_iter = map.lights.values();
	for i in 0..map.lights.len() { lights_arr[i] = *light_iter.next().unwrap(); }

	let mut last_frame_time = std::time::Instant::now();

	let mut input_enabled = true;

	event_loop.run(move |ev, _, control_flow| {
		let listeners: Vec<&mut dyn InputListener> = vec![&mut player];
		*control_flow = glutin::event_loop::ControlFlow::Poll;
		match ev {
			glutin::event::Event::WindowEvent { event, .. } => match event {
				glutin::event::WindowEvent::CloseRequested => {
					*control_flow = glutin::event_loop::ControlFlow::Exit;
					return;
				},
				glutin::event::WindowEvent::KeyboardInput { input, .. } => {
					if let Some(keycode) = input.virtual_keycode {
						match keycode {
							glutin::event::VirtualKeyCode::Escape => {
								*control_flow = glutin::event_loop::ControlFlow::Exit;
								return;
							},
							glutin::event::VirtualKeyCode::T => {
								if input.state == glutin::event::ElementState::Released {
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
			glutin::event::Event::NewEvents(cause) => match cause {
				glutin::event::StartCause::ResumeTimeReached { .. } => (),
				glutin::event::StartCause::Init => {
					center_cursor(&ctr.display, false);
				},
				glutin::event::StartCause::Poll => (),
				_ => return
			},
			_ => return
		}

		let new_frame_time = std::time::Instant::now();
		let time_delta = new_frame_time.duration_since(last_frame_time).as_secs_f32();
		last_frame_time = new_frame_time;

		net_update(&mut client_container, &mut peer_map, &mut player, &sound_stream, &mut packs, time_delta);

		player_pack_counts.clear();
		for pack in &mut packs.packs {
			pack.client_update(map.quadoctree.as_ref().unwrap(), time_delta);
			if let Some(pid) = pack.owner {
				player_pack_counts.insert(pid, player_pack_counts.get(&pid).unwrap_or(&0) + 1);
			}
		}

		let mut target = ctr.display.draw();

		let perspective_mat = perspective_matrix(&mut target);
		let env_info = EnvDrawInfo {
			perspective_mat: perspective_mat,
			view_mat: player.camera.view_matrix(),
			lights: lights_arr,
			light_count: map.lights.len(),
			params: &ctr.params,
			textures: &ctr.textures
		};

		target.clear_color_and_depth((0.85, 0.85, 0.85, 1.0), 1.0); 

		for o in map.objects.values() {
			basic_render(&mut target, &env_info, &map_info, &o, &ctr.main_program, None);
		}

		for peer_player in peer_map.values_mut() {
			peer_player.draw(&mut target, &env_info, &ctr.main_program, &wolf_anim, 
				&wolf_standing, wolf_anim.get_keyframe_by_index(5));
		}

		for pack in &packs.packs {
			pack.draw(&mut target, &env_info, &ctr.main_program, &wolf_anim, &wolf_standing);
		}

		skybox.draw(&mut target, &env_info, &ctr.skybox_program);

		leaderboard.draw(&mut target, &ctr.display, &ctr.ui_program, &client_container.peers,
			&player_pack_counts).unwrap();

		target.finish().unwrap();
	});
}
