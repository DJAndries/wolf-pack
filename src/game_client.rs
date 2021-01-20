use cubik::glium::{Frame, Surface};
use cubik::draw::{ObjDrawInfo, EnvDrawInfo, ObjDef, basic_render, MAX_LIGHTS, Light};
use cubik::camera::perspective_matrix;
use cubik::skybox::Skybox;
use cubik::animation::{ObjAnimation, ObjAnimationError};
use cubik::player::{Player, PlayerControlType};
use cubik::fonts::FontError;
use cubik::wavefront::WavefrontLoadError;
use cubik::peer_player::PeerPlayer;
use crate::constants::{APP_ID, PORT};
use cubik::audio::{buffer_sound, get_sound_stream, SoundStream, AudioError};
use cubik::fps_count::DebugFPSCounter;
use cubik::container::RenderContainer;
use cubik::math::mult_vector;
use cubik::fonts::LoadedFont;
use cubik::skybox::SkyboxError;
use std::collections::{HashMap, BTreeMap};
use cubik::client::{ClientContainer, ClientError};
use cubik::map::{GameMap, GameMapError};
use crate::msg::AppMessage;
use crate::leaderboard::Leaderboard;
use crate::minipack::MiniPacks;
use crate::constants::player_color;
use crate::stage::GameStageManager;
use std::time::Instant;
use derive_more::{From, Error};

const FONT_SIZE: f32 = 80.;

#[derive(From, Error, derive_more::Display, Debug)]
pub enum GameClientError {
	NetClientError(ClientError),
	FontError(FontError),
	AudioError(AudioError),
	GameMapError(GameMapError),
	WavefrontError(WavefrontLoadError),
	ObjAnimationError(ObjAnimationError),
	SkyboxError(SkyboxError)
}

pub struct GameClient {
	map_info: ObjDrawInfo,
	sound_stream: SoundStream,

	main_font: LoadedFont,

	peer_map: HashMap<u8, PeerPlayer>,

	client_container: ClientContainer<AppMessage>,
	pub player: Player,

	map: GameMap,

	packs: MiniPacks,

	wolf_standing: BTreeMap<String, ObjDef>,
	wolf_anim: ObjAnimation,
	skybox: Skybox,

	player_pack_counts: HashMap<u8, usize>,
	leaderboard: Leaderboard,

	lights_arr: [Light; MAX_LIGHTS],

	last_frame_time: Instant,

	game_stage_manager: GameStageManager,

	fps_count: DebugFPSCounter
}

impl GameClient {

	pub fn init(ctr: &mut RenderContainer, host: String, username: String, fps_count_enabled: bool) -> Result<Self, GameClientError> {
		let mut map_info: ObjDrawInfo = Default::default();
		map_info.generate_matrix();

		let mut client_container: ClientContainer<AppMessage> = ClientContainer::new(format!("{}:{}", host, PORT).as_str())?;
		client_container.state_name(username)?;
		let mut player = Player::new([0.0, 1.5, 0.0], PlayerControlType::MultiplayerClient,
			[0.0, 0.275, 0.0], [0.44, 0.275, 0.08]);

		player.walking_sound = Some(buffer_sound("./audio/running.wav", APP_ID)?);

		let map = GameMap::load_map("models/map3", APP_ID, Some(&ctr.display), Some(&mut ctr.textures), true)?;

		let mut lights_arr: [Light; MAX_LIGHTS] = Default::default();
		let mut light_iter = map.lights.values();
		for i in 0..map.lights.len() { lights_arr[i] = *light_iter.next().unwrap(); }

		Ok(Self {
			map_info: map_info,
			main_font: LoadedFont::load(&ctr.display, "fonts/Quebab-Shadow-ffp.otf", APP_ID, FONT_SIZE)?,
			peer_map: HashMap::new(),
			client_container: client_container,
			sound_stream: get_sound_stream()?,
			last_frame_time: Instant::now(),
			player: player,
			map: map,
			packs: MiniPacks::new(),
			skybox: Skybox::new(&ctr.display, "skybox1", APP_ID, 512, 50.)?,

			lights_arr: lights_arr,
			player_pack_counts: HashMap::new(),
			leaderboard: Leaderboard::new(),

			game_stage_manager: GameStageManager::new(),

			wolf_standing: cubik::wavefront::load_obj("models/wolf_standing.obj", APP_ID, Some(&ctr.display), Some(&mut ctr.textures),
				&[1., 1., 1.], None, None, None)?,
			wolf_anim: ObjAnimation::load_wavefront("models/wolfrunning", APP_ID, &ctr.display, &mut ctr.textures, 0.041)?,

			fps_count: DebugFPSCounter::new(fps_count_enabled)
		})
	}

