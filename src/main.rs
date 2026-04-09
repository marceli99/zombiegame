use macroquad::prelude::*;
use macroquad::audio::{Sound, load_sound_from_bytes, play_sound, PlaySoundParams};

// ── Constants ──────────────────────────────────────────────
const TILE: f32 = 32.0;
const MAP_W: usize = 25;
const MAP_H: usize = 19;
const PLAYER_SPEED: f32 = 150.0;
const BULLET_SPEED: f32 = 500.0;
const ZOMBIE_BASE_SPEED: f32 = 45.0;
const FIRE_COOLDOWN: f32 = 0.18;
const PICKUP_RANGE: f32 = 40.0;

const RESOLUTIONS: [(i32, i32); 5] = [
    (800, 608),
    (1024, 768),
    (1280, 720),
    (1600, 900),
    (1920, 1080),
];

#[derive(PartialEq)]
enum Screen {
    Menu,
    Playing,
    GameOver,
}

// ── Map ────────────────────────────────────────────────────
// 0=grass, 1=path, 2=wall, 3=tree, 4=water, 5=flowers, 6=dark grass
const MAP: [[u8; MAP_W]; MAP_H] = [
    [3,3,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,3,3],
    [3,0,0,5,0,0,6,0,0,0,1,1,1,1,1,0,0,0,6,0,0,5,0,0,3],
    [2,0,6,0,0,0,0,0,5,0,1,0,0,0,1,0,5,0,0,0,0,0,6,0,2],
    [2,0,0,0,3,0,0,0,0,0,1,0,0,0,1,0,0,0,0,0,3,0,0,0,2],
    [2,0,5,0,0,0,0,6,0,0,1,0,0,0,1,0,0,6,0,0,0,0,5,0,2],
    [2,0,0,0,0,0,0,0,0,1,1,0,0,0,1,1,0,0,0,0,0,0,0,0,2],
    [2,6,0,0,0,4,4,0,0,1,0,0,0,0,0,1,0,0,4,4,0,0,0,6,2],
    [2,0,0,0,0,4,4,0,0,1,0,0,0,0,0,1,0,0,4,4,0,0,0,0,2],
    [2,0,0,0,0,0,0,0,1,1,0,0,0,0,0,1,1,0,0,0,0,0,0,0,2],
    [1,1,1,1,1,1,1,1,1,0,0,0,0,0,0,0,1,1,1,1,1,1,1,1,1],
    [2,0,0,0,0,0,0,0,1,1,0,0,0,0,0,1,1,0,0,0,0,0,0,0,2],
    [2,0,0,0,0,4,4,0,0,1,0,0,0,0,0,1,0,0,4,4,0,0,0,0,2],
    [2,6,0,0,0,4,4,0,0,1,0,0,0,0,0,1,0,0,4,4,0,0,0,6,2],
    [2,0,0,0,0,0,0,0,0,1,1,0,0,0,1,1,0,0,0,0,0,0,0,0,2],
    [2,0,5,0,0,0,0,6,0,0,1,0,0,0,1,0,0,6,0,0,0,0,5,0,2],
    [2,0,0,0,3,0,0,0,0,0,1,0,0,0,1,0,0,0,0,0,3,0,0,0,2],
    [2,0,6,0,0,0,0,0,5,0,1,0,0,0,1,0,5,0,0,0,0,0,6,0,2],
    [3,0,0,5,0,0,6,0,0,0,1,1,1,1,1,0,0,0,6,0,0,5,0,0,3],
    [3,3,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,3,3],
];

fn is_solid(tile: u8) -> bool {
    matches!(tile, 2 | 3 | 4)
}

fn tile_at(x: f32, y: f32) -> u8 {
    let tx = (x / TILE) as usize;
    let ty = (y / TILE) as usize;
    if tx < MAP_W && ty < MAP_H { MAP[ty][tx] } else { 2 }
}

