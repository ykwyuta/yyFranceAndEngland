//! 地理データ（ブリテン島とフランスの海岸線・河川・山地）と国家・街・諸侯のデータ、地形生成

use crate::geodata::{LAKE_RINGS, LAND_RINGS, NE_RIVERS};
use crate::rng::{fbm, hash2, Rng};

/// 地図の細かさの倍率（旧来の名残。1で固定）
pub const MAP_SCALE: usize = 1;
/// 経度1度あたり20マス、緯度1度あたり30マス（北緯49度付近で縦横がほぼ揃う）
pub const MAP_W: usize = 284;
pub const MAP_H: usize = 408;
pub const LON0: f32 = -5.8;
pub const LON1: f32 = 8.4;
pub const LAT_TOP: f32 = 56.0;
pub const LAT_BOT: f32 = 42.4;

pub fn ll_to_tile(lon: f32, lat: f32) -> (f32, f32) {
    (
        (lon - LON0) / (LON1 - LON0) * MAP_W as f32,
        (LAT_TOP - lat) / (LAT_TOP - LAT_BOT) * MAP_H as f32,
    )
}
pub fn tile_to_ll(x: f32, y: f32) -> (f32, f32) {
    (
        LON0 + x / MAP_W as f32 * (LON1 - LON0),
        LAT_TOP - y / MAP_H as f32 * (LAT_TOP - LAT_BOT),
    )
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Terrain {
    Sea,
    Lake,
    River,
    Plain,
    /// 荒れ地（ランド・ヒース）
    Steppe,
    Forest,
    Hill,
    Mountain,
    Desert,
}

impl Terrain {
    pub fn passable(self) -> bool {
        !matches!(self, Terrain::Sea | Terrain::Lake | Terrain::Mountain)
    }
    pub fn move_cost(self) -> f32 {
        match self {
            Terrain::Plain | Terrain::Steppe => 1.0,
            Terrain::Desert => 1.3,
            Terrain::Forest => 1.6,
            Terrain::Hill => 1.5,
            Terrain::River => 2.5,
            _ => f32::INFINITY,
        }
    }
    /// 船の移動コスト（海・湖・大河を進む）
    pub fn sail_cost(self) -> f32 {
        match self {
            Terrain::Sea | Terrain::Lake => 1.0,
            Terrain::River => 1.4,
            _ => f32::INFINITY,
        }
    }
    pub fn water(self) -> bool {
        matches!(self, Terrain::Sea | Terrain::Lake)
    }
    /// 建物を置ける地形
    pub fn buildable(self) -> bool {
        matches!(self, Terrain::Plain | Terrain::Steppe | Terrain::Hill | Terrain::Desert)
    }
}

// ---------------------------------------------------------------- 河川（手で補うもの）
struct River {
    pts: &'static [(f32, f32)],
    major: bool,
}
const RIVERS: &[River] = &[
    // ソンム川
    River { major: false, pts: &[(3.20, 49.85), (2.90, 49.86), (2.40, 49.90), (2.00, 50.02), (1.70, 50.15)] },
    // オワーズ川
    River { major: false, pts: &[(3.70, 49.80), (3.30, 49.65), (2.95, 49.42), (2.60, 49.25), (2.30, 49.07), (2.10, 48.99)] },
    // シャラント川
    River { major: false, pts: &[(0.40, 45.90), (0.15, 45.66), (-0.33, 45.70), (-0.63, 45.75), (-0.95, 45.95)] },
    // ロット川
    River { major: false, pts: &[(3.20, 44.50), (2.50, 44.50), (1.80, 44.48), (1.44, 44.45), (1.00, 44.50), (0.40, 44.33)] },
    // シェール川
    River { major: false, pts: &[(2.60, 46.40), (2.40, 47.00), (1.80, 47.25), (1.20, 47.30), (0.70, 47.35)] },
    // アリエ川
    River { major: false, pts: &[(3.70, 44.80), (3.40, 45.30), (3.30, 45.90), (3.30, 46.50), (3.10, 46.95)] },
    // ヴィレーヌ川
    River { major: false, pts: &[(-1.20, 48.20), (-1.68, 48.10), (-1.90, 47.80), (-2.20, 47.50)] },
    // ウーズ川（ヨーク）
    River { major: false, pts: &[(-1.60, 54.20), (-1.30, 54.05), (-1.08, 53.96), (-0.90, 53.75), (-0.70, 53.70)] },
    // ツイード川（スコットランドとの境）
    River { major: false, pts: &[(-3.20, 55.62), (-2.70, 55.60), (-2.30, 55.65), (-2.00, 55.76)] },
    // タイン川
    River { major: false, pts: &[(-2.30, 54.97), (-1.90, 54.97), (-1.61, 54.97), (-1.45, 55.00)] },
];

// 山脈（ポリライン, 幅 deg）。hill=true は丘陵までにとどめる（越えられる）
struct Range {
    pts: &'static [(f32, f32)],
    w: f32,
    hill: bool,
}
macro_rules! rg {
    ($w:expr, $p:expr) => {
        Range { w: $w, hill: false, pts: $p }
    };
    (hill $w:expr, $p:expr) => {
        Range { w: $w, hill: true, pts: $p }
    };
}
const RANGES: &[Range] = &[
    // ---- フランス
    rg!(0.30, &[(-1.60, 43.05), (-0.80, 42.86), (0.20, 42.72), (1.00, 42.62), (1.80, 42.55), (2.50, 42.48), (3.00, 42.45)]), // ピレネー
    rg!(0.40, &[(7.00, 44.00), (6.95, 44.60), (6.85, 45.20), (7.00, 45.80), (7.60, 46.10), (8.40, 46.40)]),                     // アルプス
    rg!(hill 0.30, &[(5.60, 44.40), (5.75, 45.10), (6.20, 45.80)]),                                                                 // ドーフィネの前山
    rg!(hill 0.22, &[(5.60, 46.30), (6.20, 46.80), (6.90, 47.30)]),                                                                 // ジュラ
    rg!(hill 0.22, &[(6.85, 47.80), (7.10, 48.30), (7.20, 48.80)]),                                                                 // ヴォージュ
    rg!(0.18, &[(2.50, 45.20), (2.80, 45.50), (3.00, 45.75)]),                                                                       // オーヴェルニュの火山（ピュイ山地）
    rg!(hill 0.28, &[(3.40, 44.10), (3.90, 44.40), (4.30, 44.90)]),                                                                 // セヴェンヌ
    // ---- ブリテン
    rg!(hill 0.25, &[(-2.10, 53.00), (-2.20, 53.60), (-2.20, 54.20), (-2.40, 54.80)]),                                             // ペナイン
    rg!(0.12, &[(-3.30, 54.40), (-3.00, 54.55)]),                                                                                    // 湖水地方
    rg!(0.14, &[(-4.10, 52.95), (-3.80, 53.10)]),                                                                                    // スノードニア
    rg!(hill 0.25, &[(-3.70, 55.35), (-3.00, 55.40), (-2.30, 55.40)]),                                                             // サザン・アップランド／チェヴィオット
];

/// 山越えの道（山脈を貫いて通れる幅を残す）
pub struct Road {
    pub pts: &'static [(f32, f32)],
}
pub const ROADS: &[Road] = &[
    // ロンスヴォーの峠（ピレネー越え）
    Road { pts: &[(-1.45, 43.40), (-1.30, 43.00), (-1.25, 42.60)] },
];

/// 海を陸に変える細い地峡（地図の目が粗く切れてしまう狭い所を、歩いて渡れるようにする）
const BRIDGES: &[&[(f32, f32)]] = &[];

fn dist_seg(px: f32, py: f32, a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((px - a.0) * dx + (py - a.1) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    let (cx, cy) = (a.0 + dx * t, a.1 + dy * t);
    ((px - cx).powi(2) + (py - cy).powi(2)).sqrt()
}

fn dist_poly(px: f32, py: f32, pts: &[(f32, f32)]) -> f32 {
    let mut d = f32::MAX;
    for w in pts.windows(2) {
        d = d.min(dist_seg(px, py, w[0], w[1]));
    }
    d
}

/// 輪の集まりを偶奇規則で塗る（行ごとに辺との交点を求める）
fn scan_fill(rings: &[&[(f32, f32)]], mut f: impl FnMut(i32, i32)) {
    let mut xs: Vec<f32> = Vec::new();
    for y in 0..MAP_H as i32 {
        let (_, lat) = tile_to_ll(0.0, y as f32 + 0.5);
        xs.clear();
        for r in rings {
            let n = r.len();
            for i in 0..n {
                let (x0, y0) = r[i];
                let (x1, y1) = r[(i + 1) % n];
                if (y0 > lat) != (y1 > lat) {
                    xs.push(x0 + (lat - y0) / (y1 - y0) * (x1 - x0));
                }
            }
        }
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for k in (0..xs.len().saturating_sub(1)).step_by(2) {
            let (a, _) = ll_to_tile(xs[k], lat);
            let (b, _) = ll_to_tile(xs[k + 1], lat);
            let a = (a - 0.5).ceil().max(0.0) as i32;
            let b = ((b - 0.5).floor() as i32).min(MAP_W as i32 - 1);
            for x in a..=b {
                f(x, y);
            }
        }
    }
}

/// 楕円の内側か（中心・半径は経緯度）
fn in_ell(lon: f32, lat: f32, c: (f32, f32, f32, f32)) -> bool {
    ((lon - c.0) / c.2).powi(2) + ((lat - c.1) / c.3).powi(2) < 1.0
}

/// 丘陵の多い地域（中心経度・緯度・半径）
const HILLS: &[(f32, f32, f32, f32)] = &[
    (2.95, 45.20, 1.35, 1.00),  // 中央山地
    (3.95, 47.15, 0.35, 0.30),  // モルヴァン
    (4.90, 49.90, 1.00, 0.35),  // アルデンヌ
    (-3.75, 52.30, 0.85, 0.95), // ウェールズ
    (-3.10, 54.50, 0.35, 0.30), // カンブリア
    (-3.90, 50.60, 0.30, 0.15), // ダートムーア
    (-3.60, 48.35, 0.80, 0.25), // アレー山地（ブルターニュ）
    (-3.40, 55.70, 1.60, 0.40), // スコットランド南部の丘
];

/// 荒れ地（ランドの松林と砂地）
const HEATHS: &[(f32, f32, f32, f32)] = &[
    (-0.90, 44.20, 0.55, 0.55), // ランド
    (1.90, 47.55, 0.45, 0.20),  // ソローニュ
];

// ---------------------------------------------------------------- 国家データ
pub struct NationDef {
    pub name: &'static str,
    pub lon: f32,
    pub lat: f32,
    /// 生産能力バイアス（始まりの国力。出来事で上下する）
    pub bias: f32,
    pub color: [f32; 3],
    /// 大国（始まりの時点で強い）
    pub major: bool,
    pub desc: &'static str,
}

pub const NATIONS: &[NationDef] = &[
    NationDef { name: "イングランド", lon: -0.13, lat: 51.51, bias: 1.05, color: [0.82, 0.16, 0.20], major: false, desc: "ロンドンを都とする王国。海峡の向こうにボルドーとポンティユーを持ち、フランス王位を求める。船を造れる" },
    NationDef { name: "フランス", lon: 2.35, lat: 48.86, bias: 1.25, color: [0.20, 0.36, 0.86], major: true, desc: "パリを都とする西欧一の大国。諸侯の忠誠は揺れ動く。船は造れず、ガレー船を雇う" },
];

pub const ENGLAND: usize = 0;
pub const FRANCE: usize = 1;

/// 相手の国
pub fn foe(n: usize) -> usize {
    if n == ENGLAND {
        FRANCE
    } else {
        ENGLAND
    }
}

// ---------------------------------------------------------------- 諸侯
/// 街を治める諸侯。史実の出来事で、諸侯が寝返ったり同盟したりすると、その街はもう一方の王に付く
pub struct LordDef {
    pub name: &'static str,
    pub desc: &'static str,
}

pub const LORDS: &[LordDef] = &[
    LordDef { name: "王領", desc: "王が直接治める土地" },
    LordDef { name: "フランドル", desc: "毛織物の都市ヘント・ブルッヘ・リール。イングランドの羊毛なしには立ち行かない" },
    LordDef { name: "ブルターニュ", desc: "ブルターニュ公の土地。継承戦争の後も、英仏の間で揺れ動いた" },
    LordDef { name: "モンフォール派", desc: "ブルターニュ継承戦争でイングランドと結んだモンフォール家の拠点（ブレスト・ヴァンヌ）" },
    LordDef { name: "ナバラ王", desc: "エヴルー家のナバラ王シャルル（悪王）がノルマンディーに持つ土地" },
    LordDef { name: "ブルゴーニュ公", desc: "ヴァロワ家の分家。1419年のモントローの暗殺の後、イングランドと結ぶ" },
    LordDef { name: "ガスコーニュ諸侯", desc: "アルブレ家・アルマニャック家などのガスコーニュの領主たち" },
    LordDef { name: "パリ市民", desc: "1418年にブルゴーニュ派が掌握したパリ" },
    LordDef { name: "シャンパーニュの諸都市", desc: "ランス・トロワ・ラン・コンピエーニュなど、トロワ条約でイングランド王に従った北の都市" },
    LordDef { name: "ウェールズ", desc: "1400年にオワイン・グリンドゥールが反乱を起こし、フランスと結んだ" },
    LordDef { name: "ギュイエンヌの都市", desc: "ボルドーとバイヨンヌ。三百年来イングランド王に仕え、ワインの交易で結ばれている" },
];

pub const L_CROWN: u8 = 0;
pub const L_FLANDERS: u8 = 1;
pub const L_BRITTANY: u8 = 2;
pub const L_MONTFORT: u8 = 3;
pub const L_NAVARRE: u8 = 4;
pub const L_BURGUNDY: u8 = 5;
pub const L_GASCON: u8 = 6;
pub const L_PARIS: u8 = 7;
pub const L_CHAMPAGNE: u8 = 8;
pub const L_WALES: u8 = 9;
pub const L_GUYENNE: u8 = 10;

// ---------------------------------------------------------------- 街
/// 最初からある主要な街（街は新しく築けない）
pub struct TownDef {
    pub name: &'static str,
    pub lon: f32,
    pub lat: f32,
    /// 1337年に治める国
    pub owner: usize,
    /// 治める諸侯
    pub lord: u8,
}

macro_rules! town {
    ($n:expr, $lon:expr, $lat:expr, $o:ident, $l:ident) => {
        TownDef { name: $n, lon: $lon, lat: $lat, owner: $o, lord: $l }
    };
}

/// 街の一覧。各国の最初の街が都
pub const TOWNS: &[TownDef] = &[
    // ---- イングランド本国
    town!("ロンドン", -0.13, 51.51, ENGLAND, L_CROWN),
    town!("ドーヴァー", 1.31, 51.13, ENGLAND, L_CROWN),
    town!("サウサンプトン", -1.40, 50.90, ENGLAND, L_CROWN),
    town!("エクセター", -3.53, 50.72, ENGLAND, L_CROWN),
    town!("プリマス", -4.14, 50.37, ENGLAND, L_CROWN),
    town!("ブリストル", -2.59, 51.45, ENGLAND, L_CROWN),
    town!("コヴェントリー", -1.51, 52.41, ENGLAND, L_CROWN),
    town!("ノリッジ", 1.30, 52.63, ENGLAND, L_CROWN),
    town!("リンカン", -0.54, 53.23, ENGLAND, L_CROWN),
    town!("ヨーク", -1.08, 53.96, ENGLAND, L_CROWN),
    town!("チェスター", -2.89, 53.19, ENGLAND, L_CROWN),
    town!("ニューカッスル", -1.61, 54.97, ENGLAND, L_CROWN),
    town!("カーライル", -2.93, 54.89, ENGLAND, L_CROWN),
    town!("ベリック", -2.00, 55.77, ENGLAND, L_CROWN),
    town!("ハーレック", -4.05, 52.86, ENGLAND, L_WALES),
    town!("カーマーゼン", -4.31, 51.86, ENGLAND, L_WALES),
    // ---- 大陸のイングランド領（ギュイエンヌ・ポンティユー）
    town!("ボルドー", -0.58, 44.84, ENGLAND, L_GUYENNE),
    town!("バイヨンヌ", -1.47, 43.49, ENGLAND, L_GUYENNE),
    town!("モン＝ド＝マルサン", -0.50, 43.89, ENGLAND, L_GASCON),
    town!("アブヴィル", 1.83, 50.10, ENGLAND, L_CROWN),
    // ---- フランス
    town!("パリ", 2.35, 48.86, FRANCE, L_PARIS),
    town!("ルーアン", 1.10, 49.44, FRANCE, L_CROWN),
    town!("アルフルール", 0.20, 49.51, FRANCE, L_CROWN),
    town!("ディエップ", 1.08, 49.92, FRANCE, L_CROWN),
    town!("カーン", -0.36, 49.18, FRANCE, L_CROWN),
    town!("シェルブール", -1.62, 49.64, FRANCE, L_NAVARRE),
    town!("エヴルー", 1.15, 49.02, FRANCE, L_NAVARRE),
    town!("モン＝サン＝ミシェル", -1.51, 48.64, FRANCE, L_CROWN),
    town!("カレー", 1.86, 50.95, FRANCE, L_CROWN),
    town!("アミアン", 2.30, 49.89, FRANCE, L_CROWN),
    town!("ボーヴェ", 2.08, 49.43, FRANCE, L_CHAMPAGNE),
    town!("コンピエーニュ", 2.83, 49.42, FRANCE, L_CHAMPAGNE),
    town!("ラン", 3.62, 49.56, FRANCE, L_CHAMPAGNE),
    town!("ランス", 4.03, 49.26, FRANCE, L_CHAMPAGNE),
    town!("トロワ", 4.07, 48.30, FRANCE, L_CHAMPAGNE),
    town!("アラス", 2.78, 50.29, FRANCE, L_BURGUNDY),
    town!("リール", 3.06, 50.63, FRANCE, L_FLANDERS),
    town!("ヘント", 3.72, 51.05, FRANCE, L_FLANDERS),
    town!("ブルッヘ", 3.22, 51.21, FRANCE, L_FLANDERS),
    town!("ディジョン", 5.04, 47.32, FRANCE, L_BURGUNDY),
    town!("オセール", 3.57, 47.80, FRANCE, L_BURGUNDY),
    town!("マコン", 4.83, 46.31, FRANCE, L_BURGUNDY),
    town!("ヴォークルール", 5.67, 48.60, FRANCE, L_CROWN),
    town!("シャルトル", 1.49, 48.45, FRANCE, L_CROWN),
    town!("ル・マン", 0.20, 48.00, FRANCE, L_CROWN),
    town!("アランソン", 0.09, 48.43, FRANCE, L_CROWN),
    town!("フージェール", -1.20, 48.35, FRANCE, L_BRITTANY),
    town!("レンヌ", -1.68, 48.11, FRANCE, L_BRITTANY),
    town!("サン＝マロ", -2.00, 48.65, FRANCE, L_BRITTANY),
    town!("ナント", -1.55, 47.22, FRANCE, L_BRITTANY),
    town!("ヴァンヌ", -2.76, 47.66, FRANCE, L_MONTFORT),
    town!("ブレスト", -4.49, 48.39, FRANCE, L_MONTFORT),
    town!("アンジェ", -0.55, 47.47, FRANCE, L_CROWN),
    town!("トゥール", 0.69, 47.39, FRANCE, L_CROWN),
    town!("ブロワ", 1.33, 47.59, FRANCE, L_CROWN),
    town!("オルレアン", 1.91, 47.90, FRANCE, L_CROWN),
    town!("ブールジュ", 2.40, 47.08, FRANCE, L_CROWN),
    town!("ヌヴェール", 3.16, 46.99, FRANCE, L_CROWN),
    town!("ポワティエ", 0.34, 46.58, FRANCE, L_CROWN),
    town!("ラ・ロシェル", -1.15, 46.16, FRANCE, L_CROWN),
    town!("サント", -0.63, 45.75, FRANCE, L_CROWN),
    town!("アングレーム", 0.16, 45.65, FRANCE, L_CROWN),
    town!("リモージュ", 1.26, 45.83, FRANCE, L_CROWN),
    town!("ペリグー", 0.72, 45.18, FRANCE, L_CROWN),
    town!("ベルジュラック", 0.48, 44.85, FRANCE, L_CROWN),
    town!("アジャン", 0.62, 44.20, FRANCE, L_CROWN),
    town!("カオール", 1.44, 44.45, FRANCE, L_CROWN),
    town!("トゥールーズ", 1.44, 43.60, FRANCE, L_CROWN),
    town!("オーシュ", 0.59, 43.65, FRANCE, L_GASCON),
    town!("タルブ", 0.07, 43.23, FRANCE, L_GASCON),
    town!("カルカソンヌ", 2.35, 43.21, FRANCE, L_CROWN),
    town!("ロデーズ", 2.57, 44.35, FRANCE, L_GASCON),
    town!("クレルモン", 3.08, 45.78, FRANCE, L_CROWN),
    town!("ル・ピュイ", 3.88, 45.04, FRANCE, L_CROWN),
    town!("リヨン", 4.84, 45.76, FRANCE, L_CROWN),
    town!("ニーム", 4.36, 43.84, FRANCE, L_CROWN),
];

/// 街の番号（名前から。テスト・年表用）
pub fn town_id(name: &str) -> usize {
    TOWNS.iter().position(|t| t.name == name).unwrap_or_else(|| panic!("街が無い: {}", name))
}

// ---------------------------------------------------------------- 地図
pub struct Mine {
    pub x: i32,
    pub y: i32,
    pub amount: f32,
}

pub struct Map {
    pub terrain: Vec<Terrain>,
    pub wood: Vec<f32>,
    pub shade: Vec<f32>,
    pub mines: Vec<Mine>,
}

impl Map {
    #[inline]
    pub fn idx(x: i32, y: i32) -> usize {
        y as usize * MAP_W + x as usize
    }
    #[inline]
    pub fn inb(x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < MAP_W as i32 && y < MAP_H as i32
    }
    pub fn t(&self, x: i32, y: i32) -> Terrain {
        if Self::inb(x, y) {
            self.terrain[Self::idx(x, y)]
        } else {
            Terrain::Mountain
        }
    }
}

fn raster_line(map: &mut Map, a: (f32, f32), b: (f32, f32), thick: bool) {
    let (ax, ay) = ll_to_tile(a.0, a.1);
    let (bx, by) = ll_to_tile(b.0, b.1);
    let len = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt();
    let steps = (len * 4.0).ceil().max(1.0) as i32;
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let x = (ax + (bx - ax) * t).floor() as i32;
        let y = (ay + (by - ay) * t).floor() as i32;
        let pts: &[(i32, i32)] = if thick { &[(0, 0), (1, 0), (0, 1)] } else { &[(0, 0)] };
        for &(dx, dy) in pts {
            let (xx, yy) = (x + dx, y + dy);
            if Map::inb(xx, yy) {
                let i = Map::idx(xx, yy);
                if map.terrain[i] != Terrain::Sea && map.terrain[i] != Terrain::Lake {
                    map.terrain[i] = Terrain::River;
                }
            }
        }
    }
}

/// 線に沿ってマスを書き換える（f が真を返したマスだけ）
fn along_line(pts: &[(f32, f32)], width: i32, mut f: impl FnMut(usize)) {
    for w in pts.windows(2) {
        let (ax, ay) = ll_to_tile(w[0].0, w[0].1);
        let (bx, by) = ll_to_tile(w[1].0, w[1].1);
        let len = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt();
        let steps = (len * 3.0).ceil().max(1.0) as i32;
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let x = (ax + (bx - ax) * t).floor() as i32;
            let y = (ay + (by - ay) * t).floor() as i32;
            for dy in -width..=width {
                for dx in -width..=width {
                    if Map::inb(x + dx, y + dy) {
                        f(Map::idx(x + dx, y + dy));
                    }
                }
            }
        }
    }
}

