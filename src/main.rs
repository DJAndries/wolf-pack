mod server;
mod client;
mod game_client;
mod msg;
mod constants;
mod minipack;
mod leaderboard;
mod stage;
mod menu;
mod settings;

use std::env;

fn main() {
	if env::args().any(|s| s == "--server") {
		server::start_server();
	} else {
		let mut fullscreen = true;
		let mut fps_count_enabled = false;
		let mut input_switcher_enabled = false;
		let mut host: Option<String> = None;
		let mut username: Option<String> = None;

		let mut path_consumed = false;

		for arg in env::args() {
			if !path_consumed {
				path_consumed = true;
				continue;
			}
			if arg.starts_with("--") {
				if arg == "--windowed" {
					fullscreen = false;
				}
				if arg == "--fps" {
					fps_count_enabled = true;
				}
				if arg == "--switcher" {
					input_switcher_enabled = true;
				}
			} else {
				if host.is_none() {
					host = Some(arg);
				} else {
					username = Some(arg);
				}
			}
		}
		
		client::start_client(fullscreen, host, username, fps_count_enabled, input_switcher_enabled);
	}
}
