use macroquad::prelude::*;
use crate::map::*;
use crate::types::*;

// ── Particle helpers ──────────────────────────────────────
pub fn spawn_blood(particles: &mut Vec<Particle>, x: f32, y: f32, count: usize) {
    for _ in 0..count {
        let angle = rand::gen_range(0.0f32, std::f32::consts::TAU);
        let speed = rand::gen_range(30.0f32, 120.0);
        particles.push(Particle {
            x, y,
            dx: angle.cos() * speed,
            dy: angle.sin() * speed,
            life: rand::gen_range(0.2, 0.6),
            color: Color::new(
                rand::gen_range(0.5, 0.8),
                rand::gen_range(0.0, 0.1),
                rand::gen_range(0.0, 0.05),
                1.0,
            ),
            size: rand::gen_range(2.0, 5.0),
        });
    }
}

pub fn spawn_sparks(particles: &mut Vec<Particle>, x: f32, y: f32) {
    for _ in 0..4 {
        let angle = rand::gen_range(0.0f32, std::f32::consts::TAU);
        let speed = rand::gen_range(60.0f32, 150.0);
        particles.push(Particle {
            x, y,
            dx: angle.cos() * speed,
            dy: angle.sin() * speed,
            life: rand::gen_range(0.1, 0.3),
            color: Color::new(1.0, rand::gen_range(0.7, 1.0), 0.2, 1.0),
            size: rand::gen_range(1.5, 3.0),
        });
    }
}

// ── Input ─────────────────────────────────────────────────
pub fn gather_local_input(state: &GameState, my_slot: u8) -> LocalInput {
    let mut dx = 0.0f32;
    let mut dy = 0.0f32;
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) { dy -= 1.0; }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) { dy += 1.0; }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) { dx -= 1.0; }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) { dx += 1.0; }

    let (mx, my) = mouse_position();
    let cam = Camera2D {
        target: vec2(MAP_W as f32 * TILE / 2.0, MAP_H as f32 * TILE / 2.0),
        zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
        offset: vec2(0.0, 0.0),
        ..Default::default()
    };
    let wp = cam.screen_to_world(vec2(mx, my));

    let me = if my_slot == 1 { &state.player2 } else { &state.player1 };
    let angle = (wp.y - me.y).atan2(wp.x - me.x);

    let shooting = is_mouse_button_down(MouseButton::Left) || is_key_down(KeyCode::Space);

    LocalInput { dx, dy, angle, shooting }
}

// ── New game ──────────────────────────────────────────────
pub fn new_player(x: f32, y: f32) -> Player {
    Player {
        x, y, angle: 0.0,
        hp: 100, max_hp: 100,
        fire_timer: 0.0, damage_flash: 0.0,
        ammo: 60, alive: true,
    }
}

pub fn new_game(two_player: bool) -> GameState {
    let cx = MAP_W as f32 * TILE / 2.0;
    let cy = MAP_H as f32 * TILE / 2.0;
    GameState {
        player1: new_player(cx - 20.0, cy),
        player2: new_player(cx + 20.0, cy),
        two_player,
        bullets: Vec::new(),
        zombies: Vec::new(),
        pickups: Vec::new(),
        particles: Vec::new(),
        flashes: Vec::new(),
        dmg_numbers: Vec::new(),
        wave: 0,
        zombies_to_spawn: 0,
        spawn_timer: 0.0,
        wave_delay: 2.0,
        score: 0,
        kills: 0,
        game_over: false,
        screen_shake: 0.0,
        time: 0.0,
        events: Vec::new(),
    }
}

