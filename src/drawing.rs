use macroquad::prelude::*;
use crate::map::{MAP, MAP_W, MAP_H};
use crate::types::*;

pub fn draw_text_centered(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    let dims = measure_text(text, None, font_size as u16, 1.0);
    draw_text(text, x - dims.width / 2.0, y, font_size, color);
}

pub fn draw_tile(tx: usize, ty: usize, time: f32) {
    let x = tx as f32 * TILE;
    let y = ty as f32 * TILE;
    let tile = MAP[ty][tx];
    match tile {
        0 => {
            draw_rectangle(x, y, TILE, TILE, Color::new(0.22, 0.52, 0.18, 1.0));
            let seed = (tx * 7 + ty * 13) as f32;
            if (seed % 3.0) < 1.0 {
                draw_rectangle(x + 8.0, y + 10.0, 2.0, 4.0, Color::new(0.28, 0.58, 0.22, 1.0));
                draw_rectangle(x + 20.0, y + 18.0, 2.0, 3.0, Color::new(0.18, 0.48, 0.15, 1.0));
            }
        }
        1 => {
            draw_rectangle(x, y, TILE, TILE, Color::new(0.55, 0.40, 0.25, 1.0));
            let seed = (tx * 11 + ty * 17) as f32;
            if (seed % 4.0) < 1.0 {
                draw_rectangle(x + 6.0, y + 14.0, 3.0, 3.0, Color::new(0.50, 0.36, 0.22, 1.0));
            }
        }
        2 => {
            draw_rectangle(x, y, TILE, TILE, Color::new(0.38, 0.38, 0.40, 1.0));
            draw_rectangle(x, y, TILE, 2.0, Color::new(0.48, 0.48, 0.50, 1.0));
            draw_rectangle(x, y + 15.0, TILE, 2.0, Color::new(0.30, 0.30, 0.32, 1.0));
            draw_rectangle(x + 15.0, y + 2.0, 2.0, 12.0, Color::new(0.32, 0.32, 0.34, 1.0));
            draw_rectangle(x + 7.0, y + 16.0, 2.0, 14.0, Color::new(0.32, 0.32, 0.34, 1.0));
        }
        3 => {
            draw_rectangle(x, y, TILE, TILE, Color::new(0.22, 0.52, 0.18, 1.0));
            draw_rectangle(x + 12.0, y + 18.0, 8.0, 14.0, Color::new(0.40, 0.26, 0.13, 1.0));
            draw_rectangle(x + 14.0, y + 20.0, 4.0, 10.0, Color::new(0.45, 0.30, 0.16, 1.0));
            draw_rectangle(x + 4.0, y + 2.0, 24.0, 18.0, Color::new(0.15, 0.45, 0.12, 1.0));
            draw_rectangle(x + 8.0, y, 16.0, 6.0, Color::new(0.18, 0.50, 0.15, 1.0));
            draw_rectangle(x + 2.0, y + 8.0, 28.0, 8.0, Color::new(0.12, 0.40, 0.10, 1.0));
        }
        4 => {
            let wave = (time * 2.0 + tx as f32 * 0.5 + ty as f32 * 0.3).sin() * 0.05;
            draw_rectangle(x, y, TILE, TILE, Color::new(0.15 + wave, 0.35 + wave, 0.65 + wave, 1.0));
            let shimmer = ((time * 3.0 + tx as f32 * 1.5).sin() + 1.0) * 0.5;
            if shimmer > 0.7 {
                draw_rectangle(x + 8.0, y + 6.0, 6.0, 2.0, Color::new(0.4, 0.6, 0.85, 0.6));
            }
        }
        5 => {
            draw_rectangle(x, y, TILE, TILE, Color::new(0.22, 0.52, 0.18, 1.0));
            let sway = (time * 2.0 + tx as f32).sin() * 1.0;
            draw_rectangle(x + 8.0 + sway, y + 8.0, 4.0, 4.0, Color::new(0.9, 0.3, 0.3, 1.0));
            draw_rectangle(x + 20.0 - sway, y + 16.0, 4.0, 4.0, Color::new(0.9, 0.85, 0.2, 1.0));
            draw_rectangle(x + 14.0 + sway, y + 24.0, 4.0, 4.0, Color::new(0.6, 0.3, 0.8, 1.0));
        }
        6 => {
            draw_rectangle(x, y, TILE, TILE, Color::new(0.18, 0.44, 0.14, 1.0));
            draw_rectangle(x + 4.0, y + 6.0, 2.0, 5.0, Color::new(0.15, 0.40, 0.12, 1.0));
            draw_rectangle(x + 22.0, y + 20.0, 2.0, 4.0, Color::new(0.20, 0.48, 0.16, 1.0));
        }
        _ => {}
    }
}

