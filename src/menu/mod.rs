mod start;
mod main;
mod settings;
mod info;

use crate::settings::Settings;

const NORMAL_COLOR: [f32; 4] = [0.94, 0.94, 0.94, 1.];
const HOVER_COLOR: [f32; 4] = [1., 1., 0.2, 1.];
const WHITE: [f32; 4] = [1., 1., 1., 1.];

#[derive(Clone)]
pub enum MenuResult {
	Start { host: String, name: String },
	SettingsChange(Settings),
	Quit
}

pub use self::main::MainMenu;