// ── Update single player ──────────────────────────────────
fn update_single_player(
    p: &mut Player, inp_dx: f32, inp_dy: f32, inp_angle: f32, inp_shooting: bool,
    bullets: &mut Vec<Bullet>, flashes: &mut Vec<MuzzleFlash>,
    events: &mut Vec<u8>, owner: u8, dt: f32,
) {
    if !p.alive { return; }

    let mut dx = inp_dx;
    let mut dy = inp_dy;
    let len = (dx * dx + dy * dy).sqrt();
    if len > 0.0 {
        dx /= len;
        dy /= len;
        let nx = p.x + dx * PLAYER_SPEED * dt;
        let ny = p.y + dy * PLAYER_SPEED * dt;
        if can_move(nx, p.y, 6.0) { p.x = nx; }
        if can_move(p.x, ny, 6.0) { p.y = ny; }
    }

    p.angle = inp_angle;
    p.fire_timer -= dt;
    p.damage_flash = (p.damage_flash - dt * 5.0).max(0.0);

    if inp_shooting && p.fire_timer <= 0.0 && p.ammo > 0 {
        p.fire_timer = FIRE_COOLDOWN;
        p.ammo -= 1;
        let spread = rand::gen_range(-0.08f32, 0.08);
        let angle = p.angle + spread;
        let gx = p.x + angle.cos() * 16.0;
        let gy = p.y + angle.sin() * 16.0;
        bullets.push(Bullet {
            x: gx, y: gy,
            dx: angle.cos() * BULLET_SPEED,
            dy: angle.sin() * BULLET_SPEED,
            alive: true, owner,
        });
        flashes.push(MuzzleFlash { x: gx, y: gy, life: 0.06 });
        events.push(SND_SHOOT);
    } else if inp_shooting && p.fire_timer <= 0.0 && p.ammo == 0 {
        p.fire_timer = FIRE_COOLDOWN * 2.0;
        events.push(SND_NO_AMMO);
    }
}