pub fn generate_map(seed: u32) -> Map {
    let n = MAP_W * MAP_H;
    let mut map = Map { terrain: vec![Terrain::Sea; n], wood: vec![0.0; n], shade: vec![0.0; n], mines: Vec::new() };
    // 陸地
    let mut land = vec![false; n];
    scan_fill(LAND_RINGS, |x, y| land[Map::idx(x, y)] = true);

    for y in 0..MAP_H as i32 {
        for x in 0..MAP_W as i32 {
            let (lon, lat) = tile_to_ll(x as f32 + 0.5, y as f32 + 0.5);
            let i = Map::idx(x, y);
            let nz = fbm(x as f32 * 0.08, y as f32 * 0.08, seed, 4);
            let nz2 = fbm(x as f32 * 0.15, y as f32 * 0.15, seed ^ 0x55aa, 3);
            map.shade[i] = nz;
            if !land[i] {
                map.terrain[i] = Terrain::Sea;
                continue;
            }
            let mut t = Terrain::Plain;
            let wlon = lon + (nz - 0.5) * 0.9 + (nz2 - 0.5) * 0.4;
            let wlat = lat + (nz2 - 0.5) * 0.6 + (nz - 0.5) * 0.3;
            // 荒れ地（ランド・ソローニュ）
            if HEATHS.iter().any(|&d| in_ell(wlon, wlat, d)) {
                t = Terrain::Steppe;
            }
            // 丘陵の多い地域：中央山地・アルデンヌ・ウェールズ・ブルターニュ内陸など
            let hilly = HILLS.iter().any(|&d| in_ell(wlon, wlat, d)) || lat > 55.0;
            if hilly {
                if nz2 > 0.40 {
                    t = Terrain::Hill;
                }
                if nz2 > 0.72 {
                    t = Terrain::Mountain;
                }
            }
            // 山脈
            for r in RANGES {
                let d = dist_poly(lon, lat, r.pts);
                let jitter = (nz2 - 0.5) * r.w * 0.8;
                if d < r.w * 0.55 + jitter {
                    t = if r.hill { Terrain::Hill } else { Terrain::Mountain };
                    if !r.hill {
                        break;
                    }
                } else if d < r.w * 1.5 + jitter && t != Terrain::Mountain {
                    t = Terrain::Hill;
                }
            }
            // 森林（オルレアンの森・アルデンヌ・ブロセリアンド・シャーウッドなど、中世の大森林）
            if matches!(t, Terrain::Plain | Terrain::Steppe) {
                let fthr = if t == Terrain::Steppe {
                    0.56
                } else if in_ell(lon, lat, (4.9, 49.9, 1.1, 0.45)) {
                    0.48
                } else if lat < 44.0 && lon > 2.5 {
                    // ラングドックは森が少ない
                    0.75
                } else {
                    0.62
                };
                if nz > fthr {
                    t = Terrain::Forest;
                }
            }
            map.terrain[i] = t;
        }
    }
    // 湖
    scan_fill(LAKE_RINGS, |x, y| map.terrain[Map::idx(x, y)] = Terrain::Lake);
    // 河川
    // 地図の目が細かいので、大河も一筋で描く（太く描くと蛇行が湖のようになる）
    for &(_major, pts) in NE_RIVERS {
        for w in pts.windows(2) {
            raster_line(&mut map, w[0], w[1], false);
        }
    }
    for r in RIVERS {
        for w in r.pts.windows(2) {
            raster_line(&mut map, w[0], w[1], r.major);
        }
    }
    // 狭い海峡を陸でつなぐ
    for b in BRIDGES {
        let t = &mut map.terrain;
        along_line(b, 0, |i| {
            if t[i] == Terrain::Sea {
                t[i] = Terrain::Hill;
            }
        });
    }
    // 山越えの道：山を丘に削り、通れるようにする
    for r in ROADS {
        let t = &mut map.terrain;
        along_line(r.pts, 0, |i| {
            if matches!(t[i], Terrain::Mountain | Terrain::Forest) {
                t[i] = Terrain::Hill;
            }
        });
    }
    // 国都の周囲を平地にし、木材・鉄を保証
    let mut rng = Rng::new(seed as u64 ^ 0x1234);
    for td in TOWNS {
        {
            let (lon, lat) = (td.lon, td.lat);
            let (cx, cy) = ll_to_tile(lon, lat);
            let (cx, cy) = (cx as i32, cy as i32);
            for dy in -4..=5i32 {
                for dx in -4..=5i32 {
                    let (x, y) = (cx + dx, cy + dy);
                    if Map::inb(x, y) && dx * dx + dy * dy <= 22 {
                        let i = Map::idx(x, y);
                        if !matches!(map.terrain[i], Terrain::Sea | Terrain::River | Terrain::Lake) || (dx.abs() <= 2 && dy.abs() <= 2 && map.terrain[i] != Terrain::Sea) {
                            map.terrain[i] = Terrain::Plain;
                        }
                    }
                }
            }
            ensure_forest(&mut map, cx, cy, &mut rng);
        }
    }
    for i in 0..n {
        if map.terrain[i] == Terrain::Forest {
            map.wood[i] = 120.0;
        }
    }
    // 鉱山
    for y in 2..MAP_H as i32 - 2 {
        for x in 2..MAP_W as i32 - 2 {
            if map.terrain[Map::idx(x, y)] == Terrain::Hill && hash2(x, y, seed ^ 0x77) < 0.035 {
                if map.mines.iter().all(|m| (m.x - x).abs() + (m.y - y).abs() > 10) {
                    map.mines.push(Mine { x, y, amount: 2500.0 });
                }
            }
        }
    }
    for td in TOWNS {
        let (cx, cy) = ll_to_tile(td.lon, td.lat);
        ensure_mine(&mut map, cx as i32, cy as i32, &mut rng);
    }
    connect_seas(&mut map);
    connect_regions(&mut map);
    map
}