pub fn draw_player_sprite(p: &Player, time: f32, is_p2: bool) {
    if !p.alive { return; }
    let bob = (time * 8.0).sin() * 1.5;
    let flash = if p.damage_flash > 0.0 { 0.5 } else { 0.0 };

    draw_ellipse(p.x, p.y + 12.0, 10.0, 4.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.3));

    let body_color = if is_p2 {
        Color::new(0.2 + flash, 0.6, 0.35, 1.0)
    } else {
        Color::new(0.2 + flash, 0.35, 0.6, 1.0)
    };
    draw_rectangle(p.x - 7.0, p.y - 8.0 + bob, 14.0, 16.0, body_color);

    let head_color = Color::new(0.85 + flash, 0.72, 0.58, 1.0);
    draw_rectangle(p.x - 6.0, p.y - 16.0 + bob, 12.0, 10.0, head_color);

    let ex = p.angle.cos() * 2.0;
    let ey = p.angle.sin() * 1.5;
    draw_rectangle(p.x - 4.0 + ex, p.y - 12.0 + bob + ey, 3.0, 3.0, Color::new(0.1, 0.1, 0.1, 1.0));
    draw_rectangle(p.x + 2.0 + ex, p.y - 12.0 + bob + ey, 3.0, 3.0, Color::new(0.1, 0.1, 0.1, 1.0));

    let gx = p.angle.cos() * 14.0;
    let gy = p.angle.sin() * 14.0;
    draw_line(p.x + 4.0, p.y - 2.0 + bob, p.x + gx, p.y + gy, 3.0, Color::new(0.3, 0.3, 0.3, 1.0));
    draw_rectangle(p.x + gx - 2.0, p.y + gy - 2.0, 5.0, 5.0, Color::new(0.25, 0.25, 0.28, 1.0));

    let label = if is_p2 { "P2" } else { "P1" };
    let label_color = if is_p2 {
        Color::new(0.3, 0.9, 0.5, 0.8)
    } else {
        Color::new(0.3, 0.5, 0.9, 0.8)
    };
    let dims = measure_text(label, None, 14, 1.0);
    draw_text(label, p.x - dims.width / 2.0, p.y - 22.0, 14.0, label_color);
}

