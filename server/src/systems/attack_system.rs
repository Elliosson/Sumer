extern crate specs;
use std::collections::{HashSet, VecDeque};

use crate::{GamePhase, GamePhaseEnum, Gold, Map, OnGoingAttack, Player, Position, WantToAttack};
use specs::prelude::*;

use super::ongoing_attack_system::adjacent_positions;

pub struct AttackSystem {}

impl<'a> System<'a> for AttackSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantToAttack>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Gold>,
        WriteStorage<'a, OnGoingAttack>,
        WriteStorage<'a, GamePhase>,
        WriteExpect<'a, Map>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut want_to_attacks,
            mut players,
            mut golds,
            mut ongoing_attacks,
            mut game_phases,
            map,
        ) = data;

        for (_entity, player, gold, want_to_attack, game_phase) in (
            &entities,
            &mut players,
            &mut golds,
            &mut want_to_attacks,
            &mut game_phases,
        )
            .join()
        {
            match game_phase.phase {
                GamePhaseEnum::Playing => {
                    match get_closest_border(&want_to_attack.pos, &player.name, &map) {
                        Ok(border_pos) => {
                            entities
                                .build_entity()
                                .with(
                                    OnGoingAttack {
                                        gold: gold.quantity,
                                        last_turn_conquest: vec![border_pos],
                                        owner: player.name.clone(),
                                        enemy: None,
                                    },
                                    &mut ongoing_attacks,
                                )
                                .build();
                            gold.quantity = 0.;
                        }
                        Err(_) => {}
                    }
                }
                GamePhaseEnum::LocationSelection => {
                    println!("attack {:?}", want_to_attack.pos);
                    entities
                        .build_entity()
                        .with(
                            OnGoingAttack {
                                gold: gold.quantity,
                                last_turn_conquest: vec![want_to_attack.pos.clone()],
                                owner: player.name.clone(),
                                enemy: None,
                            },
                            &mut ongoing_attacks,
                        )
                        .build();
                    gold.quantity = 0.;
                    game_phase.phase = GamePhaseEnum::Playing;
                }
                GamePhaseEnum::GameOver => {}
            }
        }

        want_to_attacks.clear();
    }
}

//Returns the position of the closest territory of player <username> from pos.
pub fn get_closest_border(
    pos: &Position,
    username: &String,
    map: &Map,
) -> Result<Position, &'static str> {
    let mut visited: HashSet<Position> = HashSet::new();
    let mut queue = VecDeque::from([pos.clone()]);

    while queue.len() > 0 {
        let current_pos = queue.pop_front().unwrap();
        if visited.contains(&current_pos) {
            continue;
        }
        visited.insert(current_pos);
        let tile = map.get_tile(&current_pos);
        if tile.owner == *username {
            return Ok(current_pos.clone());
        }

        //this can causes horible bug because adjacent_position is having not ok behavior
        //I need to move adjacent_position into the map and making him take into account the map borders
        for new_pos in adjacent_positions(&current_pos) {
            if !queue.contains(&new_pos) {
                queue.push_back(new_pos);
            }
        }
    }

    //ToDo add an err
    return Err("Failed to find a tile owned by this player");
}
