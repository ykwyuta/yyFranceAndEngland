//! 交易：街を巡る商人・遠方の商人（羊毛・ワイン・イタリア・ハンザ）、
//! 国ではない兵（スコットランド軍・ジェノヴァの傭兵・大傭兵団）

use super::*;

/// 部族の兵が属する仮の枠（国としては扱わない。外交・連合・終了判定には現れない）
pub const TRIBE: usize = NATIONS.len();

/// 税率（軽税・中税・重税）：税として取る割合と、商人の集まりやすさ
/// 税率の名前（0:軽税 1:中税 2:重税）
pub fn tax_name(t: usize) -> &'static str {
    match t {
        0 => t!("軽税", "Light tax", "轻税"),
        1 => t!("中税", "Normal tax", "中税"),
        _ => t!("重税", "Heavy tax", "重税"),
    }
}
pub const TAX_RATE: [f32; 3] = [0.6, 1.0, 1.5];
pub const TAX_ATTRACT: [f32; 3] = [1.5, 1.0, 0.55];

/// 市場の値（金貨）：100単位あたり
pub const MARKET_PRICE: [f32; 3] = [40.0, 50.0, 80.0];
pub const MARKET_LOT: f32 = 100.0;

/// 傭兵の契約年数と人数
pub const MERC_YEARS: f32 = 10.0;
pub const MERC_N: usize = 3;

// ---------------------------------------------------------------- 国ではない兵
pub struct TribeDef {
    pub name: &'static str,
    pub lon: f32,
    pub lat: f32,
    /// 活動する年代（西暦）
    pub from_ad: i32,
    pub to_ad: i32,
    /// 騎兵が主か（雇う画面の表示）
    pub nomad: bool,
    /// 略奪に向かいやすさ（0〜1）
    pub host: f32,
    pub desc: &'static str,
    /// 動く年代（この間だけ略奪・交易に来る。空なら from_ad〜to_ad）
    pub periods: &'static [(i32, i32)],
    /// 雇える国（ビット：1=イングランド 2=フランス）
    pub hire: u8,
    /// 襲う国（ビット）
    pub raid: u8,
    /// 兵の並び
    pub units: &'static [u8],
    /// 船（海に現れる）
    pub naval: bool,
    /// 遠くの国からでも雇える（本拠から自国の街までの距離を問わない）
    pub anywhere: bool,
}

const EN: u8 = 1;
const FR: u8 = 2;

pub const TRIBES: &[TribeDef] = &[
    TribeDef { name: "スコットランド", lon: -3.20, lat: 55.88, from_ad: 1337, to_ad: 1453, nomad: false, host: 0.75, desc: "「古い同盟」でフランスと結ぶ北の王国。国境を越えてイングランドの北を荒らす", periods: &[(1337, 1357), (1384, 1389), (1400, 1402), (1417, 1417)], hire: 0, raid: EN, units: &[UT_SCOTS, UT_SCOTS, UT_ARC], naval: false, anywhere: false },
    TribeDef { name: "ジェノヴァ弩兵", lon: 7.60, lat: 43.90, from_ad: 1337, to_ad: 1453, nomad: false, host: 0.0, desc: "リグリアの海の共和国が送り出す、名高い傭兵弩兵", periods: &[], hire: FR, raid: 0, units: &[UT_GENOESE, UT_GENOESE, UT_GENOESE], naval: false, anywhere: true },
    TribeDef { name: "ジェノヴァのガレー船", lon: 7.60, lat: 43.90, from_ad: 1337, to_ad: 1453, nomad: false, host: 0.0, desc: "フランス王が雇うイタリアの櫂船。ジブラルタルを回って海峡に現れ、兵を運び、敵の船を襲う", periods: &[], hire: FR, raid: 0, units: &[UT_GALLEY, UT_GALLEY], naval: true, anywhere: true },
    TribeDef { name: "大傭兵団", lon: 4.60, lat: 46.60, from_ad: 1357, to_ad: 1445, nomad: true, host: 0.6, desc: "休戦で職を失った兵の群れ（グランド・カンパニー、後のエコルシュール）。村を荒らすが、金で雇える", periods: &[(1357, 1370), (1435, 1445)], hire: EN | FR, raid: EN | FR, units: &[UT_ROUTIER, UT_ROUTIER, UT_ARC], naval: false, anywhere: false },
    TribeDef { name: "カスティーリャのガレー船", lon: -1.80, lat: 43.30, from_ad: 1369, to_ad: 1453, nomad: false, host: 0.0, desc: "1369年、トラスタマラ家のエンリケ2世はフランスと結んだ。ラ・ロシェルの海戦でイングランドの艦隊を焼いた", periods: &[], hire: FR, raid: 0, units: &[UT_GALLEY, UT_GALLEY], naval: true, anywhere: true },
];

/// 部族の状態
#[derive(Clone)]
pub struct TribeState {
    /// 次に動く年（ゲーム内経過年）
    pub next: f32,
    /// 国ごとの雇用契約の満了年（その間はその国を襲わない）
    pub contract: Vec<f32>,
    /// 最近の動き
    pub last: String,
}

