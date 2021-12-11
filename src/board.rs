use crate::mask::Mask;

pub struct PlayerBoard {
    forest: Mask,
    village: Mask,
    farm: Mask,
    water: Mask,
    monster: Mask,
}

impl PlayerBoard {
    pub fn new() -> Self {
        Self {
            forest: Mask::empty(),
            village: Mask::empty(),
            farm: Mask::empty(),
            water: Mask::empty(),
            monster: Mask::empty(),
        }
    }
}

pub struct GameBoard {
    mountain: Mask,
    wasteland: Mask,
    ruin: Mask,
}

impl GameBoard {
    pub fn side1() -> Self {
        Self {
            mountain: Mask::cell(3, 1)
                | Mask::cell(8, 2)
                | Mask::cell(5, 5)
                | Mask::cell(2, 8)
                | Mask::cell(7, 9),

            wasteland: Mask::empty(),

            ruin: Mask::cell(1, 2)
                | Mask::cell(5, 1)
                | Mask::cell(9, 2)
                | Mask::cell(1, 8)
                | Mask::cell(5, 9)
                | Mask::cell(9, 8),
        }
    }

    pub fn side2() -> Self {
        Self {
            mountain: Mask::cell(3, 2)
                | Mask::cell(8, 1)
                | Mask::cell(5, 7)
                | Mask::cell(2, 9)
                | Mask::cell(9, 8),

            wasteland: Mask::cell(5, 3)
                | Mask::cell(4, 4)
                | Mask::cell(5, 4)
                | Mask::cell(4, 5)
                | Mask::cell(5, 5)
                | Mask::cell(6, 5)
                | Mask::cell(5, 6),

            ruin: Mask::cell(2, 2)
                | Mask::cell(6, 1)
                | Mask::cell(6, 4)
                | Mask::cell(1, 6)
                | Mask::cell(8, 7)
                | Mask::cell(3, 9),
        }
    }
}

pub struct CombinedBoard<'a> {
    player: &'a PlayerBoard,
    game: &'a GameBoard,
    filled: Mask,
}

impl CombinedBoard<'_> {
    pub fn new<'a>(player: &'a PlayerBoard, game: &'a GameBoard) -> CombinedBoard<'a> {
        // everything except ruins is considered filled
        let filled = player.forest
            | player.village
            | player.farm
            | player.water
            | player.monster
            | game.mountain
            | game.wasteland;

        CombinedBoard {
            player,
            game,
            filled,
        }
    }

    pub fn filled(&self) -> Mask {
        self.filled
    }

    pub fn empty(&self) -> Mask {
        !self.filled()
    }

    pub fn forest(&self) -> Mask {
        self.player.forest
    }

    pub fn village(&self) -> Mask {
        self.player.village
    }

    pub fn farm(&self) -> Mask {
        self.player.farm
    }

    pub fn water(&self) -> Mask {
        self.player.water
    }

    pub fn monster(&self) -> Mask {
        self.player.monster
    }

    pub fn mountain(&self) -> Mask {
        self.game.mountain
    }

    pub fn wasteland(&self) -> Mask {
        self.game.wasteland
    }

    pub fn ruin(&self) -> Mask {
        self.game.ruin
    }
}
