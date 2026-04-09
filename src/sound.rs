use macroquad::audio::{Sound, load_sound_from_bytes, play_sound, PlaySoundParams};
use crate::types::*;

fn make_wav(samples: &[i16]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let file_len = 36 + data_len;
    let sample_rate: u32 = 44100;
    let mut buf = Vec::with_capacity(44 + data_len as usize);
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_len.to_le_bytes());
    buf.extend_from_slice(b"WAVEfmt ");
    buf.extend_from_slice(&16u32.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());
    buf.extend_from_slice(&16u16.to_le_bytes());
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_len.to_le_bytes());
    for &s in samples { buf.extend_from_slice(&s.to_le_bytes()); }
    buf
}

fn gen_shoot_sound() -> Vec<u8> {
    let n = 2200;
    let mut s = vec![0i16; n];
    for i in 0..n {
        let t = i as f32 / 44100.0;
        let env = (1.0 - i as f32 / n as f32).powi(2);
        let noise = ((i as f32 * 12345.6789).sin() * 43758.5453).fract() * 2.0 - 1.0;
        let tone = (t * 800.0 * std::f32::consts::TAU).sin() * 0.3;
        s[i] = ((noise * 0.7 + tone) * env * 12000.0) as i16;
    }
    make_wav(&s)
}

fn gen_zombie_death_sound() -> Vec<u8> {
    let n = 6600;
    let mut s = vec![0i16; n];
    for i in 0..n {
        let t = i as f32 / 44100.0;
        let env = (1.0 - i as f32 / n as f32).powi(2);
        let freq = 120.0 - t * 300.0;
        let v = (t * freq * std::f32::consts::TAU).sin();
        let noise = ((i as f32 * 7654.321).sin() * 43758.5453).fract() * 2.0 - 1.0;
        s[i] = ((v * 0.6 + noise * 0.4) * env * 14000.0) as i16;
    }
    make_wav(&s)
}

fn gen_zombie_hit_sound() -> Vec<u8> {
    let n = 2000;
    let mut s = vec![0i16; n];
    for i in 0..n {
        let t = i as f32 / 44100.0;
        let env = (1.0 - i as f32 / n as f32).powi(3);
        let v = (t * 200.0 * std::f32::consts::TAU).sin();
        let noise = ((i as f32 * 9876.543).sin() * 43758.5453).fract() * 2.0 - 1.0;
        s[i] = ((v * 0.5 + noise * 0.5) * env * 8000.0) as i16;
    }
    make_wav(&s)
}

fn gen_pickup_sound() -> Vec<u8> {
    let n = 4400;
    let mut s = vec![0i16; n];
    for i in 0..n {
        let t = i as f32 / 44100.0;
        let env = (1.0 - i as f32 / n as f32) * (i as f32 / 400.0).min(1.0);
        let freq = 600.0 + t * 1200.0;
        s[i] = ((t * freq * std::f32::consts::TAU).sin() * env * 10000.0) as i16;
    }
    make_wav(&s)
}

fn gen_hurt_sound() -> Vec<u8> {
    let n = 4400;
    let mut s = vec![0i16; n];
    for i in 0..n {
        let t = i as f32 / 44100.0;
        let env = (1.0 - i as f32 / n as f32).powi(2);
        let v = (t * 150.0 * std::f32::consts::TAU).sin();
        let v2 = (t * 180.0 * std::f32::consts::TAU).sin();
        s[i] = ((v * 0.5 + v2 * 0.5) * env * 12000.0) as i16;
    }
    make_wav(&s)
}

fn gen_wave_sound() -> Vec<u8> {
    let n = 13230;
    let mut s = vec![0i16; n];
    for i in 0..n {
        let t = i as f32 / 44100.0;
        let env = (1.0 - i as f32 / n as f32) * (i as f32 / 800.0).min(1.0);
        let freq = 400.0 + t * 800.0;
        let v = (t * freq * std::f32::consts::TAU).sin();
        let v2 = (t * freq * 1.5 * std::f32::consts::TAU).sin() * 0.3;
        s[i] = ((v + v2) * env * 8000.0) as i16;
    }
    make_wav(&s)
}

