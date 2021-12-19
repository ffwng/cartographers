use crate::{mask::Mask, search::GameState};

pub trait Board {
    fn filled(&self) -> Mask;

    fn empty(&self) -> Mask {
        !self.filled()
    }

    fn forest(&self) -> Mask;

    fn village(&self) -> Mask;

    fn farm(&self) -> Mask;

    fn water(&self) -> Mask;

    fn monster(&self) -> Mask;

    fn mountain(&self) -> Mask;

    fn wasteland(&self) -> Mask;

    fn ruin(&self) -> Mask;
}

pub type Score = i16;

pub type Scoring = fn(b: &GameState) -> Score;

pub fn mountain_gold(b: &GameState) -> Score {
    b.mountain().touches_not(b.empty()).count_cells()
}

pub fn monsters(b: &GameState) -> Score {
    -b.empty().touches(b.monster()).count_cells()
}

pub fn sentinel_wood(b: &GameState) -> Score {
    (b.forest() & Mask::border()).count_cells()
}

pub fn treetower(b: &GameState) -> Score {
    b.forest().touches_not(b.empty()).count_cells()
}

pub fn greenbough(b: &GameState) -> Score {
    let mut score = 0;

    for i in 0..Mask::SIZE {
        if (b.forest() & Mask::column(i)).has_cells() {
            score += 1;
        }

        if (b.forest() & Mask::row(i)).has_cells() {
            score += 1;
        }
    }

    score
}

pub fn stoneside_forest(b: &GameState) -> Score {
    let mut found = Mask::empty();

    for forest in b.forest().clusters() {
        let neighbor_mountains = b.mountain().touches(forest);
        if neighbor_mountains.count_cells() > 1 {
            found |= neighbor_mountains;
        }
    }

    found.count_cells() * 3
}

pub fn canal_lake(b: &GameState) -> Score {
    b.water().touches(b.farm()).count_cells() + b.farm().touches(b.water()).count_cells()
}

pub fn the_golden_granary(b: &GameState) -> Score {
    b.water().touches(b.ruin()).count_cells() + (b.farm() & b.ruin()).count_cells() * 3
}

pub fn mage_valley(b: &GameState) -> Score {
    b.water().touches(b.mountain()).count_cells() * 2 + b.farm().touches(b.mountain()).count_cells()
}

pub fn shoreside_expanse(b: &GameState) -> Score {
    let mut score = 0;

    let m = b.water().neighbors() | Mask::border();
    for farm in b.farm().clusters() {
        if (farm & m).is_empty() {
            score += 3;
        }
    }

    let m = b.farm().neighbors() | Mask::border();
    for water in b.water().clusters() {
        if (water & m).is_empty() {
            score += 3;
        }
    }

    score
}

pub fn wildholds(b: &GameState) -> Score {
    b.village()
        .clusters()
        .filter(|region| region.count_cells() >= 6)
        .count() as Score
        * 8
}

pub fn greengold_plains(b: &GameState) -> Score {
    let neighbors = [
        b.forest().neighbors(),
        b.farm().neighbors(),
        b.water().neighbors(),
        b.monster().neighbors(),
        b.mountain().neighbors(),
    ];
    let mut score = 0;

    for village in b.village().clusters() {
        let count = neighbors
            .iter()
            .filter(|&&n| (village & n).has_cells())
            .count();
        if count >= 3 {
            score += 3;
        }
    }

    score
}

pub fn great_city(b: &GameState) -> Score {
    let m = b.mountain().neighbors();
    b.village()
        .clusters()
        .filter(|&cluster| (cluster & m).is_empty())
        .map(|cluster| cluster.count_cells())
        .max()
        .unwrap_or(0)
}

pub fn shieldgate(b: &GameState) -> Score {
    let mut max1 = 0;
    let mut max2 = 0;

    for village in b.village().clusters() {
        let size = village.count_cells();
        if size > max1 {
            max2 = max1;
            max1 = size;
        } else if size > max2 {
            max2 = size;
        }
    }

    max2
}

pub fn borderlands(b: &GameState) -> Score {
    let filled = b.filled();
    let mut score = 0;

    for i in 0..Mask::SIZE {
        if filled.contains(Mask::column(i)) {
            score += 6;
        }

        if filled.contains(Mask::row(i)) {
            score += 6;
        }
    }

    score
}

pub fn the_broken_road(b: &GameState) -> Score {
    let filled = b.filled();
    let mut diagonal = Mask::empty();
    let mut cell = Mask::cell(0, Mask::SIZE - 1);
    let mut score = 0;

    for _ in 0..Mask::SIZE {
        diagonal |= cell;
        if filled.contains(diagonal) {
            score += 3;
        }

        diagonal = diagonal.shift_up();
        cell = cell.shift_right();
    }

    score
}

pub fn lost_barony(b: &GameState) -> Score {
    let filled = b.filled();
    // try every square from SIZExSIZE to 2x2
    let mut square = Mask::full();

    for size in (2..=Mask::SIZE).rev() {
        if filled.sub_masks(square).next().is_some() {
            return size as Score * 3;
        }

        // try next smaller square
        square = square.shift_left().shift_up();
    }

    // on most boards there is always at least one non-empty field (e.g. a mountain), but check to be sure
    if filled.is_empty() {
        0
    } else {
        3
    }
}

pub fn the_cauldrons(b: &GameState) -> Score {
    b.empty().touches_not(b.empty()).count_cells()
}