/// 地図の目が粗くて外海から切り離された入り江・河口（ジロンド・テムズなど）を、細い水路で外海につなぐ
fn connect_seas(map: &mut Map) {
    let mut done: Vec<u32> = Vec::new();
    for _ in 0..80 {
        let n = MAP_W * MAP_H;
        let (comp, sizes) = components(|i| map.terrain[i] == Terrain::Sea);
        if sizes.len() <= 1 {
            return;
        }
        let main = sizes.iter().enumerate().max_by_key(|(_, s)| **s).unwrap().0 as u32;
        let Some(tid) = (0..sizes.len()).find(|&k| k as u32 != main && !done.contains(&(k as u32))) else { return };
        let tid = tid as u32;
        // 切り離された水域と外海の、いちばん近い二点
        let pts: Vec<usize> = (0..n).filter(|&i| comp[i] == tid).collect();
        let mut best = (i32::MAX, 0usize, 0usize);
        for i in 0..n {
            if comp[i] != main {
                continue;
            }
            let (x, y) = ((i % MAP_W) as i32, (i / MAP_W) as i32);
            for &j in pts.iter().step_by(1 + pts.len() / 64) {
                let (sx, sy) = ((j % MAP_W) as i32, (j / MAP_W) as i32);
                let d = (x - sx).pow(2) + (y - sy).pow(2);
                if d < best.0 {
                    best = (d, j, i);
                }
            }
        }
        if best.0 > 30 * 30 {
            // 遠すぎる：湖として残す
            for &j in pts.iter() {
                map.terrain[j] = Terrain::Lake;
            }
            done.clear();
            continue;
        }
        let (sx, sy) = ((best.1 % MAP_W) as i32, (best.1 / MAP_W) as i32);
        let (ex, ey) = ((best.2 % MAP_W) as i32, (best.2 / MAP_W) as i32);
        let big_before = big_lands(map);
        let saved = map.terrain.clone();
        let steps = ((ex - sx).abs().max((ey - sy).abs())).max(1);
        for st in 0..=steps {
            let x = sx + (ex - sx) * st / steps;
            let y = sy + (ey - sy) * st / steps;
            for (dx, dy) in [(0, 0), (1, 0), (0, 1)] {
                if Map::inb(x + dx, y + dy) {
                    map.terrain[Map::idx(x + dx, y + dy)] = Terrain::Sea;
                }
            }
        }
        // 水路で陸が分かれてしまうなら、掘らずに湖として残す
        if big_lands(map) != big_before {
            map.terrain = saved;
            for &j in pts.iter() {
                map.terrain[j] = Terrain::Lake;
            }
        }
    }
}

