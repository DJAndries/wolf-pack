use derive_more::{From, Error, Display};
use dirs::home_dir;
use std::io::{self, Read, Write};
use std::fs::File;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(From, Error, Display, Debug)]
pub enum SettingsError {
	IOError(io::Error),
	NoHomeDir,
	ParseError(toml::de::Error),
	SerializeError(toml::ser::Error)
}

pub const RESOLUTION_OPTIONS: [[usize; 2]; 3] = [
	[1920, 1080],
	[1280, 720],
	[720, 480]
];

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Settings {
	pub resolution: [usize; 2],
	pub windowed: bool
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			resolution: [1280, 720],
			windowed: false
		}
	}
}

impl Settings {
	fn settings_path() -> Result<PathBuf, SettingsError> {
		let mut home = home_dir().ok_or(SettingsError::NoHomeDir)?;
		home.push(".wolfpack");
		Ok(home)
	}

	pub fn load() -> Result<Self, SettingsError> {
		let settings_path = Self::settings_path()?;
		if !settings_path.exists() {
			let default_settings: Settings = Default::default();
			default_settings.save()?;
			return Ok(default_settings);
		}

		let mut file = File::open(settings_path)?;
		let mut contents = String::new();
		file.read_to_string(&mut contents)?;
		Ok(toml::from_str(&contents)?)
	}
	
	pub fn save(&self) -> Result<(), SettingsError> {
		let serialized = toml::to_string(self)?;
		let mut file = File::create(Self::settings_path()?)?;
		file.write_all(serialized.as_bytes())?;
		Ok(())
	}
}
