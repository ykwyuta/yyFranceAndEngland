//! 国ごとの史実に基づく特徴

use super::*;

/// 特徴による補正（1.0＝補正なし）
#[derive(Clone, Copy, Debug)]
pub struct TraitMods {
    /// 生産（城邑の税・採集）
    pub eco: f32,
    /// 兵の大分類ごとの攻撃（民・歩兵・弓兵・騎兵・船）
    pub atk: [f32; 5],
    /// 兵の大分類ごとの訓練費
    pub cost: [f32; 5],
    /// 軍の維持費
    pub upkeep: f32,
    /// 城邑・建物が受ける損害
    pub def: f32,
    /// 商人の集まりやすさ
    pub attract: f32,
    /// 市場の値
    pub market: f32,
    /// 部族の略奪で失う蓄え
    pub plunder: f32,
    /// 鍛錬・教化の費用
    pub upgrade: f32,
    /// 村の略奪で奪う物資
    pub loot: f32,
}

impl TraitMods {
    pub const NONE: TraitMods = TraitMods { eco: 1.0, atk: [1.0; 5], cost: [1.0; 5], upkeep: 1.0, def: 1.0, attract: 1.0, market: 1.0, plunder: 1.0, upgrade: 1.0, loot: 1.0 };
}

pub struct Trait {
    pub nation: usize,
    pub name: &'static str,
    pub desc: &'static str,
    /// この年（西暦）から有効
    pub from_ad: i32,
    /// この年（西暦）まで有効
    pub to_ad: i32,
    pub fx: fn(&mut TraitMods),
}

const ALL: (i32, i32) = (1337, 1453);
const INF: usize = 1;
const ARC: usize = 2;
const CAVK: usize = 3;

macro_rules! tr {
    ($n:expr, $name:expr, $desc:expr, $span:expr, $f:expr) => {
        Trait { nation: $n, name: $name, desc: $desc, from_ad: $span.0, to_ad: $span.1, fx: $f }
    };
}

pub const TRAITS: &[Trait] = &[
    tr!(ENGLAND, "ロングボウ", "ウェールズとイングランドのヨーマンが引く長弓。弓兵の攻撃+25%", ALL, |m| m.atk[ARC] *= 1.25),
    tr!(ENGLAND, "騎行（シュヴォシェ）", "敵地を駆け、村を焼いて王の威信を損なう戦い方。村の略奪で奪う物資+60%、騎兵の訓練費-15%", ALL, |m| {
        m.loot *= 1.6;
        m.cost[CAVK] *= 0.85;
    }),
    tr!(ENGLAND, "羊毛の富", "フランドルの織物を支えるイングランドの羊毛。商人が集まる（×1.4）、市場の値-15%", ALL, |m| {
        m.attract *= 1.4;
        m.market *= 0.85;
    }),
    tr!(ENGLAND, "海峡の向こう", "援軍も糧も海を越えねばならない。維持費+10%", ALL, |m| m.upkeep *= 1.1),
    tr!(ENGLAND, "ランカスター家の征服", "1415年、ヘンリー5世はノルマンディーの征服に乗り出した。歩兵の攻撃+15%", (1415, 1453), |m| m.atk[INF] *= 1.15),
    tr!(FRANCE, "騎士道", "名誉を重んじる騎士の突撃。騎兵の攻撃+20%（ただしロングボウに弱い）", ALL, |m| m.atk[CAVK] *= 1.2),
    tr!(FRANCE, "西欧一の大国", "人口はイングランドの四倍。生産+10%、民の訓練費-10%", ALL, |m| {
        m.eco *= 1.1;
        m.cost[0] *= 0.9;
    }),
    tr!(FRANCE, "城壁の都市", "城壁に籠もる街は簡単には落ちない。街・建物の受ける損害-15%", ALL, |m| m.def *= 0.85),
    tr!(FRANCE, "賢明王の改革", "シャルル5世は会戦を避け、城を一つずつ取り返した。鍛錬の費用-25%、維持費-10%", (1364, 1380), |m| {
        m.upgrade *= 0.75;
        m.upkeep *= 0.9;
    }),
    tr!(FRANCE, "常備軍と砲兵", "勅令隊とビュロー兄弟の砲兵。維持費-20%、弓兵（砲兵を含む）の攻撃+15%", (1439, 1453), |m| {
        m.upkeep *= 0.8;
        m.atk[ARC] *= 1.15;
    }),
];

/// その年に有効な特徴
pub fn traits_of(n: usize, ad: i32) -> impl Iterator<Item = &'static Trait> {
    TRAITS.iter().filter(move |t| t.nation == n && ad >= t.from_ad && ad <= t.to_ad)
}

/// 年代を問わずその国の特徴すべて
pub fn all_traits_of(n: usize) -> impl Iterator<Item = &'static Trait> {
    TRAITS.iter().filter(move |t| t.nation == n)
}

impl World {
    /// 特徴の補正を計算し直す（毎年）
    pub(super) fn refresh_traits(&mut self) {
        let ad = self.year_ad();
        for i in 0..NATIONS.len() {
            let mut m = TraitMods::NONE;
            for t in traits_of(i, ad) {
                (t.fx)(&mut m);
            }
            self.nations[i].tm = m;
        }
    }

    /// 最後の街を落とされても滅びない国（この時代にはない）
    pub fn immortal(&self, _n: usize) -> bool {
        false
    }

    /// その国の兵種の訓練費
    pub fn train_cost(&self, n: usize, ut: u8) -> [f32; 3] {
        let d = udef(ut);
        let m = if n < NATIONS.len() { self.nations[n].tm.cost[d.class as usize] } else { 1.0 };
        let c = d.cost;
        [(c[0] * m).round(), (c[1] * m).round(), (c[2] * m).round()]
    }

    /// 市場の値
    pub fn market_price(&self, n: usize, k: usize) -> f32 {
        (MARKET_PRICE[k] * self.nations[n].tm.market).round()
    }
}
