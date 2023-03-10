/*  Actors are defined as entities who performs actions.
   This file defines the components and systems commonly used by them.
*/
use bracket_terminal::prelude::*;

use crate::{
    fov::ViewShed,
    map::{xy_to_idx, Map, MAP_HEIGHT, MAP_WIDTH},
    BTerm, State,
};

/// Attempts to move an entity's position given it is allowed to move there
/// Returns true if successful in moving
pub fn try_move(map: &Map, dest_tile: Position, pos: &mut Position, view: &mut ViewShed) -> bool {
    if let Some(tile) = map.tiles.get(xy_to_idx(dest_tile.x, dest_tile.y)) {
        if !tile.is_blocking && within_bounds(dest_tile) {
            *pos = dest_tile;
            view.dirty = true;
            return true;
        }
    }
    false
}

fn within_bounds(tile_pos: Position) -> bool {
    tile_pos.x < MAP_WIDTH && tile_pos.y < MAP_HEIGHT
}

/// Renders all entities that have a Position and Sprite component
pub fn render_entities(ctx: &mut BTerm, state: &State) {
    for (_, (pos, sprite)) in state.world.query::<(&Position, &CharSprite)>().iter() {
        ctx.set(pos.x, pos.y, sprite.fg, sprite.bg, sprite.glyph);
    }
}

/// Tag Component that marks the player entity
pub struct Player;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy)]
pub struct CharSprite {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

type Color = (u8, u8, u8);

impl CharSprite {
    /// Create a new sprite, bg defaults to black which is useful for items
    pub fn new(glyph: char, fg: Color, bg: Option<Color>) -> Self {
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
}
