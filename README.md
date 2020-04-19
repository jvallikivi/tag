# Agent Based Simulation of Tag

A simple simulation of the game Tag. 


## To run
```
> git clone https://github.com/jvallikivi/tag.git
> cd tag
> cargo run --release
```

## Actions

Currently, actions are defined as follows (and can be added/modified as shown in the collapsible section below):

* One step left/right/up/down
* Stand still
* Tag someone

As defined in [./src/actions.rs](./src/action.rs), action definitions follow a generic PDDL (Planning Domain Definition Language) approach. Action parameters are not included due to the simple nature of the game. 

Every agent holds a weight (preference) for every possible action, which is correlated with the probability that the agent chooses it. These preferences change over time in a random, yet mean reverting fashion.

<details>
  <summary>Show me how to add/modify an action!</summary>

Feel free to add/modify actions in `ActionContext::new` in [./src/actions.rs](./src/action.rs), where the 6 existing actions are defined. Below is an example of an action that could be added which moves the agent left and up in one move. Make sure that the agents don't leave the grid! Take care of data races as the simulation runs on multiple threads. For example if two taggers show the intent of tagging the same untagged agent, running tag_effect may panic. 

```Rust
    pub fn new() -> ActionContext {
        


        /* Other already defined actions go above here */

        // The precondition for moving left and up by one:
        // The closure is given the agent id, the agent manager and grid
        let left_up_step_precond: Precondition = |id, am, grid| {
            let position = am.get_position(id);

            // Make sure that the agent is not already standing on
            // the left-most or top-most edge of the grid
            if position.x == 0 || position.y == 0 {
                return false;
            }
            if COLLSION_DETECTION {
                // Check that the destination has no other
                // agents around its vicinity.
                // In grid.rs, check out the following methods
                // for searching the grid:
                // 'is_subgrid_free', 'is_subgrid_occupied',
                // 'get_subgrid_occupiers'
                grid.is_subgrid_free(
                    // Destination
                    Position {
                        x: position.x - 1,
                        y: position.y - 1,
                    },
                    // See the Parameters secion in Readme
                    STEP_SG_SIDE,
                    STEP_SG_SIDE,
                    // Agent ids which should be ignored in
                    // checking whether the vicinity is free
                    vec![id],
                    // Optional closure of type
                    // Option<&dyn Fn(Id) -> bool>
                    // which checks if given an id of
                    // an agent inside the defined vicinity
                    // should it be ignored or not.
                    // Check out the tag_precond function
                    // which includes a closure which
                    // tells grid to ignore other agents
                    // which are already tagged
                    None,
                )
            } else {
                true
            }
        };

        // The effect of executing the action of moving
        // left and up by one, calling this realises the change
        // in the simulation
        let left_up_step_effect: Effect = |id, am, _grid| {
            let mut position = am.get_position(id);
            position.x -= 1;
            position.y -= 1;
            am.set_position(id, position);
        };

        // Create the action by combining the precondition and effect
        let left_up_step: Action = Action {
            precond: left_up_step_precond,
            effect: left_up_step_effect,
        };

        // Put all actions (including the new, 7th action) in a vector
        let actions: Vec<Action> = vec![
            left_step,
            right_step,
            up_step,
            down_step,
            do_nothing,
            tag,
            left_up_step,
        ];
        let action_count = actions.len();

        ActionContext {
            actions,
            // Define some weights for each action (in the same order
            // as in 'actions') indicating what preferences should
            // agents have on average. For example 0.5 for left_up_step
            // shows that it should, in a longer timeframe, be picked
            // as often as left_step, right_step, up_step and down_step
            mean_preferences: vec![0.5, 0.5, 0.5, 0.5, 0.1, 0.9, 0.5],
            action_count,
        }
    }
```
</details>


## Parameters
The parameters that can be played with with are in [./src/main.rs](./src/main.rs). After every modification it is important to build (`cargo build --release`) again.
```
pub const USE_VIEWER: bool = true;
pub const COLLSION_DETECTION: bool = true;
pub const STEP_SG_SIDE: usize = 21;
pub const TAG_SG_SIDE: usize = 31;
pub const GRID_SIDE: usize = 800;
pub const NUM_STEPS: usize = 20000;
pub const NUM_AGENTS: usize = 1000;
pub const NUM_AGENTS_IT: usize = 2;
```
* `USE_VIEWER`: Whether to visualise the simulation (not recommended for benchmarking)
* `COLLISION_DETECION`: If true, the simulation does not allow agents too close to each other (overlapping agents), meaning agents act as movement barriers to each other - a more life-like approach. However it results in a simulation which is approximately twice as slow as a simulation without any collision detection
* `STEP_SG_SIDE`: If an agent wants to step into a grid location x, a square of side length `STEP_SG_SIDE` with center at x, must not contain any other agents. This is only used if `COLLISION_DETECTION` is true
* `TAG_SG_SIDE`: If an agent wants to tag a target agent (no tag-backs (see [More](#more)) or tagging someone who is already _**it**_ (in games with multiple agents being _**it**_ at the same time)) then the target agent must be in the square of side length `TAG_SG_SIDE` with center at the agent who wants to tag. This is to simulate the proximity requirement of tagging someone
* `GRID_SIDE`: The environment is a square grid with side length `GRID_SIDE`
* `NUM_AGENTS`: Number of agents in the simulation. Note that if the propsed number of agents exceeds the upper bound `AGENT_NUM_UPPER_BOUND`, then the exceeding agents will not be added
* `NUM_AGENTS_IT`: Number of agents that initially are tagged (_**it**_)

## More
https://en.wikipedia.org/wiki/Tag_(game)
