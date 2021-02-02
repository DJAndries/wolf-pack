use cubik::fonts::{LoadedFont, FontText, FontError, TextAlign};
use cubik::glium::{Display, Program, Frame};
use cubik::client::PeerMeta;
use cubik::math::mult_vector;
use crate::minipack::PACK_SIZE;
use crate::constants::player_color;
use std::collections::HashMap;

const TEXT_SIZE: f32 = 0.125;

struct LeaderboardEntry {
	text: FontText,
	count: usize,
	vertical_step: usize
}

impl LeaderboardEntry {
	fn new(pid: u8, peer: &PeerMeta, player_pack_count: usize, vertical_step: usize) -> Self {
		let mut text = FontText::new(
			format!("{}: {}", peer.name.as_ref().unwrap(), (player_pack_count * PACK_SIZE).to_string()),
			TEXT_SIZE,
			Self::gen_position(vertical_step),
			TextAlign::Right
		);
		let color = mult_vector(player_color(pid), 0.9);
		text.ui_draw_info.color = [color[0], color[1], color[2], 1.];
		Self {
			text: text,
			count: player_pack_count,
			vertical_step: vertical_step
		}
	}

	fn update_position(&mut self, vertical_step: usize) {
		self.vertical_step = vertical_step;
		self.text.ui_draw_info.position = Self::gen_position(vertical_step);
		self.text.ui_draw_info.screen_dim = (0, 0);
	}

	fn gen_position(vertical_step: usize) -> (f32, f32) {
		(1.75, 0.85 - (vertical_step as f32 * TEXT_SIZE))
	}
}

pub struct Leaderboard {
	entries: HashMap<u8, LeaderboardEntry>
}

impl Leaderboard {
	pub fn new() -> Self {
		Self {
			entries: HashMap::new()
		}
	}

	pub fn draw(&mut self, target: &mut Frame, display: &Display, program: &Program, font: &LoadedFont,
		peers: &HashMap<u8, PeerMeta>, player_pack_counts: &HashMap<u8, usize>) -> Result<(), FontError> {
		if peers.len() < self.entries.len() {
			self.entries.clear();
		}

		let mut sorted_pids: Vec<u8> = peers.keys().cloned().collect();
		sorted_pids.sort_by(|a, b| {
			player_pack_counts.get(b).unwrap_or(&0).cmp(&player_pack_counts.get(a).unwrap_or(&0))
		});

		let mut vertical_step = 0;
		for pid in sorted_pids {
			if let Some(peer) = peers.get(&pid) {
				let player_pack_count = player_pack_counts.get(&pid).unwrap_or(&0);

				if peer.name.is_none() { continue; }
				
				if let Some(existing) = self.entries.get_mut(&pid) {
					if existing.count == *player_pack_count {
						if existing.vertical_step != vertical_step {
							existing.update_position(vertical_step);
						}
						vertical_step += 1;
						continue;
					}
				}

				self.entries.insert(pid, LeaderboardEntry::new(pid, peer, *player_pack_count, vertical_step));

				vertical_step += 1;
			}
		}

		for entry in self.entries.values_mut() {
			entry.text.draw(target, display, program, font)?;
		}

		Ok(())
	}
}
