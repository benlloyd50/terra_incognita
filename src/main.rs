use bracket_terminal::prelude::*;
use hecs::*;
use rand::{seq::SliceRandom, Rng};
use std::fs;

mod data_read;
use data_read::{named_living_builder, ENTITY_DB};
mod gui;
mod map;
mod menu;
mod messagelog;
mod monster;
mod prefab;
mod worldgen;
use map::Map;
use worldgen::generate_map;
mod actor;
mod fov;
mod item;
use actor::{CharSprite, Player, Position};
mod combat;
use combat::CombatStats;
mod config;
mod input;
mod map_scanning;
mod save_system;
mod state;

use crate::{
    config::Config,
    data_read::load_data_for_entities,
    messagelog::Message,
    state::{RunState, State},
};

bracket_terminal::embedded_resource!(TILE_FONT, "../resources/RDE.png");
bracket_terminal::embedded_resource!(CAVE_ENTRANCE, "../resources/rex/cave_entrance.xp");
bracket_terminal::embedded_resource!(INTRO_SCREEN, "../resources/rex/intro_screen.xp");
bracket_terminal::embedded_resource!(MENU_OPTIONS, "../resources/rex/options_box.xp");

fn main() -> BError {
    load_data_for_entities();

    // Reads in a config file to setup the game
    let contents: String = fs::read_to_string("resources/config.toml")?;
    let config: Config = toml::from_str(&contents).unwrap();

    bracket_terminal::link_resource!(CAVE_ENTRANCE, "../resources/rex/cave_entrance.xp");
    bracket_terminal::link_resource!(INTRO_SCREEN, "../resources/rex/intro_screen.xp");
    bracket_terminal::link_resource!(MENU_OPTIONS, "../resources/rex/options_box.xp");

    // Setup terminal renderer
    bracket_terminal::link_resource!(TILE_FONT, "resources/RDE.png");
    let context = BTermBuilder::new()
        .with_title("Terra Incognita [ALPHA]")
        .with_fullscreen(config.fullscreen)
        .with_dimensions(config.screensize_x, config.screensize_y)
        .with_tile_dimensions(config.font_size, config.font_size)
        .with_font_bg(
            &config.font_file,
            config.font_size,
            config.font_size,
            RGB::from_u8(255, 0, 255),
        )
        .with_simple_console(config.screensize_x, config.screensize_y, &config.font_file)
        .build()?;

    let gs = if config.dev_mode {
        State::dev(&config)
    } else {
        State::new(&config)
    };

    main_loop(context, gs)
}

/// Creates a new map and setups world for the start of a fresh run
pub fn start_new_game(world: &mut World, seed: u64) -> Map {
    let (mut map, player_start) = generate_map(seed, 0);
    add_player_to_room(world, player_start);
    furnish_map(world, &mut map);
    map
}

/// Adds the life and decor to the map
fn furnish_map(world: &mut World, map: &mut Map) {
    add_beings_to_rooms(world, map);
}

pub fn add_player_to_room(world: &mut World, player_start: Position) {
    let player_builder = named_living_builder(&ENTITY_DB.lock().unwrap(), "Player", player_start);
    if let Some(mut pb) = player_builder {
        let p_entity = world.spawn(pb.build());
        match world.insert(p_entity, (CombatStats::new(200, 10, 1),)) {
            Ok(..) => {}
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}

fn add_beings_to_rooms(world: &mut World, map: &mut Map) {
    let beings = vec!["Centipede", "Mole", "Star Nosed Mole"];
    for room in map.rooms.iter() {
        let monster_spawns_per_room = room.tiles.len() / 15;
        for _ in 0..monster_spawns_per_room {
            let chance: f32 = rand::thread_rng().gen();
            if chance > 0.6 {
                continue;
            }

            let being_pos = room.get_random_point();
            let being_name = beings.choose(&mut rand::thread_rng()).unwrap();
            let e_builder = named_living_builder(&ENTITY_DB.lock().unwrap(), being_name, Position(being_pos));
            if let Some(mut eb) = e_builder {
                let e = world.spawn(eb.build());
                map.beings[being_pos.to_index(map.width)] = Some(e);
            }
        }
    }
}
