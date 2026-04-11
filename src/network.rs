use std::net::UdpSocket;
use crate::types::*;

// ── Byte write helpers ────────────────────────────────────
fn wb_f32(b: &mut Vec<u8>, v: f32) { b.extend_from_slice(&v.to_le_bytes()); }
fn wb_i32(b: &mut Vec<u8>, v: i32) { b.extend_from_slice(&v.to_le_bytes()); }
fn wb_u32(b: &mut Vec<u8>, v: u32) { b.extend_from_slice(&v.to_le_bytes()); }
fn wb_u16(b: &mut Vec<u8>, v: u16) { b.extend_from_slice(&v.to_le_bytes()); }
fn wb_u8(b: &mut Vec<u8>, v: u8) { b.push(v); }

// ── Byte read helper ──────────────────────────────────────
pub struct BR<'a> { d: &'a [u8], p: usize }
impl<'a> BR<'a> {
    pub fn new(d: &'a [u8]) -> Self { Self { d, p: 0 } }
    pub fn f32(&mut self) -> f32 { let v = f32::from_le_bytes(self.d[self.p..self.p+4].try_into().unwrap()); self.p += 4; v }
    pub fn i32(&mut self) -> i32 { let v = i32::from_le_bytes(self.d[self.p..self.p+4].try_into().unwrap()); self.p += 4; v }
    pub fn u32(&mut self) -> u32 { let v = u32::from_le_bytes(self.d[self.p..self.p+4].try_into().unwrap()); self.p += 4; v }
    pub fn u16(&mut self) -> u16 { let v = u16::from_le_bytes(self.d[self.p..self.p+2].try_into().unwrap()); self.p += 2; v }
    pub fn u8(&mut self) -> u8 { let v = self.d[self.p]; self.p += 1; v }
}

// ── Serialization ─────────────────────────────────────────
pub fn serialize_input(inp: &RemoteInput) -> Vec<u8> {
    let mut b = Vec::with_capacity(18);
    wb_u8(&mut b, 3);
    wb_f32(&mut b, inp.dx);
    wb_f32(&mut b, inp.dy);
    wb_f32(&mut b, inp.angle);
    wb_u8(&mut b, if inp.shooting { 1 } else { 0 });
    b
}

pub fn deserialize_input(data: &[u8]) -> RemoteInput {
    let mut r = BR::new(&data[1..]);
    RemoteInput {
        dx: r.f32(), dy: r.f32(), angle: r.f32(),
        shooting: r.u8() != 0,
    }
}

fn serialize_player(b: &mut Vec<u8>, p: &Player) {
    wb_f32(b, p.x); wb_f32(b, p.y); wb_f32(b, p.angle);
    wb_i32(b, p.hp); wb_i32(b, p.max_hp); wb_i32(b, p.ammo);
    wb_f32(b, p.fire_timer); wb_f32(b, p.damage_flash);
    wb_u8(b, if p.alive { 1 } else { 0 });
}

fn deserialize_player(r: &mut BR) -> Player {
    Player {
        x: r.f32(), y: r.f32(), angle: r.f32(),
        hp: r.i32(), max_hp: r.i32(), ammo: r.i32(),
        fire_timer: r.f32(), damage_flash: r.f32(),
        alive: r.u8() != 0,
    }
}

pub fn serialize_state(state: &GameState) -> Vec<u8> {
    let mut b = Vec::with_capacity(4096);
    wb_u8(&mut b, MSG_STATE);
    wb_f32(&mut b, state.time);
    wb_u32(&mut b, state.wave);
    wb_u32(&mut b, state.score);
    wb_u32(&mut b, state.kills);
    wb_u8(&mut b, if state.game_over { 1 } else { 0 });
    wb_f32(&mut b, state.screen_shake);
    wb_f32(&mut b, state.wave_delay);
    wb_u32(&mut b, state.zombies_to_spawn);

    wb_u8(&mut b, state.num_players);
    for p in &state.players {
        serialize_player(&mut b, p);
    }

    wb_u16(&mut b, state.zombies.len() as u16);
    for z in &state.zombies {
        wb_f32(&mut b, z.x); wb_f32(&mut b, z.y);
        wb_i32(&mut b, z.hp); wb_i32(&mut b, z.max_hp);
        wb_f32(&mut b, z.speed); wb_f32(&mut b, z.damage_flash);
        wb_u8(&mut b, z.variant); wb_u8(&mut b, if z.alive { 1 } else { 0 });
        wb_f32(&mut b, z.attack_timer);
    }

    wb_u16(&mut b, state.bullets.len() as u16);
    for bl in &state.bullets {
        wb_f32(&mut b, bl.x); wb_f32(&mut b, bl.y);
        wb_f32(&mut b, bl.dx); wb_f32(&mut b, bl.dy);
        wb_u8(&mut b, bl.owner);
        wb_u8(&mut b, if bl.alive { 1 } else { 0 });
    }

    wb_u16(&mut b, state.pickups.len() as u16);
    for pk in &state.pickups {
        wb_f32(&mut b, pk.x); wb_f32(&mut b, pk.y);
        wb_u8(&mut b, if pk.kind == PickupKind::Health { 0 } else { 1 });
        wb_u8(&mut b, if pk.alive { 1 } else { 0 });
        wb_f32(&mut b, pk.timer);
    }

    wb_u16(&mut b, state.events.len() as u16);
    for &e in &state.events { wb_u8(&mut b, e); }

    b
}

pub fn deserialize_state(data: &[u8], state: &mut GameState) {
    let mut r = BR::new(&data[1..]);
    state.time = r.f32();
    state.wave = r.u32();
    state.score = r.u32();
    state.kills = r.u32();
    state.game_over = r.u8() != 0;
    state.screen_shake = r.f32();
    state.wave_delay = r.f32();
    state.zombies_to_spawn = r.u32();

    let np = r.u8();
    state.num_players = np;
    state.players.clear();
    for _ in 0..np {
        state.players.push(deserialize_player(&mut r));
    }

    let nz = r.u16() as usize;
    state.zombies.clear();
    for _ in 0..nz {
        state.zombies.push(Zombie {
            x: r.f32(), y: r.f32(), hp: r.i32(), max_hp: r.i32(),
            speed: r.f32(), damage_flash: r.f32(),
            variant: r.u8(), alive: r.u8() != 0,
            attack_timer: r.f32(),
        });
    }

    let nb = r.u16() as usize;
    state.bullets.clear();
    for _ in 0..nb {
        state.bullets.push(Bullet {
            x: r.f32(), y: r.f32(), dx: r.f32(), dy: r.f32(),
            owner: r.u8(), alive: r.u8() != 0,
        });
    }

    let npk = r.u16() as usize;
    state.pickups.clear();
    for _ in 0..npk {
        state.pickups.push(Pickup {
            x: r.f32(), y: r.f32(),
            kind: if r.u8() == 0 { PickupKind::Health } else { PickupKind::Ammo },
            alive: r.u8() != 0, timer: r.f32(),
        });
    }

    state.events.clear();
    let ne = r.u16() as usize;
    for _ in 0..ne { state.events.push(r.u8()); }
}

pub fn get_local_ip() -> String {
    if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
        let _ = sock.connect("8.8.8.8:80");
        if let Ok(addr) = sock.local_addr() {
            return addr.ip().to_string();
        }
    }
    if let Ok(sock) = UdpSocket::bind("[::]:0") {
        let _ = sock.connect("[2001:4860:4860::8888]:80");
        if let Ok(addr) = sock.local_addr() {
            return addr.ip().to_string();
        }
    }
    "?.?.?.?".to_string()
}

