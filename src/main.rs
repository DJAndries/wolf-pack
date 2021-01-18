mod server;
mod client;
mod msg;
mod constants;
mod minipack;
mod leaderboard;
mod stage;

use std::env;

fn main() {
	if env::args().any(|s| s == "--server") {
		server::start_server();
	} else {
		let mut fullscreen = true;
		let mut fps_count_enabled = false;
		let mut host = "127.0.0.1".to_string();
		let mut username = "Player".to_string();

		let mut path_consumed = false;
		let mut host_specified = false;

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
			} else {
				if !host_specified {
					host = arg;
					host_specified = true;
				} else {
					username = arg;		
				}
			}
		}
		
		client::start_client(fullscreen, host, username, fps_count_enabled);
	}
}
