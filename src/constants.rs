pub const APP_ID: &'static str = "wolfpack-game";
pub const PORT: u16 = 27020;

pub const PLAYER_COLORS: [[f32; 3]; 6] = [
	[1.0, 0.4, 0.4],
	[1.0, 1.0, 0.4],
	[0.4, 1.0, 0.4],
	[0.4, 1.0, 1.0],
	[0.4, 0.4, 1.0],
	[1.0, 0.4, 1.0]
];

pub fn player_color(pid: u8) -> &'static [f32; 3] {
	&PLAYER_COLORS[(pid as usize - 1) % 6]
}
