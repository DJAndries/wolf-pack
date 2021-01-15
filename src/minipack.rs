use cubik::glium::{Frame, Program};
use cubik::map::GameMap;
use cubik::player::Player;
use cubik::animation::ObjAnimation;
use cubik::quadoctree::QuadOctreeNode;
use cubik::collision::check_player_collision;
use cubik::draw::{ObjDrawInfo, EnvDrawInfo, ObjDef, basic_render};
use cubik::math::{vector_length, add_vector, normalize_vector, mult_vector};
use cubik::cube::generate_cube_collideobj;
use cubik::interpolation::{Interpolate, InterpolationHelper};
use cubik::peer_player::PeerPlayer;
use crate::msg::AppMessage;
use std::collections::{BTreeMap, HashMap};
use rand::Rng;
use serde::{Serialize, Deserialize};

const SPAWN_PREFIX: &str = "misc_minipack_spawn_";
const MIN_PACK_SIZE: usize = 5;
const MAX_PACK_SIZE: usize = 5;
const POSITION_VARIANCE: f32 = 1.;
const YAW_VARIANCE: f32 = 2.;
const MIN_MEMBER_DISTANCE: f32 = 0.3;
const COLLIDE_CHECK_DECR: f32 = 0.002;

const PICKUP_DISTANCE: f32 = 1.5;
const STARTING_FOLLOW_DISTANCE: f32 = 2.;
const FOLLOW_DISTANCE_INCR: f32 = 1.;

const SERVER_UPDATE_INTERVAL: f32 = 0.1;

#[derive(Copy, Clone)]
struct PosYawValue {
	pos: (f32, f32),
	yaw: f32
}

impl Interpolate for PosYawValue {
	fn linear_interpolate(a: &Self, b: &Self, progress: f32) -> Self {
		let yaw_diff = (a.yaw - b.yaw).abs();
		let mut a_yaw = a.yaw;
		let mut b_yaw = b.yaw;
		// if angle direction flips, allow smooth transition by bringing a and b closer together
		// by adding 2*pi radians to the smaller value
		if yaw_diff > 1.5 * std::f32::consts::PI {
			if b.yaw > a.yaw {
				a_yaw += 2. * std::f32::consts::PI;
			} else {
				b_yaw += 2. * std::f32::consts::PI;
			}
		}
		Self {
			pos: <(f32, f32)>::linear_interpolate(&a.pos, &b.pos, progress),
			yaw: f32::linear_interpolate(&a_yaw, &b_yaw, progress)
		}
	}
}

#[derive(Default)]
pub struct PackMember {
	pos_offset: (f32, f32),
	standing_yaw: f32,
	draw_info: ObjDrawInfo
}

pub struct MiniPacks {
	pub packs: Vec<MiniPack>,
	net_update_time_count: f32
}

pub struct MiniPack {
	position: (f32, f32),
	yaw: f32,
	members: Vec<PackMember>,
	owner: Option<u8>,
	is_moving: bool,

	trailing_player_distance: f32,

	interpolation: InterpolationHelper<PosYawValue>
}

#[derive(Serialize, Deserialize)]
pub struct MiniPackUpdate {
	position: (f32, f32),
	yaw: f32,
	owner: Option<u8>,
	is_moving: bool
}

impl MiniPacks {
	pub fn create_from_map(map: &mut GameMap) -> MiniPacks {
		let mut result = MiniPacks {
			packs: Vec::new(),
			net_update_time_count: 0.
		};
		
		let mut rng = rand::thread_rng();
		let spawn_keys: Vec<String> = map.misc_objs.keys().filter(|k| k.starts_with(SPAWN_PREFIX)).cloned().collect();
		for spawn_key in spawn_keys {
			let obj = map.misc_objs.get(&spawn_key).unwrap();
			let member_count = rng.gen_range(MIN_PACK_SIZE..=MAX_PACK_SIZE);
			let mut new_pack = MiniPack {
				position: (obj[0], obj[2]),
				yaw: rng.gen_range(0.0..(std::f32::consts::PI * 2.)),
				members: Vec::new(),
				owner: None,
				is_moving: false,
				interpolation: InterpolationHelper::new(),
				trailing_player_distance: STARTING_FOLLOW_DISTANCE
			};
			for _ in 0..member_count {
				let mut member = PackMember {
					standing_yaw: rng.gen_range(0.0..YAW_VARIANCE),
					..Default::default()
				};
				loop {
					member.pos_offset = (
						rng.gen_range(-POSITION_VARIANCE..POSITION_VARIANCE),
						rng.gen_range(-POSITION_VARIANCE..POSITION_VARIANCE)
					);

					member.draw_info.position[1] = obj[1];

					if new_pack.members.iter().filter(|other| {
						let distance = vector_length(
							&add_vector(&[member.pos_offset.0, 0., member.pos_offset.1], &[other.pos_offset.0, 0., member.pos_offset.1], -1.)
						);
						distance < MIN_MEMBER_DISTANCE
					}).count() == 0 {
						break;
					}
				}
				new_pack.members.push(member);
			}
			result.packs.push(new_pack);

			map.misc_objs.remove(&spawn_key);
		}

		result
	}

