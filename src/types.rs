use macroquad::prelude::*;

// ── Constants ─────────────────────────────────────────────
pub const TILE: f32 = 32.0;
pub const PLAYER_SPEED: f32 = 150.0;
pub const BULLET_SPEED: f32 = 500.0;
pub const ZOMBIE_BASE_SPEED: f32 = 45.0;
pub const FIRE_COOLDOWN: f32 = 0.18;
pub const PICKUP_RANGE: f32 = 40.0;
pub const NET_PORT: u16 = 7777;
pub const NET_SEND_RATE: f32 = 1.0 / 30.0;

pub const RESOLUTIONS: [(i32, i32); 5] = [
    (800, 608),
    (1024, 768),
    (1280, 720),
    (1600, 900),
    (1920, 1080),
];

// ── Sound event IDs ───────────────────────────────────────
pub const SND_SHOOT: u8 = 0;
pub const SND_ZOMBIE_HIT: u8 = 1;
pub const SND_ZOMBIE_DEATH: u8 = 2;
pub const SND_HURT: u8 = 3;
pub const SND_PICKUP: u8 = 4;
pub const SND_WAVE: u8 = 5;
pub const SND_NO_AMMO: u8 = 6;

// ── Enums ─────────────────────────────────────────────────
#[derive(PartialEq)]
pub enum Screen { Menu, Lobby, Playing, GameOver }

#[derive(PartialEq, Clone, Copy)]
pub enum NetRole { Solo, Host, Client }

#[derive(Clone, Copy, PartialEq)]
pub enum PickupKind { Health, Ammo }

// ── Structs ───────────────────────────────────────────────
pub struct Player {
    pub x: f32, pub y: f32, pub angle: f32,
    pub hp: i32, pub max_hp: i32,
    pub fire_timer: f32, pub damage_flash: f32,
    pub ammo: i32, pub alive: bool,
}

pub struct Bullet {
    pub x: f32, pub y: f32, pub dx: f32, pub dy: f32,
    pub alive: bool, pub owner: u8,
}

pub struct Zombie {
    pub x: f32, pub y: f32, pub hp: i32, pub max_hp: i32,
    pub alive: bool, pub speed: f32, pub damage_flash: f32, pub variant: u8,
}

pub struct Pickup {
    pub x: f32, pub y: f32, pub kind: PickupKind,
    pub alive: bool, pub timer: f32,
}

pub struct Particle {
    pub x: f32, pub y: f32, pub dx: f32, pub dy: f32,
    pub life: f32, pub color: Color, pub size: f32,
}

pub struct MuzzleFlash { pub x: f32, pub y: f32, pub life: f32 }
pub struct DamageNumber { pub x: f32, pub y: f32, pub value: i32, pub life: f32 }

#[derive(Default)]
pub struct RemoteInput { pub dx: f32, pub dy: f32, pub angle: f32, pub shooting: bool }

pub struct LocalInput {
    pub dx: f32, pub dy: f32, pub angle: f32, pub shooting: bool,
}

pub struct GameState {
    pub player1: Player,
    pub player2: Player,
    pub two_player: bool,
    pub bullets: Vec<Bullet>,
    pub zombies: Vec<Zombie>,
    pub pickups: Vec<Pickup>,
    pub particles: Vec<Particle>,
    pub flashes: Vec<MuzzleFlash>,
    pub dmg_numbers: Vec<DamageNumber>,
    pub wave: u32,
    pub zombies_to_spawn: u32,
    pub spawn_timer: f32,
    pub wave_delay: f32,
    pub score: u32,
    pub kills: u32,
    pub game_over: bool,
    pub screen_shake: f32,
    pub time: f32,
    pub events: Vec<u8>,
}

pub struct AppState {
    pub screen: Screen,
    pub game: GameState,
    pub selected_res: usize,
    pub camera_offset: Vec2,
    pub net_role: NetRole,
    pub socket: Option<std::net::UdpSocket>,
    pub peer_addr: Option<std::net::SocketAddr>,
    pub remote_input: RemoteInput,
    pub net_timer: f32,
    pub ip_input: String,
    pub connected: bool,
}
