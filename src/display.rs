extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;

use crate::{BODY_PIXEL_SIZE, MAP_SIDE, WINDOW_SIDE};

use crate::grid::Position;

use glutin_window::GlutinWindow as Window;
use graphics::*;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::RenderArgs;
use piston::window::WindowSettings;

const WHITE: [f32; 4] = [0.75, 0.75, 0.75, 1.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 0.8];
const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 0.8];

pub type RenderObject = (Position, bool);

pub struct Graphics {
    gl: GlGraphics,
    scale: f64,
}

impl Graphics {
    pub fn render(&mut self, args: &RenderArgs, objects: &Vec<RenderObject>) {
        let circle = ellipse::circle(0.0, 0.0, BODY_PIXEL_SIZE as f64 / 2.0 as f64);
        let scale = self.scale;

        self.gl.draw(args.viewport(), |c, gl| {
            clear(WHITE, gl);
            for obj in objects {
                let color = if obj.1 { RED } else { YELLOW };
                let transform = c
                    .transform
                    .trans(obj.0.x as f64 * scale, obj.0.y as f64 * scale);
                ellipse(color, circle, transform, gl);
            }
        });
    }
}

pub struct Display {
    pub window: Window,
    pub graphics: Graphics,
    pub events: Events,
}

impl Display {
    pub fn new() -> Display {
        let opengl = OpenGL::V3_2;

        let window: Window = WindowSettings::new("Tag!", [WINDOW_SIDE, WINDOW_SIDE])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();

        let graphics = Graphics {
            gl: GlGraphics::new(opengl),
            scale: WINDOW_SIDE / MAP_SIDE as f64,
        };

        let settings = EventSettings {
            max_fps: 120,
            ups: 120,
            ups_reset: 0,
            swap_buffers: true,
            bench_mode: true,
            lazy: true,
        };
        let events = Events::new(settings);
        Display {
            window,
            graphics,
            events,
        }
    }
}
