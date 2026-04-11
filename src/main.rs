mod types;
mod map;
mod drawing;
mod sound;
mod network;
mod game;

use macroquad::prelude::*;
use std::net::{UdpSocket, SocketAddr};
use std::time::SystemTime;

use types::*;
use drawing::*;
use sound::*;
use network::*;
use game::*;

fn window_conf() -> Conf {
    let mut conf = Conf {
        window_title: "Zombie Survival".to_string(),
        window_width: 1120,
        window_height: 864,
        window_resizable: true,
        ..Default::default()
    };
    conf.platform.swap_interval = Some(0);
    conf
}

#[macroquad::main(window_conf)]
async fn main() {
    let sounds = load_sounds().await;
    play_music(&sounds.menu_music, 0.35);
    let mut menu_music_playing = true;
    let mut app = AppState {
        screen: Screen::Menu,
        game: new_game(1),
        selected_res: 0,
        camera_offset: Vec2::ZERO,
        net_role: NetRole::Solo,
        socket: None,
        peer_addr: None,
        net_timer: 0.0,
        ip_input: String::new(),
        connected: false,
        servers: Vec::new(),
        browser_selected: 0,
        discovery_timer: 0.0,
        player_slot: 0,
        dedicated: false,
        host_clients: [None, None, None],
        my_ready: false,
        lobby_slots: [(false, false); 4],
    };
    let mut recv_buf = [0u8; 65536];

    loop {
        let dt = get_frame_time().min(0.05);

        match app.screen {
            Screen::Menu => {
                if !menu_music_playing {
                    play_music(&sounds.menu_music, 0.35);
                    menu_music_playing = true;
                }

                if is_key_pressed(KeyCode::Left) && app.selected_res > 0 {
                    app.selected_res -= 1;
                    let (w, h) = RESOLUTIONS[app.selected_res];
                    request_new_screen_size(w as f32, h as f32);
                    play_sfx(&sounds.menu_navigate, 0.5);
                }
                if is_key_pressed(KeyCode::Right) && app.selected_res < RESOLUTIONS.len() - 1 {
                    app.selected_res += 1;
                    let (w, h) = RESOLUTIONS[app.selected_res];
                    request_new_screen_size(w as f32, h as f32);
                    play_sfx(&sounds.menu_navigate, 0.5);
                }
                if is_key_pressed(KeyCode::Key1) {
                    play_sfx(&sounds.menu_select, 0.6);
                    stop_music(&sounds.menu_music);
                    menu_music_playing = false;
                    app.net_role = NetRole::Solo;
                    app.game = new_game(1);
                    app.screen = Screen::Playing;
                    app.socket = None;
                    play_sfx(&sounds.wave_start, 0.5);
                    play_music(&sounds.game_music, 0.12);
                }
                if is_key_pressed(KeyCode::Key2) {
                    play_sfx(&sounds.menu_select, 0.6);
                    stop_music(&sounds.menu_music);
                    menu_music_playing = false;
                    app.net_role = NetRole::Host;
                    app.connected = false;
                    app.peer_addr = None;
                    app.my_ready = false;
                    app.host_clients = [None, None, None];
                    app.lobby_slots = [(false, false); 4];
                    if let Ok(sock) = UdpSocket::bind(format!("0.0.0.0:{}", NET_PORT)) {
                        sock.set_nonblocking(true).ok();
                        app.socket = Some(sock);
                        app.screen = Screen::Lobby;
                    }
                }
                if is_key_pressed(KeyCode::Key3) {
                    play_sfx(&sounds.menu_select, 0.6);
                    stop_music(&sounds.menu_music);
                    menu_music_playing = false;
                    app.net_role = NetRole::Client;
                    app.connected = false;
                    app.ip_input = String::new();
                    app.my_ready = false;
                    app.lobby_slots = [(false, false); 4];
                    app.screen = Screen::Lobby;
                    app.socket = None;
                }
                if is_key_pressed(KeyCode::Key4) {
                    play_sfx(&sounds.menu_select, 0.6);
                    app.screen = Screen::Browser;
                    app.servers.clear();
                    app.browser_selected = 0;
                    app.socket = None;
                }
                draw_menu(app.selected_res);
            }

            Screen::Lobby => {
                clear_background(Color::new(0.08, 0.08, 0.12, 1.0));

                if is_key_pressed(KeyCode::Escape) {
                    play_sfx(&sounds.menu_navigate, 0.5);
                    app.screen = Screen::Menu;
                    app.socket = None;
                    app.connected = false;
                    app.host_clients = [None, None, None];
                    app.my_ready = false;
                    next_frame().await;
                    continue;
                }

                if app.net_role == NetRole::Host {
                    // ── Host Lobby ─────────────��────────────────
                    // Process incoming messages
                    if let Some(ref sock) = app.socket {
                        loop {
                            match sock.recv_from(&mut recv_buf) {
                                Ok((n, addr)) if n > 0 => {
                                    match recv_buf[0] {
                                        MSG_JOIN => {
                                            let mut assigned = None;
                                            for i in 0..3usize {
                                                if app.host_clients[i].is_none() {
                                                    app.host_clients[i] = Some(HostClient {
                                                        addr,
                                                        input: RemoteInput::default(),
                                                        ready: false,
                                                    });
                                                    assigned = Some(i);
                                                    break;
                                                }
                                            }
                                            if let Some(slot) = assigned {
                                                let actual_slot = (slot + 1) as u8;
                                                let _ = sock.send_to(&[MSG_ACCEPT, actual_slot], addr);
                                            } else {
                                                let _ = sock.send_to(&[MSG_DISCONNECT], addr);
                                            }
                                        }
                                        MSG_READY => {
                                            for i in 0..3usize {
                                                if let Some(ref mut c) = app.host_clients[i] {
                                                    if c.addr == addr {
                                                        c.ready = !c.ready;
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        MSG_DISCONNECT => {
                                            for i in 0..3usize {
                                                if let Some(ref c) = app.host_clients[i] {
                                                    if c.addr == addr {
                                                        app.host_clients[i] = None;
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                _ => break,
                            }
                        }
                    }

                    // Toggle ready
                    if is_key_pressed(KeyCode::Space) {
                        app.my_ready = !app.my_ready;
                        play_sfx(&sounds.menu_navigate, 0.5);
                    }

                    // Build lobby slots for display
                    app.lobby_slots = [(false, false); 4];
                    app.lobby_slots[0] = (true, app.my_ready);
                    for i in 0..3 {
                        if let Some(ref c) = app.host_clients[i] {
                            app.lobby_slots[i + 1] = (true, c.ready);
                        }
                    }

                    // Send lobby state to all clients
                    app.net_timer += dt;
                    if app.net_timer >= NET_SEND_RATE {
                        app.net_timer = 0.0;
                        let lobby_data = build_lobby_state(&app.lobby_slots);
                        if let Some(ref sock) = app.socket {
                            for i in 0..3 {
                                if let Some(ref c) = app.host_clients[i] {
                                    let _ = sock.send_to(&lobby_data, c.addr);
                                }
                            }
                        }
                    }

                    // Check if all ready
                    if app.my_ready {
                        let all_clients_ready = app.host_clients.iter().all(|c| match c {
                            Some(c) => c.ready,
                            None => true,
                        });
                        if all_clients_ready {
                            let num = 1 + app.host_clients.iter().filter(|c| c.is_some()).count() as u8;
                            // Send game start
                            if let Some(ref sock) = app.socket {
                                for i in 0..3 {
                                    if let Some(ref c) = app.host_clients[i] {
                                        let _ = sock.send_to(&[MSG_GAME_START, num], c.addr);
                                    }
                                }
                            }
                            app.game = new_game(num);
                            app.screen = Screen::Playing;
                            app.net_timer = 0.0;
                            play_sfx(&sounds.wave_start, 0.5);
                            play_music(&sounds.game_music, 0.12);
                        }
                    }

                    let ip = get_local_ip();
                    draw_lobby_ready(&app.lobby_slots, 0, true, &format!("{}:{}", ip, NET_PORT));

                } else {
                    // ── Client Lobby ──────────────────────────────
                    if !app.connected {
                        // Connection phase
                        let cx = screen_width() / 2.0;
                        let cy = screen_height() / 2.0;
                        draw_text_centered("DOLACZ DO GRY", cx, cy - 80.0, 40.0, Color::new(1.0, 0.8, 0.3, 1.0));
                        draw_text_centered("Wpisz IP hosta:", cx, cy - 30.0, 24.0, GRAY);

                        for k in [KeyCode::Key0, KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4,
                                  KeyCode::Key5, KeyCode::Key6, KeyCode::Key7, KeyCode::Key8, KeyCode::Key9] {
                            if is_key_pressed(k) {
                                let digit = match k {
                                    KeyCode::Key0 => '0', KeyCode::Key1 => '1', KeyCode::Key2 => '2',
                                    KeyCode::Key3 => '3', KeyCode::Key4 => '4', KeyCode::Key5 => '5',
                                    KeyCode::Key6 => '6', KeyCode::Key7 => '7', KeyCode::Key8 => '8',
                                    KeyCode::Key9 => '9', _ => '0',
                                };
                                app.ip_input.push(digit);
                            }
                        }
                        for k in [KeyCode::A, KeyCode::B, KeyCode::C, KeyCode::D, KeyCode::E, KeyCode::F] {
                            if is_key_pressed(k) {
                                let ch = match k {
                                    KeyCode::A => 'a', KeyCode::B => 'b', KeyCode::C => 'c',
                                    KeyCode::D => 'd', KeyCode::E => 'e', KeyCode::F => 'f',
                                    _ => 'a',
                                };
                                app.ip_input.push(ch);
                            }
                        }
                        if is_key_pressed(KeyCode::Period) || is_key_pressed(KeyCode::KpDecimal) {
                            app.ip_input.push('.');
                        }
                        if is_key_pressed(KeyCode::Semicolon) && is_key_down(KeyCode::LeftShift) ||
                           is_key_pressed(KeyCode::Semicolon) && is_key_down(KeyCode::RightShift) {
                            app.ip_input.push(':');
                        }
                        if is_key_pressed(KeyCode::LeftBracket) {
                            app.ip_input.push('[');
                        }
                        if is_key_pressed(KeyCode::RightBracket) {
                            app.ip_input.push(']');
                        }
                        if is_key_pressed(KeyCode::Backspace) {
                            app.ip_input.pop();
                        }

                        let display = format!("{}|", app.ip_input);
                        draw_text_centered(&display, cx, cy + 10.0, 32.0, WHITE);

                        draw_text_centered("[ENTER] Polacz", cx, cy + 60.0, 24.0, Color::new(0.3, 1.0, 0.3, 1.0));

                        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
                            let addr_str = if app.ip_input.contains(':') && !app.ip_input.starts_with('[') {
                                format!("[{}]:{}", app.ip_input, NET_PORT)
                            } else {
                                format!("{}:{}", app.ip_input, NET_PORT)
                            };
                            let is_ipv6 = addr_str.starts_with('[');
                            let bind_addr = if is_ipv6 { "[::]:0" } else { "0.0.0.0:0" };
                            if let Ok(sock) = UdpSocket::bind(bind_addr) {
                                sock.set_nonblocking(true).ok();
                                if let Ok(addr) = addr_str.parse() {
                                    app.peer_addr = Some(addr);
                                    app.socket = Some(sock);
                                    if let (Some(s), Some(a)) = (&app.socket, &app.peer_addr) {
                                        let _ = s.send_to(&[MSG_JOIN], a);
                                    }
                                }
                            }
                        }

                        // Waiting for accept
                        if let Some(ref sock) = app.socket {
                            if app.peer_addr.is_some() {
                                draw_text_centered("Laczenie...", cx, cy + 100.0, 20.0, YELLOW);
                                app.net_timer += dt;
                                if app.net_timer > 0.5 {
                                    app.net_timer = 0.0;
                                    if let Some(ref addr) = app.peer_addr {
                                        let _ = sock.send_to(&[MSG_JOIN], addr);
                                    }
                                }
                            }
                            if let Ok((n, _)) = sock.recv_from(&mut recv_buf) {
                                if n >= 2 && recv_buf[0] == MSG_ACCEPT {
                                    app.connected = true;
                                    app.dedicated = true;
                                    app.player_slot = recv_buf[1];
                                    app.my_ready = false;
                                    app.net_timer = 0.0;
                                }
                            }
                        }

                        draw_text_centered("[ESC] Powrot", cx, cy + 140.0, 20.0, GRAY);
                    } else {
                        // Connected - lobby ready phase
                        // Process network messages
                        let mut disconnected = false;
                        if let Some(ref sock) = app.socket {
                            loop {
                                match sock.recv_from(&mut recv_buf) {
                                    Ok((n, _)) if n > 0 => {
                                        match recv_buf[0] {
                                            MSG_LOBBY_STATE if n >= 9 => {
                                                app.lobby_slots = parse_lobby_state(&recv_buf[..n]);
                                            }
                                            MSG_GAME_START if n >= 2 => {
                                                let num = recv_buf[1];
                                                app.game = new_game(num);
                                                app.screen = Screen::Playing;
                                                app.net_timer = 0.0;
                                                play_sfx(&sounds.wave_start, 0.5);
                                                play_music(&sounds.game_music, 0.12);
                                            }
                                            MSG_DISCONNECT => {
                                                disconnected = true;
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => break,
                                }
                            }
                        }
                        if disconnected {
                            app.connected = false;
                            app.socket = None;
                            app.screen = Screen::Menu;
                        }

                        // Toggle ready
                        if is_key_pressed(KeyCode::Space) {
                            app.my_ready = !app.my_ready;
                            play_sfx(&sounds.menu_navigate, 0.5);
                            if let (Some(sock), Some(addr)) = (&app.socket, &app.peer_addr) {
                                let _ = sock.send_to(&[MSG_READY], addr);
                            }
                        }

                        draw_lobby_ready(&app.lobby_slots, app.player_slot, false, "");
                    }
                }
            }

            Screen::Browser => {
                if is_key_pressed(KeyCode::Escape) {
                    play_sfx(&sounds.menu_navigate, 0.5);
                    app.screen = Screen::Menu;
                    app.socket = None;
                    app.servers.clear();
                    next_frame().await;
                    continue;
                }

                if app.socket.is_none() {
                    if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
                        sock.set_nonblocking(true).ok();
                        sock.set_broadcast(true).ok();
                        app.socket = Some(sock);
                        app.discovery_timer = 0.0;
                    }
                }

                app.discovery_timer -= dt;
                if app.discovery_timer <= 0.0 {
                    app.discovery_timer = 1.0;
                    if let Some(ref sock) = app.socket {
                        let _ = sock.send_to(&[MSG_DISCOVERY_REQ],
                            format!("255.255.255.255:{}", DISCOVERY_PORT));
                        let ts = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap().as_millis() as u64;
                        let mut ping = Vec::with_capacity(9);
                        ping.push(MSG_PING);
                        ping.extend_from_slice(&ts.to_le_bytes());
                        for srv in &app.servers {
                            let ping_addr = SocketAddr::new(srv.addr.ip(), DISCOVERY_PORT);
                            let _ = sock.send_to(&ping, ping_addr);
                        }
                    }
                }

                if let Some(ref sock) = app.socket {
                    loop {
                        match sock.recv_from(&mut recv_buf) {
                            Ok((n, addr)) if n > 0 => {
                                if recv_buf[0] == MSG_DISCOVERY_RESP {
                                    if let Some((name, players, max_players, wave, score, game_port)) =
                                        deserialize_server_info(&recv_buf[..n])
                                    {
                                        let server_addr = SocketAddr::new(addr.ip(), game_port);
                                        let now = get_time();
                                        if let Some(existing) = app.servers.iter_mut()
                                            .find(|s| s.addr == server_addr)
                                        {
                                            existing.name = name;
                                            existing.players = players;
                                            existing.max_players = max_players;
                                            existing.wave = wave;
                                            existing.score = score;
                                            existing.last_seen = now;
                                        } else {
                                            app.servers.push(ServerInfo {
                                                name, players, max_players, wave, score,
                                                addr: server_addr, ping_ms: None, last_seen: now,
                                            });
                                        }
                                    }
                                } else if recv_buf[0] == MSG_PONG && n >= 9 {
                                    let ts = u64::from_le_bytes(
                                        recv_buf[1..9].try_into().unwrap()
                                    );
                                    let now_ms = SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap().as_millis() as u64;
                                    let ping = now_ms.saturating_sub(ts) as u32;
                                    for srv in &mut app.servers {
                                        if srv.addr.ip() == addr.ip() {
                                            srv.ping_ms = Some(ping);
                                        }
                                    }
                                }
                            }
                            _ => break,
                        }
                    }
                }

                let now = get_time();
                app.servers.retain(|s| now - s.last_seen < 5.0);

                if is_key_pressed(KeyCode::Up) && app.browser_selected > 0 {
                    app.browser_selected -= 1;
                    play_sfx(&sounds.menu_navigate, 0.5);
                }
                if is_key_pressed(KeyCode::Down) && app.browser_selected < app.servers.len().saturating_sub(1) {
                    app.browser_selected += 1;
                    play_sfx(&sounds.menu_navigate, 0.5);
                }

                if is_key_pressed(KeyCode::R) {
                    app.servers.clear();
                    app.browser_selected = 0;
                    app.discovery_timer = 0.0;
                    play_sfx(&sounds.menu_navigate, 0.5);
                }

                if is_key_pressed(KeyCode::T) {
                    play_sfx(&sounds.menu_select, 0.6);
                    app.net_role = NetRole::Client;
                    app.connected = false;
                    app.ip_input = String::new();
                    app.screen = Screen::Lobby;
                    app.socket = None;
                    app.dedicated = false;
                    app.player_slot = 0;
                    app.my_ready = false;
                }

                if (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter))
                    && !app.servers.is_empty()
                {
                    let idx = app.browser_selected.min(app.servers.len() - 1);
                    let addr = app.servers[idx].addr;
                    play_sfx(&sounds.menu_select, 0.6);
                    app.net_role = NetRole::Client;
                    app.connected = false;
                    app.peer_addr = Some(addr);
                    app.dedicated = true;
                    app.player_slot = 0;
                    app.my_ready = false;

                    if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
                        sock.set_nonblocking(true).ok();
                        let _ = sock.send_to(&[MSG_JOIN], addr);
                        app.socket = Some(sock);
                        app.screen = Screen::Lobby;
                        app.ip_input = addr.ip().to_string();
                        app.net_timer = 0.0;
                    }
                }

                draw_server_browser(&app.servers, app.browser_selected);
            }

            Screen::Playing => {
                if is_key_pressed(KeyCode::Escape) {
                    if app.net_role == NetRole::Client {
                        if let (Some(sock), Some(addr)) = (&app.socket, &app.peer_addr) {
                            let _ = sock.send_to(&[MSG_DISCONNECT], addr);
                        }
                    }
                    play_sfx(&sounds.menu_navigate, 0.5);
                    stop_music(&sounds.game_music);
                    app.screen = Screen::Menu;
                    app.socket = None;
                    app.connected = false;
                    app.dedicated = false;
                    app.player_slot = 0;
                    app.host_clients = [None, None, None];
                    app.my_ready = false;
                    next_frame().await;
                    continue;
                }

                match app.net_role {
                    NetRole::Solo => {
                        app.game.time += dt;
                        let local_input = gather_local_input(&app.game, 0);
                        let inputs = [RemoteInput {
                            dx: local_input.dx, dy: local_input.dy,
                            angle: local_input.angle, shooting: local_input.shooting,
                        }];
                        update_game(&mut app.game, &inputs, dt);
                        play_events(&app.game.events, &sounds);
                        app.game.events.clear();
                    }
                    NetRole::Host => {
                        // Receive input from clients
                        if let Some(ref sock) = app.socket {
                            loop {
                                match sock.recv_from(&mut recv_buf) {
                                    Ok((n, addr)) if n > 0 => {
                                        match recv_buf[0] {
                                            MSG_INPUT => {
                                                for i in 0..3 {
                                                    if let Some(ref mut c) = app.host_clients[i] {
                                                        if c.addr == addr {
                                                            c.input = deserialize_input(&recv_buf[..n]);
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                            MSG_DISCONNECT => {
                                                for i in 0..3 {
                                                    if let Some(ref c) = app.host_clients[i] {
                                                        if c.addr == addr {
                                                            app.host_clients[i] = None;
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                            MSG_JOIN => {
                                                // Late join during game - assign slot if available
                                                let mut assigned = None;
                                                for i in 0..3usize {
                                                    if app.host_clients[i].is_none() {
                                                        app.host_clients[i] = Some(HostClient {
                                                            addr,
                                                            input: RemoteInput::default(),
                                                            ready: true,
                                                        });
                                                        assigned = Some(i);
                                                        break;
                                                    }
                                                }
                                                if let Some(slot) = assigned {
                                                    let actual_slot = (slot + 1) as u8;
                                                    let _ = sock.send_to(&[MSG_ACCEPT, actual_slot], addr);
                                                    let _ = sock.send_to(&[MSG_GAME_START, app.game.num_players], addr);
                                                } else {
                                                    let _ = sock.send_to(&[MSG_DISCONNECT], addr);
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => break,
                                }
                            }
                        }

                        // Build inputs
                        let np = app.game.num_players as usize;
                        let local_input = gather_local_input(&app.game, 0);
                        let mut inputs = vec![RemoteInput::default(); np];
                        inputs[0] = RemoteInput {
                            dx: local_input.dx, dy: local_input.dy,
                            angle: local_input.angle, shooting: local_input.shooting,
                        };
                        for i in 0..3 {
                            if let Some(ref c) = app.host_clients[i] {
                                let slot = i + 1;
                                if slot < np {
                                    inputs[slot] = c.input.clone();
                                }
                            }
                        }

                        app.game.time += dt;
                        update_game(&mut app.game, &inputs, dt);
                        play_events(&app.game.events, &sounds);

                        // Send state to all clients
                        app.net_timer += dt;
                        if app.net_timer >= NET_SEND_RATE {
                            app.net_timer = 0.0;
                            let data = serialize_state(&app.game);
                            if let Some(ref sock) = app.socket {
                                for i in 0..3 {
                                    if let Some(ref c) = app.host_clients[i] {
                                        let _ = sock.send_to(&data, c.addr);
                                    }
                                }
                            }
                        }
                        app.game.events.clear();
                    }
                    NetRole::Client => {
                        let local_input = gather_local_input(&app.game, app.player_slot);
                        let rinp = RemoteInput {
                            dx: local_input.dx,
                            dy: local_input.dy,
                            angle: local_input.angle,
                            shooting: local_input.shooting,
                        };
                        if let (Some(sock), Some(addr)) = (&app.socket, &app.peer_addr) {
                            let data = serialize_input(&rinp);
                            let _ = sock.send_to(&data, addr);
                        }

                        if let Some(ref sock) = app.socket {
                            let mut got_state = false;
                            loop {
                                match sock.recv_from(&mut recv_buf) {
                                    Ok((n, _)) if n > 1 && recv_buf[0] == MSG_STATE => {
                                        deserialize_state(&recv_buf[..n], &mut app.game);
                                        got_state = true;
                                    }
                                    _ => break,
                                }
                            }
                            if got_state {
                                play_events(&app.game.events, &sounds);
                                app.game.events.clear();
                            }
                        }

                        client_extrapolate(&mut app.game, &local_input, app.player_slot, dt);
                        update_visuals(&mut app.game, dt);
                    }
                }

                let map_pw = map::MAP_W as f32 * TILE;
                let map_ph = map::MAP_H as f32 * TILE;
                app.camera_offset = Vec2::new(
                    (screen_width() - map_pw) / 2.0,
                    (screen_height() - map_ph) / 2.0,
                ).max(Vec2::ZERO);

                draw_game(&app.game, app.camera_offset);

                if app.game.game_over {
                    if app.dedicated {
                        draw_rectangle(0.0, 0.0, screen_width(), screen_height(),
                            Color::new(0.0, 0.0, 0.0, 0.6));
                        let cx = screen_width() / 2.0;
                        let cy = screen_height() / 2.0;
                        draw_text_centered("GAME OVER", cx, cy - 30.0, 50.0, RED);
                        draw_text_centered("Serwer restartuje gre...", cx, cy + 20.0, 24.0, GRAY);
                        draw_text_centered("[ESC] Wyjdz", cx, cy + 60.0, 20.0, GRAY);
                    } else {
                        stop_music(&sounds.game_music);
                        app.screen = Screen::GameOver;
                    }
                }
            }

            Screen::GameOver => {
                app.game.time += dt;
                update_visuals(&mut app.game, dt);
                draw_game(&app.game, app.camera_offset);
                draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.7));
                let cx = screen_width() / 2.0;
                let cy = screen_height() / 2.0;

                draw_text_centered("GAME OVER", cx, cy - 60.0, 60.0, RED);
                draw_text_centered(
                    &format!("Wave: {}  Score: {}  Kills: {}", app.game.wave, app.game.score, app.game.kills),
                    cx, cy, 28.0, WHITE,
                );
                draw_text_centered("Nacisnij [R] aby zagrac ponownie", cx, cy + 40.0, 24.0, GRAY);
                draw_text_centered("Nacisnij [ESC] - menu glowne", cx, cy + 70.0, 24.0, GRAY);

                if is_key_pressed(KeyCode::R) {
                    play_sfx(&sounds.menu_select, 0.6);
                    let np = app.game.num_players;
                    app.game = new_game(np);
                    app.screen = Screen::Playing;
                    play_sfx(&sounds.wave_start, 0.5);
                }
                if is_key_pressed(KeyCode::Escape) {
                    play_sfx(&sounds.menu_navigate, 0.5);
                    stop_music(&sounds.game_music);
                    app.screen = Screen::Menu;
                    app.socket = None;
                    app.connected = false;
                    app.host_clients = [None, None, None];
                }
            }
        }

        next_frame().await;
    }
}
