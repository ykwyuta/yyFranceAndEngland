//! 兵種：イングランドとフランスで作れる兵が違う。年代が進むと新しい兵（大砲・常備軍）が加わる。
//! 船はイングランドだけが造れる。フランスはイタリアの都市からガレー船を雇う（trade.rs）。

use super::*;

/// 兵種の定義
pub struct UnitDef {
    pub name: &'static str,
    /// 戦い方の大分類（相性・見た目・維持費の区分に使う）
    pub class: UnitKind,
    pub hp: f32,
    pub atk: f32,
    /// 射程（1.5 より長ければ遠くから射る）
    pub range: f32,
    pub cd: f32,
    pub speed: f32,
    /// [糧, 木, 鉄]
    pub cost: [f32; 3],
    pub time: f32,
    /// 建物への攻撃力の倍率
    pub vs_bld: f32,
    /// 特に強い相手（大分類, 倍率）
    pub extra: Option<(UnitKind, f32)>,
    /// 作れるようになる年（西暦）
    pub from_ad: i32,
    pub desc: &'static str,
}

macro_rules! ud {
    ($n:expr, $c:ident, $hp:expr, $atk:expr, $r:expr, $cd:expr, $sp:expr, $cost:expr, $t:expr, $vb:expr, $ex:expr, $d:expr) => {
        UnitDef { name: $n, class: UnitKind::$c, hp: $hp, atk: $atk, range: $r, cd: $cd, speed: $sp, cost: $cost, time: $t, vs_bld: $vb, extra: $ex, from_ad: 0, desc: $d }
    };
    ($n:expr, $c:ident, $hp:expr, $atk:expr, $r:expr, $cd:expr, $sp:expr, $cost:expr, $t:expr, $vb:expr, $ex:expr, from $f:expr, $d:expr) => {
        UnitDef { name: $n, class: UnitKind::$c, hp: $hp, atk: $atk, range: $r, cd: $cd, speed: $sp, cost: $cost, time: $t, vs_bld: $vb, extra: $ex, from_ad: $f, desc: $d }
    };
}

const CAV: Option<(UnitKind, f32)> = Some((UnitKind::Cavalry, 1.8));