/// 大きな陸地（2000マス以上）の数
fn big_lands(map: &Map) -> usize {
    let (_, sizes) = components(|i| map.terrain[i].passable());
    sizes.iter().filter(|&&s| s >= 2000).count()
}

fn ensure_forest(map: &mut Map, cx: i32, cy: i32, rng: &mut Rng) {
    let mut cnt = 0;
    for dy in -10..=10 {
        for dx in -10..=10 {
            if map.t(cx + dx, cy + dy) == Terrain::Forest {
                cnt += 1;
            }
        }
    }
    if cnt >= 14 {
        return;
    }
    for _ in 0..30 {
        let ang = rng.range(0.0, 6.283);
        let d = rng.range(5.0, 8.0);
        let fx = cx + (ang.cos() * d) as i32;
        let fy = cy + (ang.sin() * d) as i32;
        let mut placed = 0;
        for dy in -2..=2 {
            for dx in -2..=2 {
                if dx * dx + dy * dy > 5 {
                    continue;
                }
                let (x, y) = (fx + dx, fy + dy);
                if Map::inb(x, y) && matches!(map.terrain[Map::idx(x, y)], Terrain::Plain | Terrain::Steppe | Terrain::Desert) {
                    map.terrain[Map::idx(x, y)] = Terrain::Forest;
                    placed += 1;
                }
            }
        }
        if placed > 8 {
            return;
        }
    }
}

