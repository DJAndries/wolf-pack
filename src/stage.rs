use std::collections::HashMap;
use cubik::map::GameMap;
use cubik::fonts::{FontText, LoadedFont, FontError, TextAlign};
use cubik::player::Player;
use crate::minipack::MiniPacks;
use crate::msg::AppMessage;
use std::time::Instant;
use cubik::glium::{Display, Program, Frame};
use serde::{Serialize, Deserialize};

const WARMUP_SECONDS: usize = 5;
const GAME_SECONDS: usize = 60;
const FINISH_SECONDS: usize = 12;
const EARLY_FINISH_SECONDS: usize = 30;

const WARMUP_TEXT_SIZE: f32 = 0.11;
const MAIN_TEXT_SIZE: f32 = 0.15;

pub struct GameStageManager {
	pub current_stage: GameStage,
	stage_start_time: Instant,
	last_update_time: Instant,
	text: Option<FontText>
}

#[derive(Debug)]
pub enum GameStage {
	Standby,
	Warmup,
	InProgress,
	Finished
}

#[derive(Serialize, Deserialize)]
pub enum GameStageUpdate {
	Warmup { time_remaining: u8 },
	InProgress { time_remaining: u16 },
	Finished
}

impl GameStageManager {
	pub fn new() -> Self {
		Self {
			current_stage: GameStage::Standby,
			stage_start_time: Instant::now(),
			last_update_time: Instant::now(),
			text: None
		}
	}

	fn server_start_game(&mut self, map: &GameMap, player_map: &mut HashMap<u8, Player>, packs: &mut MiniPacks,
		player_pack_counts: &mut HashMap<u8, usize>) -> Option<GameStageUpdate> {
		packs.packs.clear();
		player_pack_counts.clear();

		packs.spawn(map);
		for player in player_map.values_mut() {
			player.respawn();
		}

		self.current_stage = GameStage::InProgress;
		self.stage_start_time = Instant::now();
		Some(GameStageUpdate::InProgress { time_remaining: GAME_SECONDS as u16 })
	}

	pub fn server_update(&mut self, map: &GameMap, player_map: &mut HashMap<u8, Player>, packs: &mut MiniPacks,
		player_pack_counts: &mut HashMap<u8, usize>) -> Option<AppMessage> {

		if self.last_update_time.elapsed().as_secs() < 1 {
			return None;
		}
		self.last_update_time = Instant::now();

		let elapsed_secs = self.stage_start_time.elapsed().as_secs() as usize;
		let update = match self.current_stage {
			GameStage::Standby => {
				if !player_map.is_empty() {
					self.current_stage = GameStage::Warmup;
					self.stage_start_time = Instant::now();
					Some(GameStageUpdate::Warmup { time_remaining: WARMUP_SECONDS as u8 })
				} else {
					None
				}
			},
			GameStage::Warmup => {
				if elapsed_secs >= WARMUP_SECONDS {
					self.server_start_game(map, player_map, packs, player_pack_counts)
				} else {
					Some(GameStageUpdate::Warmup { 
						time_remaining: (WARMUP_SECONDS as u8) - (elapsed_secs as u8)
					})
				}
			},
			GameStage::InProgress => {
				if (elapsed_secs > EARLY_FINISH_SECONDS && player_pack_counts.values().filter(|v| **v > 0).count() == 1) ||
					elapsed_secs >= GAME_SECONDS {
					
					self.current_stage = GameStage::Finished;
					self.stage_start_time = Instant::now();
					Some(GameStageUpdate::Finished)
				} else {
					Some(GameStageUpdate::InProgress { 
						time_remaining: (GAME_SECONDS as u16) - (elapsed_secs as u16)
					})
				}
			},
			GameStage::Finished => {
				if elapsed_secs >= FINISH_SECONDS {
					self.current_stage = GameStage::Warmup;
					self.stage_start_time = Instant::now();
					packs.packs.clear();
					player_pack_counts.clear();
					Some(GameStageUpdate::Warmup { time_remaining: WARMUP_SECONDS as u8 })
				} else {
					None
				}
			}
		};

		update.map(|u| AppMessage::StageChange(u))
	}

	fn format_time(seconds: usize) -> String {
		format!("{:0>2}:{:0>2}", seconds / 60, seconds % 60)
	}

	pub fn client_update(&mut self, update: GameStageUpdate, map: &GameMap, packs: &mut MiniPacks) {
		match update {
			GameStageUpdate::Warmup { time_remaining } => {
				if let GameStage::Finished = self.current_stage {
					packs.packs.clear();
				}
				self.current_stage = GameStage::Warmup;
				self.text = Some(FontText::new(format!("Warmup {}", Self::format_time(time_remaining as usize)),
					WARMUP_TEXT_SIZE, (0., 0.3), TextAlign::Center));
			},
			GameStageUpdate::InProgress { time_remaining } => {
				if packs.packs.is_empty() {
					packs.spawn(map);
				}
				self.current_stage = GameStage::InProgress;
				self.text = Some(FontText::new(Self::format_time(time_remaining as usize), MAIN_TEXT_SIZE,
					(0., 0.83), TextAlign::Center));
			},
			GameStageUpdate::Finished => {
				self.current_stage = GameStage::Finished;
				self.text = Some(FontText::new("Game Over".to_string(), MAIN_TEXT_SIZE,
					(0., 0.1), TextAlign::Center));
			}
		};
		self.text.as_mut().unwrap().ui_draw_info.color = [1.0, 0.25, 0.25, 1.0];
	}

	pub fn draw(&mut self, target: &mut Frame, display: &Display, program: &Program,
		font: &LoadedFont) -> Result<(), FontError> {
		if let Some(text) = self.text.as_mut() {
			text.draw(target, display, program, font)?;
		}

		Ok(())
	}
}
