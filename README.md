# Agent Based Simulation of Tag

A simple simulation of the game Tag. 


## To run
```
> git clone https://github.com/jvallikivi/tag.git
> cd tag
> cargo run --release
```

## Actions

Currently, actions are defined as follows:

* One step left/right/up/down
* Stand still
* Tag someone

As defined in [./src/actions.rs](./src/action.rs), action definitions follow a generic PDDL (Planning Domain Definition Language) approach. Action parameters are not included due to the simple nature of the game. 

Every agent holds a weight (preference) for every possible action, which is correlated with the probability that the agent chooses it. These preferences change over time in a random, yet mean reverting fashion.

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