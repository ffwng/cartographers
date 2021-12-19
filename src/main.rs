use std::{collections::HashMap, time::Duration};

use search::{find_best_move, GameState, InitialState};
use socketio::SocketIOExt;

use crate::{game::PlayerTerrain, protocol::Message};

mod card;
mod deck;
mod game;
mod mask;
mod protocol;
mod scoring;
mod search;
mod socketio;

fn main() {
    let mut socket = socketio::connect("ws://localhost:3000/socket.io/?EIO=4&transport=websocket")
        .expect("cannot connect");

    socket
        .write_event("enterGame", "Bot")
        .expect("failed to send message");

    socket
        .write_event("startGame", "")
        .expect("failed to send message");

    let initial_state = loop {
        let (event, data) = socket.read_event().expect("failed to read message");
        if let Some(data) = data {
            if let Some(Message::NewDegrees(degrees)) = Message::parse(&event, &data) {
                break InitialState::new(degrees);
            }
        }
    };

    let mut game_state = GameState::new(&initial_state);
    let mut card_counter = 0;

    loop {
        let (event, data) = socket.read_event().expect("failed to read message");
        if let Some(data) = data {
            let msg = Message::parse(&event, &data);
            match msg {
                Some(Message::NewSeason(season)) => {
                    game_state.new_season(season);
                    card_counter = 0;
                }
                Some(Message::NewTurn {
                    player_id,
                    board,
                    drawn_cards,
                }) => {
                    let mut on_ruin = false;
                    let mut is_ambush = false;
                    for c in &drawn_cards[card_counter..] {
                        println!("Got card {}", c);
                        if c == "tempelruinen" || c == "verfallenerAussenposten" {
                            on_ruin = true;
                        } else {
                            let card = game_state.reveal_card(c);
                            is_ambush = is_ambush || card.is_ambush();
                        }
                    }

                    if !is_ambush {
                        card_counter = drawn_cards.len();
                    }

                    game_state.new_board(board);

                    let (turn, statistics) = find_best_move(
                        &game_state,
                        drawn_cards.last().unwrap(),
                        on_ruin,
                        Duration::from_secs(2),
                    );
                    println!(
                        "Positions evaluated: {}, depth reached: {}",
                        statistics.positions_evaluated, statistics.depth_reached
                    );
                    println!("{:?}", turn);

                    let terrain_name = match turn.terrain {
                        PlayerTerrain::Forest => "FOREST",
                        PlayerTerrain::Village => "VILLAGE",
                        PlayerTerrain::Farm => "FARM",
                        PlayerTerrain::Water => "WATER",
                        PlayerTerrain::Monster => "MONSTER",
                    };

                    let mut fields = HashMap::new();
                    for pos in turn.cells.cells() {
                        fields.insert(pos.to_string(), terrain_name.to_string());
                    }

                    socket
                        .write_json_event(
                            "finishTurn",
                            &serde_json::json!({ "playerId": player_id, "fields": fields }),
                        )
                        .expect("failed to send turn");
                }
                Some(Message::FinalScoring(value)) => {
                    println!("Final scores: {}", value);
                    break;
                }
                _ => {}
            }
        }
    }
}