// ── Main update ───────────────────────────────────────────
pub fn update_game(state: &mut GameState, local_input: &LocalInput, remote_input: &RemoteInput, dt: f32) {
    state.screen_shake = (state.screen_shake - dt * 30.0).max(0.0);

    update_single_player(
        &mut state.player1, local_input.dx, local_input.dy,
        local_input.angle, local_input.shooting,
        &mut state.bullets, &mut state.flashes,
        &mut state.events, 0, dt,
    );
    if local_input.shooting && state.player1.alive {
        state.screen_shake = state.screen_shake.max(2.0);
    }

    if state.two_player {
        update_single_player(
            &mut state.player2, remote_input.dx, remote_input.dy,
            remote_input.angle, remote_input.shooting,
            &mut state.bullets, &mut state.flashes,
            &mut state.events, 1, dt,
        );
    }

    // Bullets
    for b in &mut state.bullets {
        if !b.alive { continue; }
        b.x += b.dx * dt;
        b.y += b.dy * dt;
        if is_solid(tile_at(b.x, b.y)) {
            b.alive = false;
            spawn_sparks(&mut state.particles, b.x, b.y);
            continue;
        }
        if b.x < 0.0 || b.x > MAP_W as f32 * TILE || b.y < 0.0 || b.y > MAP_H as f32 * TILE {
            b.alive = false;
        }
    }

    // Zombies
    let p1_alive = state.player1.alive;
    let p2_alive = state.two_player && state.player2.alive;
    let p1x = state.player1.x; let p1y = state.player1.y;
    let p2x = state.player2.x; let p2y = state.player2.y;

    for z in &mut state.zombies {
        if !z.alive { continue; }
        z.damage_flash = (z.damage_flash - dt * 5.0).max(0.0);
        z.attack_timer = (z.attack_timer - dt).max(0.0);

        let (tx, ty) = {
            let d1 = if p1_alive { ((p1x - z.x).powi(2) + (p1y - z.y).powi(2)).sqrt() } else { f32::MAX };
            let d2 = if p2_alive { ((p2x - z.x).powi(2) + (p2y - z.y).powi(2)).sqrt() } else { f32::MAX };
            if d1 <= d2 { (p1x, p1y) } else { (p2x, p2y) }
        };

        let to_x = tx - z.x;
        let to_y = ty - z.y;
        let dist = (to_x * to_x + to_y * to_y).sqrt();
        if dist > 1.0 {
            let nx = z.x + (to_x / dist) * z.speed * dt;
            let ny = z.y + (to_y / dist) * z.speed * dt;
            if can_move(nx, z.y, 6.0) { z.x = nx; }
            if can_move(z.x, ny, 6.0) { z.y = ny; }
        }

        if z.attack_timer <= 0.0 {
            for (player_idx, is_alive) in [(0u8, p1_alive), (1u8, p2_alive)] {
                let (px, py) = if player_idx == 0 { (p1x, p1y) } else { (p2x, p2y) };
                let dx = px - z.x;
                let dy = py - z.y;
                if dx * dx + dy * dy < 18.0 * 18.0 && is_alive {
                    let player = if player_idx == 0 { &mut state.player1 } else { &mut state.player2 };
                    let dmg = if z.variant == 3 { ZOMBIE_FIRE_ATTACK_DMG } else { ZOMBIE_ATTACK_DMG };
                    player.hp -= dmg;
                    player.damage_flash = 1.0;
                    z.attack_timer = ZOMBIE_ATTACK_INTERVAL;
                    state.events.push(SND_HURT);
                    if player.hp <= 0 {
                        player.alive = false;
                        if !state.player1.alive && (!state.two_player || !state.player2.alive) {
                            state.game_over = true;
                        }
                    }
                    break;
                }
            }
        }

        for b in &mut state.bullets {
            if !b.alive { continue; }
            let bx = b.x - z.x;
            let by = b.y - z.y;
            if bx * bx + by * by < 144.0 {
                b.alive = false;
                let dmg = rand::gen_range(20, 35);
                z.hp -= dmg;
                z.damage_flash = 1.0;
                spawn_blood(&mut state.particles, z.x, z.y, 6);
                state.dmg_numbers.push(DamageNumber {
                    x: z.x + rand::gen_range(-5.0, 5.0),
                    y: z.y - 20.0,
                    value: dmg, life: 0.8,
                });
                state.events.push(SND_ZOMBIE_HIT);
                if z.hp <= 0 {
                    z.alive = false;
                    spawn_blood(&mut state.particles, z.x, z.y, 15);
                    state.score += 10 * state.wave;
                    state.kills += 1;
                    state.events.push(SND_ZOMBIE_DEATH);
                    if rand::gen_range(0.0f32, 1.0) < 0.25 {
                        let kind = if rand::gen_range(0.0f32, 1.0) < 0.4 {
                            PickupKind::Health
                        } else {
                            PickupKind::Ammo
                        };
                        state.pickups.push(Pickup {
                            x: z.x, y: z.y, kind, alive: true, timer: 30.0,
                        });
                    }
                }
                break;
            }
        }
    }

    // Pickups
    for pk in &mut state.pickups {
        if !pk.alive { continue; }
        pk.timer -= dt;
        if pk.timer <= 0.0 { pk.alive = false; continue; }

        for pidx in 0..2u8 {
            if pidx == 1 && !state.two_player { continue; }
            let player = if pidx == 0 { &state.player1 } else { &state.player2 };
            if !player.alive { continue; }
            let ddx = player.x - pk.x;
            let ddy = player.y - pk.y;
            if ddx * ddx + ddy * ddy < PICKUP_RANGE * PICKUP_RANGE {
                pk.alive = false;
                state.events.push(SND_PICKUP);
                let player = if pidx == 0 { &mut state.player1 } else { &mut state.player2 };
                match pk.kind {
                    PickupKind::Health => player.hp = (player.hp + 25).min(player.max_hp),
                    PickupKind::Ammo => player.ammo += 15,
                }
                break;
            }
        }
    }

    update_visuals(state, dt);

    // Cleanup
    state.bullets.retain(|b| b.alive);
    state.zombies.retain(|z| z.alive);
    state.pickups.retain(|p| p.alive);
    state.particles.retain(|p| p.life > 0.0);
    state.flashes.retain(|f| f.life > 0.0);
    state.dmg_numbers.retain(|d| d.life > 0.0);

    // Wave logic
    let alive_zombies = state.zombies.len() as u32;
    if state.zombies_to_spawn == 0 && alive_zombies == 0 {
        state.wave_delay -= dt;
        if state.wave_delay <= 0.0 {
            state.wave += 1;
            let base = if state.two_player { 5 } else { 3 };
            state.zombies_to_spawn = base + state.wave * 2;
            state.wave_delay = 3.0;
            let ammo_bonus = 10 + (state.wave as i32) * 2;
            state.player1.ammo += ammo_bonus;
            if state.two_player { state.player2.ammo += ammo_bonus; }
            state.events.push(SND_WAVE);

            if state.two_player {
                if !state.player1.alive {
                    state.player1.alive = true;
                    state.player1.hp = 50;
                    state.player1.x = MAP_W as f32 * TILE / 2.0 - 20.0;
                    state.player1.y = MAP_H as f32 * TILE / 2.0;
                }
                if !state.player2.alive {
                    state.player2.alive = true;
                    state.player2.hp = 50;
                    state.player2.x = MAP_W as f32 * TILE / 2.0 + 20.0;
                    state.player2.y = MAP_H as f32 * TILE / 2.0;
                }
            }
        }
    }

    // Spawn zombies
    if state.zombies_to_spawn > 0 {
        state.spawn_timer -= dt;
        if state.spawn_timer <= 0.0 {
            state.spawn_timer = 0.4;
            state.zombies_to_spawn -= 1;
            let (sx, sy) = loop {
                let edge = rand::gen_range(0u32, 4);
                let (x, y) = match edge {
                    0 => (rand::gen_range(1.0, MAP_W as f32 - 1.0) * TILE, TILE * 1.5),
                    1 => (rand::gen_range(1.0, MAP_W as f32 - 1.0) * TILE, (MAP_H as f32 - 1.5) * TILE),
                    2 => (TILE * 1.5, rand::gen_range(1.0, MAP_H as f32 - 1.0) * TILE),
                    _ => ((MAP_W as f32 - 1.5) * TILE, rand::gen_range(1.0, MAP_H as f32 - 1.0) * TILE),
                };
                if !is_solid(tile_at(x, y)) { break (x, y); }
            };
            let wave = state.wave;
            let hp = 50 + (wave as i32) * 15;
            let is_fire = wave >= 2 && rand::gen_range(0.0f32, 1.0) < 0.3;
            let variant = if is_fire { 3 } else { rand::gen_range(0u8, 3) };
            let base_speed = ZOMBIE_BASE_SPEED + (wave as f32) * 3.0 + rand::gen_range(-10.0, 10.0);
            let speed = if is_fire { base_speed * 1.3 } else { base_speed };
            state.zombies.push(Zombie {
                x: sx, y: sy, hp, max_hp: hp, alive: true,
                speed, damage_flash: 0.0, variant, attack_timer: 0.0,
            });
        }
    }
}

pub fn update_visuals(state: &mut GameState, dt: f32) {
    let damping = 0.95_f32.powf(dt * 60.0);
    for p in &mut state.particles {
        p.x += p.dx * dt;
        p.y += p.dy * dt;
        p.dx *= damping;
        p.dy *= damping;
        p.life -= dt;
    }
    for f in &mut state.flashes { f.life -= dt; }
    for d in &mut state.dmg_numbers { d.y -= 40.0 * dt; d.life -= dt; }
    state.particles.retain(|p| p.life > 0.0);
    state.flashes.retain(|f| f.life > 0.0);
    state.dmg_numbers.retain(|d| d.life > 0.0);
}
