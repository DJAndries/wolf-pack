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
			(1.7, 0.85 - (vertical_step as f32 * TEXT_SIZE)),
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

		let mut len = self.entries.len();

		for (id, peer) in peers {
			let player_pack_count = player_pack_counts.get(id).unwrap_or(&0);

			if peer.name.is_none() { continue; }
			
			let vertical_step = if let Some(existing) = self.entries.get(id) {
				if existing.count == *player_pack_count { continue; }
				existing.vertical_step
			} else { len };

			if self.entries.insert(*id, LeaderboardEntry::new(*id, peer, *player_pack_count, vertical_step)).is_none() {
				len += 1;
			}
		}

		for entry in self.entries.values_mut() {
			entry.text.draw(target, display, program, font)?;
		}

		Ok(())
	}
}
