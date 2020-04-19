use crate::{COLLSION_DETECTION, GRID_SIDE, STEP_SG_SIDE, TAG_SG_SIDE};

use crate::agent::{AgentManager, Id};
use crate::grid::{Grid, Position};

use rand::seq::SliceRandom;

pub type Precondition = fn(Id, &AgentManager, &Grid) -> bool;
pub type Effect = fn(Id, &mut AgentManager, &Grid);

pub struct Action {
    pub precond: Precondition,
    pub effect: Effect,
}

pub struct ActionContext {
    /// Contains all actions, and hence all preconditions and effects of actions
    actions: Vec<Action>,
    /// These contain the weights (preferences) for every action which is used as
    /// an update guideline for agent's own preferences
    mean_preferences: Vec<f32>,
    /// Number of actions
    pub action_count: usize,
}

impl ActionContext {
    pub fn get_mean_preferences(&self) -> &Vec<f32> {
        &self.mean_preferences
    }

    pub fn maybe_get_allowed_effect(
        &self,
        actions_ordering: &[usize],
        id: Id,
        am: &AgentManager,
        grid: &Grid,
    ) -> Option<Effect> {
        for j in 0..self.actions.len() {
            let action: &Action = &self.actions[actions_ordering[j]];
            let action_allowed = (action.precond)(id, am, grid);
            if action_allowed {
                return Some(action.effect);
            }
        }
        None
    }

    pub fn new() -> ActionContext {
        let left_step_precond: Precondition = |id, am, grid| {
            let position = am.get_position(id);
            if position.x == 0 {
                return false;
            }
            if COLLSION_DETECTION {
                grid.is_subgrid_free(
                    Position {
                        x: position.x - 1,
                        y: position.y,
                    },
                    STEP_SG_SIDE,
                    STEP_SG_SIDE,
                    vec![id],
                    None,
                )
            } else {
                true
            }
        };
        let left_step_effect: Effect = |id, am, _| {
            let mut position = am.get_position(id);
            position.x -= 1;
            am.set_position(id, position);
        };
        let left_step: Action = Action {
            precond: left_step_precond,
            effect: left_step_effect,
        };

        let right_step_precond: Precondition = |id, am, grid| {
            let position = am.get_position(id);
            if position.x == GRID_SIDE - 1 {
                return false;
            }
            if COLLSION_DETECTION {
                grid.is_subgrid_free(
                    Position {
                        x: position.x + 1,
                        y: position.y,
                    },
                    STEP_SG_SIDE,
                    STEP_SG_SIDE,
                    vec![id],
                    None,
                )
            } else {
                true
            }
        };
        let right_step_effect: Effect = |id, am, _| {
            let mut position = am.get_position(id);
            position.x += 1;
            am.set_position(id, position);
        };
        let right_step: Action = Action {
            precond: right_step_precond,
            effect: right_step_effect,
        };

        let up_step_precond: Precondition = |id, am, grid| {
            let position = am.get_position(id);
            if position.y == 0 {
                return false;
            }
            if COLLSION_DETECTION {
                grid.is_subgrid_free(
                    Position {
                        x: position.x,
                        y: position.y - 1,
                    },
                    STEP_SG_SIDE,
                    STEP_SG_SIDE,
                    vec![id],
                    None,
                )
            } else {
                true
            }
        };
        let up_step_effect: Effect = |id, am, _| {
            let mut position = am.get_position(id);
            position.y -= 1;
            am.set_position(id, position);
        };
        let up_step: Action = Action {
            precond: up_step_precond,
            effect: up_step_effect,
        };

        let down_step_precond: Precondition = |id, am, grid| {
            let position = am.get_position(id);
            if position.y == GRID_SIDE - 1 {
                return false;
            }
            if COLLSION_DETECTION {
                grid.is_subgrid_free(
                    Position {
                        x: position.x,
                        y: position.y + 1,
                    },
                    STEP_SG_SIDE,
                    STEP_SG_SIDE,
                    vec![id],
                    None,
                )
            } else {
                true
            }
        };
        let down_step_effect: Effect = |id, am, _| {
            let mut position = am.get_position(id);
            position.y += 1;
            am.set_position(id, position);
        };
        let down_step: Action = Action {
            precond: down_step_precond,
            effect: down_step_effect,
        };

        let do_nothing_precond: Precondition = |_, _, _| true;
        let do_nothing_effect: Effect = |_, _, _| {};
        let do_nothing: Action = Action {
            precond: do_nothing_precond,
            effect: do_nothing_effect,
        };

        let tag_precond: Precondition = |id, am, grid| {
            if am.get_is_it(id) {
                let mut excluded_ids = vec![id];
                let maybe_tagged_by: Option<Id> = am.maybe_get_tagged_by(id);
                if let Some(tagged_by) = maybe_tagged_by {
                    excluded_ids.push(tagged_by);
                }
                let am_immut = &*am;
                let ignore_tagged = move |id: Id| am_immut.get_is_it(id);

                grid.is_subgrid_occupied(
                    am.get_position(id),
                    TAG_SG_SIDE,
                    TAG_SG_SIDE,
                    excluded_ids,
                    Some(&ignore_tagged),
                )
            } else {
                false
            }
        };

        let tag_effect: Effect = |id, am, grid| {
            let mut excluded_ids = vec![id];
            let maybe_tagged_by: Option<Id> = am.maybe_get_tagged_by(id);
            if let Some(tagged_by) = maybe_tagged_by {
                excluded_ids.push(tagged_by);
            }
            let am_immut = &*am;
            let ignore_tagged = move |id: Id| am_immut.get_is_it(id);
            let ids: Vec<Id> = grid.get_subgrid_occupiers(
                am.get_position(id),
                TAG_SG_SIDE,
                TAG_SG_SIDE,
                excluded_ids,
                Some(&ignore_tagged),
            );
            
            if let Some(target_id) = ids.choose(&mut am.rng) {
                am.set_is_it(id, false);
                am.set_is_it(*target_id, true);
                am.set_tagged_by(id, None);
                am.set_tagged_by(*target_id, Some(id));
                am.increment_tagged();
            }
        };
        let tag: Action = Action {
            precond: tag_precond,
            effect: tag_effect,
        };
        let actions: Vec<Action> = vec![left_step, right_step, up_step, down_step, do_nothing, tag];
        let action_count = actions.len();
        ActionContext {
            actions,
            mean_preferences: vec![0.5, 0.5, 0.5, 0.5, 0.1, 0.9],
            action_count,
        }
    }
}