pub const UNIT_DEFS: &[UnitDef] = &[
    // 0
    ud!("民", Worker, 40.0, 3.0, 0.75, 1.2, 1.9, [50.0, 0.0, 0.0], 8.0, 0.4, None, "資源を集め、村や建物を建てる"),
    // 1〜3 共通
    ud!("槍兵", Infantry, 120.0, 12.0, 0.75, 1.2, 1.9, [55.0, 0.0, 25.0], 10.0, 1.0, CAV, "長槍で騎兵を止める歩兵。城攻めに優れる"),
    ud!("弓兵", Archer, 75.0, 9.0, 4.5, 1.6, 1.75, [50.0, 35.0, 15.0], 11.0, 0.35, None, "遠くから射る。歩兵に強いが騎兵に弱い"),
    ud!("騎兵", Cavalry, 140.0, 14.0, 0.75, 1.4, 3.2, [80.0, 45.0, 40.0], 14.0, 0.7, None, "速い突撃兵。弓兵と民に強い"),
    // 4〜8 イングランド
    ud!("ロングボウ兵", Archer, 80.0, 11.0, 5.6, 1.3, 1.8, [55.0, 40.0, 10.0], 11.0, 0.35, Some((UnitKind::Cavalry, 1.6)), "イチイの長弓を引くヨーマン。射程が長く速射し、突撃する騎士を射すくめる"),
    ud!("下馬重装兵", Infantry, 150.0, 14.0, 0.75, 1.2, 1.8, [70.0, 0.0, 35.0], 12.0, 1.1, CAV, "馬を降りて弓兵と並ぶ重装の騎士・従者。騎兵の突撃を受け止める"),
    ud!("騎乗弓兵", Cavalry, 100.0, 9.0, 4.4, 1.4, 3.3, [70.0, 40.0, 20.0], 12.0, 0.4, None, "馬で駆け、降りて射る弓兵。騎行（シュヴォシェ）で村々を荒らす"),
    ud!("騎士", Cavalry, 170.0, 16.0, 0.75, 1.5, 2.9, [100.0, 45.0, 55.0], 15.0, 0.7, None, "イングランド王に仕える騎士"),
    ud!("コグ船", Ship, 320.0, 10.0, 4.2, 1.8, 2.7, [60.0, 160.0, 40.0], 18.0, 0.5, None, "兵6人を乗せて海峡を渡る丸い帆船。船首と船尾の楼から射る"),
    // 9〜10 フランス
    ud!("フランス騎士", Cavalry, 190.0, 18.0, 0.75, 1.5, 2.9, [110.0, 45.0, 60.0], 16.0, 0.7, None, "ヨーロッパに名高いフランスの騎士。突撃は何よりも重い"),
    ud!("弩兵", Archer, 80.0, 12.0, 4.6, 2.0, 1.7, [55.0, 40.0, 25.0], 12.0, 0.4, None, "鎧を射抜くクロスボウ兵。強いが装填は遅い"),
    // 11〜12 ジェノヴァ（傭兵）
    ud!("ジェノヴァ弩兵", Archer, 90.0, 13.0, 5.0, 2.0, 1.7, [60.0, 40.0, 30.0], 12.0, 0.4, None, "大盾（パヴィース）を携えたジェノヴァの傭兵弩兵"),
    ud!("ガレー船", Ship, 360.0, 13.0, 4.2, 1.7, 3.0, [60.0, 170.0, 50.0], 18.0, 0.6, None, "ジェノヴァなどイタリアの都市から雇う櫂船。兵6人を乗せて海を渡る"),
    // 13〜14 フランス（後期の改革）
    ud!("勅令隊", Cavalry, 210.0, 19.0, 0.75, 1.4, 3.0, [115.0, 45.0, 65.0], 16.0, 0.8, None, from 1445, "シャルル7世が置いたヨーロッパ初の常備軍の重騎兵（1445年〜）"),
    ud!("フラン・アルシェ", Archer, 85.0, 11.0, 5.0, 1.5, 1.8, [50.0, 35.0, 15.0], 11.0, 0.4, None, from 1448, "教区ごとに一人出す、税を免じられた自由弓兵（1448年〜）"),
    // 15 両国
    ud!("ボンバード", Archer, 90.0, 30.0, 6.5, 4.5, 1.2, [60.0, 80.0, 90.0], 18.0, 3.2, Some((UnitKind::Infantry, 1.3)), from 1375, "鉄の輪を束ねた大砲。遅いが城壁を崩す（1375年〜）"),
    // 16〜17 国ではない兵
    ud!("スコットランド槍兵", Infantry, 140.0, 13.0, 0.75, 1.2, 2.0, [60.0, 0.0, 25.0], 10.0, 1.0, Some((UnitKind::Cavalry, 2.0)), "長槍を林のように並べるシルトロンの兵"),
    ud!("傭兵団", Cavalry, 150.0, 14.0, 0.75, 1.4, 3.1, [85.0, 40.0, 40.0], 14.0, 0.8, None, "休戦で雇い主を失い、村々を荒らす大傭兵団（グランド・カンパニー）"),
    // 18 フランス
    ud!("王の砲兵", Archer, 100.0, 36.0, 7.0, 4.0, 1.3, [60.0, 90.0, 100.0], 18.0, 3.8, Some((UnitKind::Infantry, 1.4)), from 1436, "ビュロー兄弟が鍛えたフランス王の砲兵。フォルミニーとカスティヨンで勝負を決めた（1436年〜）"),
    // 19〜20 天
    ud!("天使", Infantry, 400.0, 40.0, 1.2, 1.0, 2.6, [0.0, 0.0, 0.0], 1.0, 4.0, None, "ジャンヌ・ダルクの亡霊に率いられた天の軍勢。人の剣も矢も届かない"),
    ud!("ジャンヌの亡霊", Cavalry, 600.0, 50.0, 1.2, 1.0, 3.0, [0.0, 0.0, 0.0], 1.0, 5.0, None, "ルーアンで灰となった乙女の亡霊。百年の戦を終わらせなかった両国を裁く"),
];

