//! ゲーム世界のシミュレーション（ユニット・建物・経済・戦闘・外交・イベント）

use crate::ai::AiState;
use crate::geo::*;
use crate::path::PathFinder;
use crate::rng::Rng;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

mod history;
mod trade;
mod traits;
mod descent;
mod heroes;
mod save;
mod exploration;
mod units;
mod conquest;
mod naval;
pub use history::{Chron, Person, HISTORY, INITIAL_HOUSE};
#[cfg(test)]
pub use history::Fx;
pub use trade::*;
pub use traits::*;
pub use descent::*;
pub use heroes::*;
pub use units::*;
pub use save::SAVE_VERSION;
pub use crate::lang::{sep, tx};

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}
pub fn v2(x: f32, y: f32) -> V2 {
    V2 { x, y }
}
impl std::ops::Add for V2 {
    type Output = V2;
    fn add(self, o: V2) -> V2 {
        v2(self.x + o.x, self.y + o.y)
    }
}
impl std::ops::Sub for V2 {
    type Output = V2;
    fn sub(self, o: V2) -> V2 {
        v2(self.x - o.x, self.y - o.y)
    }
}
impl std::ops::Mul<f32> for V2 {
    type Output = V2;
    fn mul(self, s: f32) -> V2 {
        v2(self.x * s, self.y * s)
    }
}
impl V2 {
    pub fn len(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn dist(self, o: V2) -> f32 {
        (self - o).len()
    }
    pub fn norm(self) -> V2 {
        let l = self.len();
        if l > 1e-6 {
            self * (1.0 / l)
        } else {
            v2(0.0, 0.0)
        }
    }
    pub fn tile(self) -> (i32, i32) {
        (self.x.floor() as i32, self.y.floor() as i32)
    }
}

/// 1年の長さ（秒）
pub const SEC_PER_YEAR: f32 = 30.0;
/// ランカスター朝の戦争（1415年〜）は時間の流れを少し遅くする（1年＝39秒）
pub const EMPIRE_SLOW: f32 = 1.3;
/// 始まりの年（フィリップ6世がギュイエンヌを没収し、エドワード3世がフランス王位を求めた）
pub const START_AD: f32 = 1337.0;
/// ランカスター朝の戦争（ヘンリー5世のノルマンディー上陸）
pub const EMPIRE_AD: f32 = 1415.0;
/// 百年戦争の終わりの年（カスティヨンの戦い）。この年が暮れても決着がつかなければ、ジャンヌ・ダルクの亡霊が両国を滅ぼす
pub const END_AD: f32 = 1453.0;
pub const DT: f32 = 1.0 / 30.0;
pub const NONE: u8 = 255;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Era {
    /// エドワード朝・カロリング朝の戦争（1337〜1414年）
    Rise,
    /// ランカスター朝の戦争（1415年〜）
    Empire,
}

/// 兵の大分類（戦い方・相性・見た目）。個々の兵種は units.rs の UNIT_DEFS
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnitKind {
    Worker,
    Infantry,
    Archer,
    Cavalry,
    Ship,
}

impl UnitKind {
    pub fn military(self) -> bool {
        self != UnitKind::Worker
    }
    pub fn naval(self) -> bool {
        self == UnitKind::Ship
    }
    /// 視界半径（タイル）
    pub fn vision(self) -> i32 {
        match self {
            UnitKind::Worker => 6,
            UnitKind::Infantry => 7,
            UnitKind::Archer => 8,
            UnitKind::Cavalry => 9,
            UnitKind::Ship => 9,
        }
    }
    /// 大分類の名前
    pub fn name(self) -> &'static str {
        match self {
            UnitKind::Worker => t!("民", "Worker", "民"),
            UnitKind::Infantry => t!("歩兵", "Infantry", "步兵"),
            UnitKind::Archer => t!("弓兵", "Archers", "弓兵"),
            UnitKind::Cavalry => t!("騎兵", "Cavalry", "骑兵"),
            UnitKind::Ship => t!("船", "Ships", "船"),
        }
    }
    pub fn sight(self) -> f32 {
        match self {
            UnitKind::Worker => 4.0,
            UnitKind::Archer | UnitKind::Ship => 7.0,
            _ => 6.0,
        }
    }
}

