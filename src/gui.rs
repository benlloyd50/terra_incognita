use bracket_terminal::prelude::*;
use hecs::{With, World};

use crate::{
    actor::{Player, Position},
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
    let mut msg_offset = 0;
    for i in lastmsgidx.saturating_sub(7)..=lastmsgidx {
        ctx.print(1, screenheight - 9 + msg_offset, &state.message_log[i].contents);
        msg_offset += 1;
    }
}

fn draw_right_box(ctx: &mut BTerm, state: &State, screenheight: usize) {
    let right_map_edge_x = 101;
    ctx.draw_box(right_map_edge_x - 1, 0, 19, screenheight - 1, WHITE, BLACK);
    ctx.print(right_map_edge_x, 1, "TerraIncognita");
    ctx.print(right_map_edge_x, 2, "Dev Build V0.1.0");
    ctx.print(right_map_edge_x, 3, format!("Turn: {}", state.turn_counter));
    ctx.print(right_map_edge_x, 4, format!("Seed: {}", state.config.world_seed));

    if let Some(player_pos) = get_player_pos(&state.world, &state.map) {
        ctx.print(
            right_map_edge_x,
            5,
            format!("X: {}, Y: {}", player_pos.0.x, player_pos.0.y),
        );
        ctx.print(right_map_edge_x, 6, format!("Tile Index: {}", player_pos.1));
    }
}

fn get_player_pos(world: &World, map: &Map) -> Option<(Position, usize)> {
    for (_, player_pos) in world.query::<With<&Position, &Player>>().iter() {
        return Some((*player_pos, map.xy_to_idx(player_pos.x, player_pos.y)));
    }
    None
}