fn can_move(x: f32, y: f32, half: f32) -> bool {
    !is_solid(tile_at(x - half, y - half))
        && !is_solid(tile_at(x + half, y - half))
        && !is_solid(tile_at(x - half, y + half))
        && !is_solid(tile_at(x + half, y + half))
}

// ── Structs ────────────────────────────────────────────────
struct Player {
    x: f32,
    y: f32,
    angle: f32,
    hp: i32,
    max_hp: i32,
    fire_timer: f32,
    damage_flash: f32,
    ammo: i32,
}

struct Bullet {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    alive: bool,
}

struct Zombie {
    x: f32,
    y: f32,
    hp: i32,
    max_hp: i32,
    alive: bool,
    speed: f32,
    damage_flash: f32,
    variant: u8,
}

#[derive(Clone, Copy, PartialEq)]
enum PickupKind {
    Health,
    Ammo,
}

struct Pickup {
    x: f32,
    y: f32,
    kind: PickupKind,
    alive: bool,
    timer: f32,
}

struct Particle {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    life: f32,
    color: Color,
    size: f32,
}

struct MuzzleFlash {
    x: f32,
    y: f32,
    life: f32,
}

struct DamageNumber {
    x: f32,
    y: f32,
    value: i32,
    life: f32,
}

struct GameState {
    player: Player,
    bullets: Vec<Bullet>,
    zombies: Vec<Zombie>,
    pickups: Vec<Pickup>,
    particles: Vec<Particle>,
    flashes: Vec<MuzzleFlash>,
    dmg_numbers: Vec<DamageNumber>,
    wave: u32,
    zombies_to_spawn: u32,
    spawn_timer: f32,
    wave_delay: f32,
    score: u32,
    kills: u32,
    game_over: bool,
    screen_shake: f32,
    time: f32,
    use_keyboard_aim: bool,
    kb_aim_angle: f32,
}

struct AppState {
    screen: Screen,
    game: GameState,
    selected_res: usize,
    camera_offset: Vec2,
}

// ── Drawing helpers ────────────────────────────────────────
fn draw_tile(tx: usize, ty: usize, time: f32) {
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

fn draw_player(p: &Player, time: f32) {
    let bob = (time * 8.0).sin() * 1.5;
    let flash = if p.damage_flash > 0.0 { 0.5 } else { 0.0 };

    draw_ellipse(p.x, p.y + 12.0, 10.0, 4.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.3));

    let body_color = Color::new(0.2 + flash, 0.35, 0.6, 1.0);
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
}

fn draw_zombie(z: &Zombie, time: f32) {
    let bob = (time * 6.0 + z.x * 0.1).sin() * 2.0;
    let flash = if z.damage_flash > 0.0 { 0.6 } else { 0.0 };

    draw_ellipse(z.x, z.y + 12.0, 9.0, 4.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.3));

    let (body_r, body_g) = match z.variant {
        0 => (0.35 + flash, 0.55),
        1 => (0.50 + flash, 0.45),
        _ => (0.30 + flash, 0.50),
    };

    draw_rectangle(z.x - 7.0, z.y - 6.0 + bob, 14.0, 14.0, Color::new(body_r, body_g, 0.2, 1.0));
    draw_rectangle(z.x - 6.0, z.y - 14.0 + bob, 12.0, 10.0, Color::new(body_r - 0.05, body_g + 0.05, 0.22, 1.0));

    let glow = ((time * 5.0).sin() + 1.0) * 0.15;
    draw_rectangle(z.x - 4.0, z.y - 11.0 + bob, 3.0, 3.0, Color::new(0.9 + glow, 0.15, 0.1, 1.0));
    draw_rectangle(z.x + 2.0, z.y - 11.0 + bob, 3.0, 3.0, Color::new(0.9 + glow, 0.15, 0.1, 1.0));

    let arm_sway = (time * 4.0 + z.y * 0.1).sin() * 3.0;
    draw_rectangle(z.x - 11.0 + arm_sway, z.y - 4.0 + bob, 5.0, 4.0, Color::new(body_r, body_g, 0.2, 1.0));
    draw_rectangle(z.x + 7.0 - arm_sway, z.y - 2.0 + bob, 5.0, 4.0, Color::new(body_r, body_g, 0.2, 1.0));

    if z.hp < z.max_hp {
        let bar_w = 18.0;
        let ratio = z.hp as f32 / z.max_hp as f32;
        draw_rectangle(z.x - bar_w / 2.0, z.y - 20.0, bar_w, 3.0, Color::new(0.2, 0.0, 0.0, 0.8));
        draw_rectangle(z.x - bar_w / 2.0, z.y - 20.0, bar_w * ratio, 3.0, Color::new(0.8, 0.1, 0.1, 0.9));
    }
}

