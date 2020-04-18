use crate::action::*;
use crate::agent::AgentManager;
use crate::display::Display;
use crate::grid::Grid;

use piston::input::RenderEvent;
pub struct Engine {
    /// 2D grid, which is used for collision detection and 'tagging'
    grid: Grid,
    /// All required information on choosing actions
    ac: ActionContext,
    /// All agents and agent context and stats
    am: AgentManager,
    display: Option<Display>,
    show_graphics: bool,
    step_counter: usize,
}

impl Engine {
    pub fn new(grid: Grid, ac: ActionContext, am: AgentManager, show_graphics: bool) -> Engine {
        let mut display: Option<Display> = None;
        if show_graphics {
            display = Some(Display::new());
        }
        Engine {
            grid,
            ac,
            am,
            display,
            show_graphics,
            step_counter: 0,
        }
    }

    fn update(&mut self) {
        self.am.perform_actions(&self.grid, &self.ac);
        self.grid.update(self.am.flush_log());
        self.step_counter += 1;
    }

    pub fn step(&mut self) {
        if self.show_graphics {
            let display: &mut Display = self.display.as_mut().unwrap();
            let maybe_e = display.events.next(&mut display.window);
            if maybe_e.is_some() {
                self.update();
                let e = maybe_e.unwrap();
                let render_objects = self.am.get_render_info();
                let display: &mut Display = self.display.as_mut().unwrap();

                if let Some(args) = e.render_args() {
                    display.graphics.render(&args, &render_objects);
                }
            }
        } else {
            self.update();
        }
    }

    pub fn stats(&mut self) {
        println!(
            "Steps done: {} \nNumber of times tagged: {}",
            self.step_counter,
            self.am.get_tagged_count()
        )
    }
}