	pub fn client_update_msg(&mut self, msg: AppMessage) {
		if let AppMessage::PackUpdate(pack_updates) = msg {
			let mut packs_iter = self.packs.iter_mut();
			for pack_update in &pack_updates {
				if let Some(pack) = packs_iter.next() {
					pack.interpolation.post_update(PosYawValue{
						pos: pack_update.position,
						yaw: pack_update.yaw
					});
					pack.owner = pack_update.owner;
					pack.is_moving = pack_update.is_moving;
				}
			}
		}
	}

	pub fn server_update_msg(&mut self, time_delta: f32) -> Option<AppMessage> {
		self.net_update_time_count += time_delta;
		if self.net_update_time_count >= SERVER_UPDATE_INTERVAL {
			self.net_update_time_count = 0.;
			Some(AppMessage::PackUpdate(
				self.packs.iter().map(|v| {
					MiniPackUpdate {
						position: v.position,
						yaw: v.yaw,
						owner: v.owner,
						is_moving: v.is_moving
					}
				}).collect()
			))
		} else {
			None
		}
	}
}

impl MiniPack {

	fn update_follow_distance(&mut self, pack_counts: usize) {
		self.trailing_player_distance = STARTING_FOLLOW_DISTANCE + (pack_counts as f32 * FOLLOW_DISTANCE_INCR);
	}

	pub fn player_server_update(&mut self, pid: u8, player: &Player, player_pack_counts: &mut HashMap<u8, usize>) {
		let diff = add_vector(&player.camera.position, &[self.position.0, 0., self.position.1], -1.);
		let distance = vector_length(&diff);

		let own_pack_counts = *player_pack_counts.entry(pid).or_insert(0);

		match self.owner {
			Some(owner_id) => {
				let other_pack_counts = *player_pack_counts.entry(owner_id).or_insert(0);
				if owner_id == pid {
					if distance > self.trailing_player_distance {
						let mve = mult_vector(&diff, (distance - self.trailing_player_distance) / distance);
						self.yaw = (mve[2] / mve[0]).atan();
						if mve[0] < 0. {
							self.yaw -= std::f32::consts::PI;
						}
						self.position.0 += mve[0];
						self.position.1 += mve[2];
					}
				} else {
					if own_pack_counts > other_pack_counts {
						self.owner = Some(pid);
						player_pack_counts.insert(pid, own_pack_counts + 1);
						player_pack_counts.insert(owner_id, own_pack_counts - 1);
						self.update_follow_distance(own_pack_counts);
					}
				}
			},
			None => {
				if distance < PICKUP_DISTANCE {
					self.owner = Some(pid);
					player_pack_counts.insert(pid, own_pack_counts + 1);
					self.update_follow_distance(own_pack_counts);
				}
			}
		};
	}

	pub fn client_update(&mut self, quadoctree: &QuadOctreeNode, time_delta: f32) {
		if let Some(pos_yaw) = self.interpolation.value(time_delta) {
			self.position = pos_yaw.pos;
			self.yaw = pos_yaw.yaw;
		}

		let owner = self.owner.as_ref();

		for member in &mut self.members {
			member.draw_info.position[0] = member.pos_offset.0 + self.position.0;
			member.draw_info.position[2] = member.pos_offset.1 + self.position.1;

			'f: for _ in 0..15 {
				let collide_obj = generate_cube_collideobj(&[0., 0.5, 0.], &member.draw_info.position, &[0.5, 0.5, 0.5], 0.);
				let collide_result = check_player_collision(quadoctree, &member.draw_info.position, &collide_obj);
				for resolve in collide_result.polygons {
					if normalize_vector(&resolve)[1] > 0.5 {
						member.draw_info.position = add_vector(&resolve, &member.draw_info.position, 1.);
						break 'f;
					}
				}
				member.draw_info.position[1] -= COLLIDE_CHECK_DECR;
			}

			member.draw_info.rotation[1] = self.yaw + if owner.is_some() { 0. } else { member.standing_yaw };
			member.draw_info.generate_matrix();
		}
	}

	pub fn draw(&self, target: &mut Frame, env_info: &EnvDrawInfo, program: &Program, wolf_anim: &ObjAnimation, wolf_standing: &BTreeMap<String, ObjDef>) {
		for member in &self.members {
			for obj in wolf_standing.values() {
				basic_render(target, env_info, &member.draw_info, obj, program, None);
			}
		}
	}
}
