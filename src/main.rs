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
        window_width: 800,
        window_height: 608,
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
        game: new_game(false),
        selected_res: 0,
        camera_offset: Vec2::ZERO,
        net_role: NetRole::Solo,
        socket: None,
        peer_addr: None,
        remote_input: RemoteInput::default(),
        net_timer: 0.0,
        ip_input: String::new(),
        connected: false,
        servers: Vec::new(),
        browser_selected: 0,
        discovery_timer: 0.0,
        player_slot: 0,
        dedicated: false,
    };
    let mut recv_buf = [0u8; 65536];

    loop {
        let dt = get_frame_time().min(0.05);

        match app.screen {
            Screen::Menu => {
                // Ensure menu music is playing
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
                    app.game = new_game(false);
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
                let cx = screen_width() / 2.0;
                let cy = screen_height() / 2.0;

                if is_key_pressed(KeyCode::Escape) {
                    play_sfx(&sounds.menu_navigate, 0.5);
                    app.screen = Screen::Menu;
                    app.socket = None;
                    next_frame().await;
                    continue;
                }

                if app.net_role == NetRole::Host {
                    let ip = get_local_ip();
                    draw_text_centered("HOSTING GAME", cx, cy - 80.0, 40.0, Color::new(0.3, 0.8, 1.0, 1.0));
                    draw_text_centered(&format!("Twoje IP: {}:{}", ip, NET_PORT), cx, cy - 30.0, 28.0, WHITE);

                    if app.connected {
                        draw_text_centered("Gracz polaczony! Startuje...", cx, cy + 20.0, 24.0, Color::new(0.3, 1.0, 0.3, 1.0));
                        app.game = new_game(true);
                        app.screen = Screen::Playing;
                        app.net_timer = 0.0;
                        play_sfx(&sounds.wave_start, 0.5);
                        play_music(&sounds.game_music, 0.12);
                        if let (Some(sock), Some(addr)) = (&app.socket, &app.peer_addr) {
                            let _ = sock.send_to(&[2], addr);
                        }
                    } else {
                        let blink = (get_time() * 2.0).sin() > 0.0;
                        if blink {
                            draw_text_centered("Czekam na gracza...", cx, cy + 20.0, 24.0, GRAY);
                        }
                        if let Some(ref sock) = app.socket {
                            if let Ok((n, addr)) = sock.recv_from(&mut recv_buf) {
                                if n > 0 && recv_buf[0] == 1 {
                                    app.peer_addr = Some(addr);
                                    app.connected = true;
                                }
                            }
                        }
                    }
                    draw_text_centered("[ESC] Powrot", cx, cy + 80.0, 20.0, GRAY);

                } else {
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
                    if is_key_pressed(KeyCode::Period) || is_key_pressed(KeyCode::KpDecimal) {
                        app.ip_input.push('.');
                    }
                    if is_key_pressed(KeyCode::Backspace) {
                        app.ip_input.pop();
                    }

                    let display = format!("{}|", app.ip_input);
                    draw_text_centered(&display, cx, cy + 10.0, 32.0, WHITE);

                    if !app.connected {
                        draw_text_centered("[ENTER] Polacz", cx, cy + 60.0, 24.0, Color::new(0.3, 1.0, 0.3, 1.0));

                        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
                            let addr_str = format!("{}:{}", app.ip_input, NET_PORT);
                            if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
                                sock.set_nonblocking(true).ok();
                                if let Ok(addr) = addr_str.parse() {
                                    app.peer_addr = Some(addr);
                                    app.socket = Some(sock);
                                    if let (Some(s), Some(a)) = (&app.socket, &app.peer_addr) {
                                        let _ = s.send_to(&[1], a);
                                    }
                                }
                            }
                        }

                        if let Some(ref sock) = app.socket {
                            if app.peer_addr.is_some() {
                                draw_text_centered("Laczenie...", cx, cy + 100.0, 20.0, YELLOW);
                                app.net_timer += dt;
                                if app.net_timer > 0.5 {
                                    app.net_timer = 0.0;
                                    if let Some(ref addr) = app.peer_addr {
                                        let _ = sock.send_to(&[1], addr);
                                    }
                                }
                            }
                            if let Ok((n, _)) = sock.recv_from(&mut recv_buf) {
                                if n > 0 && recv_buf[0] == MSG_ACCEPT {
                                    app.connected = true;
                                    if n >= 2 {
                                        app.dedicated = true;
                                        app.player_slot = recv_buf[1];
                                    } else {
                                        app.player_slot = 1;
                                    }
                                    app.game = new_game(true);
                                    app.screen = Screen::Playing;
                                    app.net_timer = 0.0;
                                    play_sfx(&sounds.wave_start, 0.5);
                                    play_music(&sounds.game_music, 0.12);
                                }
                            }
                        }
                    }

                    draw_text_centered("[ESC] Powrot", cx, cy + 140.0, 20.0, GRAY);
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

                // Ensure discovery socket exists
                if app.socket.is_none() {
                    if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
                        sock.set_nonblocking(true).ok();
                        sock.set_broadcast(true).ok();
                        app.socket = Some(sock);
                        app.discovery_timer = 0.0;
                    }
                }

                // Broadcast discovery periodically
                app.discovery_timer -= dt;
                if app.discovery_timer <= 0.0 {
                    app.discovery_timer = 1.0;
                    if let Some(ref sock) = app.socket {
                        let _ = sock.send_to(&[MSG_DISCOVERY_REQ],
                            format!("255.255.255.255:{}", DISCOVERY_PORT));
                        // Ping known servers
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

                // Receive responses
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

                // Remove stale servers
                let now = get_time();
                app.servers.retain(|s| now - s.last_seen < 5.0);

                // Navigation
                if is_key_pressed(KeyCode::Up) && app.browser_selected > 0 {
                    app.browser_selected -= 1;
                    play_sfx(&sounds.menu_navigate, 0.5);
                }
                if is_key_pressed(KeyCode::Down) && app.browser_selected < app.servers.len().saturating_sub(1) {
                    app.browser_selected += 1;
                    play_sfx(&sounds.menu_navigate, 0.5);
                }

                // Refresh
                if is_key_pressed(KeyCode::R) {
                    app.servers.clear();
                    app.browser_selected = 0;
                    app.discovery_timer = 0.0;
                    play_sfx(&sounds.menu_navigate, 0.5);
                }

                // Manual IP entry
                if is_key_pressed(KeyCode::T) {
                    play_sfx(&sounds.menu_select, 0.6);
                    app.net_role = NetRole::Client;
                    app.connected = false;
                    app.ip_input = String::new();
                    app.screen = Screen::Lobby;
                    app.socket = None;
                    app.dedicated = false;
                    app.player_slot = 0;
                }

                // Connect to selected server
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
                    next_frame().await;
                    continue;
                }

                match app.net_role {
                    NetRole::Solo => {
                        app.game.time += dt;
                        let local_input = gather_local_input(&app.game, 0);
                        update_game(&mut app.game, &local_input, &RemoteInput::default(), dt);
                        play_events(&app.game.events, &sounds);
                        app.game.events.clear();
                    }
                    NetRole::Host => {
                        if let Some(ref sock) = app.socket {
                            loop {
                                match sock.recv_from(&mut recv_buf) {
                                    Ok((n, _)) if n > 0 && recv_buf[0] == 3 => {
                                        app.remote_input = deserialize_input(&recv_buf[..n]);
                                    }
                                    _ => break,
                                }
                            }
                        }

                        app.game.time += dt;
                        let local_input = gather_local_input(&app.game, 0);
                        update_game(&mut app.game, &local_input, &app.remote_input, dt);
                        play_events(&app.game.events, &sounds);

                        app.net_timer += dt;
                        if app.net_timer >= NET_SEND_RATE {
                            app.net_timer = 0.0;
                            if let (Some(sock), Some(addr)) = (&app.socket, &app.peer_addr) {
                                let data = serialize_state(&app.game);
                                let _ = sock.send_to(&data, addr);
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
                                    Ok((n, _)) if n > 1 && recv_buf[0] == 4 => {
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

                        // Client-side extrapolation for smooth 60fps rendering
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
                    let tp = app.game.two_player;
                    app.game = new_game(tp);
                    app.screen = Screen::Playing;
                    play_sfx(&sounds.wave_start, 0.5);
                }
                if is_key_pressed(KeyCode::Escape) {
                    play_sfx(&sounds.menu_navigate, 0.5);
                    stop_music(&sounds.game_music);
                    app.screen = Screen::Menu;
                    app.socket = None;
                    app.connected = false;
                }
            }
        }

        next_frame().await;
    }
}
