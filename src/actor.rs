/*  Actors are defined as entities who performs actions.
    This file defines the components and systems commonly used by them.
*/
use bracket_terminal::prelude::*;
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::cmp;

use crate::{
    combat::{attack, CombatStats},
    data_read::named_tile,
    fov::ViewShed,
    map::{Destructible, Map, TileType},
    messagelog::Message,
    monster::Breed,
    BTerm, State,
};

pub enum MoveResult {
    Moved(String),
    Attack(Entity),
    Mine(Destructible),
    InvalidMove(String),
}

/// Get's player information and calls bump_tile
/// Used to make bump_tile entity agnostic ie monster or player can call it
pub fn player_bump(map: &mut Map, world: &mut World, delta: Point) -> MoveResult {
    if let Some((e, (pos, view, _))) = world.query::<(&mut Position, &mut ViewShed, &Player)>().iter().next() {
        let dest_tile = match safe_position_delta(pos, delta, Point::new(map.width, map.height)) {
            Ok(dest_tile) => dest_tile,
            Err(..) => {
                return MoveResult::InvalidMove("Cannot move to that position".to_string());
            }
        };
        return bump_tile(map, &dest_tile, pos, view, e);
    }
    MoveResult::InvalidMove("No player entity found?".to_string())
}

/// Adds the delta to a position while keeping it inside 0 -> upper x or y
/// If the position is the same after the delta then the function fails
fn safe_position_delta(pos: &Position, delta: Point, upper: Point) -> Result<Position, ()> {
    let x = cmp::min(cmp::max(pos.x() + delta.x, 0), upper.x);
    let y = cmp::min(cmp::max(pos.y() + delta.y, 0), upper.y);
    if x == pos.x() && y == pos.y() {
        Err(())
    } else {
        Ok(Position::new(x, y))
    }
}

/// Attempts to move an entity's position given it is allowed to move there
/// Returns true if successful in moving
pub fn bump_tile(
    map: &mut Map,
    dest_tile: &Position,
    pos: &mut Position,
    view: &mut ViewShed,
    who: Entity, // the entity that is moving
) -> MoveResult {
    if !map.within_bounds(dest_tile.0) {
        return MoveResult::InvalidMove(format!("{},{} is out of bounds", dest_tile.0.x, dest_tile.0.y));
    }

    let dest_idx = dest_tile.0.to_index(map.width);
    if let Some(target) = map.beings[dest_idx] {
        return MoveResult::Attack(target);
    }

    if let Some(tile) = map.tiles.get_mut(dest_idx) {
        view.dirty = true; // make it dirty so the vision is updated definitely
        if !tile.is_blocking {
            let idx = pos.0.to_index(map.width);
            *pos = dest_tile.clone();
            map.beings[idx] = None;
            map.beings[dest_idx] = Some(who);
            return MoveResult::Moved("".to_string());
        } else if let Some(destructible) = map.destructibles[dest_idx] {
            return MoveResult::Mine(destructible);
        }
        return MoveResult::InvalidMove("Tile is blocked".to_string());
    }
    MoveResult::InvalidMove("No tile".to_string())
}

/// Attempts to change floor
pub fn change_floor(world: &mut World, map: &mut Map, delta: i32) -> bool {
    if let Some((_, (pos, view, _))) = world.query::<(&mut Position, &mut ViewShed, &Player)>().iter().next() {
        // positive delta means we try to descend
        if delta > 0 && map.tiles[pos.0.to_index(map.width)].tile_type == TileType::DownStairs {
            view.dirty = true;
            return true;
        } else if delta < 0 // negative delta means we try to ascend
            && map.tiles[pos.0.to_index(map.width)].tile_type == TileType::UpStairs
            && !(map.depth == 0 && delta == 1)
        {
            view.dirty = true;
            return true;
        }
    }
    false
}

pub fn player_attack(world: &mut World, message_log: &mut Vec<Message>, target: Entity, turn_sent: usize) {
    if let Some((_, (attacker_stats, _))) = world.query::<(&mut CombatStats, &Player)>().iter().next() {
        if let Ok(mut defender) = world.query_one::<(&mut CombatStats, &Breed)>(target) {
            if let Some(defender) = defender.get() {
                let damage_stmt = attack((defender.0, &defender.1.name), (attacker_stats, &"Player"));
                message_log.push(Message::new(damage_stmt, turn_sent));
            }
        } // Prevents stale enemies from being double despawned
    }
}

/// Attempts to mine a position and returns if successful
pub fn mine(map: &mut Map, world: &mut World, destructible: Destructible, delta: Point) -> bool {
    if let Some((_, (pos, view, _))) = world.query::<(&mut Position, &mut ViewShed, &Player)>().iter().next() {
        let dest_pos = match safe_position_delta(pos, delta, Point::new(map.width, map.height)) {
            Ok(pos) => pos,
            Err(..) => return false,
        };
        let dest_idx = dest_pos.0.to_index(map.width);

        match destructible {
            Destructible::Tile { max_hp, mut hp } => {
                hp -= 1;
                if hp <= 0 {
                    map.tiles[dest_idx] = match map.tiles[dest_idx].tile_type {
                        TileType::Wall => named_tile("Grass Floor"),
                        _ => named_tile("Grass Floor"),
                    };
                    map.destructibles[dest_idx] = None;
                    return true;
                }
                map.destructibles[dest_idx] = Some(Destructible::Tile { max_hp, hp });
            }
            Destructible::Entity(_e) => {
                // if let Some(e) = world.query_one::<()>()
            }
        }

        view.dirty = true;
        return true;
    }

    false
}

/// Renders all entities that have a Position and Sprite component
pub fn render_entities(ctx: &mut BTerm, state: &State) {
    for (_, (pos, sprite)) in state.world.query::<(&Position, &CharSprite)>().iter() {
        if state.visible[pos.0.to_index(state.map.width)] || state.config.dev_mode {
            ctx.set(pos.x(), pos.y(), sprite.fg, sprite.bg, sprite.glyph);
        }
    }
}

/// Tag Component that marks the player entity
#[derive(Deserialize, Debug)]
pub struct Player;

#[derive(Deserialize, Debug)]
pub struct Name(pub String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position(pub Point);

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self(Point::new(x, y))
    }

    // Personal perference to use methods rather than tuple index :P
    pub fn x(&self) -> i32 {
        self.0.x
    }

    pub fn y(&self) -> i32 {
        self.0.y
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CharSprite {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

type Color = (u8, u8, u8);

impl CharSprite {
    /// Create a new sprite, bg defaults to black which is useful for items
    pub fn with_color(glyph: char, fg: Color, bg: Option<Color>) -> Self {
        match bg {
            Some(bg) => Self {
                glyph: to_cp437(glyph),
                fg: RGB::named(fg),
                bg: RGB::named(bg),
            },
            None => Self {
                glyph: to_cp437(glyph),
                fg: RGB::named(fg),
                bg: RGB::new(),
            },
        }
    }

    pub fn new(glyph: char, fg: Color, bg: Color) -> Self {
        Self {
            glyph: to_cp437(glyph),
            fg: RGB::named(fg),
            bg: RGB::named(bg),
        }
    }

    pub fn rgb(glyph: char, fg: RGB, bg: RGB) -> Self {
        Self {
            glyph: to_cp437(glyph),
            fg,
            bg,
        }
    }

    pub fn eq(&self, other: Self) -> bool {
        self.glyph == other.glyph && self.fg == other.fg && self.bg == other.bg
    }
}