fn gen_no_ammo_sound() -> Vec<u8> {
    let n = 2200;
    let mut s = vec![0i16; n];
    for i in 0..n {
        let t = i as f32 / 44100.0;
        let env = (1.0 - i as f32 / n as f32).powi(2);
        let click = if i < 200 { 1.0 } else { 0.0 };
        let tone = (t * 300.0 * std::f32::consts::TAU).sin() * 0.5;
        s[i] = ((click + tone) * env * 6000.0) as i16;
    }
    make_wav(&s)
}

// Menu: short tick when navigating
fn gen_menu_navigate_sound() -> Vec<u8> {
    let n = 1100; // ~25ms
    let mut s = vec![0i16; n];
    for i in 0..n {
        let t = i as f32 / 44100.0;
        let env = (1.0 - i as f32 / n as f32).powi(3);
        let v = (t * 1200.0 * std::f32::consts::TAU).sin();
        s[i] = (v * env * 5000.0) as i16;
    }
    make_wav(&s)
}

// Menu: confirm selection
fn gen_menu_select_sound() -> Vec<u8> {
    let n = 6600; // ~150ms
    let mut s = vec![0i16; n];
    for i in 0..n {
        let t = i as f32 / 44100.0;
        let env = (1.0 - i as f32 / n as f32) * (i as f32 / 300.0).min(1.0);
        // Two-note ascending chirp
        let freq = if i < n / 2 { 500.0 } else { 750.0 };
        let v = (t * freq * std::f32::consts::TAU).sin();
        let v2 = (t * freq * 2.0 * std::f32::consts::TAU).sin() * 0.2;
        s[i] = ((v + v2) * env * 9000.0) as i16;
    }
    make_wav(&s)
}

// Menu: ambient background loop - dark, moody drone
fn gen_menu_music() -> Vec<u8> {
    let sr = 44100;
    let duration = 4.0; // 4 second loop
    let n = (sr as f32 * duration) as usize;
    let mut s = vec![0i16; n];
    for i in 0..n {
        let t = i as f32 / sr as f32;
        // Smooth loop: fade in/out at edges
        let fade_samples = (sr as f32 * 0.15) as usize;
        let loop_env = if i < fade_samples {
            i as f32 / fade_samples as f32
        } else if i > n - fade_samples {
            (n - i) as f32 / fade_samples as f32
        } else {
            1.0
        };

        // Low drone
        let drone = (t * 55.0 * std::f32::consts::TAU).sin() * 0.4
            + (t * 82.5 * std::f32::consts::TAU).sin() * 0.25;

        // Slow pulsing pad
        let pulse = ((t * 0.5).sin() + 1.0) * 0.5;
        let pad = (t * 110.0 * std::f32::consts::TAU).sin() * 0.15 * pulse;

        // Eerie high whistle that drifts
        let drift = (t * 0.7).sin() * 30.0;
        let whistle = (t * (330.0 + drift) * std::f32::consts::TAU).sin() * 0.06
            * ((t * 1.3).sin() * 0.5 + 0.5);

        // Subtle noise texture
        let noise = ((i as f32 * 5432.1).sin() * 43758.5453).fract() * 2.0 - 1.0;
        let noise_filtered = noise * 0.03;

        let mix = (drone + pad + whistle + noise_filtered) * loop_env;
        s[i] = (mix * 7000.0).clamp(-32000.0, 32000.0) as i16;
    }
    make_wav(&s)
}

