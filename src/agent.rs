use crate::{AGENT_NUM_UPPER_BOUND, STEP_SG_SIDE};

use crate::action::*;
use crate::display::RenderObject;
use crate::grid::{Grid, Position, PositionChange};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::collections::HashMap;

pub type Id = u32;

#[derive(Clone)]
struct Agent {
    /// Id for an agent
    id: Id,
    /// Cartesian coordinates
    position: Position,
    /// Has it been tagged most recently
    is_it: bool,
    /// If it is 'it', who was it tagged by
    tagged_by: Option<u32>,
    /// Preferences on action choice
    pref: Vec<f32>,
    /// Next action, used for concurrency
    next_action: Option<Effect>,
}

pub struct AgentManager {
    /// All agents
    agents: Vec<Agent>,
    /// Map from Ids to agent index, e.g. used
    /// to find an agent from inside Grid
    id_map: HashMap<Id, usize>,
    /// Recently changed coordinates which
    /// is used for quick Grid update
    position_log: Vec<PositionChange>,
    /// Number of actions possible
    action_count: usize,
    /// Number of times the 'Tag' action has been used
    tagged_count: usize,
    rng: StdRng,
}

impl AgentManager {
    pub fn new(
        ac: &ActionContext,
        grid: &mut Grid,
        num_agents: usize,
        num_it: usize,
    ) -> AgentManager {
        let rng = rand::rngs::StdRng::from_entropy();
        let action_count = ac.action_count;
        let mut am = AgentManager {
            agents: vec![],
            id_map: HashMap::new(),
            position_log: vec![],
            action_count,
            tagged_count: 0,
            rng,
        };
        for i in 0..num_agents {
            am.add_agent(i < num_it, None, None, grid);
        }
        am
    }

    fn add_agent(
        &mut self,
        is_it: bool,
        tagged_by: Option<u32>,
        maybe_position: Option<Position>,
        grid: &mut Grid,
    ) {
        if self.agents.len() == AGENT_NUM_UPPER_BOUND {
            return;
        }
        let id: Id = self.new_id();
        self.id_map.insert(id, self.agents.len());
        let pref: Vec<f32> = (0..self.action_count)
            .map(|_| self.rng.gen::<f32>())
            .collect();
        if let Some(position) = maybe_position {
            self.agents.push(Agent {
                id,
                position,
                is_it,
                tagged_by,
                pref,
                next_action: None,
            });
            grid.set(position, id);
        } else {
            let maybe_rand_pos: Option<Position> = self.rand_pos(grid);
            if let Some(position) = maybe_rand_pos {
                self.agents.push(Agent {
                    id,
                    position,
                    is_it,
                    tagged_by,
                    pref,
                    next_action: None,
                });
                grid.set(position, id);
            } else {
                return;
            }
        }
    }

    pub fn perform_actions(&mut self, grid: &Grid, ac: &ActionContext) {
        let mean_preferences = ac.get_mean_preferences();
        let action_count = self.action_count;
        let s = &*self;
        let mut agents = self.agents.clone();
        let v = move |agent: &mut Agent| {
            let mut rng = rand::thread_rng();
            AgentManager::update_preference(agent, mean_preferences, action_count, &mut rng);
            let ordering = &*AgentManager::get_actions_ordering(agent, &mut rng);
            agent.next_action = ac.maybe_get_allowed_effect(ordering, agent.id, s, grid);
        };
        agents.par_iter_mut().for_each(|agent| v(agent));
        self.agents = agents;
        for i in 0..self.agents.len() {
            let agent = &self.agents[i];
            if let Some(effect) = agent.next_action {
                effect(agent.id, self, grid);
            }
        }
    }

    pub fn get_position(&self, id: Id) -> Position {
        self.get(id).position
    }

    pub fn set_position(&mut self, id: Id, position: Position) {
        let before = self.get_mut(id).position;
        self.position_log.push(PositionChange {
            id,
            before: before.clone(),
            after: position.clone(),
        });
        self.get_mut(id).position = position;
    }

    pub fn get_is_it(&self, id: Id) -> bool {
        self.get(id).is_it
    }

    pub fn set_is_it(&mut self, id: Id, is_it: bool) {
        self.get_mut(id).is_it = is_it;
    }

    pub fn maybe_get_tagged_by(&self, id: Id) -> Option<Id> {
        self.get(id).tagged_by
    }

    pub fn set_tagged_by(&mut self, id: Id, tagged_by: Option<Id>) {
        self.get_mut(id).tagged_by = tagged_by;
    }

    pub fn get_tagged_count(&self) -> usize {
        self.tagged_count
    }

    pub fn increment_tagged(&mut self) {
        self.tagged_count += 1;
    }

    pub fn flush_log(&mut self) -> Vec<PositionChange> {
        self.position_log.drain(..).collect()
    }

    pub fn get_render_info(&mut self) -> Vec<RenderObject> {
        let mut v: Vec<RenderObject> = vec![];
        for agent in &self.agents {
            v.push((agent.position, agent.is_it));
        }
        v
    }

    fn rand_pos(&mut self, grid: &mut Grid) -> Option<Position> {
        let mut rand_pos: Position = Position::random();
        let mut c: usize = 0;
        while !grid.is_subgrid_free(rand_pos, STEP_SG_SIDE, STEP_SG_SIDE, vec![], None) {
            if c > 500 {
                return None;
            }
            rand_pos = Position::random();
            c += 1;
        }
        Some(rand_pos)
    }

    fn new_id(&mut self) -> Id {
        let rng = &mut self.rng;
        let mut id: Id = rng.gen_range(1, Id::max_value());
        while self.id_map.contains_key(&id) {
            id = rng.gen_range(1, Id::max_value());
        }
        id
    }

    fn get_mut(&mut self, id: Id) -> &mut Agent {
        let index: usize = *self.id_map.get(&id).unwrap();
        &mut self.agents[index]
    }

    fn get(&self, id: Id) -> &Agent {
        let index: usize = *self.id_map.get(&id).unwrap();
        &self.agents[index]
    }

    fn update_preference(
        agent: &mut Agent,
        mean_preferences: &Vec<f32>,
        action_count: usize,
        rng: &mut rand::prelude::ThreadRng,
    ) {
        let mut rand_ix: usize = rng.gen_range(0, action_count);
        let rm: f32 = 1.5;
        let rand_val: f32 = rng.gen_range(1.0 / rm, rm);
        agent.pref[rand_ix] *= rand_val;

        rand_ix = rng.gen_range(0, action_count);
        if rng.gen::<f32>() < 0.02 * mean_preferences[rand_ix] {
            agent.pref[rand_ix] = mean_preferences[rand_ix];
        }
    }

    fn get_actions_ordering(agent: &mut Agent, rng: &mut rand::prelude::ThreadRng) -> Vec<usize> {
        let pref: &Vec<f32> = &agent.pref;
        let mut vals: Vec<f32> = pref.clone();
        // let min: f32 = vals.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        for i in 0..vals.len() {
            vals[i] *= rng.gen::<f32>();
        }
        let mut ordering: Vec<(usize, &f32)> = (0 as usize..).zip(vals.iter()).collect();
        ordering.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let ix: Vec<usize> = ordering
            .iter()
            .map(|(i, _)| i.clone())
            .collect::<Vec<usize>>();
        ix
    }
}