/// 相性補正（攻め手の兵種 → 守り手の大分類）
pub fn bonus(att: &UnitDef, def: UnitKind) -> f32 {
    if let Some((k, m)) = att.extra {
        if k == def {
            return m;
        }
    }
    let ranged = att.range > 1.5;
    match (att.class, def) {
        (UnitKind::Infantry, UnitKind::Cavalry) => 1.6,
        (UnitKind::Cavalry, UnitKind::Archer) if !ranged => 1.7,
        (UnitKind::Cavalry, UnitKind::Infantry) if ranged => 1.25,
        (UnitKind::Archer, UnitKind::Infantry) => 1.4,
        (UnitKind::Cavalry, UnitKind::Worker) => 1.3,
        (UnitKind::Ship, UnitKind::Ship) => 1.3,
        _ => 1.0,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BKind {
    Capital,
    Town,
    Barracks,
    Farm,
    /// 要塞：近づく敵兵を射る（数に上限あり）
    Fort,
    /// 港：船を造る（海に面した自国領に建てる）
    Port,
    /// 村：民が建てる。糧を生み、資源の運び先になり、民・兵の上限を少し上げる。敵に壊されると略奪される
    Village,
}

/// 城邑が無くても持てる民・兵の数（滅んだ国から分けられる量）
pub const BASE_CAP: u32 = 4;

/// 城邑どうしがこれより近ければ、領土が接していなくても隣国とみなす（マス）
pub const NEIGHBOR_RANGE: f32 = 45.0;

/// 要塞の射程・攻撃間隔
pub const FORT_RANGE: f32 = 9.0;
pub const FORT_COOLDOWN: f32 = 1.5;

impl BKind {
    pub fn name(self) -> &'static str {
        match self {
            BKind::Capital => t!("都", "Capital", "都城"),
            BKind::Town => t!("邑", "Town", "邑"),
            BKind::Barracks => t!("兵営", "Barracks", "兵营"),
            BKind::Farm => t!("田", "Field", "田"),
            BKind::Fort => t!("要塞", "Fort", "要塞"),
            BKind::Port => t!("港", "Harbor", "港口"),
            BKind::Village => t!("村", "Village", "村庄"),
        }
    }
    pub fn cost(self) -> [f32; 3] {
        match self {
            BKind::Capital => [0.0, 0.0, 0.0],
            BKind::Town => [100.0, 250.0, 80.0],
            BKind::Barracks => [0.0, 150.0, 30.0],
            BKind::Farm => [0.0, 60.0, 0.0],
            BKind::Fort => [300.0, 400.0, 300.0],
            BKind::Port => [50.0, 200.0, 40.0],
            BKind::Village => [60.0, 140.0, 0.0],
        }
    }
    pub fn max_hp(self) -> f32 {
        match self {
            BKind::Capital => 3000.0,
            BKind::Town => 1600.0,
            BKind::Barracks => 1200.0,
            BKind::Farm => 300.0,
            BKind::Fort => 2500.0,
            BKind::Port => 1200.0,
            BKind::Village => 600.0,
        }
    }
    pub fn build_time(self) -> f32 {
        match self {
            BKind::Capital => 60.0,
            BKind::Town => 40.0,
            BKind::Barracks => 20.0,
            BKind::Farm => 8.0,
            BKind::Fort => 30.0,
            BKind::Port => 22.0,
            BKind::Village => 20.0,
        }
    }
    pub fn radius(self) -> i32 {
        match self {
            BKind::Capital => 12,
            BKind::Town => 9,
            BKind::Village => 4,
            _ => 0,
        }
    }
    pub fn city(self) -> bool {
        matches!(self, BKind::Capital | BKind::Town)
    }
    /// 資源の運び先になるか（街と村）
    pub fn dropsite(self) -> bool {
        matches!(self, BKind::Capital | BKind::Town | BKind::Village)
    }
    pub fn blocks(self) -> bool {
        self != BKind::Farm
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Handle {
    pub idx: u32,
    pub gen: u32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Target {
    Unit(Handle),
    Bld(Handle),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Src {
    Farm(Handle),
    Forest(i32, i32),
    Mine(usize),
}
impl Src {
    pub fn res(self) -> usize {
        match self {
            Src::Farm(_) => 0,
            Src::Forest(..) => 1,
            Src::Mine(_) => 2,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Order {
    Forestry { region: usize, target: Option<usize>, cursor: usize },
    Idle,
    Explore(Option<V2>),
    Move(V2),
    AttackMove(V2),
    Attack(Target),
    /// phase 0:資源へ 1:採集中 2:集積地へ
    Gather { src: Src, phase: u8 },
    Build(Handle),
    /// 船に乗る
    Board(Handle),
    /// 船：この地点の近くの岸へ行き、乗せた兵を降ろす
    Unload(V2),
    /// 海の向こうへ：岸で船を待ち、船で渡ってから目的地へ（目的地, 攻撃移動か）
    Ferry(V2, bool),
    /// 船：海の向こうへ向かう兵を、岸と岸の間で往復して運ぶ（目的地, 攻撃移動か）
    Shuttle(V2, bool),
}

pub struct Unit {
    pub gen: u32,
    pub alive: bool,
    pub nation: u8,
    pub kind: UnitKind,
    /// 兵種（UNIT_DEFS の番号）
    pub ut: u8,
    pub pos: V2,
    pub hp: f32,
    pub power: f32,
    pub order: Order,
    pub resume: Option<V2>,
    pub path: Vec<(i16, i16)>,
    pub path_i: usize,
    pub path_goal: (i32, i32),
    pub path_pending: bool,
    pub path_done: bool,
    pub cooldown: f32,
    pub carry_kind: usize,
    pub carry: f32,
    pub scan_t: f32,
    pub dir: V2,
    pub flash: f32,
    pub selected: bool,
    pub last_attacker: Option<Handle>,
    /// 部族の兵・傭兵の出身部族（NONE=なし）
    pub tribe: u8,
    /// 傭兵の契約満了年（0=傭兵ではない）
    pub merc_until: f32,
    /// 天の兵（英霊にしか傷つけられない）
    pub divine: bool,
    /// 英霊（HEROES の番号、NONE=英霊ではない）
    pub hero: u8,
    /// 建設の予約（今の建設を終えたら順に取りかかる）
    pub build_queue: VecDeque<Handle>,
    /// 船に乗っている兵（船のみ）
    pub cargo: Vec<Handle>,
    /// 乗っている船（乗船中は地図に現れない）
    pub aboard: Option<Handle>,
}

impl Unit {
    pub fn def(&self) -> &'static UnitDef {
        udef(self.ut)
    }
    pub fn max_hp(&self) -> f32 {
        self.def().hp * self.power
    }
    pub fn reset_nav(&mut self) {
        self.path.clear();
        self.path_i = 0;
        self.path_goal = (-9, -9);
        self.path_done = false;
    }
    /// 地図の上にいる（船に乗っていない）
    pub fn on_map(&self) -> bool {
        self.alive && self.aboard.is_none()
    }
}

pub struct Building {
    pub gen: u32,
    pub alive: bool,
    pub nation: u8,
    /// 築いた国（占領されても変わらない。服属の判定に使う）
    pub origin: u8,
    pub kind: BKind,
    pub tx: i32,
    pub ty: i32,
    pub hp: f32,
    pub progress: f32,
    pub queue: VecDeque<u8>,
    pub queue_t: f32,
    pub rally: Option<V2>,
    pub cooldown: f32,
    /// 最初からある街の番号（TOWNS。NONE=街ではない）
    pub town: u8,
    /// 治める諸侯（LORDS）
    pub lord: u8,
}

impl Building {
    pub fn center(&self) -> V2 {
        v2(self.tx as f32 + 1.0, self.ty as f32 + 1.0)
    }
    pub fn done(&self) -> bool {
        self.progress >= 1.0
    }
    pub fn dist_to_footprint(&self, p: V2) -> f32 {
        let cx = p.x.clamp(self.tx as f32, self.tx as f32 + 2.0);
        let cy = p.y.clamp(self.ty as f32, self.ty as f32 + 2.0);
        p.dist(v2(cx, cy))
    }
}

pub struct Nation {
    pub name: &'static str,
    pub color: [f32; 3],
    pub bias: f32,
    pub alive: bool,
    pub appeared: bool,
    pub res: [f32; 3],
    pub aggression: f32,
    pub hegemon_cd: f32,
    pub capital: Option<Handle>,
    pub workers: u32,
    pub soldiers: u32,
    pub q_workers: u32,
    pub q_soldiers: u32,
    pub cities: u32,
    pub mil_power: f32,
    pub eco_power: f32,
    pub tiles: u32,
    pub upkeep: f32,
    pub strength: f32,
    pub earned: [f32; 3],
    pub income: [f32; 3],
    pub home: V2,
    /// 君主家（国姓）
    pub house: &'static str,
    /// 仕えている人物
    pub figures: Vec<Person>,
    /// 人物による生産・武の倍率
    pub eco_mul: f32,
    pub mil_mul: f32,
    /// 銭（商人からの税収）
    pub coin: f32,
    pub coin_earned: f32,
    pub coin_income: f32,
    /// 税率 0:軽税 1:中税 2:重税
    pub tax: u8,
    /// 国の特徴による補正
    pub tm: TraitMods,
    /// 滅んだ国から分けられた人口上限（民・兵それぞれに加わる）
    pub cap_bonus: u32,
    /// 完成した村の数
    pub villages: u32,
}

impl Nation {
    pub fn worker_cap(&self) -> u32 {
        (16 + self.cities * 2 / 3 + self.villages).min(100) + self.cap_bonus
    }
    pub fn mil_cap(&self) -> u32 {
        (60 + self.cities + self.villages * 3).min(200) + self.cap_bonus
    }
    /// 築ける要塞の数（街3つごとに1つ、最大6）
    pub fn fort_cap(&self) -> u32 {
        (1 + self.cities / 3).min(6)
    }
}

/// プレイヤーへの和睦の申し入れ（受諾か拒否かを選ぶ）
#[derive(Clone, Debug)]
pub struct PeaceAsk {
    pub from: usize,
    pub years: f32,
    pub why: String,
    /// 身代金の請求（英雄の番号, 額）。None なら和睦の申し入れ
    pub ransom: Option<(usize, f32)>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CoalKind {
    Hegemon,
    Hezong,
}

pub struct Coalition {
    pub kind: CoalKind,
    pub target: usize,
    pub leader: usize,
    pub members: Vec<usize>,
    pub end: f32,
}

pub struct Msg {
    pub text: String,
    pub time: f32,
    pub color: [f32; 3],
}

pub struct Shot {
    pub a: V2,
    pub b: V2,
    pub t: f32,
}

pub struct Banner {
    pub title: String,
    pub text: String,
    pub t: f32,
}

pub const GCELL: i32 = 4;
pub const GW: i32 = (MAP_W as i32 + GCELL - 1) / GCELL;
pub const GH: i32 = (MAP_H as i32 + GCELL - 1) / GCELL;

/// 苗を植えてから伐採できる森になるまでの対局内秒数。
pub const FOREST_GROWTH_TIME: f32 = 30.0;
pub const PLANT_FOREST_COST: [f32; 3] = [20.0, 0.0, 0.0];
const FORESTRY_REACH: f32 = 0.85;
const MAX_FORESTRY_REGIONS: usize = 1024;
const MAX_FORESTRY_SIDE: i32 = 64;

#[derive(Clone, Copy)]
pub struct ForestryRegion {
    pub nation: usize,
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl ForestryRegion {
    fn valid(self) -> bool {
        Map::inb(self.x0, self.y0) && Map::inb(self.x1, self.y1)
            && (0..MAX_FORESTRY_SIDE).contains(&(self.x1 - self.x0))
            && (0..MAX_FORESTRY_SIDE).contains(&(self.y1 - self.y0))
    }

    pub fn contains(self, x: i32, y: i32) -> bool {
        (self.x0..=self.x1).contains(&x) && (self.y0..=self.y1).contains(&y)
    }

    fn overlaps(self, other: Self) -> bool {
        self.x0 <= other.x1 && other.x0 <= self.x1 && self.y0 <= other.y1 && other.y0 <= self.y1
    }

    fn width(self) -> usize {
        (self.x1 - self.x0 + 1) as usize
    }

    fn area(self) -> usize {
        self.width() * (self.y1 - self.y0 + 1) as usize
    }

    fn tile(self, offset: usize) -> usize {
        Map::idx(self.x0 + (offset % self.width()) as i32, self.y0 + (offset / self.width()) as i32)
    }

    fn offset_near(self, pos: V2) -> usize {
        let (x, y) = pos.tile();
        let x = x.clamp(self.x0, self.x1) - self.x0;
        let y = y.clamp(self.y0, self.y1) - self.y0;
        y as usize * self.width() + x as usize
    }
}

fn forestry_position(tile: usize) -> V2 {
    v2((tile % MAP_W) as f32 + 0.5, (tile / MAP_W) as f32 + 0.5)
}

pub struct World {
    pub map: Map,
    pub cost: Vec<f32>,
    /// 船の移動コスト（海・湖・大河）
    pub wcost: Vec<f32>,
    /// 陸の連結成分（島ごとに違う番号。船が要るかの判定）
    pub land_region: Vec<u32>,
    /// 海の連結成分
    pub sea_region: Vec<u32>,
    /// 粗い升目（GCELL×GCELL マス）ごとに、陸・海を通れるか（海路と陸路のざっくりした比較に使う）
    pub coarse_land: Vec<bool>,
    pub coarse_sea: Vec<bool>,
    pub bmap: Vec<u32>,
    pub owner: Vec<u8>,
    pub units: Vec<Unit>,
    pub blds: Vec<Building>,
    free_units: Vec<u32>,
    free_blds: Vec<u32>,
    pub nations: Vec<Nation>,
    pub war: Vec<Vec<bool>>,
    pub war_start: Vec<Vec<f32>>,
    pub truce: Vec<Vec<f32>>,
    pub peace_offer: Vec<bool>,
    /// 和睦の申し入れを断られた国は、この年まで申し入れない
    pub offer_cd: Vec<f32>,
    /// プレイヤーの返答を待つ和睦の申し入れ
    pub peace_asks: VecDeque<PeaceAsk>,
    pub years: f32,
    pub era: Era,
    pub player: usize,
    pub coalitions: Vec<Coalition>,
    pub hezong_next: f32,
    pub msgs: Vec<Msg>,
    pub banner: Option<Banner>,
    banner_queue: VecDeque<Banner>,
    /// 次に起こる史実イベント
    hist_i: usize,
    /// 史書（起きた出来事・起こらなかった出来事）
    pub chronicle: Vec<Chron>,
    pub shots: Vec<Shot>,
    pub rng: Rng,
    pf: PathFinder,
    path_queue: VecDeque<Handle>,
    pub grid: Vec<Vec<u32>>,
    pub territory_dirty: bool,
    pub territory_version: u32,
    /// 国ごとの領土のマス数（recompute_territory で数え直す）
    tile_count: Vec<u32>,
    /// 国ごとの完成した城邑（毎ステップ作り直す。運び先を探すのに全建物を見ないため）
    city_list: Vec<Vec<u32>>,
    /// 押し合いを1ステップおきにするための拍子
    sep_tick: bool,
    pub terrain_dirty: Vec<(i32, i32)>,
    /// マス番号 → 森になる対局内時刻。
    pub saplings: BTreeMap<usize, f32>,
    pub forestry_regions: Vec<ForestryRegion>,
    // Trees from older saves have no tending requirement.
    managed_saplings: BTreeSet<usize>,
    pub game_over: Option<(bool, String)>,
    /// チュートリアル中はプレイヤーへの宣戦を禁じる
    pub protect_player: bool,
    /// プレイヤーが部隊に命令した回数
    pub player_orders: u32,
    /// ジャンヌ・ダルクの亡霊の降臨（1453年までに決着がつかなかった）
    pub descent: Option<Descent>,
    /// 捕らえられた英雄（身代金を待つ）
    pub captives: Vec<Captive>,
    /// 史実の英雄の登場を確かめた年
    hero_year: i32,
    /// 撮影用：プレイヤーの国も AI が動かす
    pub ai_player: bool,
    /// 英霊モード（ゲーム開始時に選ぶ。既定は無効）
    pub heroic: bool,
    pub time: f32,
    pub ai: Vec<AiState>,
    /// 商人・西域の商人・山岳民と遊牧民
    pub trade: Trade,
    stat_t: f32,
    income_t: f32,
    year_tick: f32,
}

impl World {
    pub fn new(seed: u32) -> World {
        let map = generate_map(seed);
        let n = MAP_W * MAP_H;
        let mut cost = vec![0.0; n];
        let mut wcost = vec![0.0; n];
        for i in 0..n {
            cost[i] = map.terrain[i].move_cost();
            wcost[i] = map.terrain[i].sail_cost();
        }
        let (land_region, _) = components(|i| map.terrain[i].passable());
        let (sea_region, _) = components(|i| map.terrain[i].sail_cost().is_finite());
        // 粗い升目：半分以上のマスが通れれば通れるとみなす（細い海峡・山脈はざっくりでよい）
        let coarse = |f: &dyn Fn(Terrain) -> bool| -> Vec<bool> {
            let mut v = vec![false; (GW * GH) as usize];
            for gy in 0..GH {
                for gx in 0..GW {
                    let mut k = 0;
                    for dy in 0..GCELL {
                        for dx in 0..GCELL {
                            let (x, y) = (gx * GCELL + dx, gy * GCELL + dy);
                            if Map::inb(x, y) && f(map.terrain[Map::idx(x, y)]) {
                                k += 1;
                            }
                        }
                    }
                    v[(gy * GW + gx) as usize] = k * 2 >= GCELL * GCELL;
                }
            }
            v
        };
        let coarse_land = coarse(&|t: Terrain| t.passable());
        let coarse_sea = coarse(&|t: Terrain| t.sail_cost().is_finite());
        let nn = NATIONS.len();
        let mut w = World {
            map,
            cost,
            wcost,
            land_region,
            sea_region,
            coarse_land,
            coarse_sea,
            bmap: vec![0; n],
            owner: vec![NONE; n],
            tile_count: Vec::new(),
            city_list: Vec::new(),
            sep_tick: false,
            units: Vec::new(),
            blds: Vec::new(),
            free_units: Vec::new(),
            free_blds: Vec::new(),
            nations: Vec::new(),
            war: vec![vec![false; nn]; nn],
            war_start: vec![vec![0.0; nn]; nn],
            truce: vec![vec![0.0; nn]; nn],
            peace_offer: vec![false; nn],
            offer_cd: vec![0.0; nn],
            peace_asks: VecDeque::new(),
            years: 0.0,
            era: Era::Rise,
            player: 0,
            coalitions: Vec::new(),
            hezong_next: 0.0,
            msgs: Vec::new(),
            banner: None,
            banner_queue: VecDeque::new(),
            hist_i: 0,
            chronicle: Vec::new(),
            shots: Vec::new(),
            rng: Rng::new(seed as u64 * 7 + 3),
            pf: PathFinder::new(),
            path_queue: VecDeque::new(),
            grid: vec![Vec::new(); (GW * GH) as usize],
            territory_dirty: true,
            territory_version: 0,
            terrain_dirty: Vec::new(),
            saplings: BTreeMap::new(),
            forestry_regions: Vec::new(),
            managed_saplings: BTreeSet::new(),
            game_over: None,
            protect_player: false,
            player_orders: 0,
            descent: None,
            captives: Vec::new(),
            hero_year: 0,
            ai_player: false,
            heroic: false,
            time: 0.0,
            ai: Vec::new(),
            trade: Trade::new(&mut Rng::new(seed as u64 * 13 + 5)),
            stat_t: 0.0,
            income_t: 0.0,
            year_tick: 0.0,
        };
        for (i, d) in NATIONS.iter().enumerate() {
            let (hx, hy) = ll_to_tile(d.lon, d.lat);
            w.nations.push(Nation {
                name: d.name,
                color: d.color,
                bias: d.bias,
                alive: true,
                appeared: true,
                res: [800.0, 650.0, 400.0],
                aggression: 0.0,
                hegemon_cd: 0.0,
                capital: None,
                workers: 0,
                soldiers: 0,
                q_workers: 0,
                q_soldiers: 0,
                cities: 0,
                mil_power: 0.0,
                eco_power: 0.0,
                tiles: 0,
                upkeep: 0.0,
                strength: 0.0,
                earned: [0.0; 3],
                income: [0.0; 3],
                home: v2(hx, hy),
                house: INITIAL_HOUSE[i],
                figures: Vec::new(),
                eco_mul: 1.0,
                mil_mul: 1.0,
                coin: 0.0,
                coin_earned: 0.0,
                coin_income: 0.0,
                tax: 1,
                tm: TraitMods::NONE,
                cap_bonus: 0,
                villages: 0,
            });
            w.ai.push(AiState::new(i as u64 * 31 + seed as u64));
        }
        // 部族の兵・天の軍の仮の枠（国としては扱わない）
        for (name, color) in [("部族", [0.62, 0.42, 0.24]), ("天の軍勢", [0.95, 0.92, 0.70])] {
        w.nations.push(Nation {
            name,
            color,
            bias: 0.0,
            alive: false,
            appeared: false,
            res: [0.0; 3],
            aggression: 0.0,
            hegemon_cd: 0.0,
            capital: None,
            workers: 0,
            soldiers: 0,
            q_workers: 0,
            q_soldiers: 0,
            cities: 0,
            mil_power: 0.0,
            eco_power: 0.0,
            tiles: 0,
            upkeep: 0.0,
            strength: 0.0,
            earned: [0.0; 3],
            income: [0.0; 3],
            home: v2(0.0, 0.0),
            house: "",
            figures: Vec::new(),
            eco_mul: 1.0,
            mil_mul: 1.0,
            coin: 0.0,
            coin_earned: 0.0,
            coin_income: 0.0,
            tax: 1,
            tm: TraitMods::NONE,
            cap_bonus: 0,
            villages: 0,
        });
        }
        // 最初からある街（各国の最初の街が都）
        for (k, td) in TOWNS.iter().enumerate() {
            let i = td.owner;
            let first = w.nations[i].capital.is_none();
            let kind = if first { BKind::Capital } else { BKind::Town };
            let Some(c) = w.found_city(i, td.lon, td.lat, kind) else { continue };
            {
                let b = &mut w.blds[c.idx as usize];
                b.town = k as u8;
                b.lord = td.lord;
            }
            if first {
                w.nations[i].capital = Some(c);
                let p = w.blds[c.idx as usize].center();
                w.nations[i].home = p;
            }
        }
        // 最初の民と兵：都と、海の向こうの領地の主な街に
        for i in 0..nn {
            let r = ROSTERS[i];
            let Some(c) = w.nations[i].capital else { continue };
            for k in std::iter::repeat(UT_WORKER).take(8).chain(std::iter::repeat(r[0]).take(6)).chain(std::iter::repeat(r[1]).take(6)) {
                if let Some(sp) = w.spawn_point(c) {
                    w.spawn_unit(i, k, sp, 1.0);
                }
            }
        }
        if let Some(bdx) = w.town_bld(town_id("ボルドー")) {
            let h = w.bh(bdx);
            for k in std::iter::repeat(UT_WORKER).take(6).chain(std::iter::repeat(5).take(8)).chain(std::iter::repeat(UT_LONGBOW).take(8)) {
                if let Some(sp) = w.spawn_point(h) {
                    w.spawn_unit(ENGLAND, k, sp, 1.0);
                }
            }
        }
        // 1337年：フィリップ6世がギュイエンヌを没収し、エドワード3世はフランス王位を求めて戦が始まる
        w.war[ENGLAND][FRANCE] = true;
        w.war[FRANCE][ENGLAND] = true;
        w.recompute_territory();
        w.update_stats();
        w.refresh_traits();
        // 1337年に戦場にいる英雄（フィリップ6世）
        w.yearly_heroes();
        w.msgs.clear();
        w.banner = None;
        w.banner_queue.clear();
        w
    }

    fn found_city(&mut self, nation: usize, lon: f32, lat: f32, kind: BKind) -> Option<Handle> {
        let (fx, fy) = ll_to_tile(lon, lat);
        let (cx, cy) = (fx as i32 - 1, fy as i32 - 1);
        for r in 0..8i32 {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r {
                        continue;
                    }
                    let (tx, ty) = (cx + dx, cy + dy);
                    if self.footprint_free(tx, ty) {
                        return Some(self.spawn_building(nation, kind, tx, ty, true));
                    }
                }
            }
        }
        None
    }

    // ------------------------------------------------------------ 基本操作
    pub fn unit(&self, h: Handle) -> Option<&Unit> {
        let u = self.units.get(h.idx as usize)?;
        if u.alive && u.gen == h.gen {
            Some(u)
        } else {
            None
        }
    }
    pub fn unit_mut(&mut self, h: Handle) -> Option<&mut Unit> {
        let u = self.units.get_mut(h.idx as usize)?;
        if u.alive && u.gen == h.gen {
            Some(u)
        } else {
            None
        }
    }
    pub fn bld(&self, h: Handle) -> Option<&Building> {
        let b = self.blds.get(h.idx as usize)?;
        if b.alive && b.gen == h.gen {
            Some(b)
        } else {
            None
        }
    }
    pub fn bld_mut(&mut self, h: Handle) -> Option<&mut Building> {
        let b = self.blds.get_mut(h.idx as usize)?;
        if b.alive && b.gen == h.gen {
            Some(b)
        } else {
            None
        }
    }
    pub fn uh(&self, i: usize) -> Handle {
        Handle { idx: i as u32, gen: self.units[i].gen }
    }
    pub fn bh(&self, i: usize) -> Handle {
        Handle { idx: i as u32, gen: self.blds[i].gen }
    }

    pub fn spawn_unit(&mut self, nation: usize, ut: u8, pos: V2, power: f32) -> Handle {
        let d = udef(ut);
        let u = Unit {
            gen: 0,
            alive: true,
            nation: nation as u8,
            kind: d.class,
            ut,
            pos,
            hp: d.hp * power,
            power,
            order: Order::Idle,
            resume: None,
            path: Vec::new(),
            path_i: 0,
            path_goal: (-1, -1),
            path_pending: false,
            path_done: false,
            cooldown: 0.0,
            carry_kind: 0,
            carry: 0.0,
            scan_t: self.rng.f32() * 0.5,
            dir: v2(0.0, 1.0),
            flash: 0.0,
            selected: false,
            last_attacker: None,
            tribe: NONE,
            merc_until: 0.0,
            divine: false,
            hero: NONE,
            build_queue: VecDeque::new(),
            cargo: Vec::new(),
            aboard: None,
        };
        if let Some(i) = self.free_units.pop() {
            let gen = self.units[i as usize].gen + 1;
            self.units[i as usize] = Unit { gen, ..u };
            Handle { idx: i, gen }
        } else {
            self.units.push(u);
            Handle { idx: self.units.len() as u32 - 1, gen: 0 }
        }
    }

    pub fn kill_unit(&mut self, i: usize) {
        if self.units[i].alive {
            self.units[i].alive = false;
            self.units[i].selected = false;
            self.units[i].path.clear();
            self.free_units.push(i as u32);
            // 沈んだ船の兵も失われる
            let cargo = std::mem::take(&mut self.units[i].cargo);
            for h in cargo {
                if self.unit(h).is_some() {
                    self.kill_unit(h.idx as usize);
                }
            }
            // 乗っていた船の名簿から外す
            if let Some(s) = self.units[i].aboard.take() {
                let me = Handle { idx: i as u32, gen: self.units[i].gen };
                if let Some(sh) = self.unit_mut(s) {
                    sh.cargo.retain(|&h| h != me);
                }
            }
        }
    }

    fn forestry_ground(&self, nation: usize, tile: usize) -> bool {
        self.owner[tile] as usize == nation && self.bmap[tile] == 0
            && matches!(self.map.terrain[tile], Terrain::Plain | Terrain::Steppe)
    }

    fn forestry_plantable(&self, nation: usize, tile: usize) -> bool {
        self.forestry_ground(nation, tile) && !self.saplings.contains_key(&tile)
            && !self.map.mines.iter().any(|m| {
                m.amount > 0.0 && Map::idx(m.x, m.y) == tile
            })
    }

    pub fn forestry_region_at(&self, nation: usize, x: i32, y: i32) -> Option<usize> {
        self.forestry_regions.iter().position(|r| r.nation == nation && r.contains(x, y))
    }

    pub fn designate_forestry(&mut self, nation: usize, workers: &[Handle], a: (i32, i32), b: (i32, i32)) -> Result<usize, &'static str> {
        let region = ForestryRegion {
            nation, x0: a.0.min(b.0), y0: a.1.min(b.1), x1: a.0.max(b.0), y1: a.1.max(b.1),
        };
        if !region.valid() {
            return Err(t!("地図内で最大64×64マスの範囲を指定してください", "Choose an area up to 64 × 64 tiles inside the map", "请在地图内选择不超过64×64格的区域"));
        }
        if self.forestry_regions.len() >= MAX_FORESTRY_REGIONS {
            return Err(t!("植林区域が上限に達しています", "Forestry region limit reached", "植林区域已达上限"));
        }
        if self.forestry_regions.iter().any(|&r| r.nation == nation && region.overlaps(r)) {
            return Err(t!("既存の植林区域と重なっています。区域をクリックすると民を追加できます", "This overlaps a forestry region. Click the region to assign workers to it.", "区域重叠。点击现有区域可增加林工。"));
        }
        if !(0..region.area()).any(|offset| {
            let tile = region.tile(offset);
            self.forestry_plantable(nation, tile)
                || (self.owner[tile] as usize == nation && self.bmap[tile] == 0
                    && (self.map.terrain[tile] == Terrain::Forest || self.saplings.contains_key(&tile)))
        }) {
            return Err(t!("自国領内の森か、空いた平地・草原を含めてください", "Include forest, empty plains, or steppe in your territory", "请包含己方领土内的森林或空旷平地、草原"));
        }
        let id = self.forestry_regions.len();
        self.forestry_regions.push(region);
        let count = self.assign_forestry(nation, workers, id);
        if count == 0 {
            self.forestry_regions.pop();
            return Err(t!("植林を担当する民を選んでください", "Select workers to assign to forestry", "请选择负责植林的民"));
        }
        Ok(count)
    }

    pub fn assign_forestry(&mut self, nation: usize, workers: &[Handle], id: usize) -> usize {
        let Some(&region) = self.forestry_regions.get(id).filter(|r| r.nation == nation) else {
            return 0;
        };
        let mut count = 0;
        for &h in workers {
            let Some(u) = self.unit_mut(h) else { continue };
            if !u.on_map() || u.kind != UnitKind::Worker || u.nation as usize != nation {
                continue;
            }
            if !matches!(u.order, Order::Forestry { region, .. } if region == id) {
                u.order = Order::Forestry { region: id, target: None, cursor: region.offset_near(u.pos) };
                u.resume = None;
                u.build_queue.clear();
                u.reset_nav();
                u.scan_t = 0.0;
            }
            count += 1;
        }
        count
    }

    fn update_forestry(&mut self, i: usize, id: usize, target: Option<usize>, cursor: usize, dt: f32) {
        let nation = self.units[i].nation as usize;
        let Some(&region) = self.forestry_regions.get(id).filter(|r| r.nation == nation) else {
            self.units[i].order = Order::Idle;
            return;
        };
        if self.units[i].kind != UnitKind::Worker {
            self.units[i].order = Order::Idle;
            return;
        }
        if let Some(tile) = target {
            let goal = forestry_position(tile);
            if self.forestry_ground(nation, tile) && (self.managed_saplings.contains(&tile) || self.forestry_plantable(nation, tile)) {
                if self.units[i].pos.dist(goal) <= FORESTRY_REACH {
                    if self.managed_saplings.contains(&tile) {
                        return;
                    }
                    if self.try_pay(nation, PLANT_FOREST_COST) {
                        self.saplings.insert(tile, self.time + FOREST_GROWTH_TIME);
                        self.managed_saplings.insert(tile);
                        return;
                    }
                } else if !self.nav(i, goal, FORESTRY_REACH, dt) {
                    return;
                }
            }
            self.units[i].order = Order::Forestry { region: id, target: None, cursor };
            self.units[i].scan_t = 0.5;
            self.units[i].reset_nav();
            return;
        }

        if self.units[i].scan_t > 0.0 {
            return;
        }
        self.units[i].scan_t = 1.0;
        let reserved: BTreeSet<_> = self.units.iter().filter(|u| u.on_map()).filter_map(|u| {
            if let Order::Forestry { target, .. } = u.order { target } else { None }
        }).collect();
        let can_pay = self.nations[nation].res[0] >= PLANT_FOREST_COST[0];
        let choice = (0..region.area()).map(|n| (cursor + n) % region.area()).find(|&offset| {
            let tile = region.tile(offset);
            !reserved.contains(&tile)
                && if self.managed_saplings.contains(&tile) {
                    self.forestry_ground(nation, tile)
                } else {
                    can_pay && self.forestry_plantable(nation, tile)
                }
        });
        if let Some(offset) = choice {
            self.units[i].order = Order::Forestry { region: id, target: Some(region.tile(offset)), cursor: (offset + 1) % region.area() };
        }
    }

    fn grow_forests(&mut self, elapsed: f32) {
        let tended: BTreeSet<_> = if self.managed_saplings.is_empty() {
            BTreeSet::new()
        } else {
            self.units.iter().filter(|u| u.on_map() && u.kind == UnitKind::Worker).filter_map(|u| {
                let Order::Forestry { region, target: Some(tile), .. } = u.order else { return None };
                let nation = u.nation as usize;
                if self.forestry_regions[region].nation == nation && self.forestry_ground(nation, tile)
                    && u.pos.dist(forestry_position(tile)) <= FORESTRY_REACH
                {
                    Some(tile)
                } else {
                    None
                }
            }).collect()
        };
        self.saplings.retain(|&i, ready_at| {
            if self.bmap[i] != 0 || !matches!(self.map.terrain[i], Terrain::Plain | Terrain::Steppe) {
                return false;
            }
            if self.managed_saplings.contains(&i) && !tended.contains(&i) {
                *ready_at += elapsed;
                return true;
            }
            if self.time < *ready_at {
                return true;
            }
            self.map.terrain[i] = Terrain::Forest;
            self.map.wood[i] = 120.0;
            self.cost[i] = Terrain::Forest.move_cost();
            self.terrain_dirty.push(((i % MAP_W) as i32, (i / MAP_W) as i32));
            false
        });
        self.managed_saplings.retain(|tile| self.saplings.contains_key(tile));
    }

    pub fn footprint_free(&self, tx: i32, ty: i32) -> bool {
        for dy in 0..2 {
            for dx in 0..2 {
                let (x, y) = (tx + dx, ty + dy);
                if !Map::inb(x, y) {
                    return false;
                }
                let i = Map::idx(x, y);
                if !self.map.terrain[i].buildable() || self.bmap[i] != 0 || self.saplings.contains_key(&i) {
                    return false;
                }
                if self.map.mines.iter().any(|m| m.x == x && m.y == y && m.amount > 0.0) {
                    return false;
                }
            }
        }
        true
    }

    /// 建設可能か（理由付き）
    pub fn can_place(&self, nation: usize, kind: BKind, tx: i32, ty: i32) -> Result<(), &'static str> {
        if !self.footprint_free(tx, ty) {
            return Err(t!("ここには建設できない", "Cannot build here", "此处无法建造"));
        }
        let c = v2(tx as f32 + 1.0, ty as f32 + 1.0);
        let own = |x: i32, y: i32| self.owner[Map::idx(x, y)];
        match kind {
            BKind::Town | BKind::Capital => {
                return Err(t!("街は新しく築けない（主要な街は最初からある。村なら建てられる）", "Towns cannot be founded (the great towns already exist; you can build villages)", "无法新建城镇（主要城镇一开始就已存在，可以建造村庄）"));
            }
            BKind::Village => {
                for dy in 0..2 {
                    for dx in 0..2 {
                        if own(tx + dx, ty + dy) as usize != nation {
                            return Err(t!("村は自国の領土内に建てる", "Villages must be built inside your territory", "村庄必须建在己方领土内"));
                        }
                    }
                }
                for b in self.blds.iter() {
                    if b.alive && b.kind.dropsite() && b.center().dist(c) < 6.0 {
                        return Err(t!("他の街や村に近すぎる（6マス以上離す）", "Too close to another town or village (keep 6 tiles apart)", "离其他城镇或村庄太近（需间隔6格以上）"));
                    }
                }
            }
            _ => {
                for dy in 0..2 {
                    for dx in 0..2 {
                        if own(tx + dx, ty + dy) as usize != nation {
                            return Err(t!("自国の領土内に建設する必要がある", "Must be built inside your territory", "必须建在己方领土内"));
                        }
                    }
                }
                if kind == BKind::Fort && self.fort_count(nation) >= self.nations[nation].fort_cap() {
                    return Err(t!("要塞の数が上限に達している（城邑を増やすと上限が上がる）", "Fort limit reached (more cities raise the limit)", "要塞数量已达上限（城邑越多上限越高）"));
                }
                if kind == BKind::Port {
                    if !self.can_build_ships(nation) {
                        return Err(t!("フランスは船を造れない（交易画面でイタリアの都市からガレー船を雇う）", "France cannot build ships (hire galleys from the Italian cities on the Trade screen)", "法兰西无法造船（在贸易界面向意大利城市雇佣桨帆船）"));
                    }
                    if self.port_water(tx, ty).is_none() {
                        return Err(t!("港は海辺（海から2マス以内）に建てる", "A harbor must be built on the coast (within 2 tiles of the sea)", "港口须建在海边（离海2格以内）"));
                    }
                }
            }
        }
        Ok(())
    }

    /// 建設中を含む要塞の数
    pub fn fort_count(&self, nation: usize) -> u32 {
        self.blds.iter().filter(|b| b.alive && b.nation as usize == nation && b.kind == BKind::Fort).count() as u32
    }

    pub fn spawn_building(&mut self, nation: usize, kind: BKind, tx: i32, ty: i32, complete: bool) -> Handle {
        let b = Building {
            gen: 0,
            alive: true,
            nation: nation as u8,
            origin: nation as u8,
            kind,
            tx,
            ty,
            hp: if complete { kind.max_hp() } else { kind.max_hp() * 0.1 },
            progress: if complete { 1.0 } else { 0.0 },
            queue: VecDeque::new(),
            queue_t: 0.0,
            rally: None,
            cooldown: 0.0,
            town: NONE,
            lord: L_CROWN,
        };
        let h = if let Some(i) = self.free_blds.pop() {
            let gen = self.blds[i as usize].gen + 1;
            self.blds[i as usize] = Building { gen, ..b };
            Handle { idx: i, gen }
        } else {
            self.blds.push(b);
            Handle { idx: self.blds.len() as u32 - 1, gen: 0 }
        };
        for dy in 0..2 {
            for dx in 0..2 {
                let i = Map::idx(tx + dx, ty + dy);
                self.bmap[i] = h.idx + 1;
                if kind.blocks() {
                    self.cost[i] = f32::INFINITY;
                }
            }
        }
        if kind.dropsite() && complete {
            self.territory_dirty = true;
        }
        h
    }

    pub fn remove_building(&mut self, i: usize) {
        let b = &self.blds[i];
        if !b.alive {
            return;
        }
        let (tx, ty, city) = (b.tx, b.ty, b.kind.dropsite());
        for dy in 0..2 {
            for dx in 0..2 {
                let ti = Map::idx(tx + dx, ty + dy);
                self.bmap[ti] = 0;
                self.cost[ti] = self.map.terrain[ti].move_cost();
            }
        }
        self.blds[i].alive = false;
        self.free_blds.push(i as u32);
        if city {
            self.territory_dirty = true;
        }
    }

    pub fn spawn_point(&self, h: Handle) -> Option<V2> {
        let b = self.bld(h)?;
        if b.kind == BKind::Port {
            return self.port_water(b.tx, b.ty);
        }
        let mut fallback = None;
        for r in 1..6 {
            let mut cands = Vec::new();
            for dy in -r..=1 + r {
                for dx in -r..=1 + r {
                    let (x, y) = (b.tx + dx, b.ty + dy);
                    if dx > -r && dx < 1 + r && dy > -r && dy < 1 + r {
                        continue;
                    }
                    if Map::inb(x, y) && self.cost[Map::idx(x, y)].is_finite() {
                        cands.push(v2(x as f32 + 0.5, y as f32 + 0.5));
                    }
                }
            }
            if !cands.is_empty() {
                let free: Vec<V2> = cands
                    .iter()
                    .copied()
                    .filter(|&p| !self.units.iter().any(|u| u.alive && u.pos.dist(p) < 0.85))
                    .collect();
                if !free.is_empty() {
                    let k = (self.units.len().wrapping_mul(3) + (self.time * 7.3) as usize) % free.len();
                    return Some(free[k]);
                }
                if fallback.is_none() {
                    let k = (self.units.len() + (self.time * 7.3) as usize) % cands.len();
                    fallback = Some(cands[k]);
                }
            }
        }
        fallback
    }

    pub fn passable_at(&self, p: V2) -> bool {
        let (x, y) = p.tile();
        Map::inb(x, y) && self.cost[Map::idx(x, y)].is_finite()
    }

    /// 船なら水の上、それ以外は陸の上を通れるか
    pub fn passable_l(&self, p: V2, naval: bool) -> bool {
        let (x, y) = p.tile();
        Map::inb(x, y) && if naval { self.wcost[Map::idx(x, y)].is_finite() } else { self.cost[Map::idx(x, y)].is_finite() }
    }

    pub fn line_clear(&self, a: V2, b: V2) -> bool {
        self.line_clear_l(a, b, false)
    }

    pub fn line_clear_l(&self, a: V2, b: V2, naval: bool) -> bool {
        let d = b - a;
        let l = d.len();
        let n = (l / 0.4).ceil() as i32;
        for s in 1..=n {
            let p = a + d * (s as f32 / n as f32);
            if !self.passable_l(p, naval) {
                return false;
            }
        }
        true
    }

    /// 港の建設予定地（左上 tx,ty）から2マス以内の海のマス（船の出る所）
    pub fn port_water(&self, tx: i32, ty: i32) -> Option<V2> {
        let mut best: Option<(i32, V2)> = None;
        for dy in -2..=3 {
            for dx in -2..=3 {
                let (x, y) = (tx + dx, ty + dy);
                if Map::inb(x, y) && self.map.terrain[Map::idx(x, y)] == Terrain::Sea {
                    let d = (dx * 2 - 1).abs() + (dy * 2 - 1).abs();
                    if best.map_or(true, |b| d < b.0) {
                        best = Some((d, v2(x as f32 + 0.5, y as f32 + 0.5)));
                    }
                }
            }
        }
        best.map(|b| b.1)
    }

    /// 地点の陸の連結成分（島の番号）。水の上なら近くの陸
    pub fn land_region_at(&self, p: V2) -> u32 {
        let (cx, cy) = p.tile();
        for r in 0..4i32 {
            for dy in -r..=r {
                for dx in -r..=r {
                    let (x, y) = (cx + dx, cy + dy);
                    if Map::inb(x, y) {
                        let c = self.land_region[Map::idx(x, y)];
                        if c != u32::MAX {
                            return c;
                        }
                    }
                }
            }
        }
        u32::MAX
    }

    /// 陸づたいに行けるか
    pub fn same_land(&self, a: V2, b: V2) -> bool {
        let (ra, rb) = (self.land_region_at(a), self.land_region_at(b));
        ra != u32::MAX && ra == rb
    }

    /// 地点に近い岸辺の水のマス（船を寄せる所）。max_r マスまで探す
    pub fn shore_water_near(&self, p: V2, max_r: i32) -> Option<V2> {
        let (cx, cy) = p.tile();
        for r in 0..=max_r {
            let mut best: Option<(f32, V2)> = None;
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r {
                        continue;
                    }
                    let (x, y) = (cx + dx, cy + dy);
                    if !Map::inb(x, y) || !self.map.terrain[Map::idx(x, y)].water() {
                        continue;
                    }
                    // 隣に陸があること
                    let coast = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(ax, ay)| Map::inb(x + ax, y + ay) && self.cost[Map::idx(x + ax, y + ay)].is_finite());
                    if coast {
                        let q = v2(x as f32 + 0.5, y as f32 + 0.5);
                        let d = q.dist(p);
                        if best.map_or(true, |b| d < b.0) {
                            best = Some((d, q));
                        }
                    }
                }
            }
            if best.is_some() {
                return best.map(|b| b.1);
            }
        }
        None
    }

    /// 船の周りの陸のマス（兵を降ろす所）
    pub fn land_near(&self, p: V2, r: i32) -> Vec<V2> {
        let (cx, cy) = p.tile();
        let mut v = Vec::new();
        for dy in -r..=r {
            for dx in -r..=r {
                let (x, y) = (cx + dx, cy + dy);
                if Map::inb(x, y) && self.cost[Map::idx(x, y)].is_finite() {
                    v.push(v2(x as f32 + 0.5, y as f32 + 0.5));
                }
            }
        }
        v.sort_by(|a, b| a.dist(p).partial_cmp(&b.dist(p)).unwrap());
        v
    }

    pub fn push_msg(&mut self, text: String, color: [f32; 3]) {
        self.msgs.push(Msg { text, time: self.time, color });
        if self.msgs.len() > 60 {
            self.msgs.remove(0);
        }
    }

    /// 1年あたりの秒数（ランカスター朝の戦争の時代は少し遅くなる）
    pub fn sec_per_year(&self) -> f32 {
        if self.era == Era::Rise {
            SEC_PER_YEAR
        } else {
            SEC_PER_YEAR * EMPIRE_SLOW
        }
    }

    /// いまの年（西暦）
    pub fn year_ad(&self) -> i32 {
        (START_AD + self.years).floor() as i32
    }

    pub fn power_cap(&self) -> f32 {
        if self.era == Era::Rise {
            4.0
        } else {
            6.0
        }
    }

    /// 全国共通の時代補正（一部の国だけ文明が進むことはない）
    pub fn era_mult(&self) -> f32 {
        1.0 + 0.5 * (self.years / (END_AD - START_AD))
    }

    pub fn at_war(&self, a: usize, b: usize) -> bool {
        // 略奪に来た部族の兵・天の軍勢とはどの国も戦う
        a != b && (a >= TRIBE || b >= TRIBE || self.war[a][b])
    }

    pub fn coalition_locked(&self, a: usize, b: usize) -> bool {
        self.coalitions.iter().any(|c| (c.target == a && c.members.contains(&b)) || (c.target == b && c.members.contains(&a)))
    }

    pub fn declare_war(&mut self, a: usize, b: usize, forced: bool) -> bool {
        if a == b || self.war[a][b] || !self.nations[a].alive || !self.nations[b].alive {
            return false;
        }
        if !forced && self.truce[a][b] > self.years {
            return false;
        }
        self.war[a][b] = true;
        self.war[b][a] = true;
        self.war_start[a][b] = self.years;
        self.war_start[b][a] = self.years;
        self.peace_offer[a] = false;
        self.peace_offer[b] = false;
        if !forced {
            self.nations[a].aggression += 30.0;
        }
        let p = self.player;
        if a == p || b == p {
            let t = tf!("{} が {} に宣戦布告した", "{} declared war on {}", "{}向{}宣战", tx(self.nations[a].name), tx(self.nations[b].name));
            self.push_msg(t, [1.0, 0.45, 0.35]);
        }
        true
    }

    pub fn make_peace(&mut self, a: usize, b: usize, truce_years: f32) {
        if !self.war[a][b] {
            return;
        }
        self.war[a][b] = false;
        self.war[b][a] = false;
        self.truce[a][b] = self.years + truce_years;
        self.truce[b][a] = self.years + truce_years;
        if a == self.player || b == self.player {
            let o = if a == self.player { b } else { a };
            self.peace_offer[o] = false;
            let t = tf!("{} と和睦した（{}年間の休戦）", "Made peace with {} ({}-year truce)", "与{}议和（休战{}年）", tx(self.nations[o].name), truce_years as i32);
            self.push_msg(t, [0.6, 0.9, 0.6]);
        }
    }

    /// 和睦を申し入れる。プレイヤーが当事者なら返答を待ち（false）、それ以外は即座に和睦する（true）
    pub fn propose_peace(&mut self, a: usize, b: usize, truce_years: f32, why: &str) -> bool {
        let p = self.player;
        if (a != p && b != p) || self.ai_player {
            self.make_peace(a, b, truce_years);
            return true;
        }
        let o = if a == p { b } else { a };
        if !self.war[a][b] || self.descent.is_some() || self.peace_asks.iter().any(|q| q.from == o && q.ransom.is_none()) {
            return false;
        }
        let t = tf!("{} が和睦を申し入れてきた（{}）", "{} offers peace ({})", "{}提出议和（{}）", tx(self.nations[o].name), why);
        self.push_msg(t, [0.7, 0.9, 1.0]);
        self.peace_asks.push_back(PeaceAsk { from: o, years: truce_years, why: why.to_string(), ransom: None });
        false
    }

    /// 返答待ちの申し入れのうち、既に意味のないもの（交戦が終わった・滅んだ）を捨てる
    pub fn prune_peace_asks(&mut self) {
        let p = self.player;
        let (war, nations) = (&self.war, &self.nations);
        let captives = &self.captives;
        self.peace_asks.retain(|q| {
            q.from != p
                && nations[q.from].alive
                && match q.ransom {
                    None => war[p][q.from],
                    Some((h, _)) => captives.iter().any(|c| c.hero == h && c.captor == q.from),
                }
        });
    }

    /// 先頭の和睦の申し入れに答える
    pub fn answer_peace(&mut self, accept: bool) {
        self.prune_peace_asks();
        let Some(q) = self.peace_asks.pop_front() else { return };
        let p = self.player;
        if let Some((h, amount)) = q.ransom {
            self.answer_ransom(h, amount, accept);
            return;
        }
        if accept {
            self.make_peace(p, q.from, q.years);
        } else {
            self.peace_offer[q.from] = false;
            self.offer_cd[q.from] = self.years + 8.0;
            let t = tf!("{} の和睦の申し入れを退けた（交戦を続ける）", "Refused {}'s offer of peace (the war goes on)", "拒绝了{}的议和（继续交战）", tx(self.nations[q.from].name));
            self.push_msg(t, [1.0, 0.6, 0.4]);
        }
    }

    // ------------------------------------------------------------ 領土
    pub fn recompute_territory(&mut self) {
        let n = MAP_W * MAP_H;
        let mut best = vec![f32::MAX; n];
        self.owner.fill(NONE);
        for b in self.blds.iter() {
            if !b.alive || !b.kind.dropsite() || !b.done() || b.nation as usize >= NATIONS.len() {
                continue;
            }
            // 村は街の領土の外だけを広げる
            let low = if b.kind == BKind::Village { 1.0 } else { 0.0 };
            let r = b.kind.radius();
            let c = b.center();
            let (cx, cy) = (c.x as i32, c.y as i32);
            for dy in -r..=r {
                for dx in -r..=r {
                    let (x, y) = (cx + dx, cy + dy);
                    if !Map::inb(x, y) {
                        continue;
                    }
                    let d = ((x as f32 + 0.5 - c.x).powi(2) + (y as f32 + 0.5 - c.y).powi(2)).sqrt();
                    let rr = r as f32;
                    if d > rr {
                        continue;
                    }
                    let i = Map::idx(x, y);
                    if self.map.terrain[i] == Terrain::Sea {
                        continue;
                    }
                    let score = d / rr + low;
                    if score < best[i] {
                        best[i] = score;
                        self.owner[i] = b.nation;
                    }
                }
            }
        }
        let mut tc = vec![0u32; self.nations.len()];
        for &o in self.owner.iter() {
            if (o as usize) < tc.len() {
                tc[o as usize] += 1;
            }
        }
        self.tile_count = tc;
        self.territory_dirty = false;
        self.territory_version += 1;
    }

    // ------------------------------------------------------------ 統計
    pub fn update_stats(&mut self) {
        for n in self.nations.iter_mut() {
            n.workers = 0;
            n.soldiers = 0;
            n.q_workers = 0;
            n.q_soldiers = 0;
            n.cities = 0;
            n.villages = 0;
            n.mil_power = 0.0;
            n.eco_power = 0.0;
            n.upkeep = 0.0;
            n.tiles = 0;
        }
        for u in self.units.iter() {
            if !u.alive {
                continue;
            }
            let n = &mut self.nations[u.nation as usize];
            if u.merc_until > 0.0 || u.divine || u.hero != NONE {
                // 傭兵は銭で雇われており、天の兵は天のもの。兵数の上限・維持費の外
                n.mil_power += u.power * (u.hp / u.max_hp()).max(0.2);
            } else if u.kind.military() {
                n.soldiers += 1;
                n.mil_power += u.power * (u.hp / u.max_hp()).max(0.2);
                n.upkeep += 0.07 * u.power;
            } else {
                n.workers += 1;
                n.eco_power += u.power;
            }
        }
        for b in self.blds.iter() {
            if !b.alive {
                continue;
            }
            let n = &mut self.nations[b.nation as usize];
            if b.kind.city() && b.done() {
                n.cities += 1;
            }
            if b.kind == BKind::Village && b.done() {
                n.villages += 1;
            }
            for &k in b.queue.iter() {
                if udef(k).class.military() {
                    n.q_soldiers += 1;
                } else {
                    n.q_workers += 1;
                }
            }
        }
        for (k, n) in self.nations.iter_mut().enumerate() {
            n.tiles = self.tile_count.get(k).copied().unwrap_or(0);
        }
        for n in self.nations.iter_mut() {
            n.upkeep *= n.tm.upkeep;
            n.strength = n.mil_power * 10.0 + n.cities as f32 * 25.0 + n.villages as f32 * 5.0 + n.eco_power * 4.0 + n.tiles as f32 * 0.03;
        }
    }

    // ------------------------------------------------------------ 命令
    /// プレイヤーの部隊への命令を数える（指揮中の一時停止を解くため）
    fn note_player_order(&mut self, hs: &[Handle]) {
        let p = self.player;
        if hs.iter().any(|&h| self.unit(h).map_or(false, |u| u.nation as usize == p)) {
            self.player_orders = self.player_orders.wrapping_add(1);
        }
    }

    pub fn order_move(&mut self, hs: &[Handle], goal: V2, attack: bool) {
        self.note_player_order(hs);
        let n = hs.len();
        let cols = (n as f32).sqrt().ceil() as i32;
        let mut sea_faster: Option<bool> = None;
        for (k, &h) in hs.iter().enumerate() {
            let (r, c) = (k as i32 / cols.max(1), k as i32 % cols.max(1));
            let off = v2((c - cols / 2) as f32 * 0.8, (r - cols / 2) as f32 * 0.8);
            let mut g = goal + if n > 1 { off } else { v2(0.0, 0.0) };
            if !self.passable_at(g) {
                g = goal;
            }
            let naval = self.unit(h).map_or(false, |u| u.kind.naval());
            if !self.passable_l(g, naval) {
                g = goal;
            }
            // 海の向こうへは、自国に船があれば船で渡る（岸で待てば船が迎えに来る）。
            // 陸続きでも、海を回ったほうがずっと早ければ船を使う（部隊ごとに1回だけざっくり見積もる）
            let ferry = self.unit(h).map_or(false, |u| !u.kind.naval() && u.aboard.is_none() && self.has_ships(u.nation as usize))
                && match self.unit(h) {
                    Some(u) if !self.same_land(u.pos, g) => true,
                    Some(u) => *sea_faster.get_or_insert_with(|| self.sea_is_faster(u.nation as usize, u.pos, goal, u.def().speed)),
                    None => false,
                };
            if let Some(u) = self.unit_mut(h) {
                if u.aboard.is_some() {
                    continue;
                }
                let atk = attack && u.kind.military();
                u.order = if ferry {
                    Order::Ferry(g, atk)
                } else if atk {
                    Order::AttackMove(g)
                } else {
                    Order::Move(g)
                };
                u.resume = None;
                u.build_queue.clear();
                u.reset_nav();
            }
        }
    }

    pub fn order_attack(&mut self, hs: &[Handle], t: Target) {
        self.note_player_order(hs);
        let tp = self.target_info(t).map(|x| x.0);
        let mut sea_faster: Option<bool> = None;
        for &h in hs {
            // 海の向こうの敵へは、船があれば船で渡ってから攻めかかる（陸続きでも海路がずっと早ければ船で）
            let ferry = match (tp, self.unit(h)) {
                (Some(p), Some(u)) if !u.kind.naval() && u.aboard.is_none() && self.has_ships(u.nation as usize) => {
                    !self.same_land(u.pos, p) || *sea_faster.get_or_insert_with(|| self.sea_is_faster(u.nation as usize, u.pos, p, u.def().speed))
                }
                _ => false,
            };
            if let Some(u) = self.unit_mut(h) {
                if u.aboard.is_some() {
                    continue;
                }
                u.order = if ferry { Order::Ferry(tp.unwrap(), true) } else { Order::Attack(t) };
                u.resume = None;
                u.build_queue.clear();
                u.reset_nav();
            }
        }
    }

    pub fn order_gather(&mut self, hs: &[Handle], src: Src) {
        self.note_player_order(hs);
        for &h in hs {
            if let Some(u) = self.unit_mut(h) {
                if u.kind == UnitKind::Worker && u.aboard.is_none() {
                    u.order = Order::Gather { src, phase: 0 };
                    u.build_queue.clear();
                    u.reset_nav();
                }
            }
        }
    }

    pub fn order_build(&mut self, hs: &[Handle], b: Handle) {
        self.note_player_order(hs);
        for &h in hs {
            if let Some(u) = self.unit_mut(h) {
                if u.kind == UnitKind::Worker && u.aboard.is_none() {
                    u.order = Order::Build(b);
                    u.build_queue.clear();
                    u.reset_nav();
                }
            }
        }
    }

    /// 建設を予約する：建設中の民は今の建設を終えてから順に取りかかり、それ以外の民はすぐに建て始める
    pub fn queue_build(&mut self, hs: &[Handle], b: Handle) {
        self.note_player_order(hs);
        for &h in hs {
            let busy = match self.unit(h) {
                Some(u) if u.kind == UnitKind::Worker && u.aboard.is_none() => match u.order {
                    Order::Build(cur) => cur != b && self.bld(cur).map_or(false, |x| !x.done()),
                    _ => false,
                },
                _ => continue,
            };
            let u = self.unit_mut(h).unwrap();
            if busy {
                if !u.build_queue.contains(&b) {
                    u.build_queue.push_back(b);
                }
            } else {
                u.order = Order::Build(b);
                u.build_queue.clear();
                u.reset_nav();
            }
        }
    }

    /// 予約された次の建設に取りかかる（なければ false）
    fn next_queued_build(&mut self, i: usize) -> bool {
        let ni = self.units[i].nation as usize;
        while let Some(b) = self.units[i].build_queue.pop_front() {
            if self.bld(b).map_or(false, |x| x.nation as usize == ni && !x.done()) {
                let u = &mut self.units[i];
                u.order = Order::Build(b);
                u.reset_nav();
                return true;
            }
        }
        false
    }

    pub fn can_discard_unit(&self, nation: usize, h: Handle) -> bool {
        self.unit(h).is_some_and(|u| u.nation as usize == nation && u.cargo.is_empty())
    }

    pub fn discard_units(&mut self, nation: usize, hs: &[Handle]) -> bool {
        if hs.is_empty() || !hs.iter().all(|&h| self.can_discard_unit(nation, h)) {
            return false;
        }
        for &h in hs {
            self.kill_unit(h.idx as usize);
        }
        self.update_stats();
        true
    }

    /// 補修には失った耐久10につき木材1（端数切り上げ）が必要。
    pub fn repair_cost(&self, nation: usize, h: Handle) -> Option<[f32; 3]> {
        let b = self.bld(h)?;
        let missing = b.kind.max_hp() - b.hp;
        if b.nation as usize != nation || !b.done() || b.hp <= 0.0 || missing <= 0.0 {
            return None;
        }
        Some([0.0, (missing / 10.0).ceil(), 0.0])
    }

    pub fn repair_building(&mut self, nation: usize, h: Handle) -> bool {
        let Some(cost) = self.repair_cost(nation, h) else {
            return false;
        };
        if !self.try_pay(nation, cost) {
            return false;
        }
        let b = &mut self.blds[h.idx as usize];
        b.hp = b.kind.max_hp();
        true
    }

    pub fn try_pay(&mut self, nation: usize, cost: [f32; 3]) -> bool {
        let r = &mut self.nations[nation].res;
        if r[0] >= cost[0] && r[1] >= cost[1] && r[2] >= cost[2] {
            for k in 0..3 {
                r[k] -= cost[k];
            }
            true
        } else {
            false
        }
    }

    pub fn place_building(&mut self, nation: usize, kind: BKind, tx: i32, ty: i32) -> Result<Handle, &'static str> {
        self.can_place(nation, kind, tx, ty)?;
        if !self.try_pay(nation, kind.cost()) {
            return Err(t!("資源が足りない", "Not enough resources", "资源不足"));
        }
        Ok(self.spawn_building(nation, kind, tx, ty, false))
    }

    pub fn can_train(&self, b: &Building, ut: u8) -> bool {
        self.trainable_at(b).iter().any(|&(u, _)| u == ut)
    }

    pub fn enqueue(&mut self, bh: Handle, ut: u8) -> Result<(), &'static str> {
        let kind = udef(ut).class;
        let Some(b) = self.bld(bh) else { return Err("") };
        if !self.can_train(b, ut) {
            return Err(t!("ここでは訓練できない", "Cannot train that here", "此处无法训练"));
        }
        if b.queue.len() >= 5 {
            return Err(t!("訓練待ちが一杯", "Training queue is full", "训练队列已满"));
        }
        let ni = b.nation as usize;
        let n = &self.nations[ni];
        if kind.military() {
            if n.soldiers + n.q_soldiers >= n.mil_cap() {
                return Err(t!("軍事ユニットが上限に達している", "Soldier limit reached", "军事单位已达上限"));
            }
        } else if n.workers + n.q_workers >= n.worker_cap() {
            return Err(t!("生産ユニットが上限に達している", "Worker limit reached", "生产单位已达上限"));
        }
        if !self.try_pay(ni, self.train_cost(ni, ut)) {
            return Err(t!("資源が足りない", "Not enough resources", "资源不足"));
        }
        let b = self.bld_mut(bh).unwrap();
        b.queue.push_back(ut);
        let n = &mut self.nations[ni];
        if kind.military() {
            n.q_soldiers += 1;
        } else {
            n.q_workers += 1;
        }
        Ok(())
    }

    pub fn upgrade_cost(&self, u: &Unit) -> [f32; 3] {
        let p = u.power;
        let ni = u.nation as usize;
        let m = if ni < NATIONS.len() { self.nations[ni].tm.upgrade } else { 1.0 };
        let total = (40.0 * p * m).round();
        // 蓄えの多い資源から優先して払う（資源の偏りで余らせないため）。蓄えが乏しいときは兵は糧・銅、民は糧・木が基本
        let base = if u.kind.military() { [25.0, 0.0, 15.0] } else { [25.0, 15.0, 0.0] };
        let res = if ni < NATIONS.len() { self.nations[ni].res } else { [0.0; 3] };
        let have = res[0].max(0.0) + res[1].max(0.0) + res[2].max(0.0);
        let w: [f32; 3] = if have >= total && have > 0.0 { [res[0].max(0.0), res[1].max(0.0), res[2].max(0.0)] } else { base };
        let ws = w[0] + w[1] + w[2];
        let mut c = [0.0f32; 3];
        for k in 0..3 {
            c[k] = (total * w[k] / ws).floor();
        }
        // 端数は最も多い資源で払う
        let big = (0..3).max_by(|&a, &b| w[a].partial_cmp(&w[b]).unwrap()).unwrap();
        c[big] += total - (c[0] + c[1] + c[2]);
        c
    }

    /// 訓練（兵）・教化（民）：能力を高める
    pub fn upgrade_unit(&mut self, h: Handle) -> Result<(), &'static str> {
        let cap = self.power_cap();
        let Some(u) = self.unit(h) else { return Err("") };
        if u.merc_until > 0.0 {
            return Err(t!("傭兵は鍛錬できない", "Mercenaries cannot be trained", "佣兵无法训练"));
        }
        if u.hero != NONE {
            return Err(t!("英霊は鍛錬できない", "Heroic spirits cannot be trained", "英灵无法训练"));
        }
        if u.aboard.is_some() {
            return Err(t!("船の上では鍛錬できない", "Cannot train aboard a ship", "船上无法训练"));
        }
        if u.power + 0.01 >= cap {
            return Err(t!("この時代の能力上限に達している", "Already at this era's power limit", "已达本时代能力上限"));
        }
        let (x, y) = u.pos.tile();
        if self.owner[Map::idx(x, y)] != u.nation {
            return Err(t!("自国領内でのみ鍛錬できる", "Can only train inside your territory", "只能在己方领土内训练"));
        }
        let ni = u.nation as usize;
        let cost = self.upgrade_cost(u);
        if !self.try_pay(ni, cost) {
            return Err(t!("資源が足りない", "Not enough resources", "资源不足"));
        }
        let u = self.unit_mut(h).unwrap();
        let ratio = u.hp / u.max_hp();
        u.power = (u.power + 0.5).min(cap);
        u.hp = ratio * u.max_hp();
        Ok(())
    }

    /// 兵⇔民の転換（能力は 2/3 に目減り）
    pub fn convert_unit(&mut self, h: Handle) -> Result<(), &'static str> {
        let Some(u) = self.unit(h) else { return Err("") };
        if u.merc_until > 0.0 {
            return Err(t!("傭兵は民にできない", "Mercenaries cannot become workers", "佣兵无法转为民"));
        }
        if u.hero != NONE {
            return Err(t!("英霊は民にできない", "Heroic spirits cannot become workers", "英灵无法转为民"));
        }
        if u.kind.naval() || u.aboard.is_some() {
            return Err(t!("船や船上の兵は民にできない", "Ships and troops aboard cannot become workers", "船与船上士兵无法转为民"));
        }
        let ni = u.nation as usize;
        let n = &self.nations[ni];
        let to_mil = !u.kind.military();
        if to_mil && n.soldiers + n.q_soldiers >= n.mil_cap() {
            return Err(t!("軍事ユニットが上限に達している", "Soldier limit reached", "军事单位已达上限"));
        }
        if !to_mil && n.workers + n.q_workers >= n.worker_cap() {
            return Err(t!("生産ユニットが上限に達している", "Worker limit reached", "生产单位已达上限"));
        }
        let u = self.unit_mut(h).unwrap();
        let ratio = u.hp / u.max_hp();
        u.power = (u.power * 2.0 / 3.0).max(0.34);
        u.ut = if to_mil { native_ut(ni, UnitKind::Infantry) } else { UT_WORKER };
        u.kind = udef(u.ut).class;
        u.hp = ratio * u.max_hp();
        u.order = Order::Idle;
        u.carry = 0.0;
        let n = &mut self.nations[ni];
        if to_mil {
            n.soldiers += 1;
            n.workers -= 1;
        } else {
            n.soldiers -= 1;
            n.workers += 1;
        }
        Ok(())
    }

    // ------------------------------------------------------------ 検索
    pub fn rebuild_grid(&mut self) {
        for c in self.grid.iter_mut() {
            c.clear();
        }
        for (i, u) in self.units.iter().enumerate() {
            if u.on_map() {
                let gx = (u.pos.x as i32 / GCELL).clamp(0, GW - 1);
                let gy = (u.pos.y as i32 / GCELL).clamp(0, GH - 1);
                self.grid[(gy * GW + gx) as usize].push(i as u32);
            }
        }
    }

    /// 敵対ユニット探索
    pub fn nearest_enemy_unit(&self, nation: usize, p: V2, r: f32) -> Option<usize> {
        let mut best: Option<(f32, usize)> = None;
        let g0x = ((p.x - r) as i32 / GCELL).clamp(0, GW - 1);
        let g1x = ((p.x + r) as i32 / GCELL).clamp(0, GW - 1);
        let g0y = ((p.y - r) as i32 / GCELL).clamp(0, GH - 1);
        let g1y = ((p.y + r) as i32 / GCELL).clamp(0, GH - 1);
        for gy in g0y..=g1y {
            for gx in g0x..=g1x {
                for &i in self.grid[(gy * GW + gx) as usize].iter() {
                    let u = &self.units[i as usize];
                    if !u.alive || !self.at_war(nation, u.nation as usize) {
                        continue;
                    }
                    let d = u.pos.dist(p);
                    // 兵を優先
                    let d2 = if u.kind.military() { d } else { d + 2.0 };
                    if d <= r && best.map_or(true, |b| d2 < b.0) {
                        best = Some((d2, i as usize));
                    }
                }
            }
        }
        best.map(|b| b.1)
    }

    pub fn nearest_enemy_bld(&self, nation: usize, p: V2, r: f32) -> Option<usize> {
        let mut best: Option<(f32, usize)> = None;
        for (i, b) in self.blds.iter().enumerate() {
            if !b.alive || !self.at_war(nation, b.nation as usize) {
                continue;
            }
            let d = b.center().dist(p);
            if d <= r && best.map_or(true, |x| d < x.0) {
                best = Some((d, i));
            }
        }
        best.map(|b| b.1)
    }

    /// 国ごとの完成した城邑の一覧を作り直す
    fn rebuild_city_list(&mut self) {
        let n = self.nations.len();
        if self.city_list.len() != n {
            self.city_list = vec![Vec::new(); n];
        }
        for l in self.city_list.iter_mut() {
            l.clear();
        }
        for (i, b) in self.blds.iter().enumerate() {
            if b.alive && b.kind.dropsite() && b.done() {
                if let Some(l) = self.city_list.get_mut(b.nation as usize) {
                    l.push(i as u32);
                }
            }
        }
    }

    pub fn nearest_dropsite(&self, nation: usize, p: V2) -> Option<usize> {
        let mut best: Option<(f32, usize)> = None;
        // 一覧は毎ステップ作り直すが、途中で落ちた城邑は確かめて除く
        let list: Vec<usize> = match self.city_list.get(nation) {
            Some(l) => l.iter().map(|&i| i as usize).collect(),
            None => (0..self.blds.len()).collect(),
        };
        for i in list {
            let b = &self.blds[i];
            if b.alive && b.nation as usize == nation && b.kind.dropsite() && b.done() {
                let d = b.center().dist(p);
                if best.map_or(true, |x| d < x.0) {
                    best = Some((d, i));
                }
            }
        }
        best.map(|b| b.1)
    }

    pub fn nearest_forest_for(&self, nation: Option<usize>, p: V2, maxr: i32) -> Option<(i32, i32)> {
        let (cx, cy) = p.tile();
        let mut best: Option<(i32, (i32, i32))> = None;
        for r in 0..=maxr {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r {
                        continue;
                    }
                    let (x, y) = (cx + dx, cy + dy);
                    if Map::inb(x, y) {
                        let idx = Map::idx(x, y);
                        if self.map.terrain[idx] == Terrain::Forest {
                            if let Some(n) = nation {
                                let o = self.owner[idx];
                                if o != NONE && o as usize != n {
                                    continue;
                                }
                            }
                            let d = dx * dx + dy * dy;
                            if best.map_or(true, |b| d < b.0) {
                                best = Some((d, (x, y)));
                            }
                        }
                    }
                }
            }
            if best.is_some() {
                return best.map(|b| b.1);
            }
        }
        None
    }

    pub fn nearest_forest(&self, p: V2, maxr: i32) -> Option<(i32, i32)> {
        self.nearest_forest_for(None, p, maxr)
    }

    pub fn nearest_mine_for(&self, nation: Option<usize>, p: V2, maxd: f32) -> Option<usize> {
        let mut best: Option<(f32, usize)> = None;
        for (i, m) in self.map.mines.iter().enumerate() {
            if m.amount <= 0.0 {
                continue;
            }
            if let Some(n) = nation {
                if Map::inb(m.x, m.y) {
                    let o = self.owner[Map::idx(m.x, m.y)];
                    if o != NONE && o as usize != n {
                        continue;
                    }
                }
            }
            let d = v2(m.x as f32 + 0.5, m.y as f32 + 0.5).dist(p);
            if d <= maxd && best.map_or(true, |b| d < b.0) {
                best = Some((d, i));
            }
        }
        best.map(|b| b.1)
    }

    pub fn nearest_mine(&self, p: V2, maxd: f32) -> Option<usize> {
        self.nearest_mine_for(None, p, maxd)
    }

    pub fn target_info(&self, t: Target) -> Option<(V2, f32, usize)> {
        match t {
            Target::Unit(h) => self.unit(h).filter(|u| u.aboard.is_none()).map(|u| (u.pos, if u.kind.naval() { 0.6 } else { 0.3 }, u.nation as usize)),
            Target::Bld(h) => self.bld(h).map(|b| (b.center(), 1.0, b.nation as usize)),
        }
    }

    // ------------------------------------------------------------ 戦闘
    fn damage_unit(&mut self, ti: usize, dmg: f32, attacker: Option<Handle>, an: usize) {
        let killed;
        {
            let u = &mut self.units[ti];
            // 天の軍勢には人の剣も矢も届かない
            if !u.alive || u.divine {
                return;
            }
            u.hp -= dmg;
            u.flash = 0.15;
            u.last_attacker = attacker;
            killed = u.hp <= 0.0;
            if !killed && u.kind.military() && u.order == Order::Idle && u.aboard.is_none() {
                if let Some(a) = attacker {
                    u.order = Order::Attack(Target::Unit(a));
                }
            }
        }
        if killed {
            let hero = self.units[ti].hero;
            if (hero as usize) < HEROES.len() {
                // 英雄は討たれると捕らえられる（身代金を請求できる）
                self.hero_fallen(ti, an);
                return;
            }
            self.kill_unit(ti);
            if an < NATIONS.len() {
                self.nations[an].aggression += 1.5;
            }
        }
    }

    fn damage_bld(&mut self, bi: usize, dmg: f32, an: usize) {
        let vn = self.blds[bi].nation as usize;
        let dmg = if vn < NATIONS.len() { dmg * self.nations[vn].tm.def } else { dmg };
        let b = &mut self.blds[bi];
        if !b.alive {
            return;
        }
        b.hp -= dmg;
        if b.hp > 0.0 {
            return;
        }
        if b.kind == BKind::Village && b.done() {
            // 村は壊されると略奪される
            self.plunder_village(bi, an);
        } else if b.kind.city() && b.done() && an >= TRIBE {
            if an == HEAVEN {
                self.smite_town(bi);
            } else {
                self.plunder_city(bi);
            }
        } else if b.kind.city() && b.done() {
            self.capture_city(bi, an);
        } else {
            let (vn, kind) = (b.nation as usize, b.kind);
            self.remove_building(bi);
            if vn == self.player {
                self.push_msg(tf!("{}が破壊された", "Our {} was destroyed", "{}被摧毁了", kind.name()), [1.0, 0.5, 0.4]);
            }
        }
    }

    fn capture_city(&mut self, bi: usize, an: usize) {
        let old = self.blds[bi].nation as usize;
        let was_cap = self.blds[bi].kind == BKind::Capital;
        {
            let b = &mut self.blds[bi];
            b.nation = an as u8;
            b.hp = b.kind.max_hp() * 0.3;
            b.queue.clear();
            b.queue_t = 0.0;
            b.rally = None;
            b.kind = BKind::Town;
        }
        let bh = self.bh(bi);
        // 街のまわりの村も従う
        self.villages_follow(bi, old, an);
        if self.nations[an].capital.and_then(|h| self.bld(h)).is_none() {
            self.blds[bi].kind = BKind::Capital;
            self.nations[an].capital = Some(bh);
        }
        self.territory_dirty = true;
        self.nations[an].aggression += 25.0;
        let p = self.player;
        let place = self.town_name(bi);
        let text = tf!("{0} が {1} の{2}を攻略した", "{0} took {2} from {1}", "{0}从{1}手中攻占了{2}", tx(self.nations[an].name), tx(self.nations[old].name), place);
        let col = if an == p { [0.5, 1.0, 0.5] } else if old == p { [1.0, 0.35, 0.3] } else { [0.85, 0.85, 0.7] };
        if an == p || old == p || was_cap {
            self.push_msg(text, col);
        }
        if was_cap {
            self.move_capital(old, self.blds[bi].center());
        }
        let has_city = self.blds.iter().any(|b| b.alive && b.nation as usize == old && b.kind.city() && b.done());
        if !has_city {
            self.eliminate(old, an);
        }
    }

    pub fn eliminate(&mut self, n: usize, by: usize) {
        if !self.nations[n].alive {
            return;
        }
        self.nations[n].alive = false;
        for i in 0..self.units.len() {
            if self.units[i].alive && self.units[i].nation as usize == n {
                self.kill_unit(i);
            }
        }
        for i in 0..self.blds.len() {
            if self.blds[i].alive && self.blds[i].nation as usize == n {
                self.remove_building(i);
            }
        }
        for o in 0..NATIONS.len() {
            self.war[n][o] = false;
            self.war[o][n] = false;
        }
        self.share_cap(n, by);
        let by_name = if by < self.nations.len() { tx(self.nations[by].name) } else { "?" };
        let text = tf!("{} は {} に滅ぼされた", "{} was destroyed by {}", "{}被{}所灭", tx(self.nations[n].name), by_name);
        self.push_msg(text, [1.0, 0.8, 0.3]);
        if n == self.player {
            let t = if self.descent.is_some() {
                tf!("{} はジャンヌ・ダルクの亡霊と天の軍勢に滅ぼされた……", "{} was destroyed by the ghost of Joan of Arc and the hosts of Heaven...", "{}被贞德的亡灵与天之军势所灭……", tx(self.nations[n].name))
            } else {
                tf!("{} は {} に滅ぼされた……", "{} was destroyed by {}...", "{}被{}所灭……", tx(self.nations[n].name), by_name)
            };
            self.game_over = Some((false, t));
        }
        self.check_victory();
    }

    /// 勝敗の判定：フランスからイングランドを追い出せばフランスの勝ち、フランスを占領し尽くせばイングランドの勝ち
    pub fn check_victory(&mut self) {
        if self.game_over.is_some() || self.descent.is_some() {
            return;
        }
        let winner = if self.continental_towns(ENGLAND) == 0 {
            Some(FRANCE)
        } else if !self.blds.iter().any(|b| b.alive && b.kind.city() && b.done() && b.nation as usize == FRANCE) {
            Some(ENGLAND)
        } else {
            None
        };
        let Some(win) = winner else { return };
        let p = self.player;
        let ad = self.year_ad();
        let text = if win == FRANCE {
            if p == FRANCE {
                tf!("{}年、イングランド人はフランスから追い払われた。百年の戦はフランスの勝利に終わった！", "In {}, the English have been driven out of France. The Hundred Years' War ends in victory for France!", "{}年，英格兰人被逐出法兰西。百年战争以法兰西的胜利告终！", ad)
            } else {
                tf!("{}年、大陸の領地をすべて失い、イングランドの王はフランスの王冠を諦めた……", "In {}, every English holding on the Continent is lost; the King of England's claim to the French crown is over...", "{}年，英格兰失去了大陆上的全部领地，英王对法兰西王冠的主张就此落空……", ad)
            }
        } else if p == ENGLAND {
            tf!("{}年、フランス全土がイングランド王に服した。二つの王冠はひとつになった！", "In {}, all of France submits to the King of England. The two crowns are one!", "{}年，法兰西全境臣服于英格兰国王。两顶王冠合而为一！", ad)
        } else {
            tf!("{}年、最後の街が落ち、ヴァロワ家のフランスは滅んだ……", "In {}, the last town has fallen and Valois France is no more...", "{}年，最后一座城镇陷落，瓦卢瓦王朝的法兰西灭亡了……", ad)
        };
        self.game_over = Some((win == p, text));
    }

    /// 大陸（ブリテン島の外）にある、その国の街の数
    pub fn continental_towns(&self, n: usize) -> usize {
        let (lx, ly) = ll_to_tile(TOWNS[0].lon, TOWNS[0].lat);
        let britain = self.land_region_at(v2(lx, ly));
        self.blds.iter().filter(|b| b.alive && b.kind.city() && b.done() && b.nation as usize == n && self.land_region_at(b.center()) != britain).count()
    }

    /// 滅んだ国の人口上限（基本の4と、それまでに分けられた分）を生き残った国に分ける。
    /// 等分し、余りは攻め滅ぼした国、次いで国力の高い国から順に1ずつ
    fn share_cap(&mut self, n: usize, by: usize) {
        let pts = BASE_CAP + self.nations[n].cap_bonus;
        self.nations[n].cap_bonus = 0;
        let mut order: Vec<usize> = (0..NATIONS.len()).filter(|&i| i != n && self.nations[i].alive).collect();
        if order.is_empty() {
            return;
        }
        order.sort_by(|&a, &b| self.nations[b].strength.partial_cmp(&self.nations[a].strength).unwrap());
        if let Some(k) = order.iter().position(|&i| i == by) {
            let c = order.remove(k);
            order.insert(0, c);
        }
        let (each, rest) = (pts / order.len() as u32, pts as usize % order.len());
        let mut mine = 0;
        for (k, &i) in order.iter().enumerate() {
            let add = each + if k < rest { 1 } else { 0 };
            self.nations[i].cap_bonus += add;
            if i == self.player {
                mine = add;
            }
        }
        if mine > 0 {
            let p = self.player;
            let cap = self.nations[p].worker_cap();
            let t = tf!("{}の滅亡で、民・兵の上限がそれぞれ +{}（いま{}）", "With the fall of {}, our worker and soldier limits each rise by {} (now {})", "因{}灭亡，民与兵的上限各+{}（现为{}）", tx(self.nations[n].name), mine, cap);
            self.push_msg(t, [0.7, 1.0, 0.8]);
        }
    }

    // ------------------------------------------------------------ 更新
    pub fn step(&mut self) {
        let dt = DT;
        let prev_time = self.time;
        self.time += dt;
        let spy = self.sec_per_year();
        self.years += dt / spy;
        if self.game_over.is_some() {
            return;
        }
        self.grow_forests(self.time - prev_time);
        if self.era == Era::Rise && START_AD + self.years >= EMPIRE_AD {
            self.begin_empire();
        }
        self.stat_t -= dt;
        if self.stat_t <= 0.0 {
            self.stat_t = 0.25;
            self.update_stats();
            self.assign_ferries();
        }
        self.economy(dt);
        self.update_trade(dt);
        self.update_buildings(dt);
        self.rebuild_city_list();
        self.rebuild_grid();
        for i in 0..self.units.len() {
            if self.units[i].on_map() {
                self.update_unit(i, dt);
            }
        }
        // 船に乗っている兵は船とともに動く
        for i in 0..self.units.len() {
            if let (true, Some(sh)) = (self.units[i].alive, self.units[i].aboard) {
                match self.unit(sh).map(|s| s.pos) {
                    Some(p) => self.units[i].pos = p,
                    None => self.kill_unit(i),
                }
            }
        }
        // 押し合いは1ステップおきに、2ステップ分の強さで（見た目は変わらず計算は半分）
        self.sep_tick = !self.sep_tick;
        if self.sep_tick {
            self.separate(dt * 2.0);
        }
        self.process_paths();
        for s in self.shots.iter_mut() {
            s.t -= dt;
        }
        self.shots.retain(|s| s.t > 0.0);
        if self.territory_dirty {
            self.recompute_territory();
        }
        if let Some(b) = &mut self.banner {
            b.t -= dt;
            if b.t <= 0.0 {
                self.banner = self.banner_queue.pop_front();
            }
        }
        self.year_tick += dt;
        if self.year_tick >= spy {
            self.year_tick -= spy;
            self.yearly();
        }
        crate::ai::update_all(self, dt);
    }

    fn economy(&mut self, dt: f32) {
        let em = self.era_mult();
        // 城邑からの税収
        for b in self.blds.iter() {
            if b.alive && b.kind.dropsite() && b.done() && (b.nation as usize) < NATIONS.len() {
                let n = &mut self.nations[b.nation as usize];
                let m = n.bias * n.eco_mul * n.tm.eco * em * if b.kind == BKind::Capital { 1.3 } else { 1.0 };
                // 街の数が多いので、街ひとつの税は軽い（村は民が建てるぶん実入りが多い）
                let add = if b.kind == BKind::Village { [0.16 * m * dt, 0.04 * m * dt, 0.0] } else { [0.09 * m * dt, 0.04 * m * dt, 0.02 * m * dt] };
                for k in 0..3 {
                    n.res[k] += add[k];
                    n.earned[k] += add[k];
                }
            }
        }
        // 軍の維持費（存在するだけで糧を消費）
        let spy = self.sec_per_year();
        let mut starving = vec![false; self.nations.len()];
        for (i, n) in self.nations.iter_mut().enumerate() {
            if !n.alive {
                continue;
            }
            n.res[0] -= n.upkeep * dt;
            if n.res[0] < 0.0 {
                n.res[0] = 0.0;
                starving[i] = true;
            }
            n.aggression = (n.aggression - dt * 2.0 / spy).max(0.0);
        }
        for u in self.units.iter_mut() {
            if u.alive && u.hero != NONE {
                // 英霊は傷が癒える
                u.hp = (u.hp + u.max_hp() * 0.015 * dt).min(u.max_hp());
            } else if u.alive && u.kind.military() && !u.divine && starving[u.nation as usize] {
                u.hp -= u.max_hp() * 0.01 * dt;
            }
        }
        for i in 0..self.units.len() {
            if self.units[i].alive && self.units[i].hp <= 0.0 {
                self.kill_unit(i);
            }
        }
        self.income_t += dt;
        if self.income_t >= 5.0 {
            for n in self.nations.iter_mut() {
                for k in 0..3 {
                    let r = n.earned[k] / self.income_t;
                    n.income[k] = n.income[k] * 0.5 + r * 0.5;
                    n.earned[k] = 0.0;
                }
                let r = n.coin_earned / self.income_t;
                n.coin_income = n.coin_income * 0.5 + r * 0.5;
                n.coin_earned = 0.0;
            }
            self.income_t = 0.0;
        }
    }

    fn update_buildings(&mut self, dt: f32) {
        for bi in 0..self.blds.len() {
            if !self.blds[bi].alive || !self.blds[bi].done() {
                continue;
            }
            // 訓練
            if let Some(&ut) = self.blds[bi].queue.front() {
                let k = udef(ut).class;
                self.blds[bi].queue_t += dt;
                if self.blds[bi].queue_t >= udef(ut).time {
                    let h = self.bh(bi);
                    if let Some(sp) = self.spawn_point(h) {
                        let b = &mut self.blds[bi];
                        b.queue.pop_front();
                        b.queue_t = 0.0;
                        let (ni, rally) = (b.nation as usize, b.rally);
                        let uh = self.spawn_unit(ni, ut, sp, 1.0);
                        if let Some(r) = rally {
                            self.order_move(&[uh], r, true);
                        } else if k == UnitKind::Worker {
                            self.auto_gather(uh);
                        }
                        if k.military() {
                            self.nations[ni].soldiers += 1;
                            self.nations[ni].q_soldiers = self.nations[ni].q_soldiers.saturating_sub(1);
                        } else {
                            self.nations[ni].workers += 1;
                            self.nations[ni].q_workers = self.nations[ni].q_workers.saturating_sub(1);
                        }
                    }
                }
            }
            // 城邑・要塞の防衛射撃
            let fort = self.blds[bi].kind == BKind::Fort;
            if self.blds[bi].kind.city() || fort {
                let b = &mut self.blds[bi];
                b.cooldown -= dt;
                if b.cooldown <= 0.0 {
                    let (c, ni) = (b.center(), b.nation as usize);
                    let (range, cd, dmg) = match (fort, self.era == Era::Rise) {
                        (true, true) => (FORT_RANGE, FORT_COOLDOWN, 14.0),
                        (true, false) => (FORT_RANGE, FORT_COOLDOWN, 22.0),
                        (false, true) => (6.0, 2.0, 10.0),
                        (false, false) => (6.0, 2.0, 16.0),
                    };
                    b.cooldown = cd;
                    if let Some(t) = self.nearest_enemy_unit(ni, c, range) {
                        let tp = self.units[t].pos;
                        self.shots.push(Shot { a: c, b: tp, t: 0.25 });
                        self.damage_unit(t, dmg, None, ni);
                    }
                }
            }
        }
    }

    /// 新しい民に仕事を割り当てる
    pub fn auto_gather(&mut self, h: Handle) {
        let Some(u) = self.unit(h) else { return };
        let ni = u.nation as usize;
        let p = u.pos;
        // 未完成で作業者のいない建物があれば最優先で建設に向かう
        if let Some(bh) = self.unfinished_bld(ni, p) {
            self.order_build(&[h], bh);
            return;
        }
        // 空いている田
        if let Some(f) = self.free_farm(ni, p) {
            self.order_gather(&[h], Src::Farm(f));
            return;
        }
        let res = self.nations[ni].res;
        if res[2] < res[1] * 0.6 {
            if let Some(m) = self.nearest_mine_for(Some(ni), p, 16.0) {
                self.order_gather(&[h], Src::Mine(m));
                return;
            }
        }
        if let Some((x, y)) = self.nearest_forest_for(Some(ni), p, 14) {
            self.order_gather(&[h], Src::Forest(x, y));
        }
    }

    pub fn bld_builders(&self, bh: Handle) -> usize {
        self.units.iter().filter(|u| u.alive && u.order == Order::Build(bh)).count()
    }

    pub fn unfinished_bld(&self, ni: usize, p: V2) -> Option<Handle> {
        let mut best: Option<(f32, Handle)> = None;
        for (i, b) in self.blds.iter().enumerate() {
            if b.alive && b.nation as usize == ni && !b.done() {
                let h = self.bh(i);
                if self.bld_builders(h) == 0 {
                    let d = b.center().dist(p);
                    if best.map_or(true, |x| d < x.0) {
                        best = Some((d, h));
                    }
                }
            }
        }
        best.map(|b| b.1)
    }

    /// 建物へ接近する際の通行可能な目標地点を返す（2×2の通行不可タイル内部ではなく隣接する通行可能タイル上の点）
    pub fn bld_approach_point(&self, bh: Handle, from: V2) -> V2 {
        let Some(b) = self.bld(bh) else { return from };
        let c = b.center();
        if !b.kind.blocks() && self.passable_at(c) {
            return c;
        }
        let mut best: Option<(f32, V2)> = None;
        for dy in -1..=2 {
            for dx in -1..=2 {
                if dx >= 0 && dx <= 1 && dy >= 0 && dy <= 1 {
                    continue;
                }
                let (x, y) = (b.tx + dx, b.ty + dy);
                if !Map::inb(x, y) || !self.cost[Map::idx(x, y)].is_finite() {
                    continue;
                }
                let tc = v2(x as f32 + 0.5, y as f32 + 0.5);
                let bx = tc.x.clamp(b.tx as f32, b.tx as f32 + 2.0);
                let by = tc.y.clamp(b.ty as f32, b.ty as f32 + 2.0);
                let ap = tc + (v2(bx, by) - tc) * 0.35;
                let corner_pen = if (dx == -1 || dx == 2) && (dy == -1 || dy == 2) { 0.45 } else { 0.0 };
                let score = from.dist(ap) + corner_pen;
                if best.map_or(true, |bs| score < bs.0) {
                    best = Some((score, ap));
                }
            }
        }
        best.map(|bs| bs.1).unwrap_or(c)
    }

    pub fn farm_users(&self, f: Handle) -> usize {
        self.units.iter().filter(|u| u.alive && matches!(u.order, Order::Gather { src: Src::Farm(h), .. } if h == f)).count()
    }

    pub fn free_farm(&self, ni: usize, p: V2) -> Option<Handle> {
        // 田ごとに全ユニットを数えると田の数×ユニット数になるので、先に使われている田を集める
        let mut used = vec![false; self.blds.len()];
        for u in self.units.iter() {
            if let (true, Order::Gather { src: Src::Farm(h), .. }) = (u.alive, u.order) {
                if let Some(x) = used.get_mut(h.idx as usize) {
                    if self.blds[h.idx as usize].gen == h.gen {
                        *x = true;
                    }
                }
            }
        }
        let mut best: Option<(f32, Handle)> = None;
        for (i, b) in self.blds.iter().enumerate() {
            if b.alive && b.nation as usize == ni && b.kind == BKind::Farm && b.done() {
                let h = self.bh(i);
                if !used[i] {
                    let d = b.center().dist(p);
                    if best.map_or(true, |x| d < x.0) {
                        best = Some((d, h));
                    }
                }
            }
        }
        best.map(|b| b.1)
    }

    // ---- 移動
    fn request_path(&mut self, i: usize, goal: (i32, i32)) {
        let u = &mut self.units[i];
        u.path_goal = goal;
        u.path_done = false;
        if !u.path_pending {
            u.path_pending = true;
            let h = Handle { idx: i as u32, gen: u.gen };
            self.path_queue.push_back(h);
        }
    }

    fn process_paths(&mut self) {
        // 1ステップで展開してよいマスの数。実時間で区切ると機械の速さで結果が変わるので、仕事量で区切る
        const BUDGET: usize = 30_000;
        let mut n = 0;
        let mut spent = 0;
        // 同じステップで、隣り合う地点から同じ目的地へ求めた経路は使い回す（部隊でまとめて動かすとき）
        let mut memo: Vec<((i32, i32), (i32, i32), Vec<(i16, i16)>, bool)> = Vec::new();
        while let Some(h) = self.path_queue.pop_front() {
            let i = h.idx as usize;
            if !(self.units[i].alive && self.units[i].gen == h.gen && self.units[i].path_pending) {
                continue;
            }
            let start = self.units[i].pos.tile();
            let goal = self.units[i].path_goal;
            let pos = self.units[i].pos;
            let naval = self.units[i].kind.naval();
            let reuse = memo.iter().find(|(s, g, p, nv)| {
                *nv == naval && *g == goal && !p.is_empty() && (s.0 - start.0).abs().max((s.1 - start.1).abs()) <= 1 && self.line_clear_l(pos, v2(s.0 as f32 + 0.5, s.1 as f32 + 0.5), naval)
            });
            // 島や別の海へは道が無い：探さずに諦める（陸地全体を探し回らないように）
            let reachable = {
                let reg = if naval { &self.sea_region } else { &self.land_region };
                let at = |t: (i32, i32), r: i32| -> u32 {
                    for d in 0..=r {
                        for dy in -d..=d {
                            for dx in -d..=d {
                                let (x, y) = (t.0 + dx, t.1 + dy);
                                if Map::inb(x, y) && reg[Map::idx(x, y)] != u32::MAX {
                                    return reg[Map::idx(x, y)];
                                }
                            }
                        }
                    }
                    u32::MAX
                };
                let (a, b) = (at(start, 1), at(goal, 3));
                a == u32::MAX || b == u32::MAX || a == b
            };
            let path = if !reachable {
                Vec::new()
            } else if let Some((_, _, p, _)) = reuse {
                p.clone()
            } else {
                let cost = if naval { &self.wcost } else { &self.cost };
                let r = if Map::inb(goal.0, goal.1) && cost[Map::idx(goal.0, goal.1)].is_finite() { 0 } else { 2 };
                let p = self.pf.find(cost, start, goal, r, 160000, crate::path::UNIT_WEIGHT);
                spent += self.pf.last_exp;
                memo.push((start, goal, p.clone(), naval));
                p
            };
            let u = &mut self.units[i];
            u.path = path;
            u.path_i = 0;
            u.path_pending = false;
            u.path_done = u.path.is_empty();
            n += 1;
            if n >= 4 && spent > BUDGET {
                break;
            }
        }
    }

    fn step_towards(&mut self, i: usize, to: V2, dt: f32) {
        let u = &self.units[i];
        let naval = u.kind.naval();
        let sp = u.def().speed;
        let (tx, ty) = u.pos.tile();
        let tc = if !Map::inb(tx, ty) {
            1.0
        } else if naval {
            self.map.terrain[Map::idx(tx, ty)].sail_cost()
        } else {
            self.map.terrain[Map::idx(tx, ty)].move_cost()
        };
        let sp = sp / tc.clamp(1.0, 3.0).sqrt();
        let d = to - u.pos;
        let l = d.len();
        if l < 1e-4 {
            return;
        }
        let mv = (sp * dt).min(l);
        let np = u.pos + d * (mv / l);
        let dir = d * (1.0 / l);
        let ok = self.passable_l(np, naval) || !self.passable_l(u.pos, naval);
        let u = &mut self.units[i];
        u.dir = dir;
        if ok {
            u.pos = np;
        } else {
            let a = v2(np.x, u.pos.y);
            let b = v2(u.pos.x, np.y);
            if self.passable_l(a, naval) {
                self.units[i].pos = a;
            } else if self.passable_l(b, naval) {
                self.units[i].pos = b;
            }
        }
    }

    /// 目的地へ移動。到着（または到達不能）なら true
    fn nav(&mut self, i: usize, goal: V2, r: f32, dt: f32) -> bool {
        let pos = self.units[i].pos;
        let naval = self.units[i].kind.naval();
        let dist = pos.dist(goal);
        if dist <= r {
            self.units[i].path.clear();
            return true;
        }
        if dist < 4.0 && self.line_clear_l(pos, goal, naval) {
            self.units[i].path.clear();
            self.step_towards(i, goal, dt);
            return false;
        }
        let gt = goal.tile();
        let pg = self.units[i].path_goal;
        let same = if self.units[i].path_done {
            pg == gt
        } else {
            (pg.0 - gt.0).abs().max((pg.1 - gt.1).abs()) <= 1
        };
        if !same {
            self.units[i].path.clear();
            self.request_path(i, gt);
            return false;
        }
        let u = &self.units[i];
        if u.path_i < u.path.len() {
            let (wx, wy) = u.path[u.path_i];
            let wp = v2(wx as f32 + 0.5, wy as f32 + 0.5);
            if pos.dist(wp) < 0.45 {
                let u = &mut self.units[i];
                u.path_i += 1;
                if u.path_i >= u.path.len() {
                    u.path.clear();
                    u.path_i = 0;
                    u.path_done = true;
                }
            } else {
                // 先読み：次の点が直接見えるなら飛ばす
                let ni = u.path_i + 1;
                if ni < u.path.len() {
                    let (nx, ny) = u.path[ni];
                    let np = v2(nx as f32 + 0.5, ny as f32 + 0.5);
                    if pos.dist(wp) < 1.2 && self.line_clear_l(pos, np, naval) {
                        self.units[i].path_i = ni;
                    }
                }
                self.step_towards(i, wp, dt);
            }
            return false;
        }
        if u.path_pending {
            return false;
        }
        if u.path_done {
            // 経路の終端に着いた：直進できなければ到達不能
            if dist < 6.0 && self.line_clear_l(pos, goal, naval) {
                self.step_towards(i, goal, dt);
                return false;
            }
            return true;
        }
        self.request_path(i, gt);
        false
    }

    fn update_unit(&mut self, i: usize, dt: f32) {
        {
            let u = &mut self.units[i];
            u.cooldown -= dt;
            u.flash = (u.flash - dt).max(0.0);
            u.scan_t -= dt;
        }
        let order = self.units[i].order;
        let ni = self.units[i].nation as usize;
        let kind = self.units[i].kind;
        match order {
            Order::Forestry { region, target, cursor } => self.update_forestry(i, region, target, cursor, dt),
            Order::Idle => {
                if kind.military() && self.units[i].scan_t <= 0.0 {
                    self.units[i].scan_t = 0.5;
                    if let Some(t) = self.nearest_enemy_unit(ni, self.units[i].pos, kind.sight()) {
                        let th = self.uh(t);
                        self.units[i].order = Order::Attack(Target::Unit(th));
                    }
                }
                // 自国領内で待機中は回復
                let (x, y) = self.units[i].pos.tile();
                if Map::inb(x, y) && self.owner[Map::idx(x, y)] as usize == ni {
                    let u = &mut self.units[i];
                    u.hp = (u.hp + u.max_hp() * 0.01 * dt).min(u.max_hp());
                }
            }
            Order::Explore(Some(goal)) => self.update_explorer(i, goal, dt),
            Order::Explore(None) => {}
            Order::Move(g) => {
                if self.nav(i, g, 0.25, dt) {
                    self.units[i].order = Order::Idle;
                }
            }
            Order::AttackMove(g) => {
                if self.units[i].scan_t <= 0.0 {
                    self.units[i].scan_t = 0.4;
                    let p = self.units[i].pos;
                    let found = self
                        .nearest_enemy_unit(ni, p, kind.sight())
                        .map(|t| Target::Unit(self.uh(t)))
                        .or_else(|| self.nearest_enemy_bld(ni, p, kind.sight()).map(|b| Target::Bld(self.bh(b))));
                    if let Some(t) = found {
                        self.units[i].order = Order::Attack(t);
                        self.units[i].resume = Some(g);
                        return;
                    }
                }
                if self.nav(i, g, 0.6, dt) {
                    self.units[i].order = Order::Idle;
                }
            }
            Order::Attack(t) => {
                let info = self.target_info(t);
                let valid = match info {
                    Some((_, _, tn)) => self.at_war(ni, tn),
                    None => false,
                };
                if !valid {
                    let r = self.units[i].resume.take();
                    self.units[i].order = match r {
                        Some(g) => Order::AttackMove(g),
                        None => Order::Idle,
                    };
                    self.units[i].scan_t = 0.0;
                    return;
                }
                let (tp, rad, _) = info.unwrap();
                let pos = self.units[i].pos;
                let def = self.units[i].def();
                let range = def.range + rad;
                let d = pos.dist(tp);
                // 進軍中なら深追いしない
                if let (Some(_), Target::Unit(_)) = (self.units[i].resume, t) {
                    if d > kind.sight() * 1.8 {
                        let g = self.units[i].resume.take().unwrap();
                        self.units[i].order = Order::AttackMove(g);
                        return;
                    }
                }
                // 攻撃中も近くに敵兵がいれば建物より優先
                if let Target::Bld(_) = t {
                    if self.units[i].scan_t <= 0.0 {
                        self.units[i].scan_t = 0.6;
                        if let Some(e) = self.nearest_enemy_unit(ni, pos, kind.sight() * 0.8) {
                            if self.units[e].kind.military() {
                                if self.units[i].resume.is_none() {
                                    self.units[i].resume = Some(tp);
                                }
                                self.units[i].order = Order::Attack(Target::Unit(self.uh(e)));
                                return;
                            }
                        }
                    }
                }
                let in_range = match t {
                    Target::Bld(bh) => d <= range || self.bld(bh).map_or(false, |b| b.dist_to_footprint(pos) <= def.range + 0.35),
                    Target::Unit(_) => d <= range,
                };
                if in_range {
                    self.units[i].dir = (tp - pos).norm();
                    if self.units[i].cooldown <= 0.0 {
                        self.units[i].cooldown = def.cd;
                        // 英雄のそばで戦う兵は奮い立つ
                        let aura = if ni < NATIONS.len() { self.hero_aura(ni, pos) } else { 1.0 };
                        let mods = if ni < NATIONS.len() { self.nations[ni].mil_mul * self.nations[ni].tm.atk[kind as usize] * aura } else { 1.0 };
                        let atk = def.atk * self.units[i].power * mods * self.rng.range(0.85, 1.15);
                        let me = self.uh(i);
                        if def.range > 1.5 {
                            self.shots.push(Shot { a: pos, b: tp, t: 0.3 });
                        }
                        match t {
                            Target::Unit(h) => {
                                let ti = h.idx as usize;
                                let tk = self.units[ti].kind;
                                let (tx, ty) = self.units[ti].pos.tile();
                                let hill = Map::inb(tx, ty) && self.map.terrain[Map::idx(tx, ty)] == Terrain::Hill;
                                let dmg = atk * bonus(def, tk) * if hill { 0.8 } else { 1.0 };
                                self.damage_unit(ti, dmg, Some(me), ni);
                            }
                            Target::Bld(h) => {
                                let dmg = atk * def.vs_bld * 0.8;
                                self.damage_bld(h.idx as usize, dmg, ni);
                            }
                        }
                    }
                } else {
                    let (nav_goal, nav_r) = match t {
                        Target::Bld(bh) if def.range < 1.5 && !kind.naval() => (self.bld_approach_point(bh, pos), 0.45),
                        _ => (tp, range * 0.9),
                    };
                    if self.nav(i, nav_goal, nav_r, dt) {
                        let unreachable = match t {
                            Target::Bld(bh) => d > range + 0.5 && self.bld(bh).map_or(true, |b| b.dist_to_footprint(self.units[i].pos) > def.range + 0.65),
                            Target::Unit(_) => d > range + 0.5,
                        };
                        if unreachable {
                            self.units[i].order = Order::Idle;
                        }
                    }
                }
            }
            Order::Gather { src, phase } => self.update_gather(i, src, phase, dt),
            Order::Board(sh) => self.update_board(i, sh, dt),
            Order::Unload(target) => self.update_unload(i, target, dt),
            Order::Ferry(goal, attack) => self.update_ferry(i, goal, attack, dt),
            Order::Shuttle(goal, attack) => self.update_shuttle(i, goal, attack, dt),
            Order::Build(bh) => {
                let Some(b) = self.bld(bh) else {
                    if !self.next_queued_build(i) {
                        self.units[i].order = Order::Idle;
                    }
                    return;
                };
                if b.done() || b.nation as usize != ni {
                    let (kind_b, own) = (b.kind, b.nation as usize == ni);
                    // 予約があれば次の建設へ
                    if self.next_queued_build(i) {
                        return;
                    }
                    if kind_b == BKind::Farm && own && self.farm_users(bh) == 0 {
                        self.units[i].order = Order::Gather { src: Src::Farm(bh), phase: 0 };
                    } else {
                        self.units[i].order = Order::Idle;
                    }
                    return;
                }
                let c = b.center();
                let pos = self.units[i].pos;
                let near = if b.kind.blocks() {
                    b.dist_to_footprint(pos) <= 1.05
                } else {
                    pos.dist(c) <= 1.15 || b.dist_to_footprint(pos) <= 0.45
                };
                if near {
                    self.units[i].path.clear();
                    let d = (c - pos).norm();
                    if d.len() > 0.0 {
                        self.units[i].dir = d;
                    }
                    let pw = self.units[i].power;
                    let b = self.bld_mut(bh).unwrap();
                    let inc = dt * pw / b.kind.build_time();
                    b.progress += inc;
                    b.hp = (b.hp + b.kind.max_hp() * 0.9 * inc).min(b.kind.max_hp());
                    if b.progress >= 1.0 {
                        b.progress = 1.0;
                        let (drop, city) = (b.kind.dropsite(), b.kind.city());
                        if drop {
                            self.territory_dirty = true;
                        }
                        if city {
                            if self.nations[ni].capital.and_then(|h| self.bld(h)).is_none() {
                                let b = self.bld_mut(bh).unwrap();
                                b.kind = BKind::Capital;
                                self.nations[ni].capital = Some(bh);
                            }
                        }
                        if ni == self.player {
                            let nm = self.bld(bh).unwrap().kind.name();
                            self.push_msg(tf!("{}が完成した", "{} completed", "{}建成了", nm), [0.7, 0.9, 1.0]);
                        }
                    }
                } else {
                    let ap = self.bld_approach_point(bh, pos);
                    let nr = if b.kind.blocks() { 0.45 } else { 0.85 };
                    if self.nav(i, ap, nr, dt) {
                        let reached = self.bld(bh).map_or(false, |b| {
                            if b.kind.blocks() {
                                b.dist_to_footprint(self.units[i].pos) <= 1.25
                            } else {
                                self.units[i].pos.dist(b.center()) <= 1.5 || b.dist_to_footprint(self.units[i].pos) <= 0.75
                            }
                        });
                        if !reached {
                            // 辿り着けなければ次の予約へ
                            if !self.next_queued_build(i) {
                                self.units[i].order = Order::Idle;
                            }
                        }
                    }
                }
            }
        }
    }

    fn update_gather(&mut self, i: usize, src: Src, phase: u8, dt: f32) {
        let ni = self.units[i].nation as usize;
        let res = src.res();
        // 資源の位置と有効性
        let (sp, valid, sr) = match src {
            Src::Farm(h) => match self.bld(h) {
                Some(b) if b.nation as usize == ni && b.done() => (b.center(), true, 1.0),
                _ => (v2(0.0, 0.0), false, 0.0),
            },
            Src::Forest(x, y) => (v2(x as f32 + 0.5, y as f32 + 0.5), Map::inb(x, y) && self.map.terrain[Map::idx(x, y)] == Terrain::Forest, 1.2),
            Src::Mine(m) => {
                let mm = &self.map.mines[m];
                (v2(mm.x as f32 + 0.5, mm.y as f32 + 0.5), mm.amount > 0.0, 1.0)
            }
        };
        let cap = 10.0 * self.units[i].power;
        if phase != 2 && !valid {
            // 近くの同種資源へ
            let p = self.units[i].pos;
            let next = match src {
                Src::Forest(..) => self.nearest_forest(p, 8).map(|(x, y)| Src::Forest(x, y)),
                Src::Mine(_) => self.nearest_mine(p, 10.0).map(Src::Mine),
                Src::Farm(_) => self.free_farm(ni, p).map(Src::Farm),
            };
            match next {
                Some(s) => self.units[i].order = Order::Gather { src: s, phase: 0 },
                None => {
                    self.units[i].order = if self.units[i].carry > 0.0 { Order::Gather { src, phase: 2 } } else { Order::Idle };
                }
            }
            return;
        }
        match phase {
            0 => {
                if self.units[i].carry_kind != res {
                    self.units[i].carry = 0.0;
                    self.units[i].carry_kind = res;
                }
                if self.units[i].carry >= cap {
                    self.units[i].order = Order::Gather { src, phase: 2 };
                    return;
                }
                if self.nav(i, sp, sr, dt) {
                    if self.units[i].pos.dist(sp) <= sr + 0.6 {
                        self.units[i].order = Order::Gather { src, phase: 1 };
                    } else {
                        self.units[i].order = Order::Idle;
                    }
                }
            }
            1 => {
                let base = [0.9, 0.8, 0.5][res];
                let n = &self.nations[ni];
                let amt = base * self.units[i].power * n.bias * self.era_mult() * dt;
                let mut take = amt;
                match src {
                    Src::Forest(x, y) => {
                        let ti = Map::idx(x, y);
                        take = take.min(self.map.wood[ti]);
                        self.map.wood[ti] -= take;
                        if self.map.wood[ti] <= 0.0 {
                            self.map.terrain[ti] = Terrain::Plain;
                            if self.bmap[ti] == 0 {
                                self.cost[ti] = 1.0;
                            }
                            self.terrain_dirty.push((x, y));
                        }
                    }
                    Src::Mine(m) => {
                        take = take.min(self.map.mines[m].amount);
                        self.map.mines[m].amount -= take;
                    }
                    Src::Farm(_) => {}
                }
                let sp_dir = (sp - self.units[i].pos).norm();
                let u = &mut self.units[i];
                if sp_dir.len() > 0.0 {
                    u.dir = sp_dir;
                }
                u.carry += take;
                if u.carry >= cap {
                    u.carry = cap;
                    u.order = Order::Gather { src, phase: 2 };
                }
            }
            _ => {
                let p = self.units[i].pos;
                let Some(di) = self.nearest_dropsite(ni, p) else {
                    self.units[i].order = Order::Idle;
                    return;
                };
                let dh = self.bh(di);
                if self.blds[di].dist_to_footprint(p) <= 1.15 {
                    self.units[i].path.clear();
                    let carry = self.units[i].carry * self.nations[ni].eco_mul * self.nations[ni].tm.eco;
                    let k = self.units[i].carry_kind;
                    self.nations[ni].res[k] += carry;
                    self.nations[ni].earned[k] += carry;
                    self.units[i].carry = 0.0;
                    self.units[i].order = Order::Gather { src, phase: 0 };
                } else {
                    let ap = self.bld_approach_point(dh, p);
                    if self.nav(i, ap, 0.5, dt) && self.blds[di].dist_to_footprint(self.units[i].pos) > 1.35 {
                        self.units[i].order = Order::Idle;
                    }
                }
            }
        }
    }

    fn separate(&mut self, dt: f32) {
        // 押し合いは 0.55 以内のユニット同士だけ。格子（4マス）のうち、その範囲にかかる升（たいてい1つ、多くて4つ）だけを見る
        const R: f32 = 0.55;
        let mut pushes: Vec<(usize, V2)> = Vec::new();
        for a in 0..self.units.len() {
            let ua = &self.units[a];
            if !ua.on_map() {
                continue;
            }
            let p = ua.pos;
            let g0x = ((p.x - R) as i32 / GCELL).clamp(0, GW - 1);
            let g1x = ((p.x + R) as i32 / GCELL).clamp(0, GW - 1);
            let g0y = ((p.y - R) as i32 / GCELL).clamp(0, GH - 1);
            let g1y = ((p.y + R) as i32 / GCELL).clamp(0, GH - 1);
            let mut push = v2(0.0, 0.0);
            for gy in g0y..=g1y {
                for gx in g0x..=g1x {
                    for &b in self.grid[(gy * GW + gx) as usize].iter() {
                        let b = b as usize;
                        if a == b {
                            continue;
                        }
                        let ub = &self.units[b];
                        if !ub.on_map() || ub.kind.naval() != ua.kind.naval() {
                            continue;
                        }
                        let d = p - ub.pos;
                        let l2 = d.x * d.x + d.y * d.y;
                        if l2 < R * R {
                            let l = l2.sqrt();
                            let dir = if l > 1e-4 { d * (1.0 / l) } else { v2(((a * 7) % 5) as f32 - 2.0, ((a * 3) % 5) as f32 - 2.0).norm() };
                            push = push + dir * (R - l);
                        }
                    }
                }
            }
            if push.x != 0.0 || push.y != 0.0 {
                pushes.push((a, push));
            }
        }
        for (i, p) in pushes {
            let np = self.units[i].pos + p * (dt * 3.0).min(0.5);
            if self.passable_l(np, self.units[i].kind.naval()) {
                self.units[i].pos = np;
            }
        }
    }

    // ------------------------------------------------------------ 年次イベント
    fn yearly(&mut self) {
        // 連合の終了
        let mut ended = Vec::new();
        for (k, c) in self.coalitions.iter().enumerate() {
            if self.years >= c.end || !self.nations[c.target].alive {
                ended.push(k);
            }
        }
        for k in ended.into_iter().rev() {
            let c = self.coalitions.remove(k);
            for &m in c.members.iter() {
                if self.nations[m].alive && self.nations[c.target].alive {
                    self.propose_peace(m, c.target, 8.0, t!("連合の解散", "the coalition disbanded", "联军解散"));
                }
            }
            let t = tf!("反{}同盟が解散した", "The Anti-{} League has disbanded", "反{}同盟已解散", tx(self.nations[c.target].name));
            self.push_msg(t, [0.9, 0.9, 0.6]);
        }
        self.yearly_heroes();
        self.run_history();
        self.yearly_trade();
        self.refresh_traits();
        // 終了判定
        if self.descent.is_none() {
            self.check_victory();
            // 1453年が暮れても決着がつかなければ、ジャンヌ・ダルクの亡霊が天の軍勢を率いて現れる
            if self.game_over.is_none() && self.year_ad() > END_AD as i32 {
                self.begin_descent();
            }
        } else {
            self.yearly_descent();
        }
    }

    fn start_coalition(&mut self, kind: CoalKind, target: usize) {
        self.start_coalition_ex(kind, target, true);
    }

    /// その国と隣り合う国：領土が接する国と、城邑が近い国（海の向こうの国は船を造れる場合だけ）
    pub fn neighbors_of(&self, t: usize) -> Vec<usize> {
        let nn = NATIONS.len();
        let mut adj = vec![false; nn];
        let tt = t as u8;
        for y in 0..MAP_H as i32 {
            for x in 0..MAP_W as i32 {
                let o = self.owner[Map::idx(x, y)];
                for (dx, dy) in [(1, 0), (0, 1)] {
                    if !Map::inb(x + dx, y + dy) {
                        continue;
                    }
                    let p = self.owner[Map::idx(x + dx, y + dy)];
                    if o == p || o == NONE || p == NONE {
                        continue;
                    }
                    if o == tt && (p as usize) < nn {
                        adj[p as usize] = true;
                    } else if p == tt && (o as usize) < nn {
                        adj[o as usize] = true;
                    }
                }
            }
        }
        let mine: Vec<V2> = self.blds.iter().filter(|b| b.alive && b.kind.city() && b.done() && b.nation as usize == t).map(|b| b.center()).collect();
        for b in self.blds.iter() {
            let o = b.nation as usize;
            if !b.alive || !b.kind.city() || !b.done() || o == t || o >= nn || adj[o] {
                continue;
            }
            let c = b.center();
            if mine.iter().any(|&m| m.dist(c) <= NEIGHBOR_RANGE && (self.same_land(m, c) || self.can_build_ships(o))) {
                adj[o] = true;
            }
        }
        (0..nn).filter(|&i| adj[i] && i != t && self.nations[i].alive).collect()
    }

    /// 反〇〇同盟を結成する（標的と隣り合う国だけが加わる）。announce が偽なら通知は呼び出し側に任せる
    fn start_coalition_ex(&mut self, kind: CoalKind, target: usize, announce: bool) -> bool {
        let members: Vec<usize> = self.neighbors_of(target);
        // 最も強い参加国（記録用。同盟を率いる盟主ではない）
        let leader = members.iter().copied().max_by(|&a, &b| self.nations[a].strength.partial_cmp(&self.nations[b].strength).unwrap());
        let Some(leader) = leader else {
            self.nations[target].hegemon_cd = self.years + 10.0;
            return false;
        };
        if members.len() < 2 {
            self.nations[target].hegemon_cd = self.years + 10.0;
            return false;
        }
        // 参加国同士は休戦し、一斉に宣戦
        for a in 0..members.len() {
            for b in a + 1..members.len() {
                let (ma, mb) = (members[a], members[b]);
                if self.war[ma][mb] {
                    self.propose_peace(ma, mb, 15.0, t!("連合への参加", "joining a coalition", "加入联军"));
                }
            }
        }
        for &m in members.iter() {
            self.declare_war(m, target, true);
        }
        let dur = 15.0;
        let tn = tx(self.nations[target].name);
        let mut names: Vec<&str> = members.iter().take(5).map(|&m| tx(self.nations[m].name)).collect();
        if members.len() > 5 {
            names.push(t!("など", "and others", "等"));
        }
        let list = names.join(sep());
        let title = tf!("反{}同盟", "The Anti-{} League", "反{}同盟", tn);
        let text = match kind {
            CoalKind::Hegemon => tf!(
                "{0} の度重なる侵略に、国境を接する {1} の {2} か国が反{0}同盟を結び、ともに兵を挙げた（{3}年間）",
                "Alarmed by {0}'s repeated aggression, its {2} neighbours ({1}) have formed the Anti-{0} League and taken up arms together ({3} years)",
                "面对{0}屡次侵略，接壤的{1}等{2}国结成反{0}同盟，共同起兵（{3}年）",
                tn, list, members.len(), dur as i32
            ),
            CoalKind::Hezong => tf!(
                "強大になりすぎた {0} を恐れ、隣り合う {1} の {2} か国が反{0}同盟を結んだ（{3}年間）",
                "Fearing the overwhelming might of {0}, its {2} neighbours ({1}) have formed the Anti-{0} League ({3} years)",
                "畏惧过于强大的{0}，相邻的{1}等{2}国结成反{0}同盟（{3}年）",
                tn, list, members.len(), dur as i32
            ),
        };
        if announce {
            self.push_msg(format!("【{}】{}", title, text), [1.0, 0.85, 0.3]);
            self.queue_banner(Banner { title, text, t: 7.0 });
        }
        self.nations[target].aggression = 0.0;
        self.nations[target].hegemon_cd = self.years + 40.0;
        self.coalitions.push(Coalition { kind, target, leader, members, end: self.years + dur });
        true
    }

    /// ランカスター朝の戦争（1415年）：時の流れが少し遅くなる
    fn begin_empire(&mut self) {
        self.era = Era::Empire;
        self.hezong_next = self.years + 15.0;
        let text = t!("ヘンリー5世の時代。オルレアン派とブルゴーニュ派に割れたフランスへ、イングランドは征服の戦を挑む。大砲が城壁を崩し、鍛錬の上限が上がる。", "The age of Henry V. With France split between Armagnacs and Burgundians, England launches a war of conquest. Cannon batter down walls, and the training limit rises.", "亨利五世的时代。法兰西分裂为阿马尼亚克派与勃艮第派，英格兰发动征服之战。大炮轰塌城墙，训练上限提高。").to_string();
        self.push_msg(tf!("【ランカスター朝の戦争】{}", "[The Lancastrian War] {}", "【兰开斯特王朝战争】{}", text), [1.0, 0.85, 0.3]);
        self.queue_banner(Banner { title: t!("ランカスター朝の戦争", "The Lancastrian War", "兰开斯特王朝战争").into(), text, t: 8.0 });
        self.update_stats();
    }

    fn found_city_free(&mut self, nation: usize, lon: f32, lat: f32) -> Option<Handle> {
        let (fx, fy) = ll_to_tile(lon, lat);
        let (cx, cy) = (fx as i32 - 1, fy as i32 - 1);
        for r in 0..12i32 {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r {
                        continue;
                    }
                    let (tx, ty) = (cx + dx, cy + dy);
                    if !self.footprint_free(tx, ty) {
                        continue;
                    }
                    let c = v2(tx as f32 + 1.0, ty as f32 + 1.0);
                    if self.blds.iter().any(|b| b.alive && b.kind.city() && b.center().dist(c) < 6.0) {
                        continue;
                    }
                    return Some(self.spawn_building(nation, BKind::Capital, tx, ty, true));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests;