// Gameplay: light ambient melody - gentle, loopable, quiet
fn gen_game_music() -> Vec<u8> {
    let sr = 44100;
    let duration = 8.0; // 8 second loop
    let n = (sr as f32 * duration) as usize;
    let mut s = vec![0i16; n];

    // Simple pentatonic melody notes (C D E G A) in Hz, two octaves
    let melody: &[f32] = &[
        261.6, 293.7, 329.6, 392.0, 440.0,
        523.3, 587.3, 659.3, 784.0, 880.0,
    ];
    // Note sequence — gentle ascending/descending pattern
    let seq: &[usize] = &[0, 2, 4, 7, 5, 3, 1, 4, 6, 8, 5, 2, 3, 1, 0, 2];
    let note_dur = duration / seq.len() as f32;

    for i in 0..n {
        let t = i as f32 / sr as f32;

        // Smooth loop envelope
        let fade = (sr as f32 * 0.3) as usize;
        let loop_env = if i < fade {
            i as f32 / fade as f32
        } else if i > n - fade {
            (n - i) as f32 / fade as f32
        } else {
            1.0
        };

        // Melody: soft sine with gentle attack/release per note
        let note_idx = ((t / note_dur) as usize) % seq.len();
        let note_t = (t % note_dur) / note_dur;
        let note_env = (note_t * 8.0).min(1.0) * (1.0 - note_t).powf(0.5);
        let freq = melody[seq[note_idx]];
        let mel = (t * freq * std::f32::consts::TAU).sin() * 0.25 * note_env;
        // Soft overtone
        let mel2 = (t * freq * 2.0 * std::f32::consts::TAU).sin() * 0.06 * note_env;

        // Warm pad — slow chord (root + fifth)
        let pad_pulse = ((t * 0.4).sin() * 0.5 + 0.5) * 0.12;
        let pad = (t * 130.8 * std::f32::consts::TAU).sin() * pad_pulse
                + (t * 196.0 * std::f32::consts::TAU).sin() * pad_pulse * 0.7;

        // Very subtle high shimmer
        let shimmer_freq = 1318.5 + (t * 0.6).sin() * 40.0;
        let shimmer = (t * shimmer_freq * std::f32::consts::TAU).sin()
            * 0.02 * ((t * 1.1).sin() * 0.5 + 0.5);

        let mix = (mel + mel2 + pad + shimmer) * loop_env;
        s[i] = (mix * 6000.0).clamp(-32000.0, 32000.0) as i16;
    }
    make_wav(&s)
}

pub struct Sounds {
    pub shoot: Sound,
    pub zombie_hit: Sound,
    pub zombie_death: Sound,
    pub pickup: Sound,
    pub hurt: Sound,
    pub wave_start: Sound,
    pub no_ammo: Sound,
    pub menu_navigate: Sound,
    pub menu_select: Sound,
    pub menu_music: Sound,
    pub game_music: Sound,
}

pub async fn load_sounds() -> Sounds {
    Sounds {
        shoot: load_sound_from_bytes(&gen_shoot_sound()).await.unwrap(),
        zombie_hit: load_sound_from_bytes(&gen_zombie_hit_sound()).await.unwrap(),
        zombie_death: load_sound_from_bytes(&gen_zombie_death_sound()).await.unwrap(),
        pickup: load_sound_from_bytes(&gen_pickup_sound()).await.unwrap(),
        hurt: load_sound_from_bytes(&gen_hurt_sound()).await.unwrap(),
        wave_start: load_sound_from_bytes(&gen_wave_sound()).await.unwrap(),
        no_ammo: load_sound_from_bytes(&gen_no_ammo_sound()).await.unwrap(),
        menu_navigate: load_sound_from_bytes(&gen_menu_navigate_sound()).await.unwrap(),
        menu_select: load_sound_from_bytes(&gen_menu_select_sound()).await.unwrap(),
        menu_music: load_sound_from_bytes(&gen_menu_music()).await.unwrap(),
        game_music: load_sound_from_bytes(&gen_game_music()).await.unwrap(),
    }
}

pub fn play_music(sound: &Sound, volume: f32) {
    play_sound(sound, PlaySoundParams { looped: true, volume });
}

pub fn stop_music(sound: &Sound) {
    macroquad::audio::stop_sound(sound);
}

pub fn play_sfx(sound: &Sound, volume: f32) {
    play_sound(sound, PlaySoundParams { looped: false, volume });
}

pub fn play_events(events: &[u8], sounds: &Sounds) {
    for &e in events {
        match e {
            SND_SHOOT => play_sfx(&sounds.shoot, 0.3),
            SND_ZOMBIE_HIT => play_sfx(&sounds.zombie_hit, 0.2),
            SND_ZOMBIE_DEATH => play_sfx(&sounds.zombie_death, 0.5),
            SND_HURT => play_sfx(&sounds.hurt, 0.4),
            SND_PICKUP => play_sfx(&sounds.pickup, 0.5),
            SND_WAVE => play_sfx(&sounds.wave_start, 0.6),
            SND_NO_AMMO => play_sfx(&sounds.no_ammo, 0.4),
            _ => {}
        }
    }
}