fn ensure_mine(map: &mut Map, cx: i32, cy: i32, rng: &mut Rng) {
    if map.mines.iter().any(|m| (m.x - cx).abs().max((m.y - cy).abs()) <= 12) {
        return;
    }
    for _ in 0..40 {
        let ang = rng.range(0.0, 6.283);
        let d = rng.range(6.0, 9.0);
        let x = cx + (ang.cos() * d) as i32;
        let y = cy + (ang.sin() * d) as i32;
        if Map::inb(x, y) && map.terrain[Map::idx(x, y)].passable() && map.terrain[Map::idx(x, y)] != Terrain::River {
            map.terrain[Map::idx(x, y)] = Terrain::Hill;
            map.mines.push(Mine { x, y, amount: 2500.0 });
            return;
        }
    }
}

/// 通行可能なマスの連結成分（4近傍）。番号と大きさを返す
pub fn components(pass: impl Fn(usize) -> bool) -> (Vec<u32>, Vec<usize>) {
    let n = MAP_W * MAP_H;
    let mut comp = vec![u32::MAX; n];
    let mut sizes: Vec<usize> = Vec::new();
    let mut stack = Vec::new();
    for s in 0..n {
        if comp[s] != u32::MAX || !pass(s) {
            continue;
        }
        let id = sizes.len() as u32;
        stack.push(s);
        comp[s] = id;
        let mut sz = 0;
        while let Some(c) = stack.pop() {
            sz += 1;
            let (x, y) = ((c % MAP_W) as i32, (c / MAP_W) as i32);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if Map::inb(nx, ny) {
                    let ni = Map::idx(nx, ny);
                    if comp[ni] == u32::MAX && pass(ni) {
                        comp[ni] = id;
                        stack.push(ni);
                    }
                }
            }
        }
        sizes.push(sz);
    }
    (comp, sizes)
}

