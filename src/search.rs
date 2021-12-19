use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use enum_map::{enum_map, EnumMap};

use crate::{
    card::{explore_cards, monster_cards, splitterland, splitterland_monster, ExploreCard},
    deck::{Deck, DrawnCard},
    game::{PlayerBoard, PlayerTerrain, Season},
    mask::Mask,
    scoring::{monsters, mountain_gold, Board, Score, Scoring},
};

// state that does not change during the game
pub struct InitialState {
    deck: Vec<ExploreCard>,
    monster_deck: Vec<ExploreCard>,
    // all scoring relevant for each season,(four degrees + gold + monsters), with factor
    scoring: EnumMap<Season, [(Scoring, f32); 6]>,
    ruin: Mask,
    mountain: Mask,
    wasteland: Mask,
}

impl InitialState {
    pub fn new(degrees: [Scoring; 4]) -> Self {
        let scoring = enum_map! {
            Season::Spring => [(degrees[0], 2.0), (degrees[1], 2.0), (degrees[2], 2.0), (degrees[3], 2.0), (gold, 4.0), (monsters, 4.0)],
            Season::Summer => [(degrees[0], 1.0), (degrees[1], 1.0), (degrees[2], 2.0), (degrees[3], 2.0), (gold, 3.0), (monsters, 3.0)],
            Season::Fall => [(degrees[0], 1.0), (zero, 0.0), (degrees[2], 1.0), (degrees[3], 2.0), (gold, 2.0), (monsters, 2.0)],
            Season::Winter => [(degrees[0], 1.0), (zero, 0.0), (zero, 0.0), (degrees[3], 1.0), (gold, 1.0), (monsters, 1.0)],
        };

        Self {
            deck: explore_cards(),
            monster_deck: monster_cards(),
            scoring,
            ruin: Mask::from_cells(&[16, 23, 31, 89, 97, 104]),
            mountain: Mask::from_cells(&[14, 30, 60, 90, 106]),
            wasteland: Mask::empty(),
        }
    }
}

#[derive(Clone, Copy)]
// state that potentially changes after a turn
pub struct GameState<'a> {
    initial_state: &'a InitialState,
    season: Season,
    season_timer: u16,
    deck: Deck,
    board: PlayerBoard,
    gold: i16,
    total_score: i16,
}

impl GameState<'_> {
    pub fn new(initial_state: &InitialState) -> GameState<'_> {
        GameState {
            initial_state,
            season: Season::Spring,
            season_timer: 0,
            deck: Deck::empty(),
            board: PlayerBoard::new_with(|_| None),
            gold: 0,
            total_score: 0,
        }
    }

    pub fn new_season(&mut self, season: Season) {
        self.season = season;
        self.season_timer = 0;
        self.deck.new_season();
    }

    pub fn reveal_card(&mut self, card: &str) -> &ExploreCard {
        if let Some(idx) = self
            .initial_state
            .deck
            .iter()
            .position(|c| c.name() == card)
        {
            self.deck.remove_explore_card(idx as u16);
            &self.initial_state.deck[idx]
        } else if let Some(idx) = self
            .initial_state
            .monster_deck
            .iter()
            .position(|c| c.name() == card)
        {
            self.deck.remove_monster_card(idx as u16);
            &self.initial_state.monster_deck[idx]
        } else {
            panic!("card {} was not found in deck", card);
        }
    }

    pub fn new_board(&mut self, board: PlayerBoard) {
        self.board = board;
    }
}

fn gold(state: &GameState) -> Score {
    state.gold + mountain_gold(state)
}

fn zero(_: &GameState) -> Score {
    0
}

#[derive(Debug)]
pub struct Turn {
    pub terrain: PlayerTerrain,
    pub cells: Mask,
}

pub struct Statistics {
    pub positions_evaluated: u32,
    pub depth_reached: u32,
    pub end_reached: bool,
}

pub fn find_best_move(
    state: &GameState,
    card: &str,
    on_ruin: bool,
    duration: Duration,
) -> (Turn, Statistics) {
    let card = state
        .initial_state
        .deck
        .iter()
        .find(|c| c.name() == card)
        .unwrap_or_else(|| {
            state
                .initial_state
                .monster_deck
                .iter()
                .find(|c| c.name() == card)
                .expect("card was not found")
        });

    let mut statistics = Statistics {
        positions_evaluated: 0,
        depth_reached: 0,
        end_reached: false,
    };

    let timeout_reached = Arc::new(AtomicBool::new(false));
    let timeout_reached_clone = timeout_reached.clone();
    let mut best_turn = None;

    thread::spawn(move || {
        thread::sleep(duration);
        timeout_reached_clone.store(true, Ordering::Relaxed);
    });

    for depth in 0.. {
        let (turn, _) = search_explore_move(
            state,
            card,
            on_ruin,
            depth,
            &mut statistics,
            &timeout_reached,
            false,
        );

        if timeout_reached.load(Ordering::Relaxed) {
            break;
        } else {
            best_turn = Some(turn);
            statistics.depth_reached = depth;
        }
    }

    (best_turn.unwrap(), statistics)
}