pub const UT_WORKER: u8 = 0;
pub const UT_INF: u8 = 1;
pub const UT_ARC: u8 = 2;
pub const UT_CAV: u8 = 3;
pub const UT_LONGBOW: u8 = 4;
pub const UT_MOUNTED_ARCHER: u8 = 6;
pub const UT_KNIGHT_E: u8 = 7;
pub const UT_COG: u8 = 8;
pub const UT_KNIGHT_F: u8 = 9;
pub const UT_CROSSBOW: u8 = 10;
pub const UT_GENOESE: u8 = 11;
pub const UT_GALLEY: u8 = 12;
pub const UT_BOMBARD: u8 = 15;
pub const UT_SCOTS: u8 = 16;
pub const UT_ROUTIER: u8 = 17;
pub const UT_ROYAL_GUNS: u8 = 18;
pub const UT_ANGEL: u8 = 19;
pub const UT_GHOST: u8 = 20;

/// 国ごとに作れる兵（民を除く）。並びは NATIONS と同じ
pub const ROSTERS: &[&[u8]] = &[
    &[5, 4, 6, 7, 15, 8],      // イングランド
    &[1, 10, 9, 15, 18, 13, 14], // フランス
];

/// 船の定員
pub const SHIP_CAP: usize = 6;

pub fn udef(ut: u8) -> &'static UnitDef {
    UNIT_DEFS.get(ut as usize).unwrap_or(&UNIT_DEFS[0])
}

/// その国の基本の兵種（大分類に合うもの）
pub fn native_ut(n: usize, class: UnitKind) -> u8 {
    if class == UnitKind::Worker {
        return UT_WORKER;
    }
    if let Some(r) = ROSTERS.get(n) {
        if let Some(&u) = r.iter().find(|&&u| udef(u).class == class) {
            return u;
        }
    }
    match class {
        UnitKind::Infantry => UT_INF,
        UnitKind::Archer => UT_ARC,
        _ => UT_CAV,
    }
}

/// その国の最も強い陸兵（史実の出来事で馳せ参じる兵）
pub fn best_land_ut(n: usize, ad: i32) -> u8 {
    ROSTERS
        .get(n)
        .and_then(|r| r.iter().copied().filter(|&u| udef(u).class != UnitKind::Ship && udef(u).from_ad <= ad && u != UT_BOMBARD && u != UT_ROYAL_GUNS).max_by(|&a, &b| (udef(a).hp * udef(a).atk).partial_cmp(&(udef(b).hp * udef(b).atk)).unwrap()))
        .unwrap_or(UT_INF)
}

/// 船を造れる国か（自国の兵種に船がある）
pub fn native_ships(n: usize) -> bool {
    ROSTERS.get(n).map_or(false, |r| r.iter().any(|&u| udef(u).class == UnitKind::Ship))
}

impl World {
    /// その国が作れる兵種（自国の兵のうち、いまの年に作れるもの）。(兵種, 出身国)
    pub fn available_units(&self, n: usize) -> Vec<(u8, usize)> {
        let ad = self.year_ad();
        ROSTERS.get(n).map_or(Vec::new(), |r| r.iter().filter(|&&u| udef(u).from_ad <= ad).map(|&u| (u, n)).collect())
    }

    /// その国が船を造れるか（自国の兵に船がある）
    pub fn can_build_ships(&self, n: usize) -> bool {
        n < NATIONS.len() && native_ships(n)
    }

    /// 建物で訓練できる兵種
    pub fn trainable_at(&self, b: &Building) -> Vec<(u8, usize)> {
        if !b.done() {
            return Vec::new();
        }
        let n = b.nation as usize;
        if n >= NATIONS.len() {
            return Vec::new();
        }
        match b.kind {
            BKind::Capital | BKind::Town => vec![(UT_WORKER, n)],
            BKind::Barracks => self.available_units(n).into_iter().filter(|&(u, _)| udef(u).class != UnitKind::Ship).collect(),
            BKind::Port => self.available_units(n).into_iter().filter(|&(u, _)| udef(u).class == UnitKind::Ship).collect(),
            BKind::Farm | BKind::Fort | BKind::Village => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rosters_match_nations() {
        assert_eq!(ROSTERS.len(), NATIONS.len());
        for (n, r) in ROSTERS.iter().enumerate() {
            for &u in r.iter() {
                assert!((u as usize) < UNIT_DEFS.len());
                assert_ne!(u, UT_WORKER);
            }
            // 船を造れるのはイングランドだけ（フランスはイタリアのガレー船を雇う）
            assert_eq!(native_ships(n), n == ENGLAND, "{}", NATIONS[n].name);
        }
    }
}