pub fn draw_zombie(z: &Zombie, time: f32) {
    let bob = (time * 6.0 + z.x * 0.1).sin() * 2.0;
    let flash = if z.damage_flash > 0.0 { 0.6 } else { 0.0 };

    draw_ellipse(z.x, z.y + 12.0, 9.0, 4.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.3));

    let is_fire = z.variant == 3;
    let (body_r, body_g) = match z.variant {
        0 => (0.35 + flash, 0.55),
        1 => (0.50 + flash, 0.45),
        3 => (0.85 + flash, 0.30),
        _ => (0.30 + flash, 0.50),
    };

    // Fire zombie flame aura
    if is_fire {
        let flicker1 = ((time * 12.0 + z.x * 0.3).sin() + 1.0) * 0.5;
        let flicker2 = ((time * 9.0 + z.y * 0.5).cos() + 1.0) * 0.5;
        draw_circle(z.x, z.y - 4.0 + bob, 14.0 + flicker1 * 4.0, Color::new(1.0, 0.4, 0.0, 0.15));
        draw_circle(z.x - 3.0, z.y - 10.0 + bob, 6.0 + flicker2 * 3.0, Color::new(1.0, 0.6, 0.0, 0.25));
        draw_circle(z.x + 4.0, z.y - 12.0 + bob, 5.0 + flicker1 * 2.0, Color::new(1.0, 0.3, 0.0, 0.2));
        // Flame tips above head
        let tip_y = z.y - 18.0 + bob - flicker1 * 5.0;
        draw_rectangle(z.x - 2.0, tip_y, 4.0, 6.0, Color::new(1.0, 0.8, 0.0, 0.4 * flicker2));
        draw_rectangle(z.x + 3.0, tip_y + 2.0, 3.0, 4.0, Color::new(1.0, 0.5, 0.0, 0.3 * flicker1));
        draw_rectangle(z.x - 5.0, tip_y + 3.0, 3.0, 3.0, Color::new(1.0, 0.6, 0.1, 0.3 * flicker2));
    }

    let body_b = if is_fire { 0.05 } else { 0.2 };
    draw_rectangle(z.x - 7.0, z.y - 6.0 + bob, 14.0, 14.0, Color::new(body_r, body_g, body_b, 1.0));
    draw_rectangle(z.x - 6.0, z.y - 14.0 + bob, 12.0, 10.0, Color::new(body_r - 0.05, body_g + 0.05, body_b + 0.02, 1.0));

    let glow = ((time * 5.0).sin() + 1.0) * 0.15;
    let eye_color = if is_fire {
        Color::new(1.0, 0.8 + glow, 0.0, 1.0)
    } else {
        Color::new(0.9 + glow, 0.15, 0.1, 1.0)
    };
    draw_rectangle(z.x - 4.0, z.y - 11.0 + bob, 3.0, 3.0, eye_color);
    draw_rectangle(z.x + 2.0, z.y - 11.0 + bob, 3.0, 3.0, eye_color);

    let arm_sway = (time * 4.0 + z.y * 0.1).sin() * 3.0;
    draw_rectangle(z.x - 11.0 + arm_sway, z.y - 4.0 + bob, 5.0, 4.0, Color::new(body_r, body_g, body_b, 1.0));
    draw_rectangle(z.x + 7.0 - arm_sway, z.y - 2.0 + bob, 5.0, 4.0, Color::new(body_r, body_g, body_b, 1.0));

    if z.hp < z.max_hp {
        let bar_w = 18.0;
        let ratio = z.hp as f32 / z.max_hp as f32;
        draw_rectangle(z.x - bar_w / 2.0, z.y - 20.0, bar_w, 3.0, Color::new(0.2, 0.0, 0.0, 0.8));
        draw_rectangle(z.x - bar_w / 2.0, z.y - 20.0, bar_w * ratio, 3.0, Color::new(0.8, 0.1, 0.1, 0.9));
    }
}

pub fn draw_pickup(p: &Pickup, time: f32) {
    let bob = (time * 3.0 + p.x * 0.1).sin() * 3.0;
    let glow = ((time * 4.0).sin() + 1.0) * 0.2;
    match p.kind {
        PickupKind::Health => {
            draw_rectangle(p.x - 6.0, p.y - 6.0 + bob, 12.0, 12.0, Color::new(0.9 + glow, 0.2, 0.2, 0.9));
            draw_rectangle(p.x - 2.0, p.y - 8.0 + bob, 4.0, 16.0, Color::new(1.0, 0.95, 0.95, 0.95));
            draw_rectangle(p.x - 8.0, p.y - 2.0 + bob, 16.0, 4.0, Color::new(1.0, 0.95, 0.95, 0.95));
        }
        PickupKind::Ammo => {
            draw_rectangle(p.x - 7.0, p.y - 5.0 + bob, 14.0, 10.0, Color::new(0.6 + glow, 0.55, 0.2, 0.9));
            draw_rectangle(p.x - 5.0, p.y - 3.0 + bob, 10.0, 6.0, Color::new(0.75, 0.7, 0.3, 0.9));
            draw_rectangle(p.x - 3.0, p.y - 2.0 + bob, 2.0, 4.0, Color::new(0.4, 0.35, 0.1, 1.0));
            draw_rectangle(p.x + 1.0, p.y - 2.0 + bob, 2.0, 4.0, Color::new(0.4, 0.35, 0.1, 1.0));
        }
    }
}