fn search_explore_move(
    state: &GameState,
    card: &ExploreCard,
    on_ruin: bool,
    depth: u32,
    statistics: &mut Statistics,
    timeout_reached: &AtomicBool,
    tried_rift_land: bool,
) -> (Turn, f32) {
    let is_ambush = card.is_ambush();
    let mut best_score = if is_ambush { f32::MAX } else { f32::MIN };
    let mut best_turn = None;

    let empty = state.empty();
    let ruin = state.ruin();

    // check all possible terrains and patterns at all positions
    for &(pattern, gold) in card.patterns() {
        let state = state.add_gold(gold);
        for cells in empty.sub_masks(pattern) {
            for &terrain in card.terrains() {
                if on_ruin && !is_ambush && (cells & ruin).is_empty() {
                    // invalid turn
                    continue;
                }

                let mut state = state.place_cells(terrain, cells);
                let score = search_game_move(&mut state, depth, statistics, timeout_reached);

                let is_better = if is_ambush {
                    score < best_score
                } else {
                    score > best_score
                };

                if is_better {
                    best_score = score;
                    best_turn = Some(Turn { terrain, cells });
                }
            }
        }
    }

    if best_turn.is_none() && !tried_rift_land {
        // there was no possible turn, try to place a rift land anywhere
        let rift_land = if is_ambush {
            splitterland_monster()
        } else {
            splitterland()
        };
        return search_explore_move(
            state,
            &rift_land,
            false,
            depth,
            statistics,
            timeout_reached,
            true,
        );
    }

    (best_turn.expect("no possible turn found"), best_score)
}

fn search_game_move(
    state: &mut GameState,
    depth: u32,
    statistics: &mut Statistics,
    timeout_reached: &AtomicBool,
) -> f32 {
    if timeout_reached.load(Ordering::Relaxed) {
        return 0.0;
    }

    statistics.positions_evaluated += 1;

    if !state.handle_season_end() {
        // game has ended
        statistics.end_reached = true;
        return state.final_score();
    }

    if depth == 0 {
        return state.heuristic_score();
    }

    let mut weighted_score_sum = 0.0;

    // check every possible next card
    for (card, prob, next_state) in state.draw_cards() {
        let (_, score) = search_explore_move(
            &next_state,
            card,
            false,
            depth - 1,
            statistics,
            timeout_reached,
            false,
        );
        weighted_score_sum += score * prob;
    }

    weighted_score_sum
}

impl GameState<'_> {
    fn add_gold(&self, gold: i16) -> Self {
        Self {
            gold: self.gold + gold,
            ..*self
        }
    }

    fn place_cells(&self, terrain: PlayerTerrain, cells: Mask) -> Self {
        Self {
            board: self.board.place_cells(terrain, cells),
            ..*self
        }
    }

    // returns false only when the game is finished
    fn handle_season_end(&mut self) -> bool {
        if self.season_timer < self.season.time() {
            // season still ongoing
            return true;
        }

        for (scoring, _) in self.initial_state.scoring[self.season] {
            self.total_score += scoring(self);
        }

        if let Some(season) = self.season.next() {
            self.new_season(season);
            true
        } else {
            // game ends after winter
            false
        }
    }

    fn final_score(&self) -> f32 {
        self.total_score as f32
    }

    fn heuristic_score(&self) -> f32 {
        self.total_score as f32
            + self.initial_state.scoring[self.season]
                .iter()
                .map(|(scoring, factor)| scoring(self) as f32 * factor)
                .sum::<f32>()
    }

    fn draw_cards(&self) -> impl Iterator<Item = (&ExploreCard, f32, GameState<'_>)> {
        self.deck.draw_cards().map(|(c, prob, deck)| {
            let card = match c {
                DrawnCard::ExploreCard(idx) => &self.initial_state.deck[idx as usize],
                DrawnCard::MonsterCard(idx) => &self.initial_state.monster_deck[idx as usize],
            };

            let mut new_state = *self;
            new_state.deck = deck;
            new_state.season_timer += card.time();

            (card, prob, new_state)
        })
    }
}

impl Board for GameState<'_> {
    fn filled(&self) -> Mask {
        // everything except ruins is considered filled
        self.forest()
            | self.village()
            | self.farm()
            | self.water()
            | self.monster()
            | self.mountain()
            | self.wasteland()
    }

    fn forest(&self) -> Mask {
        self.board.get_cells(PlayerTerrain::Forest)
    }

    fn village(&self) -> Mask {
        self.board.get_cells(PlayerTerrain::Village)
    }

    fn farm(&self) -> Mask {
        self.board.get_cells(PlayerTerrain::Farm)
    }

    fn water(&self) -> Mask {
        self.board.get_cells(PlayerTerrain::Water)
    }

    fn monster(&self) -> Mask {
        self.board.get_cells(PlayerTerrain::Monster)
    }

    fn mountain(&self) -> Mask {
        self.initial_state.mountain
    }

    fn wasteland(&self) -> Mask {
        self.initial_state.wasteland
    }

    fn ruin(&self) -> Mask {
        self.initial_state.ruin
    }
}
