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

fn spawn_explosion(particles: &mut Vec<Particle>, x: f32, y: f32) {
    for _ in 0..30 {
        let angle = rand::gen_range(0.0f32, std::f32::consts::TAU);
        let speed = rand::gen_range(50.0f32, 200.0);
        let is_fire = rand::gen_range(0.0f32, 1.0) < 0.6;
        let color = if is_fire {
            Color::new(1.0, rand::gen_range(0.3, 0.8), 0.0, 1.0)
        } else {
            Color::new(rand::gen_range(0.3, 0.6), rand::gen_range(0.8, 1.0), 0.0, 1.0)
        };
        particles.push(Particle {
            x, y,
            dx: angle.cos() * speed,
            dy: angle.sin() * speed,
            life: rand::gen_range(0.3, 0.8),
            color,
            size: rand::gen_range(3.0, 7.0),
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
    let map_w = MAP_W as f32 * TILE;
    let map_h = MAP_H as f32 * TILE;
    let s = (screen_width() / map_w).min(screen_height() / map_h);
    let cam = Camera2D {
        target: vec2(map_w / 2.0, map_h / 2.0),
        zoom: vec2(s * 2.0 / screen_width(), s * 2.0 / screen_height()),
        ..Default::default()
    };
    let wp = cam.screen_to_world(vec2(mx, my));

    let slot = my_slot as usize;
    let me = if slot < state.players.len() { &state.players[slot] } else { &state.players[0] };
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
        ammo: 999, alive: true,
    }
}

pub fn new_game(num_players: u8) -> GameState {
    let cx = MAP_W as f32 * TILE / 2.0;
    let cy = MAP_H as f32 * TILE / 2.0;
    let spawn_offsets: [(f32, f32); 4] = [
        (-30.0, -20.0), (30.0, -20.0),
        (-30.0,  20.0), (30.0,  20.0),
    ];
    let np = (num_players as usize).min(4).max(1);
    let mut players = Vec::with_capacity(np);
    for i in 0..np {
        let (ox, oy) = spawn_offsets[i];
        players.push(new_player(cx + ox, cy + oy));
    }
    GameState {
        players,
        num_players: np as u8,
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

// ── Main update ───────────────────────────────────────────
pub fn update_game(state: &mut GameState, inputs: &[RemoteInput], dt: f32) {
    state.screen_shake = (state.screen_shake - dt * 30.0).max(0.0);
    let np = state.num_players as usize;

    // Update each player
    for i in 0..np.min(state.players.len()) {
        let inp = if i < inputs.len() { &inputs[i] } else { &RemoteInput::default() };
        if !state.players[i].alive { continue; }

        let mut pdx = inp.dx;
        let mut pdy = inp.dy;
        let len = (pdx * pdx + pdy * pdy).sqrt();
        if len > 0.0 {
            pdx /= len;
            pdy /= len;
            let nx = state.players[i].x + pdx * PLAYER_SPEED * dt;
            let ny = state.players[i].y + pdy * PLAYER_SPEED * dt;
            if can_move(nx, state.players[i].y, 6.0) { state.players[i].x = nx; }
            if can_move(state.players[i].x, ny, 6.0) { state.players[i].y = ny; }
        }

        state.players[i].angle = inp.angle;
        state.players[i].fire_timer -= dt;
        state.players[i].damage_flash = (state.players[i].damage_flash - dt * 5.0).max(0.0);

        if inp.shooting && state.players[i].fire_timer <= 0.0 && state.players[i].ammo > 0 {
            state.players[i].fire_timer = FIRE_COOLDOWN;
            state.players[i].ammo -= 1;
            let spread = rand::gen_range(-0.08f32, 0.08);
            let angle = state.players[i].angle + spread;
            let gx = state.players[i].x + angle.cos() * 16.0;
            let gy = state.players[i].y + angle.sin() * 16.0;
            state.bullets.push(Bullet {
                x: gx, y: gy,
                dx: angle.cos() * BULLET_SPEED,
                dy: angle.sin() * BULLET_SPEED,
                alive: true, owner: i as u8,
            });
            state.flashes.push(MuzzleFlash { x: gx, y: gy, life: 0.06 });
            state.events.push(SND_SHOOT);
            state.screen_shake = state.screen_shake.max(2.0);
        } else if inp.shooting && state.players[i].fire_timer <= 0.0 && state.players[i].ammo == 0 {
            state.players[i].fire_timer = FIRE_COOLDOWN * 2.0;
            state.events.push(SND_NO_AMMO);
        }
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

    // Pre-compute player positions for zombie AI
    let player_data: Vec<(f32, f32, bool)> = (0..np.min(state.players.len()))
        .map(|i| (state.players[i].x, state.players[i].y, state.players[i].alive))
        .collect();

    // Zombies (index-based for split borrows)
    for zi in 0..state.zombies.len() {
        if !state.zombies[zi].alive { continue; }
        state.zombies[zi].damage_flash = (state.zombies[zi].damage_flash - dt * 5.0).max(0.0);
        state.zombies[zi].attack_timer = (state.zombies[zi].attack_timer - dt).max(0.0);

        // Find closest player
        let zx = state.zombies[zi].x;
        let zy = state.zombies[zi].y;
        let mut min_dist = f32::MAX;
        let mut tx = zx;
        let mut ty = zy;
        for &(px, py, alive) in &player_data {
            if !alive { continue; }
            let d = ((px - zx).powi(2) + (py - zy).powi(2)).sqrt();
            if d < min_dist { min_dist = d; tx = px; ty = py; }
        }

        // Movement
        let to_x = tx - zx;
        let to_y = ty - zy;
        let dist = (to_x * to_x + to_y * to_y).sqrt();
        if dist > 1.0 {
            let spd = state.zombies[zi].speed;
            let nx = zx + (to_x / dist) * spd * dt;
            let ny = zy + (to_y / dist) * spd * dt;
            if can_move(nx, zy, 6.0) { state.zombies[zi].x = nx; }
            let new_zx = state.zombies[zi].x;
            if can_move(new_zx, ny, 6.0) { state.zombies[zi].y = ny; }
        }

        // Attack players
        if state.zombies[zi].attack_timer <= 0.0 {
            for pi in 0..np.min(state.players.len()) {
                if !state.players[pi].alive { continue; }
                let adx = state.players[pi].x - state.zombies[zi].x;
                let ady = state.players[pi].y - state.zombies[zi].y;
                if adx * adx + ady * ady < 18.0 * 18.0 {
                    let dmg = if state.zombies[zi].variant == 3 { ZOMBIE_FIRE_ATTACK_DMG } else { ZOMBIE_ATTACK_DMG };
                    state.players[pi].hp -= dmg;
                    state.players[pi].damage_flash = 1.0;
                    state.zombies[zi].attack_timer = ZOMBIE_ATTACK_INTERVAL;
                    state.events.push(SND_HURT);
                    if state.players[pi].hp <= 0 {
                        state.players[pi].alive = false;
                        if (0..np.min(state.players.len())).all(|j| !state.players[j].alive) {
                            state.game_over = true;
                        }
                    }
                    break;
                }
            }
        }

        // Bullet collision
        for bi in 0..state.bullets.len() {
            if !state.bullets[bi].alive { continue; }
            let bx = state.bullets[bi].x - state.zombies[zi].x;
            let by = state.bullets[bi].y - state.zombies[zi].y;
            if bx * bx + by * by < 144.0 {
                state.bullets[bi].alive = false;
                let dmg = rand::gen_range(20, 35);
                state.zombies[zi].hp -= dmg;
                state.zombies[zi].damage_flash = 1.0;
                let zpos_x = state.zombies[zi].x;
                let zpos_y = state.zombies[zi].y;
                spawn_blood(&mut state.particles, zpos_x, zpos_y, 6);
                state.dmg_numbers.push(DamageNumber {
                    x: zpos_x + rand::gen_range(-5.0, 5.0),
                    y: zpos_y - 20.0,
                    value: dmg, life: 0.8,
                });
                state.events.push(SND_ZOMBIE_HIT);
                if state.zombies[zi].hp <= 0 {
                    state.zombies[zi].alive = false;
                    spawn_blood(&mut state.particles, zpos_x, zpos_y, 15);
                    state.score += 10 * state.wave;
                    state.kills += 1;
                    state.events.push(SND_ZOMBIE_DEATH);
                    if rand::gen_range(0.0f32, 1.0) < 0.25 {
                        let kind = if rand::gen_range(0.0f32, 1.0) < 0.4 { PickupKind::Health } else { PickupKind::Ammo };
                        state.pickups.push(Pickup {
                            x: zpos_x, y: zpos_y, kind, alive: true, timer: 30.0,
                        });
                    }
                }
                break;
            }
        }
    }

    // Explosive zombie detonation (variant 4)
    let explosions: Vec<(f32, f32)> = state.zombies.iter()
        .filter(|z| !z.alive && z.variant == 4)
        .map(|z| (z.x, z.y))
        .collect();

    for &(ex, ey) in &explosions {
        // Damage nearby zombies
        for z in &mut state.zombies {
            if !z.alive { continue; }
            let ddx = z.x - ex;
            let ddy = z.y - ey;
            if ddx * ddx + ddy * ddy < ZOMBIE_EXPLOSION_RADIUS * ZOMBIE_EXPLOSION_RADIUS {
                z.hp -= ZOMBIE_EXPLOSION_DMG;
                z.damage_flash = 1.0;
                spawn_blood(&mut state.particles, z.x, z.y, 6);
                state.dmg_numbers.push(DamageNumber {
                    x: z.x + rand::gen_range(-5.0, 5.0),
                    y: z.y - 20.0,
                    value: ZOMBIE_EXPLOSION_DMG, life: 0.8,
                });
                if z.hp <= 0 {
                    z.alive = false;
                    spawn_blood(&mut state.particles, z.x, z.y, 15);
                    state.score += 10 * state.wave;
                    state.kills += 1;
                    state.events.push(SND_ZOMBIE_DEATH);
                }
            }
        }
        // Damage nearby players
        for pi in 0..np.min(state.players.len()) {
            if !state.players[pi].alive { continue; }
            let ddx = state.players[pi].x - ex;
            let ddy = state.players[pi].y - ey;
            if ddx * ddx + ddy * ddy < ZOMBIE_EXPLOSION_RADIUS * ZOMBIE_EXPLOSION_RADIUS {
                state.players[pi].hp -= ZOMBIE_EXPLOSION_DMG;
                state.players[pi].damage_flash = 1.0;
                state.events.push(SND_HURT);
                if state.players[pi].hp <= 0 {
                    state.players[pi].alive = false;
                    if (0..np.min(state.players.len())).all(|j| !state.players[j].alive) {
                        state.game_over = true;
                    }
                }
            }
        }
        spawn_explosion(&mut state.particles, ex, ey);
        state.screen_shake = state.screen_shake.max(8.0);
        state.events.push(SND_EXPLOSION);
    }

    // Pickups
    for pk in &mut state.pickups {
        if !pk.alive { continue; }
        pk.timer -= dt;
        if pk.timer <= 0.0 { pk.alive = false; continue; }

        for pidx in 0..np.min(state.players.len()) {
            if !state.players[pidx].alive { continue; }
            let ddx = state.players[pidx].x - pk.x;
            let ddy = state.players[pidx].y - pk.y;
            if ddx * ddx + ddy * ddy < PICKUP_RANGE * PICKUP_RANGE {
                pk.alive = false;
                state.events.push(SND_PICKUP);
                match pk.kind {
                    PickupKind::Health => {
                        let max = state.players[pidx].max_hp;
                        state.players[pidx].hp = (state.players[pidx].hp + 25).min(max);
                    }
                    PickupKind::Ammo => state.players[pidx].ammo += 15,
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
            let base = if np > 1 { 3 + np as u32 } else { 3 };
            state.zombies_to_spawn = base + state.wave * 2;
            state.wave_delay = 3.0;
            let ammo_bonus = 10 + (state.wave as i32) * 2;
            for i in 0..np.min(state.players.len()) {
                state.players[i].ammo += ammo_bonus;
            }
            state.events.push(SND_WAVE);

            // Revive dead players at wave start (multiplayer)
            if np > 1 {
                let cx = MAP_W as f32 * TILE / 2.0;
                let cy = MAP_H as f32 * TILE / 2.0;
                let spawn_offsets: [(f32, f32); 4] = [
                    (-30.0, -20.0), (30.0, -20.0),
                    (-30.0,  20.0), (30.0,  20.0),
                ];
                for i in 0..np.min(state.players.len()) {
                    if !state.players[i].alive {
                        state.players[i].alive = true;
                        state.players[i].hp = 50;
                        state.players[i].x = cx + spawn_offsets[i].0;
                        state.players[i].y = cy + spawn_offsets[i].1;
                    }
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
            let roll = rand::gen_range(0.0f32, 1.0);
            let is_fire = wave >= 2 && roll < 0.25;
            let is_explosive = wave >= 2 && !is_fire && roll < 0.35;
            let variant = if is_explosive { 4 } else if is_fire { 3 } else { rand::gen_range(0u8, 3) };
            let base_speed = ZOMBIE_BASE_SPEED + (wave as f32) * 3.0 + rand::gen_range(-10.0, 10.0);
            let speed = if is_fire { base_speed * 1.3 } else if is_explosive { base_speed * 0.8 } else { base_speed };
            let zhp = if is_explosive { hp / 2 } else { hp };
            state.zombies.push(Zombie {
                x: sx, y: sy, hp: zhp, max_hp: zhp, alive: true,
                speed, damage_flash: 0.0, variant, attack_timer: 0.0,
            });
        }
    }
}

// ── Client-side extrapolation ─────────────────────────────
pub fn client_extrapolate(state: &mut GameState, local_input: &LocalInput, my_slot: u8, dt: f32) {
    let np = state.num_players as usize;
    let slot = my_slot as usize;

    // Move local player
    if slot < np && slot < state.players.len() && state.players[slot].alive {
        let mut dx = local_input.dx;
        let mut dy = local_input.dy;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            dx /= len;
            dy /= len;
            let nx = state.players[slot].x + dx * PLAYER_SPEED * dt;
            let ny = state.players[slot].y + dy * PLAYER_SPEED * dt;
            if can_move(nx, state.players[slot].y, 6.0) { state.players[slot].x = nx; }
            if can_move(state.players[slot].x, ny, 6.0) { state.players[slot].y = ny; }
        }
        state.players[slot].angle = local_input.angle;
        state.players[slot].damage_flash = (state.players[slot].damage_flash - dt * 5.0).max(0.0);
    }

    // Update other players' damage flash
    for i in 0..np.min(state.players.len()) {
        if i == slot { continue; }
        state.players[i].damage_flash = (state.players[i].damage_flash - dt * 5.0).max(0.0);
    }

    // Extrapolate bullets
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
    state.bullets.retain(|b| b.alive);

    // Extrapolate zombies
    let player_data: Vec<(f32, f32, bool)> = (0..np.min(state.players.len()))
        .map(|i| (state.players[i].x, state.players[i].y, state.players[i].alive))
        .collect();

    for z in &mut state.zombies {
        if !z.alive { continue; }
        z.damage_flash = (z.damage_flash - dt * 5.0).max(0.0);

        let mut min_dist = f32::MAX;
        let mut tx = z.x;
        let mut ty = z.y;
        for &(px, py, alive) in &player_data {
            if !alive { continue; }
            let d = ((px - z.x).powi(2) + (py - z.y).powi(2)).sqrt();
            if d < min_dist { min_dist = d; tx = px; ty = py; }
        }

        let to_x = tx - z.x;
        let to_y = ty - z.y;
        let dist = (to_x * to_x + to_y * to_y).sqrt();
        if dist > 1.0 {
            let nx = z.x + (to_x / dist) * z.speed * dt;
            let ny = z.y + (to_y / dist) * z.speed * dt;
            if can_move(nx, z.y, 6.0) { z.x = nx; }
            if can_move(z.x, ny, 6.0) { z.y = ny; }
        }
    }

    state.screen_shake = (state.screen_shake - dt * 30.0).max(0.0);
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
