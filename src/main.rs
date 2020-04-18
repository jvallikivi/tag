extern crate rand;

mod grid;
mod action;
mod agent;
mod engine;

use agent::AgentManager;
use grid::Grid;
use engine::Engine;
use action::*;

pub const STEP_SG_SIDE: usize = 21;
pub const TAG_SG_SIDE: usize = 31;
pub const MAP_SIDE: usize = 1500;
pub const AGENT_NUM_UPPER_BOUND: usize = MAP_SIDE*MAP_SIDE/(STEP_SG_SIDE/STEP_SG_SIDE);
pub const NUM_STEPS: usize = 1000;

fn main() {
    let ac: ActionContext = ActionContext::new();
    let mut grid: Grid = Grid{val: vec![vec![0; MAP_SIDE]; MAP_SIDE]};
    let mut am: AgentManager = AgentManager::new(&ac);
    am.add_agent(true, None, None, &mut grid); 
    for _ in 0..999 {
        am.add_agent(false, None, None, &mut grid);
    }
    println!("Number of agents: {}", am.get_num_agents());

    let mut engine: Engine = Engine::new(grid, ac, am);
    for _ in 0..NUM_STEPS {
        engine.step();
    }
    engine.stats();
}