fn draw_pickup(p: &Pickup, time: f32) {
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

fn spawn_blood(particles: &mut Vec<Particle>, x: f32, y: f32, count: usize) {
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

fn spawn_sparks(particles: &mut Vec<Particle>, x: f32, y: f32) {
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

fn draw_text_centered(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    let dims = measure_text(text, None, font_size as u16, 1.0);
    draw_text(text, x - dims.width / 2.0, y, font_size, color);
}

// ── Sound generation ──────────────────────────────────────
fn make_wav(samples: &[i16]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let file_len = 36 + data_len;
    let sample_rate: u32 = 44100;
    let mut buf = Vec::with_capacity(44 + data_len as usize);
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_len.to_le_bytes());
    buf.extend_from_slice(b"WAVEfmt ");
    buf.extend_from_slice(&16u32.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes()); // PCM
    buf.extend_from_slice(&1u16.to_le_bytes()); // mono
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
    buf.extend_from_slice(&2u16.to_le_bytes()); // block align
    buf.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_len.to_le_bytes());
    for &s in samples {
        buf.extend_from_slice(&s.to_le_bytes());
    }
    buf
}

fn gen_shoot_sound() -> Vec<u8> {
    let n = 2200; // ~50ms
    let mut samples = vec![0i16; n];
    for i in 0..n {
        let t = i as f32 / 44100.0;
        let env = (1.0 - i as f32 / n as f32).powi(2);
        let noise = ((i as f32 * 12345.6789).sin() * 43758.5453).fract() * 2.0 - 1.0;
        let tone = (t * 800.0 * std::f32::consts::TAU).sin() * 0.3;
        samples[i] = ((noise * 0.7 + tone) * env * 12000.0) as i16;
    }
    make_wav(&samples)
}

fn gen_zombie_death_sound() -> Vec<u8> {
    let n = 6600; // ~150ms
    let mut samples = vec![0i16; n];
    for i in 0..n {
        let t = i as f32 / 44100.0;
        let env = (1.0 - i as f32 / n as f32).powi(2);
        let freq = 120.0 - t * 300.0;
        let s = (t * freq * std::f32::consts::TAU).sin();
        let noise = ((i as f32 * 7654.321).sin() * 43758.5453).fract() * 2.0 - 1.0;
        samples[i] = ((s * 0.6 + noise * 0.4) * env * 14000.0) as i16;
    }
    make_wav(&samples)
}

fn gen_zombie_hit_sound() -> Vec<u8> {
    let n = 2000;
    let mut samples = vec![0i16; n];
    for i in 0..n {
        let t = i as f32 / 44100.0;
        let env = (1.0 - i as f32 / n as f32).powi(3);
        let s = (t * 200.0 * std::f32::consts::TAU).sin();
        let noise = ((i as f32 * 9876.543).sin() * 43758.5453).fract() * 2.0 - 1.0;
        samples[i] = ((s * 0.5 + noise * 0.5) * env * 8000.0) as i16;
    }
    make_wav(&samples)
}

fn gen_pickup_sound() -> Vec<u8> {
    let n = 4400; // ~100ms
    let mut samples = vec![0i16; n];
    for i in 0..n {
        let t = i as f32 / 44100.0;
        let env = (1.0 - i as f32 / n as f32) * (i as f32 / 400.0).min(1.0);
        let freq = 600.0 + t * 1200.0;
        let s = (t * freq * std::f32::consts::TAU).sin();
        samples[i] = (s * env * 10000.0) as i16;
    }
    make_wav(&samples)
}

fn gen_hurt_sound() -> Vec<u8> {
    let n = 4400;
    let mut samples = vec![0i16; n];
    for i in 0..n {
        let t = i as f32 / 44100.0;
        let env = (1.0 - i as f32 / n as f32).powi(2);
        let s = (t * 150.0 * std::f32::consts::TAU).sin();
        let s2 = (t * 180.0 * std::f32::consts::TAU).sin();
        samples[i] = ((s * 0.5 + s2 * 0.5) * env * 12000.0) as i16;
    }
    make_wav(&samples)
}

fn gen_wave_sound() -> Vec<u8> {
    let n = 13230; // ~300ms
    let mut samples = vec![0i16; n];
    for i in 0..n {
        let t = i as f32 / 44100.0;
        let env = (1.0 - i as f32 / n as f32) * (i as f32 / 800.0).min(1.0);
        let freq = 400.0 + t * 800.0;
        let s = (t * freq * std::f32::consts::TAU).sin();
        let s2 = (t * freq * 1.5 * std::f32::consts::TAU).sin() * 0.3;
        samples[i] = ((s + s2) * env * 8000.0) as i16;
    }
    make_wav(&samples)
}

fn gen_no_ammo_sound() -> Vec<u8> {
    let n = 2200;
    let mut samples = vec![0i16; n];
    for i in 0..n {
        let t = i as f32 / 44100.0;
        let env = (1.0 - i as f32 / n as f32).powi(2);
        let click = if i < 200 { 1.0 } else { 0.0 };
        let tone = (t * 300.0 * std::f32::consts::TAU).sin() * 0.5;
        samples[i] = ((click + tone) * env * 6000.0) as i16;
    }
    make_wav(&samples)
}

struct Sounds {
    shoot: Sound,
    zombie_hit: Sound,
    zombie_death: Sound,
    pickup: Sound,
    hurt: Sound,
    wave_start: Sound,
    no_ammo: Sound,
}

async fn load_sounds() -> Sounds {
    Sounds {
        shoot: load_sound_from_bytes(&gen_shoot_sound()).await.unwrap(),
        zombie_hit: load_sound_from_bytes(&gen_zombie_hit_sound()).await.unwrap(),
        zombie_death: load_sound_from_bytes(&gen_zombie_death_sound()).await.unwrap(),
        pickup: load_sound_from_bytes(&gen_pickup_sound()).await.unwrap(),
        hurt: load_sound_from_bytes(&gen_hurt_sound()).await.unwrap(),
        wave_start: load_sound_from_bytes(&gen_wave_sound()).await.unwrap(),
        no_ammo: load_sound_from_bytes(&gen_no_ammo_sound()).await.unwrap(),
    }
}

fn play_sfx(sound: &Sound, volume: f32) {
    play_sound(sound, PlaySoundParams { looped: false, volume });
}

// ── Main ───────────────────────────────────────────────────
fn window_conf() -> Conf {
    Conf {
        window_title: "Zombie Survival".to_string(),
        window_width: 800,
        window_height: 608,
        window_resizable: true,
        ..Default::default()
    }
}

fn draw_menu(selected_res: usize) {
    clear_background(Color::new(0.08, 0.08, 0.12, 1.0));
    let cx = screen_width() / 2.0;
    let cy = screen_height() / 2.0;

    draw_text_centered("ZOMBIE SURVIVAL", cx, cy - 160.0, 52.0, Color::new(0.9, 0.2, 0.15, 1.0));

    // Resolution selection
    draw_text_centered("Rozdzielczosc (lewo/prawo):", cx, cy - 80.0, 24.0, GRAY);
    let (rw, rh) = RESOLUTIONS[selected_res];
    let arrow_left = if selected_res > 0 { "< " } else { "  " };
    let arrow_right = if selected_res < RESOLUTIONS.len() - 1 { " >" } else { "  " };
    draw_text_centered(
        &format!("{}{} x {}{}", arrow_left, rw, rh, arrow_right),
        cx, cy - 45.0, 32.0, WHITE,
    );

    draw_text_centered("--- Sterowanie ---", cx, cy + 10.0, 22.0, Color::new(0.6, 0.8, 1.0, 1.0));
    draw_text_centered("WASD / Strzalki - ruch", cx, cy + 40.0, 20.0, GRAY);
    draw_text_centered("Myszka - celowanie + LPM strzal", cx, cy + 65.0, 20.0, GRAY);
    draw_text_centered("IJKL - celowanie klawiatura", cx, cy + 90.0, 20.0, GRAY);
    draw_text_centered("Spacja - strzal (klawiatura)", cx, cy + 115.0, 20.0, GRAY);

    let blink = (get_time() * 3.0).sin() > 0.0;
    if blink {
        draw_text_centered("Nacisnij [ENTER] aby grac", cx, cy + 170.0, 28.0, Color::new(0.3, 1.0, 0.3, 1.0));
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let sounds = load_sounds().await;
    let mut app = AppState {
        screen: Screen::Menu,
        game: new_game(),
        selected_res: 0,
        camera_offset: Vec2::ZERO,
    };

    loop {
        let dt = get_frame_time().min(0.05);

        match app.screen {
            Screen::Menu => {
                if is_key_pressed(KeyCode::Left) && app.selected_res > 0 {
                    app.selected_res -= 1;
                    let (w, h) = RESOLUTIONS[app.selected_res];
                    request_new_screen_size(w as f32, h as f32);
                }
                if is_key_pressed(KeyCode::Right) && app.selected_res < RESOLUTIONS.len() - 1 {
                    app.selected_res += 1;
                    let (w, h) = RESOLUTIONS[app.selected_res];
                    request_new_screen_size(w as f32, h as f32);
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
                    app.game = new_game();
                    app.screen = Screen::Playing;
                    play_sfx(&sounds.wave_start, 0.5);
                }
                draw_menu(app.selected_res);
            }
            Screen::Playing => {
                if is_key_pressed(KeyCode::Escape) {
                    app.screen = Screen::Menu;
                    next_frame().await;
                    continue;
                }
                app.game.time += dt;
                update(&mut app.game, &sounds, dt);

                // Compute camera offset to center player when window is larger than map
                let map_pw = MAP_W as f32 * TILE;
                let map_ph = MAP_H as f32 * TILE;
                app.camera_offset = Vec2::new(
                    (screen_width() - map_pw) / 2.0,
                    (screen_height() - map_ph) / 2.0,
                ).max(Vec2::ZERO);

                draw_game(&app.game, app.camera_offset);

                if app.game.game_over {
                    app.screen = Screen::GameOver;
                }
            }
            Screen::GameOver => {
                app.game.time += dt;
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
                    app.game = new_game();
                    app.screen = Screen::Playing;
                    play_sfx(&sounds.wave_start, 0.5);
                }
                if is_key_pressed(KeyCode::Escape) {
                    app.screen = Screen::Menu;
                }
            }
        }

        next_frame().await;
    }
}

fn new_game() -> GameState {
    GameState {
        player: Player {
            x: MAP_W as f32 * TILE / 2.0,
            y: MAP_H as f32 * TILE / 2.0,
            angle: 0.0,
            hp: 100,
            max_hp: 100,
            fire_timer: 0.0,
            damage_flash: 0.0,
            ammo: 30,
        },
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
        use_keyboard_aim: false,
        kb_aim_angle: 0.0,
    }
}

// ── Update ─────────────────────────────────────────────────
fn update(state: &mut GameState, sounds: &Sounds, dt: f32) {
    // Player movement
    let mut dx = 0.0f32;
    let mut dy = 0.0f32;
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) { dy -= 1.0; }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) { dy += 1.0; }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) { dx -= 1.0; }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) { dx += 1.0; }
    let len = (dx * dx + dy * dy).sqrt();
    if len > 0.0 {
        dx /= len;
        dy /= len;
        let nx = state.player.x + dx * PLAYER_SPEED * dt;
        let ny = state.player.y + dy * PLAYER_SPEED * dt;
        if can_move(nx, state.player.y, 6.0) { state.player.x = nx; }
        if can_move(state.player.x, ny, 6.0) { state.player.y = ny; }
    }

    // Keyboard aim (IJKL)
    let mut aim_dx = 0.0f32;
    let mut aim_dy = 0.0f32;
    if is_key_down(KeyCode::I) { aim_dy -= 1.0; }
    if is_key_down(KeyCode::K) { aim_dy += 1.0; }
    if is_key_down(KeyCode::J) { aim_dx -= 1.0; }
    if is_key_down(KeyCode::L) { aim_dx += 1.0; }
    if aim_dx != 0.0 || aim_dy != 0.0 {
        state.kb_aim_angle = aim_dy.atan2(aim_dx);
        state.use_keyboard_aim = true;
    }

    // Mouse aim — switch back to mouse when it moves or clicked
    let (mx_screen, my_screen) = mouse_position();
    if is_mouse_button_down(MouseButton::Left) || is_mouse_button_down(MouseButton::Right) {
        state.use_keyboard_aim = false;
    }

    if state.use_keyboard_aim {
        state.player.angle = state.kb_aim_angle;
    } else {
        // Convert screen coords to world coords using the same camera as draw_game
        let cam = Camera2D {
            target: vec2(MAP_W as f32 * TILE / 2.0, MAP_H as f32 * TILE / 2.0),
            zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
            offset: vec2(0.0, 0.0),
            ..Default::default()
        };
        let world_pos = cam.screen_to_world(vec2(mx_screen, my_screen));
        state.player.angle = (world_pos.y - state.player.y).atan2(world_pos.x - state.player.x);
    }

    // Shoot (mouse LPM or Space)
    state.player.fire_timer -= dt;
    let shooting = is_mouse_button_down(MouseButton::Left) || is_key_down(KeyCode::Space);
    if shooting && state.player.fire_timer <= 0.0 && state.player.ammo > 0 {
        state.player.fire_timer = FIRE_COOLDOWN;
        state.player.ammo -= 1;
        let spread = rand::gen_range(-0.05f32, 0.05);
        let angle = state.player.angle + spread;
        let gx = state.player.x + angle.cos() * 16.0;
        let gy = state.player.y + angle.sin() * 16.0;
        state.bullets.push(Bullet {
            x: gx, y: gy,
            dx: angle.cos() * BULLET_SPEED,
            dy: angle.sin() * BULLET_SPEED,
            alive: true,
        });
        state.flashes.push(MuzzleFlash { x: gx, y: gy, life: 0.06 });
        state.screen_shake = 2.0;
        play_sfx(&sounds.shoot, 0.3);
    } else if shooting && state.player.fire_timer <= 0.0 && state.player.ammo == 0 {
        state.player.fire_timer = FIRE_COOLDOWN * 2.0;
        play_sfx(&sounds.no_ammo, 0.4);
    }

    state.screen_shake = (state.screen_shake - dt * 30.0).max(0.0);
    state.player.damage_flash = (state.player.damage_flash - dt * 5.0).max(0.0);

    // Update bullets
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

    // Update zombies
    let px = state.player.x;
    let py = state.player.y;
    for z in &mut state.zombies {
        if !z.alive { continue; }
        z.damage_flash = (z.damage_flash - dt * 5.0).max(0.0);

        let to_x = px - z.x;
        let to_y = py - z.y;
        let dist = (to_x * to_x + to_y * to_y).sqrt();
        if dist > 1.0 {
            let nx = z.x + (to_x / dist) * z.speed * dt;
            let ny = z.y + (to_y / dist) * z.speed * dt;
            if can_move(nx, z.y, 6.0) { z.x = nx; }
            if can_move(z.x, ny, 6.0) { z.y = ny; }
        }

        // Damage player on contact
        if dist < 18.0 {
            state.player.hp -= 1;
            state.player.damage_flash = 1.0;
            if state.player.hp % 10 == 0 {
                play_sfx(&sounds.hurt, 0.4);
            }
            if state.player.hp <= 0 {
                state.game_over = true;
            }
        }

        // Bullet hits
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
                    value: dmg,
                    life: 0.8,
                });
                play_sfx(&sounds.zombie_hit, 0.2);
                if z.hp <= 0 {
                    z.alive = false;
                    spawn_blood(&mut state.particles, z.x, z.y, 15);
                    state.score += 10 * state.wave;
                    state.kills += 1;
                    play_sfx(&sounds.zombie_death, 0.5);
                    // Drop pickup
                    if rand::gen_range(0.0f32, 1.0) < 0.25 {
                        let kind = if rand::gen_range(0.0f32, 1.0) < 0.4 {
                            PickupKind::Health
                        } else {
                            PickupKind::Ammo
                        };
                        state.pickups.push(Pickup {
                            x: z.x, y: z.y, kind, alive: true, timer: 10.0,
                        });
                    }
                }
                break;
            }
        }
    }

    // Update pickups
    for pk in &mut state.pickups {
        if !pk.alive { continue; }
        pk.timer -= dt;
        if pk.timer <= 0.0 { pk.alive = false; continue; }
        let ddx = px - pk.x;
        let ddy = py - pk.y;
        if ddx * ddx + ddy * ddy < PICKUP_RANGE * PICKUP_RANGE {
            pk.alive = false;
            play_sfx(&sounds.pickup, 0.5);
            match pk.kind {
                PickupKind::Health => {
                    state.player.hp = (state.player.hp + 25).min(state.player.max_hp);
                }
                PickupKind::Ammo => {
                    state.player.ammo += 15;
                }
            }
        }
    }

    // Update particles
    for p in &mut state.particles {
        p.x += p.dx * dt;
        p.y += p.dy * dt;
        p.dx *= 0.95;
        p.dy *= 0.95;
        p.life -= dt;
    }
    for f in &mut state.flashes { f.life -= dt; }
    for d in &mut state.dmg_numbers { d.y -= 40.0 * dt; d.life -= dt; }

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
            state.zombies_to_spawn = 3 + state.wave * 2;
            state.wave_delay = 3.0;
            state.player.ammo += 10 + (state.wave as i32) * 2;
            play_sfx(&sounds.wave_start, 0.6);
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
            state.zombies.push(Zombie {
                x: sx, y: sy,
                hp, max_hp: hp,
                alive: true,
                speed: ZOMBIE_BASE_SPEED + (wave as f32) * 3.0 + rand::gen_range(-10.0, 10.0),
                damage_flash: 0.0,
                variant: rand::gen_range(0, 3),
            });
        }
    }
}

