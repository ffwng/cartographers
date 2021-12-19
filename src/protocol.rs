use std::fmt::Debug;

use serde_json::Value;

use crate::{
    game::{PlayerBoard, PlayerTerrain, Season},
    scoring::{self, Scoring},
};

pub enum Message {
    NewDegrees([Scoring; 4]),
    NewSeason(Season),
    NewTurn {
        player_id: String,
        board: PlayerBoard,
        drawn_cards: Vec<String>,
    },
    FinalScoring(Value),
}

impl Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NewDegrees(_) => f.debug_tuple("NewDegrees").finish(),
            Self::NewSeason(season) => f.debug_tuple("NewSeason").field(season).finish(),
            Self::NewTurn {
                player_id,
                board,
                drawn_cards,
            } => f
                .debug_struct("NewTurn")
                .field("player_id", player_id)
                .field("board", board)
                .field("drawn_cards", drawn_cards)
                .finish(),
            Self::FinalScoring(value) => f.debug_tuple("FinalScoring").field(value).finish(),
        }
    }
}

impl Message {
    pub fn parse(event: &str, data: &Value) -> Option<Message> {
        match event {
            "newDegrees" => {
                let cards = data.as_array().expect("expected an array of degrees");
                Some(Self::NewDegrees([
                    Self::parse_degree(&cards[0]),
                    Self::parse_degree(&cards[1]),
                    Self::parse_degree(&cards[2]),
                    Self::parse_degree(&cards[3]),
                ]))
            }
            "newSeason" => Some(Self::NewSeason(Self::parse_season(data))),
            "newTurn" => {
                let data = data.as_object().expect("expected a turn object");

                let player_id = data["playerId"]
                    .as_str()
                    .expect("expected a turn player id")
                    .to_string();

                let board = Self::parse_board(&data["fields"]);

                let mut drawn_cards: Vec<_> = data["usedCards"]
                    .as_array()
                    .expect("expected an array of cards")
                    .iter()
                    .map(Self::parse_card)
                    .collect();
                drawn_cards.push(Self::parse_card(&data["exploreCard"]));

                Some(Self::NewTurn {
                    player_id,
                    board,
                    drawn_cards,
                })
            }
            "receivedTurn" => {
                assert!(
                    data.as_bool().expect("expected a turn validity boolean"),
                    "turn was not valid"
                );
                None
            }
            "finalScoring" => Some(Self::FinalScoring(data.clone())),
            "playerJoinsOrLeaves" | "scoring" => None,
            _ => panic!("unexpected game event {}", event),
        }
    }

    fn parse_degree(value: &Value) -> Scoring {
        match value.as_object().expect("expected a degree object")["card"]
            .as_str()
            .expect("expected a card name")
        {
            "wald1" => scoring::stoneside_forest,
            "wald2" => scoring::sentinel_wood,
            "wald3" => scoring::treetower,
            "wald4" => scoring::greenbough,
            "wasser1" => scoring::mage_valley,
            "wasser2" => scoring::canal_lake,
            "wasser3" => scoring::shoreside_expanse,
            "wasser4" => scoring::the_golden_granary,
            "dorf1" => scoring::greengold_plains,
            "dorf2" => scoring::shieldgate,
            "dorf3" => scoring::wildholds,
            "dorf4" => scoring::great_city,
            "distanz1" => scoring::borderlands,
            "distanz2" => scoring::the_cauldrons,
            "distanz3" => scoring::the_broken_road,
            "distanz4" => scoring::lost_barony,
            name => panic!("unknown degree {}", name),
        }
    }

    fn parse_season(value: &Value) -> Season {
        match value.as_object().expect("expected a season object")["name"]
            .as_str()
            .expect("expected a season name")
        {
            "spring" => Season::Spring,
            "sommer" => Season::Summer,
            "autmn" => Season::Fall,
            "winter" => Season::Winter,
            name => panic!("unknown season {}", name),
        }
    }

    fn parse_board(value: &Value) -> PlayerBoard {
        let cells = value.as_array().expect("expected an array of cells");
        PlayerBoard::new_with(|i| {
            match cells[i as usize]
                .as_object()
                .expect("expected a cell object")["landscape"]
                .as_str()
                .expect("expected a terrain name")
            {
                "FOREST" => Some(PlayerTerrain::Forest),
                "VILLAGE" => Some(PlayerTerrain::Village),
                "FARM" => Some(PlayerTerrain::Farm),
                "WATER" => Some(PlayerTerrain::Water),
                "MONSTER" => Some(PlayerTerrain::Monster),
                "MOUNTAIN" | "WASTELAND" | "Ruin" | "EMPTY" => None,
                name => panic!("unknown tarrain {}", name),
            }
        })
    }

    fn parse_card(value: &Value) -> String {
        value.as_object().expect("expected a card object")["name"]
            .as_str()
            .expect("expected a card name")
            .into()
    }
}
