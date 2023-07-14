use std::time::Duration;
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::particles::geometry::RectF;
use crate::particles::Particles;
use crate::particles::scale::Scale;
use crate::particles::source::ParticleSource;

pub struct ParticleRender<'a> {
    scale: Scale,
    sphere_texture: Texture<'a>,
    sphere_size: (u32, u32),
    particles: Particles
}

impl<'a> ParticleRender<'a> {
    pub fn new(particles: Particles, texture_creator: &'a TextureCreator<WindowContext>, scale: Scale) -> Result<Self, String> {
        let mut sphere_texture = texture_creator.load_texture("resource/particle/sphere.png")?;
        sphere_texture.set_blend_mode(BlendMode::Blend);
        let sphere_query = sphere_texture.query();
        Ok(Self { scale, particles, sphere_texture, sphere_size: (sphere_query.width, sphere_query.height) })
    }

    pub fn add_source(&mut self, source: ParticleSource) {
        self.particles.sources.push(source);
    }

    pub fn update(&mut self, delta: Duration) {
        self.particles.update(delta)
    }

    pub fn draw(&mut self, canvas: &mut WindowCanvas) -> Result<(), String> {
        let (sphere_width, sphere_height) = self.sphere_size;
        let sphere_width= sphere_width / 2;
        let sphere_height= sphere_height / 2;

        for particle in self.particles.particles() {
            let point = self.scale.point_to_render_space(particle.position);
            let rect = Rect::new(
                point.x() - sphere_width as i32 / 2,
                point.y() - sphere_height as i32 / 2,
                sphere_width,
                sphere_height
            );
            let (r, g, b): (u8, u8, u8) = particle.color.into();
            self.sphere_texture.set_color_mod(r, g, b);
            if particle.alpha < 1.0 {
                self.sphere_texture.set_alpha_mod((255.0 * particle.alpha).round() as u8);
            } else {
                self.sphere_texture.set_alpha_mod(255);
            }

            canvas.copy(&self.sphere_texture, None, rect)?;
        }
        Ok(())
    }
}