// ── Draw ───────────────────────────────────────────────────
fn draw_game(state: &GameState, _offset: Vec2) {
    clear_background(Color::new(0.05, 0.05, 0.08, 1.0));

    // Push camera to center the map
    let cam = Camera2D {
        target: vec2(MAP_W as f32 * TILE / 2.0, MAP_H as f32 * TILE / 2.0),
        zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
        offset: vec2(0.0, 0.0),
        ..Default::default()
    };
    set_camera(&cam);

    // Draw map
    for ty in 0..MAP_H {
        for tx in 0..MAP_W {
            draw_tile(tx, ty, state.time);
        }
    }

    // Draw pickups
    for pk in &state.pickups { draw_pickup(pk, state.time); }

    // Particles
    for p in &state.particles {
        let alpha = (p.life * 3.0).min(1.0);
        let c = Color::new(p.color.r, p.color.g, p.color.b, alpha);
        draw_rectangle(p.x - p.size / 2.0, p.y - p.size / 2.0, p.size, p.size, c);
    }

    // Bullets
    for b in &state.bullets {
        draw_rectangle(b.x - 2.0, b.y - 2.0, 4.0, 4.0, YELLOW);
        draw_line(b.x, b.y, b.x - b.dx * 0.02, b.y - b.dy * 0.02, 2.0, Color::new(1.0, 1.0, 0.5, 0.5));
    }

    // Muzzle flashes
    for f in &state.flashes {
        let size = f.life * 200.0;
        draw_circle(f.x, f.y, size, Color::new(1.0, 0.9, 0.4, f.life * 10.0));
    }

    // Zombies
    for z in &state.zombies { draw_zombie(z, state.time); }

    // Player
    draw_player(&state.player, state.time);

    // Damage numbers
    for d in &state.dmg_numbers {
        let alpha = (d.life * 2.0).min(1.0);
        draw_text(&d.value.to_string(), d.x - 8.0, d.y, 20.0, Color::new(1.0, 0.3, 0.1, alpha));
    }

    // Switch back to screen space for HUD
    set_default_camera();

    // ── HUD ────────────────────────────────────────────────
    let hud_y = screen_height() - 45.0;
    draw_rectangle(0.0, hud_y - 5.0, screen_width(), 50.0, Color::new(0.0, 0.0, 0.0, 0.7));
    draw_rectangle(0.0, hud_y - 5.0, screen_width(), 2.0, Color::new(0.3, 0.3, 0.3, 0.8));

    // HP bar
    let hp_ratio = state.player.hp as f32 / state.player.max_hp as f32;
    let hp_color = if hp_ratio > 0.5 {
        Color::new(0.2, 0.8, 0.2, 1.0)
    } else if hp_ratio > 0.25 {
        Color::new(0.9, 0.7, 0.1, 1.0)
    } else {
        Color::new(0.9, 0.15, 0.1, 1.0)
    };
    draw_text("HP", 10.0, hud_y + 20.0, 20.0, WHITE);
    draw_rectangle(35.0, hud_y + 8.0, 120.0, 16.0, Color::new(0.2, 0.0, 0.0, 0.8));
    draw_rectangle(35.0, hud_y + 8.0, 120.0 * hp_ratio, 16.0, hp_color);
    draw_text(&format!("{}/{}", state.player.hp, state.player.max_hp), 60.0, hud_y + 22.0, 16.0, WHITE);

    // Ammo
    let ammo_color = if state.player.ammo > 10 {
        Color::new(0.9, 0.85, 0.2, 1.0)
    } else if state.player.ammo > 0 {
        Color::new(0.9, 0.4, 0.1, 1.0)
    } else {
        RED
    };
    draw_text(&format!("AMMO: {}", state.player.ammo), 180.0, hud_y + 22.0, 20.0, ammo_color);

    // Wave / Score / Kills
    draw_text(&format!("WAVE: {}", state.wave), 350.0, hud_y + 22.0, 20.0, Color::new(0.5, 0.8, 1.0, 1.0));
    draw_text(&format!("SCORE: {}", state.score), 500.0, hud_y + 22.0, 20.0, Color::new(1.0, 0.85, 0.3, 1.0));
    draw_text(&format!("KILLS: {}", state.kills), 660.0, hud_y + 22.0, 20.0, Color::new(0.8, 0.5, 0.5, 1.0));

    // Wave announcement
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
        draw_text_centered("WASD - ruch | Myszka/IJKL - cel | LPM/Spacja - strzal",
            screen_width() / 2.0, screen_height() / 2.0 + 10.0, 20.0, GRAY);
    }

    // No ammo warning
    if state.player.ammo == 0 && !state.game_over {
        let blink = (state.time * 6.0).sin() > 0.0;
        if blink {
            draw_text_centered("NO AMMO! Kill zombies for drops!", screen_width() / 2.0, 80.0, 24.0, RED);
        }
    }

    // Low HP vignette
    if state.player.hp < 30 {
        let alpha = ((state.time * 4.0).sin() + 1.0) * 0.15;
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.8, 0.0, 0.0, alpha));
    }
}
