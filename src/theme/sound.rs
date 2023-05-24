use crate::config::Config;
use sdl2::mixer::Chunk;

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
