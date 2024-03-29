use bracket_terminal::prelude::*;
use hecs::{With, World};

use crate::{
    actor::{Player, Position},
    combat::CombatStats,
    map::Map,
    State,
};

pub fn draw_gui(ctx: &mut BTerm, state: &State) {
    let screenheight = state.config.screensize_y;
    draw_message_box(ctx, state, screenheight);
    draw_right_box(ctx, state, screenheight);

    ctx.print_color(0, 79, WHITESMOKE, BLACK, format!("FPS: {}", ctx.fps));
}

fn draw_message_box(ctx: &mut BTerm, state: &State, screenheight: usize) {
    ctx.draw_box(0, screenheight - 10, 99, 9, WHITE, BLACK);

    let lastmsgidx = state.message_log.len() - 1;
    for (msg_offset, i) in (lastmsgidx.saturating_sub(7)..=lastmsgidx).enumerate() {
        ctx.print(1, screenheight - 9 + msg_offset, &state.message_log[i].contents);
    }
}

fn draw_right_box(ctx: &mut BTerm, state: &State, screenheight: usize) {
    let right_map_edge_x = state.map.width + 1;
    ctx.draw_box(right_map_edge_x - 1, 0, 19, screenheight - 1, WHITE, BLACK);
    ctx.print(right_map_edge_x, 1, "TerraIncognita");
    ctx.print(right_map_edge_x, 2, "Dev Build V0.1.0");
    ctx.print(right_map_edge_x, 3, format!("Turn: {}", state.turn_counter));
    ctx.print(right_map_edge_x, 4, format!("Seed: {}", state.config.world_seed));

    if let Some((pos, idx)) = get_player_pos(&state.world, &state.map) {
        ctx.print(right_map_edge_x, 5, format!("X: {} Y: {}", pos.x(), pos.y()));
        ctx.print(right_map_edge_x, 6, format!("Tile Index: {}", idx));
    }

    if let Some(stats) = get_player_stats(&state.world) {
        draw_hp_bar(
            ctx,
            "Player",
            stats.health,
            stats.max_health,
            Point::new(right_map_edge_x, 9),
        );
    }

    ctx.print(right_map_edge_x, 7, format!("Depth: {}", state.map.depth));
}

/// A fully customizable bar that splits between two characters with custom colors
type ColoredChar = (char, RGBA, RGBA);
fn draw_horizontal_split_bar(
    ctx: &mut BTerm,
    n: u32,
    max: u32,
    sx: i32,
    sy: i32,
    width: usize,
    filled: ColoredChar,
    empty: ColoredChar,
) {
    let percent = n as f32 / max as f32;
    let fill_width = (percent * width as f32) as usize;
    for x in 0..width as i32 {
        if x <= fill_width as i32 {
            ctx.set(sx + x, sy, filled.1, empty.2, to_cp437(filled.0));
        } else {
            ctx.set(sx + x, sy, empty.1, empty.2, to_cp437(empty.0));
        }
    }
}

/// Creates a health bar with a name
fn draw_hp_bar(ctx: &mut BTerm, name: impl ToString, hp: u32, max_hp: u32, starting_pos: Point) {
    ctx.print(starting_pos.x, starting_pos.y, format!("►{}", name.to_string()));

    ctx.print(starting_pos.x, starting_pos.y + 1, "HP:");
    draw_horizontal_split_bar(
        ctx,
        hp,
        max_hp,
        starting_pos.x + 3,
        starting_pos.y + 1,
        15,
        ('♥', RGBA::named(LIME_GREEN), RGBA::named(BLACK)),
        ('♥', RGBA::named(DARKRED), RGBA::named(BLACK)),
    );
}

fn get_player_pos(world: &World, map: &Map) -> Option<(Position, usize)> {
    if let Some((_, player_pos)) = world.query::<With<&Position, &Player>>().iter().next() {
        return Some((player_pos.clone(), player_pos.0.to_index(map.width)));
    }
    None
}

fn get_player_stats(world: &World) -> Option<CombatStats> {
    if let Some((_, player_combat_stats)) = world.query::<With<&CombatStats, &Player>>().iter().next() {
        return Some(player_combat_stats.clone());
    }
    None
}
