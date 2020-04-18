use crate::AGENT_NUM_UPPER_BOUND;
use crate::STEP_SG_SIDE;

use crate::action::*;
use crate::grid::Grid;
use crate::grid::Position;
use crate::grid::PositionChange;

use rand::Rng;
use std::collections::HashMap;

pub type Id = u32;

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
    rng: rand::prelude::ThreadRng,
}

impl AgentManager {
    pub fn new(ac: &ActionContext) -> AgentManager {
        let rng = rand::thread_rng();
        let action_count = ac.action_count;
        AgentManager {
            agents: vec![],
            id_map: HashMap::new(),
            position_log: vec![],
            action_count,
            tagged_count: 0,
            rng,
        }
    }

    pub fn add_agent(
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
                });
                grid.set(position, id);
            } else {
                return;
            }
        }
    }

    pub fn get_num_agents(&mut self) -> usize {
        self.agents.len()
    }

    pub fn get_ids(&mut self) -> Vec<Id> {
        (0..self.agents.len())
            .map(|i| self.agents[i].id)
            .collect::<Vec<Id>>()
    }

    pub fn get_actions_ordering(&mut self, id: Id) -> Vec<usize> {
        let pref: &Vec<f32> = &self.get(id).pref;
        let mut vals: Vec<f32> = pref.clone();
        for i in 0..vals.len() {
            vals[i] *= self.rng.gen::<f32>();
        }
        let mut ordering: Vec<(usize, &f32)> = (0 as usize..).zip(vals.iter()).collect();
        ordering.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
        let ix: Vec<usize> = ordering
            .iter()
            .map(|(i, _)| i.clone())
            .collect::<Vec<usize>>();
        ix
    }

    pub fn update_preference(&mut self, id: Id, mean_preferences: &Vec<f32>) {
        // Over time, acts similarly to a mean reverting walk
        let action_count = self.action_count;
        let mut rand_ix: usize = self.rng.gen_range(0, action_count);
        let rm: f32 = 1.3;
        let rand_val: f32 = self.rng.gen_range(1.0 / rm, rm);
        self.get(id).pref[rand_ix] *= rand_val;

        rand_ix = self.rng.gen_range(0, action_count);
        if self.rng.gen::<f32>() < 0.01 * mean_preferences[rand_ix] {
            self.get(id).pref[rand_ix] = mean_preferences[rand_ix];
        }
    }

    pub fn get_position(&mut self, id: Id) -> Position {
        self.get(id).position
    }

    pub fn set_position(&mut self, id: Id, position: Position) {
        let before = self.get(id).position;
        self.position_log.push(PositionChange {
            id,
            before: before.clone(),
            after: position.clone(),
        });
        self.get(id).position = position;
    }

    pub fn get_is_it(&mut self, id: Id) -> bool {
        self.get(id).is_it
    }

    pub fn set_is_it(&mut self, id: Id, is_it: bool) {
        self.get(id).is_it = is_it;
    }

    pub fn maybe_get_tagged_by(&mut self, id: Id) -> Option<Id> {
        self.get(id).tagged_by
    }

    pub fn set_tagged_by(&mut self, id: Id, tagged_by: Option<Id>) {
        self.get(id).tagged_by = tagged_by;
    }

    pub fn get_tagged_count(&mut self) -> usize {
        self.tagged_count
    }

    pub fn increment_tagged(&mut self) {
        self.tagged_count += 1;
    }

    pub fn flush_log(&mut self) -> Vec<PositionChange> {
        self.position_log.drain(..).collect()
    }

    fn rand_pos(&mut self, grid: &mut Grid) -> Option<Position> {
        let mut rand_pos: Position = Position::random();
        let mut c: usize = 0;
        while !grid.is_subgrid_free(rand_pos, STEP_SG_SIDE, STEP_SG_SIDE, vec![]) {
            if c > 500 {
                return None;
            }
            rand_pos = Position::random();
            c += 1;
        }
        Some(rand_pos)
    }

    fn new_id(&mut self) -> Id {
        let mut rng = self.rng;
        let mut id: Id = rng.gen_range(1, Id::max_value());
        while self.id_map.contains_key(&id) {
            id = rng.gen_range(1, Id::max_value());
        }
        id
    }

    fn get(&mut self, id: Id) -> &mut Agent {
        let index: usize = *self.id_map.get(&id).unwrap();
        &mut self.agents[index]
    }
}
