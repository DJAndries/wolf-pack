mod server;
mod client;
mod msg;
mod constants;
mod minipack;

use std::env;

fn main() {
	if env::args().any(|s| s == "--server") {
		server::start_server();
	} else {
		let fullscreen = !env::args().any(|s| s == "--windowed");
		client::start_client(fullscreen);
	}
}