/// 山に囲まれて孤立した盆地に峠を開ける（海に面した島はそのまま残す：渡るには船が要る）
fn connect_regions(map: &mut Map) {
    for _ in 0..64 {
        let n = MAP_W * MAP_H;
        let (comp, sizes) = components(|i| map.terrain[i].passable());
        if sizes.len() <= 1 {
            return;
        }
        let main = sizes.iter().enumerate().max_by_key(|(_, s)| **s).unwrap().0 as u32;
        // 海に面しているか
        let mut coastal = vec![false; sizes.len()];
        for i in 0..n {
            let c = comp[i];
            if c == u32::MAX || coastal[c as usize] {
                continue;
            }
            let (x, y) = ((i % MAP_W) as i32, (i / MAP_W) as i32);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if Map::inb(x + dx, y + dy) && map.terrain[Map::idx(x + dx, y + dy)] == Terrain::Sea {
                    coastal[c as usize] = true;
                }
            }
        }
        let mut target = None;
        for (id, &sz) in sizes.iter().enumerate() {
            if id as u32 != main && sz >= 12 && !coastal[id] {
                target = Some(id as u32);
                break;
            }
        }
        let Some(tid) = target else {
            // 小さな孤立領域（島ではないもの）は山にする
            for i in 0..n {
                let c = comp[i];
                if c != u32::MAX && c != main && !coastal[c as usize] {
                    map.terrain[i] = Terrain::Mountain;
                }
            }
            return;
        };
        let start = (0..n).find(|&i| comp[i] == tid).unwrap();
        let (sx, sy) = ((start % MAP_W) as i32, (start / MAP_W) as i32);
        let mut best = (i32::MAX, 0usize);
        for i in 0..n {
            if comp[i] == main {
                let (x, y) = ((i % MAP_W) as i32, (i / MAP_W) as i32);
                let d = (x - sx).pow(2) + (y - sy).pow(2);
                if d < best.0 {
                    best = (d, i);
                }
            }
        }
        let (ex, ey) = ((best.1 % MAP_W) as i32, (best.1 / MAP_W) as i32);
        let steps = ((ex - sx).abs().max((ey - sy).abs())).max(1);
        for s in 0..=steps {
            let x = sx + (ex - sx) * s / steps;
            let y = sy + (ey - sy) * s / steps;
            for (dx, dy) in [(0, 0), (1, 0), (0, 1)] {
                if Map::inb(x + dx, y + dy) {
                    let i = Map::idx(x + dx, y + dy);
                    if !map.terrain[i].passable() {
                        map.terrain[i] = if map.terrain[i] == Terrain::Mountain { Terrain::Hill } else { Terrain::River };
                    }
                }
            }
        }
    }
}