/// 略奪に出た部族の一団
pub struct Band {
    pub tribe: usize,
    pub target: usize,
    pub city: Handle,
    pub units: Vec<Handle>,
    /// 0:略奪へ 1:引き上げ
    pub phase: u8,
    pub until: f32,
    pub home: V2,
}

// ---------------------------------------------------------------- 遠方の商人の道
pub struct SilkRoute {
    pub name: &'static str,
    /// 地図上の入口（経度・緯度）
    pub entry: (f32, f32),
    pub from_ad: i32,
    /// 商人が訪れる地域（経度 lon0..lon1, 緯度 lat0..lat1）
    pub region: (f32, f32, f32, f32),
    pub desc: &'static str,
}

pub const ROUTES: &[SilkRoute] = &[
    SilkRoute { name: "羊毛の道（フランドル）", entry: (4.40, 51.20), from_ad: 1337, region: (1.5, 4.5, 49.5, 51.4), desc: "アントウェルペンからヘント・ブルッヘ・リールへ。イングランドの羊毛が織物になる" },
    SilkRoute { name: "ワインの道（ガスコーニュ）", entry: (-1.10, 44.60), from_ad: 1337, region: (-1.6, 1.6, 43.2, 45.9), desc: "ボルドーのクラレットを積む商人。ギュイエンヌの富の源" },
    SilkRoute { name: "イタリア商人（ロンバルディア）", entry: (6.90, 45.10), from_ad: 1337, region: (2.5, 6.5, 43.5, 48.6), desc: "アルプスを越えてリヨン・シャンパーニュの市へ来るフィレンツェやミラノの商人" },
    SilkRoute { name: "ハンザ商人（スティールヤード）", entry: (1.60, 52.70), from_ad: 1337, region: (-2.0, 1.8, 50.6, 53.5), desc: "ロンドンのスティールヤードに商館を持つドイツの商人" },
];

// ---------------------------------------------------------------- 隊商
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CaravanKind {
    /// 中原の商人
    Merchant,
    /// 西域の商人（道の番号）
    Western(u8),
    /// 部族の隊商（部族の番号）
    Tribe(u8),
}

pub struct Caravan {
    pub kind: CaravanKind,
    pub pos: V2,
    pub dir: V2,
    pub(crate) path: Vec<(i16, i16)>,
    pub(crate) path_i: usize,
    pub(crate) need_path: bool,
    pub(crate) dest: Option<Handle>,
    pub(crate) goal: V2,
    /// 市で商い中の残り秒
    pub stay: f32,
    pub(crate) trips: u8,
    pub(crate) max_trips: u8,
    /// 帰途（西域・部族）
    pub leaving: bool,
    /// 去った（取り除く）
    pub(crate) gone: bool,
    pub value: f32,
}

/// 交易全体の状態
pub struct Trade {
    pub caravans: Vec<Caravan>,
    pub tribes: Vec<TribeState>,
    pub bands: Vec<Band>,
    /// 中原の商人の目標数（中原全体の生産量に比例）
    pub target: usize,
    pub target_w: usize,
    pub(crate) spawn_t: f32,
    pub(crate) band_t: f32,
}

impl Trade {
    pub fn new(rng: &mut Rng) -> Trade {
        let tribes = TRIBES
            .iter()
            .map(|_| TribeState { next: rng.range(12.0, 26.0), contract: vec![0.0; NATIONS.len()], last: String::new() })
            .collect();
        Trade { caravans: Vec::new(), tribes, bands: Vec::new(), target: 8, target_w: 1, spawn_t: 0.0, band_t: 0.0 }
    }
    pub fn count(&self, f: impl Fn(CaravanKind) -> bool) -> usize {
        self.caravans.iter().filter(|c| f(c.kind)).count()
    }
}

/// 部族がその年に活動しているか
pub fn tribe_active(t: usize, yb: i32) -> bool {
    let d = &TRIBES[t];
    yb >= d.from_ad && yb <= d.to_ad && (d.periods.is_empty() || d.periods.iter().any(|&(a, b)| yb >= a && yb <= b))
}

/// その国を表すビット
fn nbit(n: usize) -> u8 {
    1 << n
}

pub fn route_open(r: usize, yb: i32) -> bool {
    yb >= ROUTES[r].from_ad
}

fn in_region(r: usize, p: V2) -> bool {
    let (lon, lat) = tile_to_ll(p.x, p.y);
    let (a, b, c, d) = ROUTES[r].region;
    lon >= a && lon <= b && lat >= c && lat <= d
}

fn ll_v2(lon: f32, lat: f32) -> V2 {
    let (x, y) = ll_to_tile(lon, lat);
    v2(x, y)
}

impl World {
    /// 国の生産力（商人を引き寄せる力）
    pub fn production(&self, n: usize) -> f32 {
        let nn = &self.nations[n];
        nn.income[0] + nn.income[1] + nn.income[2]
    }

    /// 部族の本拠（通行できる地点）
    pub fn tribe_home(&self, t: usize) -> V2 {
        self.passable_near(ll_v2(TRIBES[t].lon, TRIBES[t].lat))
    }

