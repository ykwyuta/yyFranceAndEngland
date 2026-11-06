//! Dedicated Hundred Years' War sprites, compiled to premultiplied BGRA.
use super::*;
const SIZE: usize = 192;
const CELL: usize = SIZE * SIZE * 4;
const ATLASES: [&[u8]; 3] = [
    include_bytes!("../../assets/hundred-years/units.bgra"),
    include_bytes!("../../assets/hundred-years/buildings.bgra"),
    include_bytes!("../../assets/hundred-years/heroes.bgra"),
];
#[derive(Default)]
pub(super) struct PeriodArt {
    normal: Vec<Option<ID2D1Bitmap>>,
    flipped: Vec<Option<ID2D1Bitmap>>,
    fog: Vec<Option<ID2D1Bitmap>>,
    angel: [Option<ID2D1Bitmap>; 2],
}
impl PeriodArt {
    pub(super) fn load(&mut self, gfx: &Gfx) {
        self.normal.clear(); self.flipped.clear(); self.fog.clear();
        for raw in ATLASES.into_iter().flat_map(|a| a.chunks_exact(CELL)) {
            self.normal.push(gfx.create_bitmap(SIZE as u32, SIZE as u32, &load_raw_sprite_bgra(raw, SIZE, false, 1.0)));
            self.flipped.push(gfx.create_bitmap(SIZE as u32, SIZE as u32, &load_raw_sprite_bgra(raw, SIZE, true, 1.0)));
            self.fog.push(gfx.create_bitmap(SIZE as u32, SIZE as u32, &load_raw_sprite_bgra(raw, SIZE, false, 0.52)));
        }
        // Fictional endgame: a spectral version of Joan's dedicated sprite.
        let raw = &ATLASES[2][crate::world::JOAN * CELL..(crate::world::JOAN + 1) * CELL];
        for (k, flip) in [false, true].into_iter().enumerate() {
            let px: Vec<u32> = load_raw_sprite_bgra(raw, SIZE, flip, 1.0).into_iter().map(|p| {
                let a = (p >> 24) & 255;
                if a == 0 { return 0; }
                let ch = |sh: u32, tint: f32| -> u32 {
                    let c = ((p >> sh) & 255) as f32 / a as f32;
                    ((0.55 + 0.45 * c) * tint * a as f32) as u32
                };
                ch(0, 0.86) | (ch(8, 0.96) << 8) | (ch(16, 1.0) << 16) | (a << 24)
            }).collect();
            self.angel[k] = gfx.create_bitmap(SIZE as u32, SIZE as u32, &px);
        }
    }
    pub(super) fn get(&self, idx: usize, flip: bool, fog: bool) -> Option<&ID2D1Bitmap> {
        (if fog { &self.fog } else if flip { &self.flipped } else { &self.normal }).get(idx).and_then(Option::as_ref)
    }
}
fn unit_slot(nation: usize, ut: u8) -> usize {
    use crate::world::*;
    match ut {
        UT_WORKER => 0, UT_INF => 1, UT_ARC | UT_LONGBOW => 2,
        5 => 28 + 3, // Dedicated standing knight for English dismounted men-at-arms.
        UT_MOUNTED_ARCHER => 3, UT_KNIGHT_E => 4, UT_KNIGHT_F => 5,
        UT_CAV => if nation == crate::geo::FRANCE { 5 } else { 4 },
        UT_CROSSBOW => 6, 13 => 7, 14 => 8, UT_GENOESE => 9,
        UT_SCOTS => 10, UT_ROUTIER => 11, UT_COG => 12, UT_GALLEY => 13,
        UT_BOMBARD => 14, UT_ROYAL_GUNS => 15, _ => 1,
    }
}
impl Art {
    pub fn national_unit(&self, nation: usize, ut: u8, flip: bool) -> Option<&ID2D1Bitmap> {
        if matches!(ut, crate::world::UT_ANGEL | crate::world::UT_GHOST) {
            return self.period.angel[flip as usize].as_ref();
        }
        self.period.get(unit_slot(nation, ut), flip, false)
    }
    pub fn national_building(&self, origin: usize, kind: BKind, fog: bool) -> Option<&ID2D1Bitmap> {
        let slot = match kind {
            BKind::Capital => if origin == crate::geo::ENGLAND { 1 } else { 0 },
            BKind::Town => 2,
            BKind::Village => if origin == crate::geo::ENGLAND { 3 } else { 8 },
            BKind::Barracks => 4, BKind::Farm => 5, BKind::Fort => 6, BKind::Port => 7,
        };
        self.period.get(16 + slot, false, fog)
    }
    pub fn period_hero(&self, hero: usize, flip: bool) -> Option<&ID2D1Bitmap> {
        if hero >= crate::world::HEROES.len() { return None; }
        self.period.get(28 + hero, flip, false)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hundred_years_art_covers_every_hero_and_unit() {
        assert_eq!(ATLASES[0].len(), CELL * 16);
        assert_eq!(ATLASES[1].len(), CELL * 12);
        assert_eq!(ATLASES[2].len(), CELL * crate::world::HEROES.len());
        for nation in 0..crate::geo::NATIONS.len() {
            for ut in 0..crate::world::UNIT_DEFS.len() {
                assert!(unit_slot(nation, ut as u8) < 28 + crate::world::HEROES.len());
            }
        }
    }
}