pub fn draw_menu(selected_res: usize) {
    clear_background(Color::new(0.08, 0.08, 0.12, 1.0));
    let cx = screen_width() / 2.0;
    let cy = screen_height() / 2.0;

    draw_text_centered("ZOMBIE SURVIVAL", cx, cy - 180.0, 52.0, Color::new(0.9, 0.2, 0.15, 1.0));

    draw_text_centered("Rozdzielczosc (lewo/prawo):", cx, cy - 100.0, 24.0, GRAY);
    let (rw, rh) = RESOLUTIONS[selected_res];
    let al = if selected_res > 0 { "< " } else { "  " };
    let ar = if selected_res < RESOLUTIONS.len() - 1 { " >" } else { "  " };
    draw_text_centered(&format!("{}{} x {}{}", al, rw, rh, ar), cx, cy - 65.0, 32.0, WHITE);

    draw_text_centered("--- Tryb gry ---", cx, cy - 20.0, 24.0, Color::new(0.6, 0.8, 1.0, 1.0));

    let blink = (get_time() * 3.0).sin() > 0.0;
    if blink {
        draw_text_centered("[1] Graj SOLO", cx, cy + 20.0, 28.0, Color::new(0.3, 1.0, 0.3, 1.0));
        draw_text_centered("[2] Hostuj gre (LAN)", cx, cy + 55.0, 28.0, Color::new(0.3, 0.8, 1.0, 1.0));
        draw_text_centered("[3] Dolacz do gry (LAN)", cx, cy + 90.0, 28.0, Color::new(1.0, 0.8, 0.3, 1.0));
        draw_text_centered("[4] Przegladarka serwerow", cx, cy + 125.0, 28.0, Color::new(0.8, 0.3, 1.0, 1.0));
    } else {
        draw_text_centered("[1] Graj SOLO", cx, cy + 20.0, 28.0, Color::new(0.2, 0.7, 0.2, 1.0));
        draw_text_centered("[2] Hostuj gre (LAN)", cx, cy + 55.0, 28.0, Color::new(0.2, 0.6, 0.8, 1.0));
        draw_text_centered("[3] Dolacz do gry (LAN)", cx, cy + 90.0, 28.0, Color::new(0.8, 0.6, 0.2, 1.0));
        draw_text_centered("[4] Przegladarka serwerow", cx, cy + 125.0, 28.0, Color::new(0.6, 0.2, 0.8, 1.0));
    }

    draw_text_centered("--- Sterowanie ---", cx, cy + 170.0, 20.0, Color::new(0.5, 0.5, 0.5, 1.0));
    draw_text_centered("WASD - ruch | Myszka - cel | LPM - strzal", cx, cy + 195.0, 18.0, GRAY);
}

pub fn draw_server_browser(servers: &[ServerInfo], selected: usize) {
    clear_background(Color::new(0.08, 0.08, 0.12, 1.0));
    let cx = screen_width() / 2.0;

    draw_text_centered("PRZEGLADARKA SERWEROW", cx, 50.0, 40.0, Color::new(0.8, 0.3, 1.0, 1.0));

    let header_y = 95.0;
    let col_name = 40.0;
    let col_players = 300.0;
    let col_wave = 400.0;
    let col_score = 490.0;
    let col_ping = 580.0;
    let col_ip = 660.0;

    draw_text("Nazwa", col_name, header_y, 20.0, Color::new(0.6, 0.6, 0.8, 1.0));
    draw_text("Gracze", col_players, header_y, 20.0, Color::new(0.6, 0.6, 0.8, 1.0));
    draw_text("Wave", col_wave, header_y, 20.0, Color::new(0.6, 0.6, 0.8, 1.0));
    draw_text("Score", col_score, header_y, 20.0, Color::new(0.6, 0.6, 0.8, 1.0));
    draw_text("Ping", col_ping, header_y, 20.0, Color::new(0.6, 0.6, 0.8, 1.0));
    draw_text("IP", col_ip, header_y, 20.0, Color::new(0.6, 0.6, 0.8, 1.0));

    draw_line(30.0, header_y + 8.0, screen_width() - 30.0, header_y + 8.0, 1.0,
        Color::new(0.3, 0.3, 0.5, 1.0));

    if servers.is_empty() {
        let blink = (get_time() * 2.0).sin() > 0.0;
        if blink {
            draw_text_centered("Szukam serwerow w sieci LAN...", cx, 200.0, 24.0, GRAY);
        }
    } else {
        for (i, srv) in servers.iter().enumerate() {
            let y = 125.0 + i as f32 * 30.0;
            if i == selected {
                draw_rectangle(30.0, y - 15.0, screen_width() - 60.0, 28.0,
                    Color::new(0.2, 0.15, 0.35, 0.8));
            }
            let color = if i == selected { WHITE } else { Color::new(0.7, 0.7, 0.7, 1.0) };
            draw_text(&srv.name, col_name, y, 20.0, color);
            draw_text(&format!("{}/{}", srv.players, srv.max_players), col_players, y, 20.0, color);
            draw_text(&format!("{}", srv.wave), col_wave, y, 20.0, color);
            draw_text(&format!("{}", srv.score), col_score, y, 20.0, color);
            let ping_color = match srv.ping_ms {
                Some(ms) if ms < 50 => Color::new(0.3, 1.0, 0.3, 1.0),
                Some(ms) if ms < 100 => Color::new(1.0, 1.0, 0.3, 1.0),
                Some(_) => Color::new(1.0, 0.3, 0.3, 1.0),
                None => GRAY,
            };
            let ping_str = match srv.ping_ms {
                Some(ms) => format!("{}ms", ms),
                None => "?".to_string(),
            };
            draw_text(&ping_str, col_ping, y, 20.0, ping_color);
            draw_text(&format!("{}", srv.addr.ip()), col_ip, y, 18.0,
                Color::new(0.5, 0.5, 0.5, 1.0));
        }
    }

    let footer_y = screen_height() - 40.0;
    draw_rectangle(0.0, footer_y - 15.0, screen_width(), 55.0, Color::new(0.0, 0.0, 0.0, 0.5));
    draw_text_centered(
        "[ENTER] Polacz  [R] Odswiez  [T] Wpisz IP  [ESC] Powrot",
        cx, footer_y + 10.0, 20.0, GRAY,
    );
}

