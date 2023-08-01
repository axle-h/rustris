use crate::config::AudioConfig;
use crate::event::GameEvent;

use rand::{thread_rng, Rng};
use sdl2::get_error;
use sdl2::mixer::{Chunk, Music};
use sdl2::rwops::RWops;
use sdl2::sys::mixer;

pub fn load_sound(buffer: &[u8], config: AudioConfig) -> Result<Chunk, String> {
    let mut chunk = chunk_from_buffer(buffer)?;
    chunk.set_volume(config.effects_volume());
    Ok(chunk)
}

pub fn play_sound(chunk: &Chunk) -> Result<(), String> {
    // TODO ignore cannot play sound
    sdl2::mixer::Channel::all().play(chunk, 0)?;
    Ok(())
}

fn chunk_from_buffer(buffer: &[u8]) -> Result<Chunk, String> {
    let raw = unsafe { mixer::Mix_LoadWAV_RW(RWops::from_bytes(buffer)?.raw(), 0) };
    if raw.is_null() {
        Err(get_error())
    } else {
        Ok(Chunk { raw, owned: true })
    }
}

#[derive(Debug, Clone)]
pub struct SoundThemeOptions {
    config: AudioConfig,
    music: &'static [u8],
    move_tetromino: &'static [u8],
    rotate: &'static [u8],
    lock: &'static [u8],
    send_garbage: Vec<&'static [u8]>,
    clear: [&'static [u8]; 4], // single, double, triple, tetris
    level_up: &'static [u8],
    game_over: &'static [u8],
    pause: &'static [u8],
    victory: &'static [u8],
    stack_drop: Option<&'static [u8]>,
    hard_drop: Option<&'static [u8]>,
    hold: Option<&'static [u8]>,
}

impl SoundThemeOptions {
    pub fn default(
        config: AudioConfig,
        music: &'static [u8],
        move_tetromino: &'static [u8],
        rotate: &'static [u8],
        lock: &'static [u8],
        send_garbage: &'static [u8],
        clear: [&'static [u8]; 4], // single, double, triple, tetris
        level_up: &'static [u8],
        game_over: &'static [u8],
        pause: &'static [u8],
        victory: &'static [u8],
    ) -> Self {
        Self {
            config,
            music,
            move_tetromino,
            rotate,
            lock,
            send_garbage: vec![send_garbage],
            clear,
            level_up,
            game_over,
            pause,
            victory,
            stack_drop: None,
            hard_drop: None,
            hold: None,
        }
    }

    pub fn with_stack_drop(mut self, value: &'static [u8]) -> Self {
        self.stack_drop = Some(value);
        self
    }

    pub fn with_hard_drop(mut self, value: &'static [u8]) -> Self {
        self.hard_drop = Some(value);
        self
    }

    pub fn with_hold(mut self, value: &'static [u8]) -> Self {
        self.hold = Some(value);
        self
    }

    pub fn with_alt_send_garbage(mut self, value: &'static [u8]) -> Self {
        self.send_garbage.push(value);
        self
    }

    fn load_music<'a>(&self) -> Result<Music<'a>, String> {
        Music::from_static_bytes(self.music)
    }

    fn load_sound(&self, buffer: &[u8]) -> Result<Chunk, String> {
        load_sound(buffer, self.config)
    }

    pub fn build(self) -> Result<SoundTheme, String> {
        SoundTheme::new(self)
    }
}

pub struct SoundTheme {
    music: Music<'static>,
    move_tetromino: Chunk,
    rotate: Chunk,
    lock: Chunk,
    send_garbage: Vec<Chunk>,
    clear: [Chunk; 4], // single, double, triple, tetris
    level_up: Chunk,
    game_over: Chunk,
    pause: Chunk,
    victory: Chunk,
    stack_drop: Option<Chunk>,
    hard_drop: Option<Chunk>,
    hold: Option<Chunk>,
}

impl SoundTheme {
    pub fn new(options: SoundThemeOptions) -> Result<Self, String> {
        let o = options.clone();
        Ok(Self {
            music: options.load_music()?,
            move_tetromino: options.load_sound(o.move_tetromino)?,
            rotate: options.load_sound(o.rotate)?,
            lock: options.load_sound(o.lock)?,
            send_garbage: o
                .send_garbage
                .into_iter()
                .map(|p| options.load_sound(p).unwrap())
                .collect(),
            clear: o.clear.map(|p| options.load_sound(p).unwrap()),
            level_up: options.load_sound(o.level_up)?,
            game_over: options.load_sound(o.game_over)?,
            pause: options.load_sound(o.pause)?,
            victory: options.load_sound(o.victory)?,
            stack_drop: o.stack_drop.map(|p| options.load_sound(p).unwrap()),
            hard_drop: o.hard_drop.map(|p| options.load_sound(p).unwrap()),
            hold: o.hold.map(|p| options.load_sound(p).unwrap()),
        })
    }

    pub fn music(&self) -> &Music {
        &self.music
    }

    pub fn receive_event(&self, event: GameEvent) -> Result<(), String> {
        match event {
            GameEvent::Move => play_sound(&self.move_tetromino),
            GameEvent::Rotate => play_sound(&self.rotate),
            GameEvent::Lock { .. } => play_sound(&self.lock),
            GameEvent::Destroy(lines) => {
                let mut count = 0;
                for line in lines {
                    if line.is_some() {
                        count += 1;
                    } else {
                        break;
                    }
                }
                if count > 0 && count < 5 {
                    play_sound(&self.clear[count - 1])
                } else {
                    Ok(())
                }
            }
            GameEvent::Destroyed { level_up, .. } => {
                if level_up {
                    play_sound(&self.level_up)
                } else if self.stack_drop.is_some() {
                    play_sound(self.stack_drop.as_ref().unwrap())
                } else {
                    Ok(())
                }
            }
            GameEvent::ReceivedGarbage { .. } => {
                if self.send_garbage.len() == 1 {
                    play_sound(&self.send_garbage[0])
                } else {
                    let sound =
                        &self.send_garbage[thread_rng().gen_range(0..self.send_garbage.len())];
                    play_sound(sound)
                }
            }
            GameEvent::GameOver { .. } => play_sound(&self.game_over),
            GameEvent::Victory { .. } => play_sound(&self.victory),
            GameEvent::Paused => play_sound(&self.pause),
            GameEvent::HardDrop { .. } if self.hard_drop.is_some() => {
                play_sound(self.hard_drop.as_ref().unwrap())
            }
            GameEvent::Hold if self.hold.is_some() => play_sound(self.hold.as_ref().unwrap()),
            _ => Ok(()),
        }
    }
}
