use crate::{game::PlayerTerrain, mask::Mask};

use PlayerTerrain::*;

pub fn explore_cards() -> Vec<ExploreCard> {
    vec![
        ackerland(),
        baumwipfeldorf(),
        fischerdorf(),
        gehöft(),
        großer_strom(),
        hinterlandbach(),
        obsthain(),
        splitterland(),
        sumpf(),
        vergessener_wald(),
        weiler(),
    ]
}

pub fn monster_cards() -> Vec<ExploreCard> {
    vec![
        gnollangriff(),
        goblinattacke(),
        grottenschratüberfall(),
        insektoideninvasion(),
        koboldansturm(),
        ogeroffensive(),
        rattenmenschenrache(),
        schindersturm(),
    ]
}

pub struct ExploreCard {
    name: String,
    time: u16,
    terrains: Vec<PlayerTerrain>,
    patterns: Vec<(Mask, i16)>,
    is_ambush: bool,
}

impl ExploreCard {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn time(&self) -> u16 {
        self.time
    }

    pub fn terrains(&self) -> &[PlayerTerrain] {
        &self.terrains
    }

    pub fn patterns(&self) -> &[(Mask, i16)] {
        &self.patterns
    }

    pub fn is_ambush(&self) -> bool {
        self.is_ambush
    }
}

// Note: english card names are not available at the moment

pub fn ackerland() -> ExploreCard {
    ExploreCardBuilder::new("ackerland", 1, &[Farm])
        .with_pattern(&[b"xx"], 1)
        .with_pattern(&[b" x ", b"xxx", b" x "], 0)
        .build()
}

pub fn baumwipfeldorf() -> ExploreCard {
    ExploreCardBuilder::new("baumwipfeldorf", 2, &[Forest, Village])
        .with_pattern(&[b"  xx", b"xxx "], 0)
        .build()
}

pub fn fischerdorf() -> ExploreCard {
    ExploreCardBuilder::new("fischerdorf", 2, &[Village, Water])
        .with_pattern(&[b"xxxx"], 0)
        .build()
}

pub fn gehöft() -> ExploreCard {
    ExploreCardBuilder::new("gehoeft", 2, &[Village, Farm])
        .with_pattern(&[b"x ", b"xx", b"x "], 0)
        .build()
}

pub fn gnollangriff() -> ExploreCard {
    ExploreCardBuilder::new("gnollangriff", 0, &[Monster])
        .with_pattern(&[b"xx", b"x ", b"xx"], 0)
        .build()
}

pub fn goblinattacke() -> ExploreCard {
    ExploreCardBuilder::new("goblinattacke", 0, &[Monster])
        .with_pattern(&[b"x  ", b" x ", b"  x"], 0)
        .build()
}

pub fn großer_strom() -> ExploreCard {
    ExploreCardBuilder::new("grosserStrom", 1, &[Water])
        .with_pattern(&[b"xxx"], 1)
        .with_pattern(&[b"  x", b" xx", b"xx "], 0)
        .build()
}

pub fn grottenschratüberfall() -> ExploreCard {
    ExploreCardBuilder::new("grottenschratueberfall", 0, &[Monster])
        .with_pattern(&[b"x x", b"x x"], 0)
        .build()
}

pub fn hinterlandbach() -> ExploreCard {
    ExploreCardBuilder::new("hinterlandbach", 2, &[Farm, Water])
        .with_pattern(&[b"xxx", b"x  ", b"x  "], 0)
        .build()
}

pub fn insektoideninvasion() -> ExploreCard {
    ExploreCardBuilder::new("insektoideninvasion", 0, &[Monster])
        .with_pattern(&[b" x", b"xx", b"x "], 0)
        .build()
}

pub fn koboldansturm() -> ExploreCard {
    ExploreCardBuilder::new("koboldansturm", 0, &[Monster])
        .with_pattern(&[b"x ", b"xx", b"x "], 0)
        .build()
}

pub fn obsthain() -> ExploreCard {
    ExploreCardBuilder::new("obsthain", 2, &[Forest, Farm])
        .with_pattern(&[b"xxx", b"  x"], 0)
        .build()
}