pub fn draw_game(state: &GameState, _offset: Vec2) {
    clear_background(Color::new(0.05, 0.05, 0.08, 1.0));

    let cam = Camera2D {
        target: vec2(MAP_W as f32 * TILE / 2.0, MAP_H as f32 * TILE / 2.0),
        zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
        offset: vec2(0.0, 0.0),
        ..Default::default()
    };
    set_camera(&cam);

    for ty in 0..MAP_H {
        for tx in 0..MAP_W { draw_tile(tx, ty, state.time); }
    }

    for pk in &state.pickups { draw_pickup(pk, state.time); }

    for p in &state.particles {
        let alpha = (p.life * 3.0).min(1.0);
        let c = Color::new(p.color.r, p.color.g, p.color.b, alpha);
        draw_rectangle(p.x - p.size / 2.0, p.y - p.size / 2.0, p.size, p.size, c);
    }

    for b in &state.bullets {
        let color = if b.owner == 0 { YELLOW } else { Color::new(0.3, 1.0, 0.5, 1.0) };
        draw_rectangle(b.x - 2.0, b.y - 2.0, 4.0, 4.0, color);
        let trail_color = if b.owner == 0 {
            Color::new(1.0, 1.0, 0.5, 0.5)
        } else {
            Color::new(0.3, 1.0, 0.5, 0.3)
        };
        draw_line(b.x, b.y, b.x - b.dx * 0.02, b.y - b.dy * 0.02, 2.0, trail_color);
    }

    for f in &state.flashes {
        let size = f.life * 200.0;
        draw_circle(f.x, f.y, size, Color::new(1.0, 0.9, 0.4, f.life * 10.0));
    }

    for z in &state.zombies { draw_zombie(z, state.time); }

    draw_player_sprite(&state.player1, state.time, false);
    if state.two_player {
        draw_player_sprite(&state.player2, state.time, true);
    }

    for d in &state.dmg_numbers {
        let alpha = (d.life * 2.0).min(1.0);
        draw_text(&d.value.to_string(), d.x - 8.0, d.y, 20.0, Color::new(1.0, 0.3, 0.1, alpha));
    }

    set_default_camera();

    // ── HUD ───────────────────────────────────────────────
    let hud_y = screen_height() - 45.0;
    draw_rectangle(0.0, hud_y - 5.0, screen_width(), 50.0, Color::new(0.0, 0.0, 0.0, 0.7));
    draw_rectangle(0.0, hud_y - 5.0, screen_width(), 2.0, Color::new(0.3, 0.3, 0.3, 0.8));

    let hp1_ratio = state.player1.hp as f32 / state.player1.max_hp as f32;
    let hp1_color = if !state.player1.alive { DARKGRAY }
        else if hp1_ratio > 0.5 { Color::new(0.2, 0.8, 0.2, 1.0) }
        else if hp1_ratio > 0.25 { Color::new(0.9, 0.7, 0.1, 1.0) }
        else { Color::new(0.9, 0.15, 0.1, 1.0) };
    draw_text("P1", 10.0, hud_y + 20.0, 18.0, Color::new(0.3, 0.5, 0.9, 1.0));
    draw_rectangle(30.0, hud_y + 8.0, 80.0, 14.0, Color::new(0.2, 0.0, 0.0, 0.8));
    draw_rectangle(30.0, hud_y + 8.0, 80.0 * hp1_ratio.max(0.0), 14.0, hp1_color);
    draw_text(&format!("{}", state.player1.ammo), 115.0, hud_y + 22.0, 18.0,
        Color::new(0.9, 0.85, 0.2, 1.0));

    if state.two_player {
        let hp2_ratio = state.player2.hp as f32 / state.player2.max_hp as f32;
        let hp2_color = if !state.player2.alive { DARKGRAY }
            else if hp2_ratio > 0.5 { Color::new(0.2, 0.8, 0.2, 1.0) }
            else if hp2_ratio > 0.25 { Color::new(0.9, 0.7, 0.1, 1.0) }
            else { Color::new(0.9, 0.15, 0.1, 1.0) };
        draw_text("P2", 150.0, hud_y + 20.0, 18.0, Color::new(0.3, 0.9, 0.5, 1.0));
        draw_rectangle(170.0, hud_y + 8.0, 80.0, 14.0, Color::new(0.2, 0.0, 0.0, 0.8));
        draw_rectangle(170.0, hud_y + 8.0, 80.0 * hp2_ratio.max(0.0), 14.0, hp2_color);
        draw_text(&format!("{}", state.player2.ammo), 255.0, hud_y + 22.0, 18.0,
            Color::new(0.9, 0.85, 0.2, 1.0));
    }

    draw_text(&format!("WAVE: {}", state.wave), 350.0, hud_y + 22.0, 20.0, Color::new(0.5, 0.8, 1.0, 1.0));
    draw_text(&format!("SCORE: {}", state.score), 500.0, hud_y + 22.0, 20.0, Color::new(1.0, 0.85, 0.3, 1.0));
    draw_text(&format!("KILLS: {}", state.kills), 660.0, hud_y + 22.0, 20.0, Color::new(0.8, 0.5, 0.5, 1.0));

    if state.wave_delay > 1.0 && state.zombies_to_spawn == 0 && state.zombies.is_empty() && state.wave > 0 {
        draw_text_centered(
            &format!("Wave {} cleared!", state.wave),
            screen_width() / 2.0, screen_height() / 2.0 - 40.0, 40.0,
            Color::new(0.3, 1.0, 0.3, 1.0),
        );
    }
    if state.wave == 0 && state.wave_delay > 0.5 {
        draw_text_centered("Get Ready!", screen_width() / 2.0, screen_height() / 2.0 - 40.0, 40.0,
            Color::new(1.0, 1.0, 0.5, 1.0));
        draw_text_centered("WASD - ruch | Myszka - cel | LPM - strzal",
            screen_width() / 2.0, screen_height() / 2.0 + 10.0, 20.0, GRAY);
    }

    let no_ammo = state.player1.ammo == 0 && state.player1.alive;
    if no_ammo && !state.game_over {
        let blink = (state.time * 6.0).sin() > 0.0;
        if blink {
            draw_text_centered("NO AMMO!", screen_width() / 2.0, 80.0, 24.0, RED);
        }
    }

    let my_hp = state.player1.hp;
    if my_hp < 30 && my_hp > 0 {
        let alpha = ((state.time * 4.0).sin() + 1.0) * 0.15;
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.8, 0.0, 0.0, alpha));
    }

    if state.two_player {
        draw_text("LAN CO-OP", screen_width() - 100.0, 20.0, 18.0, Color::new(0.5, 0.8, 1.0, 0.6));
    }
}