	fn net_update(&mut self, time_delta: f32) -> Result<(), GameClientError> {
		let pids = self.client_container.pids();
		self.peer_map.retain(|&k, _| pids.contains(&k));

		self.client_container.update()?;

		for msg in self.client_container.get_msgs() {
			match msg {
				AppMessage::PlayerChange { msg, player_id } => {
					if self.client_container.player_id.unwrap_or(0) == player_id {
						self.player.update(0., None, Some(&self.sound_stream), Some(msg));
					} else {
						let peer_player = self.peer_map.entry(player_id)
							.or_insert_with(|| {
								let mut r = PeerPlayer::new();
								r.obj_draw_info.color = mult_vector(player_color(player_id), 10.);
								r
							});

						peer_player.update(Some(msg), time_delta);
					}
				},
				AppMessage::PackUpdate(_) => {
					self.packs.client_update_msg(msg);
				},
				AppMessage::StageChange(update) => {
					self.game_stage_manager.client_update(update, &self.map, &mut self.packs);
				}
			}
		}

		if let Some(msg) = self.player.update(time_delta, None, Some(&self.sound_stream), None) {
			self.client_container.send(AppMessage::PlayerChange {
				player_id: 0,
				msg: msg
			})?;
		}

		for peer_player in self.peer_map.values_mut() {
			peer_player.update(None, time_delta);
		}

		Ok(())
	}

	pub fn update(&mut self, target: &mut Frame, ctr: &RenderContainer) -> Result<(), GameClientError> {
		let new_frame_time = Instant::now();
		let time_delta = new_frame_time.duration_since(self.last_frame_time).as_secs_f32();
		self.last_frame_time = new_frame_time;

		self.net_update(time_delta)?;

		self.player_pack_counts.clear();
		for pack in &mut self.packs.packs {
			pack.client_update(self.map.quadoctree.as_ref().unwrap(), time_delta);
			if let Some(pid) = pack.owner {
				self.player_pack_counts.insert(pid, self.player_pack_counts.get(&pid).unwrap_or(&0) + 1);
			}
		}

		let perspective_mat = perspective_matrix(target);
		let env_info = EnvDrawInfo {
			perspective_mat: perspective_mat,
			view_mat: self.player.camera.view_matrix(),
			lights: self.lights_arr,
			light_count: self.map.lights.len(),
			params: &ctr.params,
			textures: &ctr.textures
		};

		target.clear_color_and_depth((0.85, 0.85, 0.85, 1.0), 1.0); 

		for o in self.map.objects.values() {
			basic_render(target, &env_info, &self.map_info, &o, &ctr.main_program, None);
		}

		for peer_player in self.peer_map.values_mut() {
			peer_player.draw(target, &env_info, &ctr.main_program, &self.wolf_anim, 
				&self.wolf_standing, self.wolf_anim.get_keyframe_by_index(5));
		}

		for pack in &self.packs.packs {
			pack.draw(target, &env_info, &ctr.main_program, &self.wolf_anim, &self.wolf_standing);
		}

		self.skybox.draw(target, &env_info, &ctr.skybox_program);

		self.leaderboard.draw(target, &ctr.display, &ctr.ui_program, &self.main_font, &self.client_container.peers,
			&self.player_pack_counts).unwrap();

		self.game_stage_manager.draw(target, &ctr.display, &ctr.ui_program, &self.main_font).unwrap();

		self.fps_count.update();

		Ok(())
	}
}
