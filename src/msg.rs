use serde::{Serialize, Deserialize};
use cubik::player::PlayerControlMessage;
use crate::minipack::MiniPackUpdate;

#[derive(Serialize, Deserialize)]
pub enum AppMessage {
	PlayerChange { player_id: u8, msg: PlayerControlMessage },
	PackUpdate(Vec<MiniPackUpdate>)
}
