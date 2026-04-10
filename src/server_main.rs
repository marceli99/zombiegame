mod types;
mod map;
mod game;
mod network;

use std::net::UdpSocket;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::env;

use types::*;
use network::*;
use game::*;

struct ClientSlot {
    addr: std::net::SocketAddr,
    input: RemoteInput,
    last_seen: Instant,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut server_name = "Zombie Server".to_string();
    let mut port = NET_PORT;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--name" => {
                if i + 1 < args.len() {
                    server_name = args[i + 1].clone();
                    i += 1;
                }
            }
            "--port" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse().unwrap_or(NET_PORT);
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    macroquad::rand::srand(seed);

    let game_sock = UdpSocket::bind(format!("0.0.0.0:{}", port))
        .expect("Failed to bind game socket");
    game_sock.set_nonblocking(true).ok();

    let disc_port = port + 1;
    let disc_sock = UdpSocket::bind(format!("0.0.0.0:{}", disc_port))
        .expect("Failed to bind discovery socket");
    disc_sock.set_nonblocking(true).ok();
    disc_sock.set_broadcast(true).ok();

    println!("=== Zombie Dedicated Server ===");
    println!("Name: {}", server_name);
    println!("Game port: {}", port);
    println!("Discovery port: {}", disc_port);
    println!("IP: {}", get_local_ip());
    println!("Waiting for players...");

    let mut state = new_game(true);
    let mut clients: [Option<ClientSlot>; 2] = [None, None];
    let mut recv_buf = [0u8; 65536];
    let mut last_tick = Instant::now();
    let mut net_timer = 0.0f32;
    let mut game_started = false;
    let mut restart_timer = 0.0f32;

    loop {
        let now = Instant::now();
        let dt = now.duration_since(last_tick).as_secs_f32().min(0.05);
        last_tick = now;

        // Discovery requests
        loop {
            match disc_sock.recv_from(&mut recv_buf) {
                Ok((n, addr)) if n > 0 => {
                    if recv_buf[0] == MSG_DISCOVERY_REQ {
                        let count = clients.iter().filter(|c| c.is_some()).count() as u8;
                        let resp = serialize_server_info(
                            &server_name, count, MAX_PLAYERS,
                            state.wave, state.score, port,
                        );
                        let _ = disc_sock.send_to(&resp, addr);
                    } else if recv_buf[0] == MSG_PING && n >= 9 {
                        recv_buf[0] = MSG_PONG;
                        let _ = disc_sock.send_to(&recv_buf[..n], addr);
                    }
                }
                _ => break,
            }
        }

        // Game messages
        loop {
            match game_sock.recv_from(&mut recv_buf) {
                Ok((n, addr)) if n > 0 => {
                    match recv_buf[0] {
                        MSG_JOIN => {
                            let mut assigned = None;
                            for slot in 0..2usize {
                                if clients[slot].is_none() {
                                    clients[slot] = Some(ClientSlot {
                                        addr,
                                        input: RemoteInput::default(),
                                        last_seen: Instant::now(),
                                    });
                                    assigned = Some(slot);
                                    break;
                                }
                            }
                            if let Some(slot) = assigned {
                                let _ = game_sock.send_to(&[MSG_ACCEPT, slot as u8], addr);
                                println!("Player joined slot {} from {}", slot, addr);
                                if !game_started {
                                    state = new_game(true);
                                    game_started = true;
                                    restart_timer = 0.0;
                                    println!("Game started!");
                                }
                            } else {
                                let _ = game_sock.send_to(&[MSG_DISCONNECT], addr);
                                println!("Rejected {} (server full)", addr);
                            }
                        }
                        MSG_INPUT if n > 1 => {
                            for slot in 0..2usize {
                                if let Some(ref mut c) = clients[slot] {
                                    if c.addr == addr {
                                        c.input = deserialize_input(&recv_buf[..n]);
                                        c.last_seen = Instant::now();
                                        break;
                                    }
                                }
                            }
                        }
                        MSG_DISCONNECT => {
                            for slot in 0..2usize {
                                if let Some(ref c) = clients[slot] {
                                    if c.addr == addr {
                                        println!("Player disconnected from slot {}", slot);
                                        clients[slot] = None;
                                        break;
                                    }
                                }
                            }
                        }
                        MSG_PING if n >= 9 => {
                            recv_buf[0] = MSG_PONG;
                            let _ = game_sock.send_to(&recv_buf[..n], addr);
                        }
                        _ => {}
                    }
                }
                _ => break,
            }
        }

        // Timeout
        for slot in 0..2usize {
            if let Some(ref c) = clients[slot] {
                if c.last_seen.elapsed().as_secs() > 10 {
                    println!("Player in slot {} timed out", slot);
                    clients[slot] = None;
                }
            }
        }

        let player_count = clients.iter().filter(|c| c.is_some()).count();
        if game_started && player_count == 0 {
            game_started = false;
            state = new_game(true);
            println!("All players left. Waiting...");
        }

        if game_started {
            let p1_input = if let Some(ref c) = clients[0] {
                LocalInput {
                    dx: c.input.dx, dy: c.input.dy,
                    angle: c.input.angle, shooting: c.input.shooting,
                }
            } else {
                LocalInput { dx: 0.0, dy: 0.0, angle: 0.0, shooting: false }
            };

            let p2_input = if let Some(ref c) = clients[1] {
                RemoteInput {
                    dx: c.input.dx, dy: c.input.dy,
                    angle: c.input.angle, shooting: c.input.shooting,
                }
            } else {
                RemoteInput::default()
            };

            state.time += dt;
            update_game(&mut state, &p1_input, &p2_input, dt);

            if state.game_over {
                restart_timer += dt;
                if restart_timer >= 10.0 {
                    println!("Restarting game...");
                    state = new_game(true);
                    restart_timer = 0.0;
                }
            }

            net_timer += dt;
            if net_timer >= NET_SEND_RATE {
                net_timer = 0.0;
                let data = serialize_state(&state);
                for slot in 0..2usize {
                    if let Some(ref c) = clients[slot] {
                        let _ = game_sock.send_to(&data, c.addr);
                    }
                }
                state.events.clear();
            }
        }

        let elapsed = last_tick.elapsed();
        let target = std::time::Duration::from_micros(16667);
        if elapsed < target {
            std::thread::sleep(target - elapsed);
        }
    }
}
