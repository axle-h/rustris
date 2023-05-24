use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::{Point, Rect};
use sdl2::render::BlendMode;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use std::cmp::min;
use std::time::Duration;

const MAX_ALPHA: u8 = 100;
const FRAME_DURATION: Duration = Duration::from_millis(3);

pub struct HardDropAnimation<'a> {
    texture: Texture<'a>,
    snip: Rect,
    step_y: u32,
    max_frames: u32,
    frame: u32,
    current_duration: Duration,
}

impl<'a> HardDropAnimation<'a> {
    pub fn new(
        canvas: &WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
        minos: [Rect; 4],
        dropped_pixels: u32,
    ) -> Result<Self, String> {
        let points = minos
            .iter()
            .flat_map(|mino| {
                [
                    mino.top_left(),
                    mino.top_right(),
                    mino.bottom_left(),
                    mino.bottom_right(),
                ]
            })
            .collect::<Vec<Point>>();

        let snip = Rect::from_enclose_points(&points, None).unwrap();
        let mut texture = texture_creator
            .create_texture_target(PixelFormatEnum::ARGB8888, snip.width(), snip.height())
            .map_err(|e| e.to_string())?;
        texture.set_blend_mode(BlendMode::Blend);
        for rect in minos.iter() {
            let pixels = canvas.read_pixels(*rect, PixelFormatEnum::ARGB8888)?;
            let translated_rect = Rect::new(
                rect.x() - snip.x(),
                rect.y() - snip.y(),
                rect.width(),
                rect.height(),
            );
            let pitch = PixelFormatEnum::ARGB8888.byte_size_of_pixels(rect.width() as usize);
            texture
                .update(translated_rect, &pixels, pitch)
                .map_err(|e| e.to_string())?;
        }
        let step_y = snip.height() / 4;
        let max_frames = dropped_pixels / step_y;
        Ok(Self {
            texture,
            snip,
            step_y,
            max_frames,
            frame: 1,
            current_duration: Duration::ZERO,
        })
    }

    pub fn update(&mut self, canvas: &mut WindowCanvas, delta: Duration) -> Result<bool, String> {
        self.current_duration += delta;
        if self.current_duration >= FRAME_DURATION {
            let frame_delta = self.current_duration.as_secs_f64() / FRAME_DURATION.as_secs_f64();
            self.current_duration = Duration::ZERO;
            self.frame = min(self.frame + frame_delta.round() as u32, self.max_frames);
        }
        // trail
        let trail_frames = min(self.frame, 5) as i32;
        for j in 1..=trail_frames {
            let alpha_mod = (MAX_ALPHA as f64 * j as f64 / trail_frames as f64).round() as u8;
            self.texture.set_alpha_mod(MAX_ALPHA - alpha_mod);
            canvas.copy(&self.texture, None, self.translated_snip(-j))?;
        }

        // fall
        for j in 1..=self.frame {
            let alpha_mod = (MAX_ALPHA as f64 * j as f64 / self.max_frames as f64).round() as u8;
            self.texture.set_alpha_mod(MAX_ALPHA - alpha_mod);
            canvas.copy(&self.texture, None, self.translated_snip(j as i32))?;
        }
        self.texture.set_alpha_mod(255);
        canvas.copy(&self.texture, None, self.snip)?;

        Ok(self.frame < self.max_frames)
    }

    fn translated_snip(&self, j: i32) -> Rect {
        Rect::new(
            self.snip.x(),
            self.snip.y() + j * self.step_y as i32,
            self.snip.width(),
            self.snip.height(),
        )
    }
}
