use std::{
    fmt::Debug,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not},
};

const FULL: u128 = (1 << Mask::CELL_COUNT) - 1;

const fn row(i: u8) -> u128 {
    assert!(i < Mask::SIZE);

    let row = (1 << Mask::SIZE) - 1;
    row << (i * Mask::SIZE)
}

const fn column(i: u8) -> u128 {
    assert!(i < Mask::SIZE);

    let mut column = 0;
    let mut steps = 0;
    while steps < Mask::SIZE {
        column = column << Mask::SIZE | 1;
        steps += 1;
    }

    column << i
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
pub struct Mask(u128);

impl Mask {
    pub const SIZE: u8 = 11;
    pub const CELL_COUNT: u8 = Self::SIZE * Self::SIZE;

    pub const fn to_bits(self) -> u128 {
        self.0
    }

    pub const fn from_bits(bits: u128) -> Self {
        Self(bits)
    }

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn full() -> Self {
        Self(FULL)
    }

    pub const fn cell_idx(idx: u8) -> Self {
        assert!(idx < Self::CELL_COUNT);
        Self(1 << idx)
    }

    pub const fn cell(x: u8, y: u8) -> Self {
        assert!(x < Self::SIZE);
        assert!(y < Self::SIZE);

        Self((1 << x) << (Self::SIZE * y))
    }

    pub const fn row(i: u8) -> Self {
        Self(row(i))
    }

    pub const fn column(i: u8) -> Self {
        Self(column(i))
    }

    pub const fn border() -> Self {
        Self(row(0) | row(Self::SIZE - 1) | column(0) | column(Self::SIZE - 1))
    }

    pub const fn from_cells(cells: &[u8]) -> Self {
        assert!(cells.len() < Self::CELL_COUNT as usize);
        
        let mut res = Self::empty();
        let mut idx = 0;
        while idx < cells.len() {
            res.0 |= Self::cell_idx(idx as u8).0;
            idx += 1;
        }

        res
    }

    pub const fn shift_up(self) -> Self {
        Self(self.0 >> Self::SIZE)
    }

    pub const fn shift_down(self) -> Self {
        Self((self.0 << Self::SIZE) & FULL)
    }

    pub const fn shift_right(self) -> Self {
        Self((self.0 & !column(Self::SIZE - 1)) << 1)
    }

    pub const fn shift_left(self) -> Self {
        Self((self.0 & !column(0)) >> 1)
    }

    pub fn neighbors(self) -> Self {
        self.shift_left() | self.shift_right() | self.shift_up() | self.shift_down()
    }

    pub const fn clusters(self) -> Clusters {
        Clusters(self.0)
    }

    pub const fn cells(self) -> Cells {
        Cells(self.0)
    }

    pub fn sub_masks(self, pattern: Self) -> SubMasks {
        SubMasks::new(self, pattern)
    }

    pub fn touches(self, other: Self) -> Self {
        self & other.neighbors()
    }

    pub fn touches_not(self, other: Self) -> Self {
        self & !other.neighbors()
    }

    pub fn contains(self, other: Self) -> bool {
        self & other == other
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn has_cells(self) -> bool {
        !self.is_empty()
    }

    pub const fn count_cells(self) -> i32 {
        self.0.count_ones() as i32
    }
}

impl BitAnd for Mask {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Mask {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for Mask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Mask {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Not for Mask {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0 & FULL)
    }
}

impl Debug for Mask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;

        for y in 0..Self::SIZE {
            for x in 0..Self::SIZE {
                let s = if self.contains(Mask::cell(x, y)) { "o" } else { "." };
                write!(f, "{}", s)?;
            }
            
            writeln!(f)?;
        }

        Ok(())
    }
}

pub struct Clusters(u128);

impl Iterator for Clusters {
    type Item = Mask;

    fn next(&mut self) -> Option<Self::Item> {
        if Mask(self.0).is_empty() {
            return None;
        }

        // start the region with any cell, x & (-x) isolates the lowest set bit
        let mut cluster = self.0 & self.0.wrapping_neg();
        loop {
            let next = cluster | (Mask(cluster).neighbors().0 & self.0);
            if next == cluster {
                // there are no additional neighbors also belonging to same the cluster
                self.0 ^= cluster;
                return Some(Mask(cluster));
            }

            cluster = next;
        }
    }
}

pub struct Cells(u128);

impl Iterator for Cells {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }

        let idx = self.0.trailing_zeros() as u8;
        self.0 ^= 1 << idx;

        Some(idx)
    }
}

pub struct SubMasks {
    mask: Mask,
    next_pattern: Mask,
    next_line: Mask,
}

impl SubMasks {
    fn new(mask: Mask, pattern: Mask) -> Self {
        let next_line = if (pattern & Mask::row(Mask::SIZE - 1)).is_empty() {
            pattern.shift_down()
        } else {
            Mask::empty()
        };

        Self {
            mask,
            next_pattern: pattern,
            next_line
        }
    }

    fn shift_pattern(&mut self) {
        if (self.next_pattern & Mask::column(Mask::SIZE - 1)).is_empty() {
            self.next_pattern = self.next_pattern.shift_right()
        } else {
            *self = Self::new(self.mask, self.next_line);
        }
    }
}

impl Iterator for SubMasks {
    type Item = Mask;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.next_pattern.is_empty() {
                return None;
            }

            if self.mask.contains(self.next_pattern) {
                return Some(self.next_pattern);
            }

            self.shift_pattern();
        }
    }
}
