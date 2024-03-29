use bracket_terminal::prelude::{PURPLE, RGB, WHITESMOKE};
use hecs::EntityBuilder;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json::from_str;
use std::{collections::HashMap, fs, sync::Mutex};

mod living_structs;
use living_structs::LivingData;
mod tile_structs;
use tile_structs::TileData;
mod perlin_structs;

use crate::{
    actor::{CharSprite, Name, Player, Position},
    combat::CombatStats,
    fov::ViewShed,
    map::{TileType, WorldTile},
    monster::Breed,
};

lazy_static! {
    pub static ref ENTITY_DB: Mutex<EntityDatabase> = Mutex::new(EntityDatabase::empty());
    // pub static ref PERLIN_NOISE: Mutex<PerlinNoise> = Mutex::new();
}

// Basically make this a list or something, i only need it when i generate a new map
// i could just read it in from file rather than hold on to it
// #[derive(Deserialize, Debug)]
// pub struct PerlinNoise {
//     settings: PerlinSettings
// }

#[derive(Deserialize, Debug)]
pub struct EntityDatabase {
    living: LivingData,
    tiles: TileData,

    #[serde(skip)]
    living_index: HashMap<String, usize>,
    #[serde(skip)]
    tile_index: HashMap<String, usize>,
}

impl EntityDatabase {
    fn empty() -> Self {
        Self {
            living: LivingData::default(),
            tiles: TileData::default(),
            living_index: HashMap::new(),
            tile_index: HashMap::new(),
        }
    }

    /// Initializes the entity database to hold the objects as well as where they are located
    fn load(&mut self, data: EntityDatabase) {
        *self = data;
        for (idx, monster) in self.living.all.iter().enumerate() {
            self.living_index.insert(monster.name.clone(), idx);
        }

        for (idx, tile) in self.tiles.all.iter().enumerate() {
            self.tile_index.insert(tile.name.clone(), idx);
        }
    }
}

/// Loads all the json data of entities stored in the resources folder within the data folder
pub fn load_data_for_entities() {
    let mut entity_data = EntityDatabase::empty();

    let contents: String =
        fs::read_to_string("resources/data/living.json").expect("Unable to read to a string, please check file.");
    let living: LivingData = from_str(&contents).expect("Bad JSON in living.json fix it");
    entity_data.living = living;

    let contents: String =
        fs::read_to_string("resources/data/tile.json").expect("Unable to read to a string, please check file.");
    let tile: TileData = from_str(&contents).expect("Bad JSON in tile.json fix it");
    entity_data.tiles = tile;

    ENTITY_DB.lock().unwrap().load(entity_data);
}

/// Returns a tile based on a name provided, will return an "empty" tile if the name
/// provided does not exist.
pub fn named_tile(name: &str) -> WorldTile {
    let edb = &ENTITY_DB.lock().unwrap();
    let mut builder = WorldTile::empty();

    if !edb.tile_index.contains_key(name) {
        println!("{} does not exist", name);
        return builder;
    }
    let tile_info = &edb.tiles.all[edb.tile_index[name]];

    if let Some(is_blocking) = tile_info.is_blocking {
        builder.is_blocking = is_blocking;
    }
    if let Some(is_transparent) = tile_info.is_transparent {
        builder.is_transparent = is_transparent;
    }
    // if let Some(d_info) = &tile_info.destructible_info {
    //     builder.destructible = match d_info.by_what.as_str() {
    //         "hand" => Destructible::ByHand {
    //             health: d_info.hits,
    //             dropped_item: Item {},
    //         },
    //         "pickaxe" => Destructible::_ByPick {
    //             health: d_info.hits,
    //             dropped_item: Item {},
    //         },
    //         _ => Destructible::Unbreakable,
    //     };
    // }
    if let Some(sprite) = &tile_info.sprite {
        let fg = RGB::from_hex(&sprite.fg).unwrap_or(RGB::named(PURPLE));
        let bg = RGB::from_hex(&sprite.bg).unwrap_or(RGB::named(WHITESMOKE));

        builder.sprite = CharSprite::rgb(sprite.glyph, fg, bg);
    }
    if let Some(t_type) = &tile_info.tile_type {
        builder.tile_type = match t_type.as_str() {
            "upstairs" => TileType::UpStairs,
            "downstairs" => TileType::DownStairs,
            "wall" => TileType::Wall,
            "floor" => TileType::Floor,
            "water" => TileType::Water,
            "special" => TileType::Special,
            _ => TileType::Unknown,
        }
    }

    builder
}

pub fn named_living_builder(edb: &EntityDatabase, name: &str, pos: Position) -> Option<EntityBuilder> {
    if !edb.living_index.contains_key(name) {
        return None;
    }
    let monster_info = &edb.living.all[edb.living_index[name]];
    let mut eb = EntityBuilder::new();

    eb.add(pos);

    if let Some(sprite) = &monster_info.sprite {
        let fg = RGB::from_hex(&sprite.fg).unwrap_or(RGB::named(PURPLE));
        let bg = RGB::from_hex(&sprite.bg).unwrap_or(RGB::named(WHITESMOKE));

        eb.add(CharSprite::rgb(sprite.glyph, fg, bg));
    }

    if let Some(breed) = &monster_info.breed {
        if let Some(ai) = &monster_info.ai {
            eb.add(Breed::from(name, breed, ai));
        } else {
            eb.add(Breed::from(name, breed, "basic"));
        }
    }

    if let Some(view_distance) = &monster_info.view_range {
        eb.add(ViewShed::new(*view_distance));
    }

    if let Some(_) = &monster_info.player {
        eb.add(Player);
    }

    if let Some(stats) = &monster_info.combatstats {
        eb.add(CombatStats::new(stats.hp, stats.str, stats.def));
    }

    if name == "Player" {
        eb.add(Name(name.to_string()));
    }

    Some(eb)
}
