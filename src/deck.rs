// a representation of all cards left in the deck
#[derive(Clone, Copy)]
pub struct Deck {
    explore_cards_mask: u16,
    monster_cards_mask: u16,
    monsters_in_deck: u8,
}

impl Deck {
    const EXPLORE_COUNT: u16 = 11;
    const MONSTER_COUNT: u16 = 8;

    pub fn empty() -> Self {
        Self {
            explore_cards_mask: 0,
            monster_cards_mask: 0,
            monsters_in_deck: 0,
        }
    }

    pub fn remove_explore_card(&mut self, idx: u16) {
        self.explore_cards_mask ^= 1 << idx;
    }

    pub fn remove_monster_card(&mut self, idx: u16) {
        self.monster_cards_mask ^= 1 << idx;
        self.monsters_in_deck -= 1;
    }

    pub fn new_season(&mut self) {
        // shuffle in all explore card and ruins and one additional monster card
        *self = Self {
            explore_cards_mask: (1 << Self::EXPLORE_COUNT) - 1,
            monster_cards_mask: (1 << Self::MONSTER_COUNT) - 1,
            monsters_in_deck: self.monsters_in_deck + 1,
        }
    }

    pub fn draw_cards(&self) -> impl Iterator<Item = (DrawnCard, f32, Self)> + '_ {
        let e = self.explore_cards_mask.count_ones() as f32;
        let m = self.monsters_in_deck as f32;
        let total = e + m;

        let explore_prob = 1.0 / total;

        let explore_iter = MaskIterator(self.explore_cards_mask).map(move |pos| {
            let mut new_deck = *self;
            new_deck.remove_explore_card(pos);

            (DrawnCard::ExploreCard(pos), explore_prob, new_deck)
        });

        let total_monsters = self.monster_cards_mask.count_ones() as f32;
        let monster_prob = m / (total_monsters * total);

        let monster_mask = if self.monsters_in_deck > 0 {
            self.monster_cards_mask
        } else {
            0
        };
        let monster_iter = MaskIterator(monster_mask).map(move |pos| {
            let mut new_deck = *self;
            new_deck.remove_monster_card(pos);

            (DrawnCard::MonsterCard(pos), m / monster_prob, new_deck)
        });

        explore_iter.chain(monster_iter)
    }
}

pub enum DrawnCard {
    ExploreCard(u16),
    MonsterCard(u16),
}

struct MaskIterator(u16);

impl Iterator for MaskIterator {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }

        let pos = self.0.trailing_zeros() as u16;
        self.0 ^= 1 << pos;

        Some(pos)
    }
}
