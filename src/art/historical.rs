//! Generated period artwork. Slots are normalized at build-authoring time, never at runtime.
use super::*;

const SIZE: usize = 192;
const CELL: usize = SIZE * SIZE * 4;
const ATLASES: [&[u8]; 10] = [
    include_bytes!("../../assets/period/steppe.bgra"),
    include_bytes!("../../assets/period/song.bgra"),
    include_bytes!("../../assets/period/japan.bgra"),
    include_bytes!("../../assets/period/goryeo.bgra"),
    include_bytes!("../../assets/period/islamic.bgra"),
    include_bytes!("../../assets/period/latin.bgra"),
    include_bytes!("../../assets/period/orthodox.bgra"),
    include_bytes!("../../assets/period/southern.bgra"),
    include_bytes!("../../assets/period/tibet.bgra"),
    include_bytes!("../../assets/period/ainu.bgra"),
];
const EXTRA: &[u8] = include_bytes!("../../assets/period/extra.bgra");

/// Nation order is the simulation's NATIONS order. England and France both use the Latin (western) atlas;
/// the national colour badge tells them apart. Architecture follows origin.
pub fn culture(n: usize) -> usize {
    const MAP: [usize; 2] = [5, 5];
    MAP.get(n).copied().unwrap_or(5)
}

#[derive(Default)]
pub(super) struct PeriodArt {
    normal: Vec<Option<ID2D1Bitmap>>,
    flipped: Vec<Option<ID2D1Bitmap>>,
    fog: Vec<Option<ID2D1Bitmap>>,
    /// 天使（白く輝く姿）：通常・反転
    angel: [Option<ID2D1Bitmap>; 2],
    /// 大砲（手で描く）：通常・反転
    bombard: [Option<ID2D1Bitmap>; 2],
}

impl PeriodArt {
    pub(super) fn load(&mut self, gfx: &Gfx) {
        self.normal.clear(); self.flipped.clear(); self.fog.clear();
        for raw in ATLASES.into_iter().flat_map(|a| a.chunks_exact(CELL)).chain(EXTRA.chunks_exact(CELL)) {
            self.normal.push(gfx.create_bitmap(SIZE as u32, SIZE as u32, &load_raw_sprite_bgra(raw, SIZE, false, 1.0)));
            self.flipped.push(gfx.create_bitmap(SIZE as u32, SIZE as u32, &load_raw_sprite_bgra(raw, SIZE, true, 1.0)));
            self.fog.push(gfx.create_bitmap(SIZE as u32, SIZE as u32, &load_raw_sprite_bgra(raw, SIZE, false, 0.52)));
        }
        // 天使：ラテンの白い騎士を、金色の光をまとった白に染める
        let raw = &ATLASES[5][5 * CELL..6 * CELL];
        for (k, flip) in [false, true].into_iter().enumerate() {
            let px: Vec<u32> = load_raw_sprite_bgra(raw, SIZE, flip, 1.0)
                .into_iter()
                .map(|p| {
                    let a = (p >> 24) & 255;
                    if a == 0 {
                        return 0;
                    }
                    let af = a as f32 / 255.0;
                    let ch = |sh: u32, tint: f32| -> u32 {
                        let c = ((p >> sh) & 255) as f32 / 255.0 / af;
                        let l = 0.55 + 0.45 * c;
                        ((l * tint).min(1.0) * af * 255.0) as u32
                    };
                    ch(0, 0.86) | (ch(8, 0.96) << 8) | (ch(16, 1.0) << 16) | (a << 24)
                })
                .collect();
            self.angel[k] = gfx.create_bitmap(SIZE as u32, SIZE as u32, &px);
        }
        let bs = super::bombard_sprite_size() as u32;
        self.bombard[0] = gfx.create_bitmap(bs, bs, &super::gen_bombard_sprite(false));
        self.bombard[1] = gfx.create_bitmap(bs, bs, &super::gen_bombard_sprite(true));
    }
    pub(super) fn get(&self, idx: usize, flip: bool, fog: bool) -> Option<&ID2D1Bitmap> {
        (if fog { &self.fog } else if flip { &self.flipped } else { &self.normal }).get(idx).and_then(Option::as_ref)
    }
}

impl Art {
    pub fn national_unit(&self, nation: usize, ut: u8, flip: bool) -> Option<&ID2D1Bitmap> {
        use crate::world::*;
        let _ = nation;
        const LATIN: usize = 5;
        const ORTHO: usize = 6;
        // (文化, 枠)。枠：0民 1歩兵 2弓兵 3騎射 4騎兵 5特別
        let (c, slot) = match ut {
            UT_WORKER => (LATIN, 0),
            UT_INF => (LATIN, 1),
            UT_ARC | UT_LONGBOW | 14 => (ORTHO, 2),
            5 => (ORTHO, 1),
            UT_MOUNTED_ARCHER => (LATIN, 3),
            UT_KNIGHT_E | UT_CAV => (LATIN, 4),
            UT_KNIGHT_F => (ORTHO, 4),
            UT_CROSSBOW | UT_GENOESE => (LATIN, 2),
            13 => (ORTHO, 5),
            UT_SCOTS => (ORTHO, 1),
            UT_ROUTIER => (ORTHO, 3),
            UT_COG => return self.period.get(120 + 2, flip, false),
            UT_GALLEY => return self.period.get(120 + 3, flip, false),
            UT_BOMBARD | UT_ROYAL_GUNS => return self.period.bombard[flip as usize].as_ref(),
            UT_ANGEL | UT_GHOST => return self.period.angel[flip as usize].as_ref(),
            _ => match udef(ut).class {
                UnitKind::Ship => return self.period.get(120 + 2, flip, false),
                _ => (LATIN, 1),
            },
        };
        self.period.get(c * 12 + slot, flip, false)
    }
    pub fn national_building(&self, origin: usize, kind: BKind, fog: bool) -> Option<&ID2D1Bitmap> {
        let (c, slot) = match kind {
            BKind::Capital => (culture(origin), 6),
            // 城壁に囲まれた街
            BKind::Town => (6, 10),
            BKind::Village => (culture(origin), 7),
            BKind::Barracks => (culture(origin), 8),
            BKind::Farm => (culture(origin), 9),
            BKind::Fort => (culture(origin), 10),
            BKind::Port => (culture(origin), 11),
        };
        self.period.get(c * 12 + slot, false, fog)
    }
    /// 英雄の姿：その兵種の絵（ジャンヌ・ダルクは白い陣羽織の姿）
    pub fn period_hero(&self, hero: usize, flip: bool) -> Option<&ID2D1Bitmap> {
        use crate::world::*;
        if hero >= HEROES.len() { return None; }
        if hero == JOAN {
            return self.period.get(5 * 12 + 5, flip, false);
        }
        self.national_unit(HEROES[hero].nation, HEROES[hero].ut, flip)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn atlas_layout_and_nation_coverage() {
        for a in ATLASES { assert_eq!(a.len(), CELL * 12); }
        assert_eq!(EXTRA.len(), CELL * 8);
        for n in 0..crate::geo::NATIONS.len() { assert!(culture(n) < ATLASES.len()); }
    }
}