// ── Discovery Protocol ───────────────────────────────────
pub fn serialize_server_info(name: &str, players: u8, max_players: u8, wave: u32, score: u32, game_port: u16) -> Vec<u8> {
    let mut b = Vec::with_capacity(64);
    wb_u8(&mut b, MSG_DISCOVERY_RESP);
    let name_bytes = name.as_bytes();
    let name_len = name_bytes.len().min(255);
    wb_u8(&mut b, name_len as u8);
    b.extend_from_slice(&name_bytes[..name_len]);
    wb_u8(&mut b, players);
    wb_u8(&mut b, max_players);
    wb_u32(&mut b, wave);
    wb_u32(&mut b, score);
    wb_u16(&mut b, game_port);
    b
}

pub fn deserialize_server_info(data: &[u8]) -> Option<(String, u8, u8, u32, u32, u16)> {
    if data.len() < 4 || data[0] != MSG_DISCOVERY_RESP { return None; }
    let name_len = data[1] as usize;
    if data.len() < 2 + name_len + 12 { return None; }
    let name = String::from_utf8_lossy(&data[2..2 + name_len]).to_string();
    let rest = &data[2 + name_len..];
    let mut r = BR::new(rest);
    let players = r.u8();
    let max_players = r.u8();
    let wave = r.u32();
    let score = r.u32();
    let game_port = r.u16();
    Some((name, players, max_players, wave, score, game_port))
}

// ── Lobby Protocol ───────────────────────────────────────
pub fn build_lobby_state(slots: &[(bool, bool); 4]) -> Vec<u8> {
    let mut b = Vec::with_capacity(9);
    b.push(MSG_LOBBY_STATE);
    for &(connected, ready) in slots {
        b.push(if connected { 1 } else { 0 });
        b.push(if ready { 1 } else { 0 });
    }
    b
}

pub fn parse_lobby_state(data: &[u8]) -> [(bool, bool); 4] {
    let mut slots = [(false, false); 4];
    if data.len() >= 9 && data[0] == MSG_LOBBY_STATE {
        for i in 0..4 {
            slots[i] = (data[1 + i * 2] != 0, data[2 + i * 2] != 0);
        }
    }
    slots
}
