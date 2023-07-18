use crate::config::{AudioConfig, Config};
use sdl2::mixer::{Chunk, Music};
use rand::{Rng, thread_rng};
use rand::prelude::ThreadRng;
use crate::event::GameEvent;

pub fn load_sound(theme: &str, name: &str, config: Config) -> Result<Chunk, String> {
    let mut chunk = Chunk::from_file(format!("./resource/{}/{}.ogg", theme, name))
        .map_err(|e| format!("Cannot load sound file {}: {:?}", name, e))?;
    chunk.set_volume(config.audio.effects_volume());
    Ok(chunk)
}

pub fn play_sound(chunk: &Chunk) -> Result<(), String> {
    // TODO ignore cannot play sound
    sdl2::mixer::Channel::all().play(chunk, 0)?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct SoundThemeOptions {
    theme_name: String,
    config: AudioConfig,
    music: String,
    move_tetromino: String,
    rotate: String,
    lock: String,
    send_garbage: Vec<String>,
    clear: [String; 4], // single, double, triple, tetris
    level_up: String,
    game_over: String,
    pause: String,
    victory: String,
    stack_drop: Option<String>,
    hard_drop: Option<String>,
    hold: Option<String>
}

impl SoundThemeOptions {
    pub fn default(theme_name: &str, config: AudioConfig) -> Self {
        let line_clear = "line-clear".to_string();
        Self {
            theme_name: theme_name.to_string(),
            config,
            music: "music".to_string(),
            move_tetromino: "move".to_string(),
            rotate: "rotate".to_string(),
            lock: "lock".to_string(),
            send_garbage: vec!["send-garbage".to_string()],
            clear: [
                // same sound for all line clears by default
                line_clear.clone(), line_clear.clone(), line_clear,
                "tetris".to_string()
            ],
            level_up: "level-up".to_string(),
            game_over: "game-over".to_string(),
            pause: "pause".to_string(),
            victory: "victory".to_string(),
            stack_drop: Some("stack-drop".to_string()),
            hard_drop: None,
            hold: None
        }
    }

    pub fn without_stack_drop(mut self) -> Self {
        self.stack_drop = None;
        self
    }

    pub fn with_distinct_clear(mut self) -> Self {
        self.clear[0] = "single".to_string();
        self.clear[1] = "double".to_string();
        self.clear[2] = "triple".to_string();
        self
    }

    pub fn with_hard_drop(mut self) -> Self {
        self.hard_drop = Some("hard-drop".to_string());
        self
    }

    pub fn with_hold(mut self) -> Self {
        self.hold = Some("hold".to_string());
        self
    }

    pub fn with_alt_send_garbage(mut self) -> Self {
        self.send_garbage.push("send-garbage-alt".to_string());
        self
    }

    fn load_music<'a>(&self) -> Result<Music<'a>, String> {
        let file = format!("./resource/{}/{}.ogg", self.theme_name, self.music);
        Music::from_file(file)
    }

    fn load_sound(&self, name: String) -> Result<Chunk, String> {
        let mut chunk = Chunk::from_file(format!("./resource/{}/{}.ogg", self.theme_name, name))
            .map_err(|e| format!("Cannot load sound file {}: {:?}", name, e))?;
        chunk.set_volume(self.config.effects_volume());
        Ok(chunk)
    }

    pub fn build<'a>(self) -> Result<SoundTheme<'a>, String> {
        SoundTheme::new(self)
    }
}

pub struct SoundTheme<'a> {
    rng: ThreadRng,
    music: Music<'a>,
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
    hold: Option<Chunk>
}

impl<'a> SoundTheme<'a> {
    pub fn new(options: SoundThemeOptions) -> Result<Self, String> {
        let o = options.clone();
        Ok(Self {
            rng: thread_rng(),
            music: options.load_music()?,
            move_tetromino: options.load_sound(o.move_tetromino)?,
            rotate: options.load_sound(o.rotate)?,
            lock: options.load_sound(o.lock)?,
            send_garbage: o.send_garbage.into_iter().map(|p| options.load_sound(p).unwrap()).collect(),
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

    pub fn receive_event(&mut self, event: GameEvent) -> Result<(), String> {
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
                    let sound = &self.send_garbage[self.rng.gen_range(0..self.send_garbage.len())];
                    play_sound(sound)
                }
            },
            GameEvent::GameOver(_) => play_sound(&self.game_over),
            GameEvent::Victory => play_sound(&self.victory),
            GameEvent::Paused => play_sound(&self.pause),
            GameEvent::HardDrop { .. } if self.hard_drop.is_some() => play_sound(self.hard_drop.as_ref().unwrap()),
            GameEvent::Hold if self.hold.is_some() => play_sound(self.hold.as_ref().unwrap()),
            _ => Ok(()),
        }
    }
}

