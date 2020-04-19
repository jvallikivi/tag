extern crate piston;
extern crate rand;
extern crate rayon;

mod action;
mod agent;
mod display;
mod engine;
mod grid;

use action::*;
use agent::AgentManager;
use engine::Engine;
use grid::Grid;

use std::time::Instant;

pub const WINDOW_SIDE: f64 = 720.0;
pub const AGENT_NUM_UPPER_BOUND: usize = GRID_SIDE * GRID_SIDE / (STEP_SG_SIDE / STEP_SG_SIDE);
pub const BODY_PIXEL_SIZE: usize = 15 * WINDOW_SIDE as usize / GRID_SIDE;

//         Modify these values             //
pub const USE_VIEWER: bool = true;
pub const COLLSION_DETECTION: bool = true;
pub const STEP_SG_SIDE: usize = 21;
pub const TAG_SG_SIDE: usize = 31;
pub const GRID_SIDE: usize = 800;
pub const NUM_STEPS: usize = 20000;
pub const NUM_AGENTS: usize = 500;
pub const NUM_AGENTS_IT: usize = 2;
//    More information in ../README.md     //

fn main() {
    let now = Instant::now();

    let ac: ActionContext = ActionContext::new();
    let mut grid: Grid = Grid {
        val: vec![vec![0; GRID_SIDE]; GRID_SIDE],
    };
    let am: AgentManager = AgentManager::new(&ac, &mut grid, NUM_AGENTS, NUM_AGENTS_IT);

    let mut engine: Engine = Engine::new(grid, ac, am, USE_VIEWER);

    for _ in 0..NUM_STEPS {
        engine.step();
    }
    println!("Took {} ms", now.elapsed().as_millis());
    engine.stats();
}
