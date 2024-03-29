use std::collections::HashMap;

use bracket_terminal::prelude::{BTerm, GameState};
use hecs::World;

use crate::{
    actor::render_entities,
    combat::destroy_dead_beings,
    config::Config,
    fov::update_vision,
    gui::draw_gui,
    input::{handle_player_action, player_input},
    map::{render_map, Map},
    menu::{run_menu_systems, MenuIndex},
    messagelog::Message,
    monster::handle_monster_turns,
    save_system::save_game,
    start_new_game,
    worldgen::move_to_new_floor,
};

pub struct State {
    pub world: World, // Holds all of our entities
    pub map: Map,     // Holds the tiles to the world
    pub message_log: Vec<Message>,

    pub generated_maps: HashMap<usize, Map>,

    pub runstate: RunState,
    pub config: Config,
    pub turn_counter: usize,

    pub visible: Vec<bool>, // Player's visibility
}

#[derive(Clone)]
pub enum RunState {
    InGame,
    MainMenu(MenuIndex),
    NextLevel(usize),
    SaveGame,
}

pub enum PlayerResponse {
    StateChange(RunState),
    TurnAdvance,
    Waiting,
}

impl State {
    /// Empty start of state that starts in the menu
    pub fn new(config: &Config) -> Self {
        Self {
            world: World::new(),
            map: Map::empty(),
            runstate: RunState::MainMenu(MenuIndex(0)),
            config: config.clone(),
            message_log: vec![
                Message::new("Welcome to Terra Incognita".to_string(), 0),
                Message::new("This is an alpha build from April 2023".to_string(), 0),
            ],
            turn_counter: 0,
            generated_maps: HashMap::new(),
            visible: vec![false; config.map_x * config.map_y],
        }
    }

    /// For dev purposes, we can skip the main menu
    pub fn dev(config: &Config) -> Self {
        let mut world = World::new();
        let map = start_new_game(&mut world, config.world_seed);
        State {
            world,
            map,
            runstate: RunState::InGame,
            config: config.clone(),
            message_log: vec![
                Message::new("Welcome to Terra Incognita".to_string(), 0),
                Message::new("This is a dev build from April 2023".to_string(), 0),
            ],
            turn_counter: 0,
            generated_maps: HashMap::new(),
            visible: vec![false; config.map_x * config.map_y],
        }
    }

    /// Resets state to be a blank slate for a new game
    fn clean_up(&mut self) {
        *self = State::new(&self.config);
    }

    /// Systems that are ran every frame, regardless of turn progression
    fn run_continuous_systems(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        update_vision(self);

        render_map(ctx, &self.map, &self.config, &self.visible, &self.map.discovered);
        render_entities(ctx, self);

        draw_gui(ctx, self);
    }

    fn run_pre_response_systems(&mut self) {
        destroy_dead_beings(&mut self.world, &mut self.map);
    }

    /// Response systems are ran after a player inputs something that progresses a turn
    fn run_response_systems(&mut self) {
        handle_monster_turns(self);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        let mut newstate = self.runstate.clone();

        match newstate {
            RunState::InGame => {
                self.run_continuous_systems(ctx);
                let player_action = player_input(ctx);
                match handle_player_action(self, player_action) {
                    PlayerResponse::StateChange(new_state) => {
                        ctx.cls();
                        newstate = new_state;
                    }
                    PlayerResponse::TurnAdvance => {
                        self.turn_counter += 1;
                        self.run_pre_response_systems();
                        self.run_response_systems();
                    }
                    _ => {}
                }
            }
            RunState::MainMenu(menu_idx) => {
                newstate = run_menu_systems(self, ctx, menu_idx.0);
            }
            RunState::NextLevel(new_depth) => {
                move_to_new_floor(self, new_depth);
                newstate = RunState::InGame;
            }
            RunState::SaveGame => {
                save_game(self);
                self.clean_up();
                newstate = RunState::MainMenu(MenuIndex(0));
            }
        }

        self.runstate = newstate;
    }
}
