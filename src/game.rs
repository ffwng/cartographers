use enum_map::{enum_map, Enum, EnumMap};

use crate::mask::Mask;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
}

impl Season {
    pub fn time(&self) -> u32 {
        match self {
            Self::Spring | Self::Summer => 8,
            Self::Fall => 7,
            Self::Winter => 6,
        }
    }

    pub fn next(&self) -> Option<Season> {
        match self {
            Self::Spring => Some(Self::Summer),
            Self::Summer => Some(Self::Fall),
            Self::Fall => Some(Self::Winter),
            Self::Winter => None,
        }
    }
}

#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayerTerrain {
    Forest,
    Village,
    Farm,
    Water,
    Monster,
}

#[derive(Debug)]
pub struct PlayerBoard(EnumMap<PlayerTerrain, Mask>);

impl PlayerBoard {
    pub fn new_with(mut f: impl FnMut(u8) -> Option<PlayerTerrain>) -> Self {
        let mut result = Self(enum_map! {
            _ => Mask::empty()
        });

        for idx in 0..Mask::CELL_COUNT {
            if let Some(t) = f(idx) {
                result.0[t] |= Mask::cell_idx(idx);
            }
        }

        result
    }
}