    fn passable_near(&self, p: V2) -> V2 {
        let (cx, cy) = p.tile();
        for r in 0..28i32 {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r {
                        continue;
                    }
                    let (x, y) = (cx + dx, cy + dy);
                    if Map::inb(x, y) && self.cost[Map::idx(x, y)].is_finite() && self.map.terrain[Map::idx(x, y)] != Terrain::River {
                        return v2(x as f32 + 0.5, y as f32 + 0.5);
                    }
                }
            }
        }
        p
    }

    /// 傭兵の値（金貨）
    pub fn merc_cost(&self) -> f32 {
        (150.0 + self.years * 0.3).round()
    }

    /// 部族の兵の能力
    fn tribe_power(&self) -> f32 {
        (1.0 + 2.2 * self.years / (END_AD - START_AD)).min(self.power_cap())
    }

    /// その国が傭兵を雇えるか（雇えない理由）
    pub fn can_hire(&self, n: usize, t: usize) -> Result<(), &'static str> {
        let d = &TRIBES[t];
        if n >= NATIONS.len() || d.hire & nbit(n) == 0 {
            return Err(t!("この国には雇えない", "Your kingdom cannot hire them", "本国无法雇佣"));
        }
        if !tribe_active(t, self.year_ad()) {
            return Err(t!("今はいない", "Not available now", "目前不在"));
        }
        if d.naval {
            if self.merc_port(n).is_none() {
                return Err(t!("海峡か大西洋に面した街が要る", "You need a town on the Channel or the Atlantic coast", "需要一座面向海峡或大西洋的城镇"));
            }
        } else if !d.anywhere {
            let home = self.tribe_home(t);
            let near = self.blds.iter().any(|b| b.alive && b.nation as usize == n && b.kind.city() && b.done() && b.center().dist(home) < 150.0);
            if !near {
                return Err(t!("自国の街から遠すぎる", "Too far from your towns", "离己方城镇太远"));
            }
        }
        if self.trade.bands.iter().any(|b| b.tribe == t && b.target == n && b.phase == 0) {
            return Err(t!("自国を襲っている", "They are raiding you", "正在劫掠我国"));
        }
        if self.nations[n].coin < self.merc_cost_of(t) {
            return Err(t!("金貨が足りない", "Not enough gold", "金币不足"));
        }
        Ok(())
    }

    /// 傭兵の値（金貨）。船は高い
    pub fn merc_cost_of(&self, t: usize) -> f32 {
        let base = self.merc_cost();
        if TRIBES[t].naval {
            (base * 1.5).round()
        } else {
            base
        }
    }

    /// 外海（いちばん大きな海）の番号
    pub fn main_sea(&self) -> u32 {
        let mut cnt: std::collections::HashMap<u32, usize> = Default::default();
        for (i, &r) in self.sea_region.iter().enumerate() {
            if r != u32::MAX && self.map.terrain[i] == Terrain::Sea {
                *cnt.entry(r).or_insert(0) += 1;
            }
        }
        cnt.into_iter().max_by_key(|x| x.1).map_or(u32::MAX, |x| x.0)
    }

    /// 雇った船が現れる海辺の街（イングランドの都と同じ海に面した、自国の街のうち都に近いもの）
    pub fn merc_port(&self, n: usize) -> Option<(usize, V2)> {
        // 外海（いちばん大きな海）
        let channel = self.main_sea();
        let home = self.nations[n].home;
        let mut best: Option<(f32, usize, V2)> = None;
        for (i, b) in self.blds.iter().enumerate() {
            if !b.alive || b.nation as usize != n || !b.kind.city() || !b.done() {
                continue;
            }
            let Some(wp) = self.shore_water_near(b.center(), 5) else { continue };
            let reg = self.sea_region[Map::idx(wp.x as i32, wp.y as i32)];
            if channel != reg {
                continue;
            }
            let d = b.center().dist(home);
            if best.map_or(true, |x| d < x.0) {
                best = Some((d, i, wp));
            }
        }
        best.map(|b| (b.1, b.2))
    }

    /// 街の場所の呼び方（街の名前）
    pub fn city_place(&self, bi: usize) -> String {
        self.town_name(bi)
    }

    /// 傭兵を雇う：本拠に最も近い自国の街（船なら海辺の街の沖）に現れ、契約年数のあいだ仕える。
    /// 現れた傭兵と、現れた街の番号を返す
    pub fn hire_mercs(&mut self, n: usize, t: usize) -> Result<(Vec<Handle>, usize), &'static str> {
        self.can_hire(n, t)?;
        let d = &TRIBES[t];
        let home = self.tribe_home(t);
        let (bi, at) = if d.naval {
            let Some((bi, wp)) = self.merc_port(n) else { return Err("") };
            (bi, Some(wp))
        } else {
            let mut best: Option<(f32, usize)> = None;
            for (i, b) in self.blds.iter().enumerate() {
                if b.alive && b.nation as usize == n && b.kind.city() && b.done() {
                    let dd = b.center().dist(home);
                    if best.map_or(true, |x| dd < x.0) {
                        best = Some((dd, i));
                    }
                }
            }
            let Some((_, bi)) = best else { return Err(t!("街がない", "No town", "没有城镇")) };
            (bi, None)
        };
        let cost = self.merc_cost_of(t);
        self.nations[n].coin -= cost;
        let bh = self.bh(bi);
        let power = (self.tribe_power() * 1.15).min(self.power_cap());
        let until = self.years + MERC_YEARS;
        let mut hired = Vec::new();
        for &kind in d.units.iter() {
            let sp = match at {
                Some(wp) => {
                    // 海の上の空いた所
                    let mut q = wp;
                    for r in 0..4i32 {
                        let c = v2(wp.x + (hired.len() as i32 % 2 * r) as f32, wp.y + r as f32 * 0.6);
                        if self.passable_l(c, true) {
                            q = c;
                            break;
                        }
                    }
                    Some(q)
                }
                None => self.spawn_point(bh),
            };
            if let Some(sp) = sp {
                let h = self.spawn_unit(n, kind, sp, power);
                let u = self.unit_mut(h).unwrap();
                u.tribe = t as u8;
                u.merc_until = until;
                hired.push(h);
            }
        }
        self.trade.tribes[t].contract[n] = until;
        self.trade.tribes[t].last = tf!("{}に雇われている", "In the pay of {}", "受雇于{}", tx(self.nations[n].name));
        if n == self.player {
            let s = tf!("{}を{}隊雇った（金貨{}・{}年）。{}に現れた", "Hired {} ({} units, {} gold, {} years). They appeared at {}", "雇佣了{}{}队（金币{}・{}年）。出现在{}", tx(d.name), hired.len(), cost, MERC_YEARS as i32, self.city_place(bi));
            self.push_msg(s, [0.6, 0.9, 0.6]);
        }
        Ok((hired, bi))
    }

    /// 市場で資源を買う
    pub fn buy_resource(&mut self, n: usize, k: usize) -> Result<(), &'static str> {
        let price = self.market_price(n, k);
        let nn = &mut self.nations[n];
        if nn.coin < price {
            return Err(t!("金貨が足りない", "Not enough gold", "金币不足"));
        }
        nn.coin -= price;
        nn.res[k] += MARKET_LOT;
        Ok(())
    }

    // ------------------------------------------------------------ 毎フレーム
    pub(super) fn update_trade(&mut self, dt: f32) {
        let em = self.era_mult();
        // ---- 隊商の移動と税
        let mut paths = 0;
        let mut arrivals: Vec<usize> = Vec::new();
        for i in 0..self.trade.caravans.len() {
            let c = &self.trade.caravans[i];
            let (x, y) = c.pos.tile();
            if Map::inb(x, y) {
                let o = self.owner[Map::idx(x, y)] as usize;
                if o < NATIONS.len() && self.nations[o].alive {
                    let n = &mut self.nations[o];
                    let add = c.value * 0.45 * em * TAX_RATE[n.tax as usize] * if c.stay > 0.0 { 2.5 } else { 1.0 } * dt;
                    n.coin += add;
                    n.coin_earned += add;
                }
            }
            if c.gone {
                continue;
            }
            if c.stay > 0.0 {
                let c = &mut self.trade.caravans[i];
                c.stay -= dt;
                if c.stay <= 0.0 {
                    arrivals.push(i);
                }
                continue;
            }
            if c.need_path {
                if paths >= 2 {
                    continue;
                }
                paths += 1;
                let start = c.pos.tile();
                let goal = c.goal.tile();
                let path = self.pf.find(&self.cost, start, goal, 2, 160000, crate::path::CARAVAN_WEIGHT);
                let c = &mut self.trade.caravans[i];
                c.path = path;
                c.path_i = 0;
                c.need_path = false;
                if c.path.is_empty() && c.pos.dist(c.goal) > 3.2 {
                    // 行き着けない：あきらめる
                    c.gone = true;
                }
                continue;
            }
            // 経路に沿って進む
            let sp = if matches!(c.kind, CaravanKind::Tribe(_)) { 2.6 } else { 2.0 };
            let (tx, ty) = c.pos.tile();
            let tc = if Map::inb(tx, ty) { self.map.terrain[Map::idx(tx, ty)].move_cost() } else { 1.0 };
            let mut mv = sp / tc.clamp(1.0, 3.0).sqrt() * dt;
            let c = &mut self.trade.caravans[i];
            while mv > 0.0 {
                let to = if c.path_i < c.path.len() {
                    let (px, py) = c.path[c.path_i];
                    v2(px as f32 + 0.5, py as f32 + 0.5)
                } else {
                    c.goal
                };
                let d = to - c.pos;
                let l = d.len();
                if l <= mv {
                    c.pos = to;
                    mv -= l;
                    if c.path_i < c.path.len() {
                        c.path_i += 1;
                    } else {
                        break;
                    }
                } else {
                    c.dir = d * (1.0 / l);
                    c.pos = c.pos + d * (mv / l);
                    mv = 0.0;
                }
            }
            if c.path_i >= c.path.len() {
                if c.pos.dist(c.goal) < 3.2 {
                    c.stay = if c.leaving { 0.01 } else { self.rng.range(6.0, 12.0) };
                } else {
                    c.need_path = true;
                }
            }
        }
        for i in arrivals {
            self.caravan_next(i);
        }
        // 帰り着いた・去った隊商
        self.trade.caravans.retain(|c| !c.gone);

        // ---- 隊商を送り出す
        self.trade.spawn_t -= dt;
        if self.trade.spawn_t <= 0.0 {
            self.trade.spawn_t = 0.8;
            self.spawn_caravans();
        }
        // ---- 略奪の一団
        self.trade.band_t -= dt;
        if self.trade.band_t <= 0.0 {
            self.trade.band_t = 1.0;
            self.update_bands();
        }
    }

    /// 都市を選ぶ：生産力が高く税が軽い国ほど、近いほど選ばれやすい
    fn pick_city(&mut self, from: V2, filter: impl Fn(&World, V2) -> bool, avoid: Option<Handle>) -> Option<Handle> {
        let mut cands: Vec<(f32, usize)> = Vec::new();
        let mut total = 0.0;
        for (i, b) in self.blds.iter().enumerate() {
            if !b.alive || !b.kind.city() || !b.done() || (b.nation as usize) >= NATIONS.len() {
                continue;
            }
            if avoid.map_or(false, |h| h.idx as usize == i) || !filter(self, b.center()) {
                continue;
            }
            let ni = b.nation as usize;
            let n = &self.nations[ni];
            let prod = (self.production(ni) + 0.5) / n.cities.max(1) as f32;
            let mut wgt = prod.powf(1.2) * TAX_ATTRACT[n.tax as usize] * n.tm.attract * if b.kind == BKind::Capital { 1.6 } else { 1.0 };
            if (0..NATIONS.len()).any(|o| self.at_war(ni, o)) {
                wgt *= 0.6;
            }
            wgt *= (-b.center().dist(from) / 140.0).exp();
            if wgt > 0.0 {
                total += wgt;
                cands.push((wgt, i));
            }
        }
        if cands.is_empty() {
            return None;
        }
        let mut r = self.rng.f32() * total;
        for (wgt, i) in cands.iter() {
            r -= wgt;
            if r <= 0.0 {
                return Some(self.bh(*i));
            }
        }
        Some(self.bh(cands.last().unwrap().1))
    }

    fn new_caravan(&mut self, kind: CaravanKind, pos: V2, value: f32, max_trips: u8) -> Caravan {
        Caravan {
            kind,
            pos,
            dir: v2(1.0, 0.0),
            path: Vec::new(),
            path_i: 0,
            need_path: false,
            dest: None,
            goal: pos,
            stay: self.rng.range(0.5, 2.0),
            trips: 0,
            max_trips,
            leaving: false,
            gone: false,
            value,
        }
    }

    /// 市での商いを終えた隊商の次の行き先
    fn caravan_next(&mut self, i: usize) {
        let (kind, pos, dest, trips, max_trips, leaving) = {
            let c = &self.trade.caravans[i];
            (c.kind, c.pos, c.dest, c.trips, c.max_trips, c.leaving)
        };
        if leaving {
            self.trade.caravans[i].gone = true;
            return;
        }
        let retire = match kind {
            CaravanKind::Merchant => {
                let n = self.trade.count(|k| k == CaravanKind::Merchant);
                n > self.trade.target && self.rng.chance(0.5)
            }
            _ => trips >= max_trips,
        };
        let next = if retire {
            None
        } else {
            match kind {
                CaravanKind::Merchant => self.pick_city(pos, move |w, p| w.same_land(pos, p), dest),
                CaravanKind::Western(r) => self.pick_city(pos, move |w, p| in_region(r as usize, p) && w.same_land(pos, p), dest),
                CaravanKind::Tribe(_) => None,
            }
        };
        let c = &mut self.trade.caravans[i];
        c.trips += 1;
        c.stay = 0.0;
        match next {
            Some(h) => {
                c.dest = Some(h);
                c.goal = self.blds[h.idx as usize].center();
                c.need_path = true;
            }
            None => match kind {
                CaravanKind::Merchant => c.gone = true,
                CaravanKind::Western(r) => {
                    let (lo, la) = ROUTES[r as usize].entry;
                    c.leaving = true;
                    c.goal = ll_v2(lo, la);
                    c.need_path = true;
                }
                CaravanKind::Tribe(t) => {
                    c.leaving = true;
                    c.goal = ll_v2(TRIBES[t as usize].lon, TRIBES[t as usize].lat);
                    c.need_path = true;
                }
            },
        }
        if c.leaving && c.need_path {
            let g = self.passable_near(self.trade.caravans[i].goal);
            self.trade.caravans[i].goal = g;
        }
    }

    fn spawn_caravans(&mut self) {
        let total: f32 = (0..NATIONS.len()).filter(|&i| self.nations[i].alive).map(|i| self.production(i)).sum();
        // 中原全体の生産量に比例して商人が増える
        self.trade.target = ((total * 0.16) as usize).clamp(6, 90);
        let yb = self.year_ad();
        let open: Vec<usize> = (0..ROUTES.len()).filter(|&r| route_open(r, yb)).collect();
        self.trade.target_w = if open.is_empty() { 0 } else { ((total * 0.012) as usize + open.len()).clamp(1, 12) };
        let nm = self.trade.count(|k| k == CaravanKind::Merchant);
        if nm < self.trade.target {
            let c = self.v2_center();
            if let Some(h) = self.pick_city(c, |_, _| true, None) {
                let p = self.blds[h.idx as usize].center();
                let off = v2(self.rng.range(-2.0, 2.0), self.rng.range(1.5, 2.5));
                let p = self.passable_near(p + off);
                let mut cv = self.new_caravan(CaravanKind::Merchant, p, 1.0, 255);
                cv.dest = Some(h);
                self.trade.caravans.push(cv);
            }
        }
        let nw = self.trade.count(|k| matches!(k, CaravanKind::Western(_)));
        if nw < self.trade.target_w && self.rng.chance(0.25) {
            // 道ごとの訪問先の生産力で選ぶ
            let mut best: Vec<(f32, usize)> = Vec::new();
            for &r in open.iter() {
                let mut s = 0.0;
                for b in self.blds.iter() {
                    if b.alive && b.kind.city() && b.done() && (b.nation as usize) < NATIONS.len() && in_region(r, b.center()) {
                        let ni = b.nation as usize;
                        s += (self.production(ni) + 0.5) / self.nations[ni].cities.max(1) as f32;
                    }
                }
                if s > 0.0 {
                    best.push((s, r));
                }
            }
            let tot: f32 = best.iter().map(|b| b.0).sum();
            let mut x = self.rng.f32() * tot;
            let mut pick = None;
            for (s, r) in best {
                x -= s;
                pick = Some(r);
                if x <= 0.0 {
                    break;
                }
            }
            if let Some(r) = pick {
                let (lo, la) = ROUTES[r].entry;
                let p = self.passable_near(ll_v2(lo, la));
                let trips = 2 + (self.rng.f32() * 3.0) as u8;
                let mut cv = self.new_caravan(CaravanKind::Western(r as u8), p, 3.0, trips);
                if let Some(h) = self.pick_city(p, move |_, q| in_region(r, q), None) {
                    cv.dest = Some(h);
                    cv.goal = self.blds[h.idx as usize].center();
                    cv.need_path = true;
                    cv.stay = 0.0;
                    self.trade.caravans.push(cv);
                    if self.blds[h.idx as usize].nation as usize == self.player && self.rng.chance(0.5) {
                        let s = tf!("遠方の商人が{}を通ってやって来た", "Foreign merchants arrived by the {}", "远方商人经{}而来", tx(ROUTES[r].name));
                        self.push_msg(s, [0.85, 0.75, 1.0]);
                    }
                }
            }
        }
    }

    fn v2_center(&self) -> V2 {
        v2(MAP_W as f32 / 2.0, MAP_H as f32 / 2.0)
    }

    // ------------------------------------------------------------ 毎年：部族の動き・傭兵の満了
    pub(super) fn yearly_trade(&mut self) {
        let yb = self.year_ad();
        // 傭兵の契約満了
        let mut done = Vec::new();
        for (i, u) in self.units.iter().enumerate() {
            if u.alive && u.merc_until > 0.0 && self.years >= u.merc_until {
                done.push(i);
            }
        }
        if done.iter().any(|&i| self.units[i].nation as usize == self.player) {
            let t = self.units[done[0]].tribe as usize;
            let s = tf!("{}の傭兵は契約を終えて去った", "The {} mercenaries finished their contract and left", "{}佣兵合约期满离去", TRIBES.get(t).map_or(t!("部族", "tribal", "部族"), |d| tx(d.name)));
            self.push_msg(s, [0.85, 0.85, 0.7]);
        }
        for i in done {
            self.kill_unit(i);
        }
        for t in 0..TRIBES.len() {
            if !tribe_active(t, yb) || TRIBES[t].raid == 0 || self.years < self.trade.tribes[t].next {
                continue;
            }
            self.trade.tribes[t].next = self.years + self.rng.range(7.0, 15.0);
            self.tribe_act(t);
        }
    }

    /// 国ではない兵が動く：近くの国へ略奪に出るか、商いに来る
    fn tribe_act(&mut self, t: usize) {
        self.tribe_raid(t, false);
    }

    /// 略奪に出る（force なら必ず。史実の出来事で攻め込むとき）。出たら true
    pub(super) fn tribe_raid(&mut self, t: usize, force: bool) -> bool {
        let home = self.tribe_home(t);
        let mask = TRIBES[t].raid;
        let mut cands: Vec<(f32, usize)> = Vec::new();
        for (i, b) in self.blds.iter().enumerate() {
            if b.alive && b.kind.dropsite() && b.done() && (b.nation as usize) < NATIONS.len() && mask & nbit(b.nation as usize) != 0 && self.same_land(b.center(), home) {
                let d = b.center().dist(home);
                if d < 110.0 {
                    // 村ほど狙われやすい
                    cands.push((if b.kind == BKind::Village { d * 0.6 } else { d }, i));
                }
            }
        }
        if cands.is_empty() {
            return false;
        }
        cands.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        cands.truncate(4);
        let k = (self.rng.f32() * self.rng.f32() * cands.len() as f32) as usize;
        let bi = cands[k.min(cands.len() - 1)].1;
        let target = self.blds[bi].nation as usize;
        let city = self.bh(bi);
        let size = (3.0 + self.years / 40.0).round() as usize + if force { 3 } else { 0 };
        let power = self.tribe_power();
        // 守りが薄い国ほど襲われる。雇われている国は襲わない
        let defense = self.nations[target].mil_power + 1.0;
        let raid_power = size as f32 * power;
        let mut p = TRIBES[t].host * (raid_power / (defense * 0.6)).clamp(0.3, 1.6);
        if self.trade.tribes[t].contract[target] > self.years || (target == self.player && self.protect_player) {
            p = 0.0;
        }
        if self.trade.bands.iter().any(|b| b.tribe == t) && !force {
            p = 0.0;
        }
        if force && !(target == self.player && self.protect_player) {
            p = 1.0;
        }
        let tname = tx(TRIBES[t].name);
        let nname = tx(self.nations[target].name);
        if self.rng.chance(if force { p } else { p.min(0.9) }) {
            let mut hs = Vec::new();
            let us = TRIBES[t].units;
            for k in 0..size {
                let kind = us[k % us.len()];
                let off = v2(self.rng.range(-1.5, 1.5), self.rng.range(-1.5, 1.5));
                let pos = self.passable_near(home + off);
                let h = self.spawn_unit(TRIBE, kind, pos, power);
                self.unit_mut(h).unwrap().tribe = t as u8;
                hs.push(h);
            }
            let goal = self.blds[bi].center();
            self.order_move(&hs, goal, true);
            self.trade.bands.push(Band { tribe: t, target, city, units: hs, phase: 0, until: self.years + 8.0, home });
            self.trade.tribes[t].last = tf!("{}へ略奪に向かった", "Set out to raid {}", "前往劫掠{}", nname);
            let place = self.town_name(bi);
            if target == self.player {
                let s = tf!("{}の兵{}が{}へ略奪に向かってくる！", "{} troops ({}) are coming to raid {}!", "{}的士兵{}名正前来劫掠{}！", tname, size, place);
                self.push_msg(s.clone(), [1.0, 0.4, 0.3]);
                let hint = if TRIBES[t].hire & nbit(target) != 0 { t!("傭兵として雇えば襲われなくなる。", "Hiring them as mercenaries stops their raids.", "雇其为佣兵即可免遭劫掠。") } else { "" };
                let text = tf!("{}（{}）。街と村を守れ。{}", "{} ({}). Defend your towns and villages. {}", "{}（{}）。守住城镇与村庄。{}", s, tx(TRIBES[t].desc), hint);
                self.queue_banner(Banner { title: tf!("{}の来寇", "{} Raid", "{}来犯", tname), text, t: 9.0 });
            } else {
                self.push_msg(tf!("{}が{}へ略奪に向かった", "The {} set out to raid {}", "{}前往劫掠{}", tname, place), [0.95, 0.7, 0.5]);
            }
            return true;
        } else if force || TRIBES[t].hire == 0 {
            return false;
        } else {
            let mut cv = self.new_caravan(CaravanKind::Tribe(t as u8), home, 1.6, 1);
            cv.dest = Some(city);
            cv.goal = self.blds[bi].center();
            cv.need_path = true;
            cv.stay = 0.0;
            self.trade.caravans.push(cv);
            self.trade.tribes[t].last = tf!("{}へ雇い主を探しに来た", "Came to {} looking for an employer", "前往{}寻找雇主", nname);
            if target == self.player {
                self.push_msg(tf!("{}が雇い主を探してやって来る", "The {} are coming, looking for an employer", "{}前来寻找雇主", tname), [0.8, 0.9, 0.7]);
            }
        }
        false
    }

    /// 略奪の一団：城を目指し、略奪したら（または時間切れで）引き上げる
    fn update_bands(&mut self) {
        let mut k = 0;
        while k < self.trade.bands.len() {
            let alive: Vec<Handle> = self.trade.bands[k].units.iter().copied().filter(|&h| self.unit(h).is_some()).collect();
            self.trade.bands[k].units = alive.clone();
            let b = &self.trade.bands[k];
            let (t, target, phase, home, city, until) = (b.tribe, b.target, b.phase, b.home, b.city, b.until);
            if alive.is_empty() {
                if phase == 0 {
                    self.trade.tribes[t].last = tf!("{}に撃退された", "Driven off by {}", "被{}击退", tx(self.nations[target].name));
                    if target == self.player {
                        self.push_msg(tf!("{}の来寇を撃退した", "We drove off the {} raid", "击退了{}的来犯", tx(TRIBES[t].name)), [0.5, 1.0, 0.5]);
                    }
                }
                self.trade.bands.remove(k);
                continue;
            }
            if phase == 0 {
                let city_ok = self.bld(city).map_or(false, |c| c.nation as usize == target);
                if self.years > until || !city_ok || !self.nations[target].alive {
                    self.trade.bands[k].phase = 1;
                    self.trade.bands[k].until = self.years + 15.0;
                    self.order_move(&alive, home, false);
                } else {
                    let idle: Vec<Handle> = alive.iter().copied().filter(|&h| self.unit(h).map_or(false, |u| u.order == Order::Idle)).collect();
                    if !idle.is_empty() {
                        self.order_attack(&idle, Target::Bld(city));
                    }
                }
            } else {
                let mut stay = Vec::new();
                for &h in alive.iter() {
                    let u = self.unit(h).unwrap();
                    if u.pos.dist(home) < 4.0 || self.years > until {
                        let i = h.idx as usize;
                        self.kill_unit(i);
                    } else if u.order == Order::Idle {
                        stay.push(h);
                    }
                }
                if !stay.is_empty() {
                    self.order_move(&stay, home, false);
                }
            }
            k += 1;
        }
    }

    /// 部族が城邑を陥とした：占領はせず、蓄えを奪って引き上げる
    pub(super) fn plunder_city(&mut self, bi: usize) {
        let vn = self.blds[bi].nation as usize;
        {
            let b = &mut self.blds[bi];
            b.hp = b.kind.max_hp() * 0.35;
        }
        let n = &mut self.nations[vn];
        let mut lost = [0.0; 3];
        for k in 0..3 {
            lost[k] = n.res[k] * 0.35 * n.tm.plunder;
            n.res[k] -= lost[k];
        }
        n.coin *= 1.0 - 0.3 * n.tm.plunder;
        let mut who = t!("略奪者", "Raiders", "劫掠者");
        for b in self.trade.bands.iter_mut() {
            if b.target == vn && b.phase == 0 {
                b.phase = 1;
                b.until = self.years + 15.0;
                who = tx(TRIBES[b.tribe].name);
                self.trade.tribes[b.tribe].last = tf!("{}を略奪して引き上げた", "Raided {} and withdrew", "劫掠{}后撤回", tx(self.nations[vn].name));
            }
        }
        let backs: Vec<(Vec<Handle>, V2)> = self.trade.bands.iter().filter(|b| b.target == vn && b.phase == 1).map(|b| (b.units.clone(), b.home)).collect();
        for (hs, home) in backs {
            self.order_move(&hs, home, false);
        }
        if vn == self.player {
            let s = tf!("{}に街を略奪された（糧{:.0}・木{:.0}・鉄{:.0}と金貨の三割を失った）", "The {} sacked one of our towns (lost food {:.0}, wood {:.0}, iron {:.0} and 30% of our gold)", "{}劫掠了我国城镇（损失粮{:.0}、木{:.0}、铁{:.0}及三成金币）", who, lost[0], lost[1], lost[2]);
            self.push_msg(s, [1.0, 0.35, 0.3]);
        } else {
            self.push_msg(tf!("{}が{}の街を略奪した", "The {} sacked a town of {}", "{}劫掠了{}的城镇", who, tx(self.nations[vn].name)), [0.95, 0.7, 0.5]);
        }
    }

    /// AI：金貨で足りない資源を買い、戦時には傭兵を雇う（フランスは海を渡るためにガレー船も）
    pub fn ai_trade(&mut self, n: usize, rng: &mut Rng) {
        let res = self.nations[n].res;
        let coin = self.nations[n].coin;
        let at_war = (0..NATIONS.len()).any(|o| self.at_war(n, o)) || self.trade.bands.iter().any(|b| b.target == n && b.phase == 0);
        let mercs = self.units.iter().filter(|u| u.alive && u.nation as usize == n && u.merc_until > 0.0).count();
        if at_war && mercs < 10 && coin > self.merc_cost() * 1.3 && rng.chance(0.12) {
            let ships = self.units.iter().filter(|u| u.alive && u.nation as usize == n && u.kind.naval()).count();
            let mut ts: Vec<usize> = (0..TRIBES.len()).filter(|&t| self.can_hire(n, t).is_ok()).collect();
            // 船が足りなければ船を、そうでなければ兵を
            ts.sort_by_key(|&t| if TRIBES[t].naval == (ships < 2) { 0 } else { 1 });
            if let Some(&t) = ts.first() {
                if !TRIBES[t].naval || ships < 4 {
                    let _ = self.hire_mercs(n, t);
                    return;
                }
            }
        }
        let keep = if at_war { 0.0 } else { 80.0 };
        if coin - keep >= self.market_price(n, 2) {
            // 必要量に対して最も乏しい資源
            let need = [200.0, 150.0, 120.0];
            let mut k = 0;
            for j in 1..3 {
                if res[j] / need[j] < res[k] / need[k] {
                    k = j;
                }
            }
            if res[k] < need[k] * 2.0 {
                let _ = self.buy_resource(n, k);
            }
        }
    }
}