pub fn ogeroffensive() -> ExploreCard {
    ExploreCardBuilder::new("ogeroffensive", 0, &[Monster])
        .with_pattern(&[b"xx", b"xx"], 0)
        .build()
}

pub fn rattenmenschenrache() -> ExploreCard {
    ExploreCardBuilder::new("rattenmenschenrache", 0, &[Monster])
        .with_pattern(&[b"xxx"], 0)
        .build()
}

pub fn schindersturm() -> ExploreCard {
    ExploreCardBuilder::new("schindersturm", 0, &[Monster])
        .with_pattern(&[b"x ", b"xx"], 0)
        .build()
}

pub fn splitterland() -> ExploreCard {
    ExploreCardBuilder::new("splitterland", 0, &[Forest, Village, Farm, Water, Monster])
        .with_pattern(&[b"x"], 0)
        .build()
}

pub fn splitterland_monster() -> ExploreCard {
    ExploreCardBuilder::new("splitterland_monster", 0, &[Monster])
        .with_pattern(&[b"x"], 0)
        .build()
}

pub fn sumpf() -> ExploreCard {
    ExploreCardBuilder::new("sumpf", 2, &[Forest, Water])
        .with_pattern(&[b"x  ", b"xxx", b"x  "], 0)
        .build()
}

pub fn vergessener_wald() -> ExploreCard {
    ExploreCardBuilder::new("vergessenerWald", 1, &[Forest])
        .with_pattern(&[b"x ", b" x"], 1)
        .with_pattern(&[b"x ", b"xx", b" x"], 0)
        .build()
}

pub fn weiler() -> ExploreCard {
    ExploreCardBuilder::new("weiler", 1, &[Village])
        .with_pattern(&[b"x ", b"xx"], 1)
        .with_pattern(&[b"xxx", b"xx "], 0)
        .build()
}

pub struct ExploreCardBuilder(ExploreCard);

impl ExploreCardBuilder {
    fn new(name: impl Into<String>, time: u16, terrains: &[PlayerTerrain]) -> Self {
        assert!(!terrains.is_empty(), "there must be at least one terrain");
        Self(ExploreCard {
            name: name.into(),
            time,
            terrains: terrains.to_vec(),
            patterns: Vec::new(),
            is_ambush: terrains[0] == PlayerTerrain::Monster,
        })
    }

    fn with_pattern(mut self, pattern: &[&[u8]], gold: i16) -> Self {
        let p1 = Pattern::new(pattern);
        let p2 = p1.rotate90();
        let p3 = p2.rotate90();
        let p4 = p3.rotate90();
        let p5 = p1.mirror();
        let p6 = p5.rotate90();
        let p7 = p6.rotate90();
        let p8 = p7.rotate90();

        let mut masks = vec![
            p1.to_mask(),
            p2.to_mask(),
            p3.to_mask(),
            p4.to_mask(),
            p5.to_mask(),
            p6.to_mask(),
            p7.to_mask(),
            p8.to_mask(),
        ];

        masks.sort_unstable();
        masks.dedup();

        self.0.patterns.extend(masks.into_iter().map(|m| (m, gold)));

        self
    }

    fn build(self) -> ExploreCard {
        assert!(
            !self.0.patterns.is_empty(),
            "there must be at least one pattern"
        );
        self.0
    }
}

struct Pattern(Vec<Vec<bool>>);

impl Pattern {
    fn new(pattern: &[&[u8]]) -> Self {
        Self(
            pattern
                .iter()
                .map(|row| row.iter().map(|&b| b == b'x').collect())
                .collect(),
        )
    }

    fn rotate90(&self) -> Pattern {
        Self(
            (0..self.0[0].len())
                .map(|i| self.0.iter().rev().map(|row| row[i]).collect())
                .collect(),
        )
    }

    fn mirror(&self) -> Pattern {
        Self(
            self.0
                .iter()
                .map(|row| row.iter().rev().copied().collect())
                .collect(),
        )
    }

    fn to_mask(&self) -> Mask {
        let mut m = Mask::empty();
        for (y, row) in self.0.iter().enumerate() {
            for (x, &b) in row.iter().enumerate() {
                if b {
                    m |= Mask::cell(x.try_into().unwrap(), y.try_into().unwrap())
                }
            }
        }

        m
    }
}
