use crate::action::*;
use crate::agent::AgentManager;
use crate::grid::Grid;

pub struct Engine {
    /// 2D grid, which is used for collision detection and 'tagging'
    grid: Grid,
    /// All required information on choosing actions
    ac: ActionContext,
    /// All agents and agent context and stats
    am: AgentManager,
    step_counter: usize,
}

impl Engine {
    pub fn new(grid: Grid, ac: ActionContext, am: AgentManager) -> Engine {
        Engine {
            grid,
            ac,
            am,
            step_counter: 0,
        }
    }

    pub fn step(&mut self) {
        for id in self.am.get_ids() {
            self.am
                .update_preference(id, &self.ac.get_mean_preferences());
            let vec = self.am.get_actions_ordering(id);
            let maybe_effect: Option<Effect> =
                self.ac
                    .maybe_get_allowed_effect(vec, id, &mut self.am, &mut self.grid);
            if let Some(effect) = maybe_effect {
                effect(id, &mut self.am, &mut self.grid);
            }
        }
        self.grid.update(self.am.flush_log());
        self.step_counter += 1;
    }

    pub fn stats(&mut self) {
        println!(
            "Steps done: {} \nNumber of times tagged: {}",
            self.step_counter,
            self.am.get_tagged_count()
        )
    }
}
