use bracket_pathfinding::prelude::{a_star_search, DistanceAlg};
use rand::random;
use std::cmp::max;

use hecs::{Entity, With};

use crate::{
    actor::{bump_tile, MoveResult, Player, Position},
    combat::{attack, CombatStats},
    fov::ViewShed,
    map::Map,
    Message, State,
};

pub fn handle_monster_turns(state: &mut State) {
    if let Some((_, (player_pos, player_stats))) = state
        .world
        .query::<With<(&Position, &mut CombatStats), &Player>>()
        .iter()
        .next()
    {
        for (e, (pos, view, breed, being_stats)) in state
            .world
            .query::<(&mut Position, &mut ViewShed, &mut Breed, &CombatStats)>()
            .iter()
        {
            let move_state = (
                // When move_state is changed the values that were mutable are also changed
                e,
                pos,
                *being_stats,
                view,
                player_pos.clone(),
                &mut *player_stats,
                &mut state.map,
                state.turn_counter,
                &mut state.message_log,
            );
            breed.perform_move(move_state);
        }
    }
}

/// General info about the type of monster/creature
#[derive(Clone)]
pub struct Breed {
    pub name: String,
    _species: String,
    ai: BeingAI,
}

#[derive(Clone)]
pub enum BeingAI {
    BasicPoke, // Simplest AI being able to wander, follow the player if they are visible, and poke the player
}

impl Breed {
    pub fn from(name: impl ToString, species: impl ToString, ai: impl ToString) -> Self {
        let ai = match ai.to_string().as_str() {
            "basic" => BeingAI::BasicPoke,
            _ => BeingAI::BasicPoke,
        };
        Self {
            name: name.to_string(),
            _species: species.to_string(),
            ai,
        }
    }

    fn perform_move(
        &mut self,
        move_state: (
            Entity,
            &mut Position,
            CombatStats,
            &mut ViewShed,
            Position,
            &mut CombatStats,
            &mut Map,
            usize,
            &mut Vec<Message>,
        ),
    ) {
        match self.ai {
            BeingAI::BasicPoke => simple_ai(self, move_state),
        }
    }
}

fn simple_ai(
    breed: &Breed,
    (me, pos, attacker_stats, view, player_pos, player_stats, map, turn_counter, message_log): (
        Entity,
        &mut Position,
        CombatStats,
        &mut ViewShed,
        Position,
        &mut CombatStats,
        &mut Map,
        usize,
        &mut Vec<Message>,
    ),
) {
    let dist_to_player = DistanceAlg::Pythagoras.distance2d(player_pos.0, pos.0);
    if dist_to_player < 1.5 {
        let damage_stmt = attack((player_stats, &"Player"), (&attacker_stats, &breed.name));
        message_log.push(Message::new(damage_stmt, turn_counter));
        return;
    }
    let tile_idx = pos.0.to_index(map.width);

    if view.visible_tiles.contains(&player_pos.0) {
        let path = a_star_search(tile_idx, player_pos.0.to_index(map.width), map);
        if path.success && path.steps.len() > 1 {
            let next_pos = map.idx_to_pos(path.steps[1]);
            match bump_tile(map, &next_pos, pos, view, me) {
                MoveResult::Moved(_) => return,
                _ => {}
            }
        }
    } else {
        let mut new_pos = pos.clone();
        match random::<u8>() % 4 {
            0 => {
                new_pos.0.x += 1;
            }
            1 => {
                new_pos.0.x = max(new_pos.x() - 1, 0);
            }
            2 => {
                new_pos.0.y += 1;
            }
            3 => {
                new_pos.0.y = max(new_pos.y() - 1, 0);
            }
            _ => {}
        }
        bump_tile(map, &new_pos, pos, view, me);
    }
}
