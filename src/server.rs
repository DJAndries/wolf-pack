use cubik::server::ServerContainer;
use cubik::player::{Player, PlayerControlType};
use cubik::quadoctree::{QuadOctreeNode, BoundingBox};
use cubik::map::GameMap;
use crate::msg::AppMessage;
use crate::constants::{APP_ID, PORT};
use crate::minipack::MiniPacks;
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::collections::HashMap;

pub fn start_server() {
	let mut server_container: ServerContainer<AppMessage> = ServerContainer::new(PORT, 10).unwrap();

	println!("server listening on port {}", PORT);

	let mut last_status_update = Instant::now();
	let mut player_map: HashMap<u8, Player> = HashMap::new();

	let quadoctree = QuadOctreeNode::new_tree(BoundingBox {
		start_pos: [-25., -25., -25.],
		end_pos: [25., 25., 25.]
	}, false);

	let mut map = GameMap::load_map("models/map3", APP_ID, None, None, Some(quadoctree)).unwrap();
	let mut packs = MiniPacks::create_from_map(&mut map);
	let mut player_pack_counts: HashMap<u8, usize> = HashMap::new();

	let mut last_time = Instant::now();

	loop {
		server_container.update();

		let current_pids = server_container.pids();
		player_map.retain(|&k, _| current_pids.contains(&k));

		for pid in current_pids {
			let player = player_map.entry(pid)
				.or_insert_with(|| {
					let mut player = Player::new([0., 1.5, 0.], PlayerControlType::MultiplayerServer,
						[-0.28, 0.275, 0.0], [0.44, 0.275, 0.08]);
					player.move_rate = 2.56;
					player
				});
			if let Ok(msgs) = server_container.get_msgs(pid) {
				for msg in msgs {
					if let AppMessage::PlayerChange { msg, .. } = msg {
						player.update(0., Some(&map.quadoctree.as_ref().unwrap()), None, Some(msg));
					}
				}
				if let Some(msg) = player.update(last_time.elapsed().as_secs_f32(), Some(&map.quadoctree.as_ref().unwrap()), None, None) {
					server_container.broadcast(AppMessage::PlayerChange {
						msg: msg,
						player_id: pid
					});
				}
				for pack in &mut packs.packs {
					pack.player_server_update(pid, &player, &mut player_pack_counts);
				}
			}
		}

		if let Some(msg) = packs.server_update_msg(last_time.elapsed().as_secs_f32()) {
			server_container.broadcast(msg);
		}

		if last_status_update.elapsed().as_secs_f32() > 5. {
			last_status_update = Instant::now();
			println!("peer status update:");
			for (pid, conn) in &server_container.connections {
				println!("pid: {} name: {}", pid, conn.name.as_ref().unwrap_or(&"".to_string()));
			}
			println!("");
		}

		last_time = Instant::now();

		sleep(Duration::from_millis(17));
	}
}