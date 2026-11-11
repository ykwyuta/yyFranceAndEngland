//! 画面・入力・HUD（独自実装の UI）

mod chronicle;
mod heroes;
mod market;
mod mobile;
mod save;
mod tutorial;

use crate::art::{paint_aoe_terrain_pixel, Art, CmdIcon, ResIcon};
use crate::geo::*;
use crate::gfx::{Font, Gfx, Texture};
use crate::rng::hash2;
use crate::world::*;
use crate::Ev;
use crate::gfx::Bitmap as ID2D1Bitmap;

const TEX_SCALE: usize = 8;
const OVERLAY_SCALE: usize = 4;
const TOP_H: f32 = 36.0;
const BOT_H: f32 = 202.0;
const MM_W: f32 = 118.0;
const MM_H: f32 = 168.0;

const VK_BACK: u32 = 0x08;
const VK_TAB: u32 = 0x09;
const VK_SHIFT: u32 = 0x10;
const VK_CONTROL: u32 = 0x11;
const VK_ESCAPE: u32 = 0x1B;
const VK_SPACE: u32 = 0x20;
const VK_HOME: u32 = 0x24;
const VK_LEFT: u32 = 0x25;
const VK_UP: u32 = 0x26;
const VK_RIGHT: u32 = 0x27;
const VK_DOWN: u32 = 0x28;
const VK_DELETE: u32 = 0x2E;
const VK_F1: u32 = 0x70;
const VK_F2: u32 = 0x71;
const VK_F3: u32 = 0x72;
const VK_F4: u32 = 0x73;
const VK_F5: u32 = 0x74;
/// 下の画面への操作を遮る透明なボタンの目印
const BLOCK_LABEL: &str = "#block";
const VK_F11: u32 = 0x7A;
const VK_ADD: u32 = 0x6B;
const VK_SUB: u32 = 0x6D;
const VK_OEM_PLUS: u32 = 0xBB;
const VK_OEM_MINUS: u32 = 0xBD;
const VK_OEM_PERIOD: u32 = 0xBE;

/// 費用表示用の短い資源名（英語は F/W/B）
fn res_short(k: usize) -> &'static str {
    match k {
        0 => t!("糧", "F", "粮"),
        1 => t!("木", "W", "木"),
        _ => t!("鉄", "B", "铁"),
    }
}

/// 資源の名前（0:糧 1:木 2:銅）
fn res_name(k: usize) -> &'static str {
    match k {
        0 => t!("糧", "Food", "粮"),
        1 => t!("木", "Wood", "木"),
        _ => t!("鉄", "Iron", "铁"),
    }
}
const RES_COLORS: [[f32; 4]; 3] = [[0.95, 0.85, 0.35, 1.0], [0.55, 0.85, 0.45, 1.0], [0.95, 0.6, 0.35, 1.0]];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Title,
    Playing,
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Normal,
    Place(BKind),
    PlantForest,
    AttackMove,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Act {
    /// 兵種（UNIT_DEFS の番号）を訓練する
    Train(u8),
    /// 兵営の兵種一覧の次の頁
    TrainPage,
    /// 選んだ兵を、選んだ船にまとめて乗せる
    BoardAll,
    /// 邑を焼き払う
    Raze,
    /// 船の兵をいまの岸に降ろす
    Unload,
    Place(BKind),
    Upgrade,
    Convert,
    Stop,
    AttackMove,
    SelectAllWorkers,
    SelectAllMilitary,
    SelectKind(UnitKind),
    FilterKind(UnitKind),
    WorkersAuto,
    AllWorkersAuto,
    AllWorkersWood,
    AllWorkersMine,
    AllWorkersUpgrade,
    ArmyDefend,
    AllArmyAttack,
    AllArmyDefend,
    AllArmyUpgrade,
    Speed(u32),
    Diplo,
    Help,
    War(usize),
    Peace(usize),
    /// 和睦の申し入れへの返答（true=受諾）
    PeaceAnswer(bool),
    Start,
    Home,
    Restart,
    CloseOverlay,
    Demolish,
    Discard,
    ConfirmDiscard,
    Repair,
    PlantForest,
    ResumeBuild,
    IdleWorker,
    Chron,
    ChronPage(i32),
    Market,
    Tax(u8),
    Buy(usize),
    Hire(usize),
    ToggleHeroic,
    Summon,
    SummonHero(usize),
    Tutorial,
    TutNext,
    TutQuit,
    Scale(i32),
    Fullscreen,
    #[cfg(target_arch = "wasm32")]
    RefreshCache,
    /// 表示言語を切り替える
    Lang,
    SaveMenu,
    /// 選択・建設の取り消し（タッチ操作用の ✕）
    Cancel,
    /// 出来事の告知（バナー）を閉じる
    DismissBanner,
    /// 待っている告知もまとめて閉じる
    DismissAllBanners,
    /// 連合の一覧を開く・畳む（タッチ向けの画面）
    ToggleAlerts,
    /// いま出ているお知らせを消す（タッチ向けの画面）
    ClearMsgs,
    SaveSlot(u8),
    LoadSlot(u8),
    ExportSlot(u8),
    ImportFile,
    Noop,
}

struct Button {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    label: String,
    key: u32,
    enabled: bool,
    act: Act,
    tip: String,
}

pub struct Game {
    gfx: Gfx,
    art: Art,
    world: World,
    seed: u32,
    terrain_px: Vec<u32>,
    terrain_tex: Texture,
    overlay_px: Vec<u32>,
    overlay_tex: Texture,
    overlay_ver: u32,
    minimap_px: Vec<u32>,
    minimap_bmp: Option<ID2D1Bitmap>,
    minimap_gen: u32,
    minimap_ver: u32,
    minimap_t: f32,
    minimap_fog: u32,
    /// 視界：0=未踏、1=探索済み（霧）、2=視界内
    fog: Vec<u8>,
    fog_px: Vec<u32>,
    fog_tex: Texture,
    fog_t: f32,
    fog_ver: u32,
    cam: V2,
    zoom: f32,
    mouse: (f32, f32),
    keys: [bool; 256],
    ldown: Option<(f32, f32)>,
    mdrag: Option<(f32, f32, V2)>,
    mm_drag: bool,
    screen: Screen,
    mode: Mode,
    sel: Vec<Handle>,
    sel_b: Option<Handle>,
    groups: Vec<Vec<Handle>>,
    speed: u32,
    paused: bool,
    /// 部隊を選んでから命令するまでの一時停止
    cmd_pause: bool,
    last_sel: Vec<Handle>,
    last_orders: u32,
    acc: f32,
    show_diplo: bool,
    show_help: bool,
    show_chron: bool,
    show_market: bool,
    show_summon: bool,
    /// 記録（セーブ・ロード）画面
    show_save: bool,
    discard_pending: bool,
    /// いまの入力がタッチ操作か
    touch: bool,
    /// スマートフォン向け画面で一覧（外交・交易など）をなぞってスクロールした量（UI 座標）
    ov_scroll: f32,
    ov_key: u8,
    ov_max: f32,
    /// タッチ操作でツールチップを出しておく時刻
    tip_until: f32,
    /// タッチ操作の建設：1回目のタップで場所を示し、同じ場所を2回目にタップで建てる
    place_tap: Option<(f32, f32)>,
    /// タッチ向けの画面：連合の一覧を開いているか
    alerts_open: bool,
    /// タッチ向けの画面：この時刻までのお知らせは消した
    msg_cut: f32,
    /// 最後に記録した年（自動記録の間隔）
    autosave_years: f32,
    /// タイトル画面で選ぶ英霊モード（既定は無効）
    heroic_opt: bool,
    chron_scroll: usize,
    title_sel: Option<usize>,
    notice: Option<(String, f32)>,
    buttons: Vec<Button>,
    last_click: (f32, Option<Handle>),
    clock: f32,
    marker: Option<(V2, f32, bool)>,
    idle_cycle: usize,
    /// 兵営の兵種一覧の頁
    train_page: usize,
    /// 自動撮影（YY_AUTO="国番号,秒,出力パス[,ズーム]"）
    auto: Option<(Option<usize>, f32, String, f32)>,
    script: Vec<(f32, Ev)>,
    tut: Option<tutorial::Tut>,
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn rgba(c: [f32; 3], a: f32) -> u32 {
    let r = (c[0].clamp(0.0, 1.0) * 255.0) as u32;
    let g = (c[1].clamp(0.0, 1.0) * 255.0) as u32;
    let b = (c[2].clamp(0.0, 1.0) * 255.0) as u32;
    let a = (a.clamp(0.0, 1.0) * 255.0) as u32;
    r | (g << 8) | (b << 16) | (a << 24)
}

fn bgra(c: [f32; 3]) -> u32 {
    let r = (c[0].clamp(0.0, 1.0) * 255.0) as u32;
    let g = (c[1].clamp(0.0, 1.0) * 255.0) as u32;
    let b = (c[2].clamp(0.0, 1.0) * 255.0) as u32;
    b | (g << 8) | (r << 16) | (255 << 24)
}

fn c4(c: [f32; 3], a: f32) -> [f32; 4] {
    [c[0], c[1], c[2], a]
}

fn darker(c: [f32; 3], k: f32) -> [f32; 3] {
    [c[0] * k, c[1] * k, c[2] * k]
}

/// タイルの基本色
fn base_color(t: Terrain, x: i32, y: i32, shade: f32) -> [f32; 3] {
    let lat_t = (y as f32 / MAP_H as f32).clamp(0.0, 1.0);
    match t {
        Terrain::Sea => lerp3([0.10, 0.22, 0.42], [0.14, 0.30, 0.52], shade),
        Terrain::Lake => [0.20, 0.38, 0.58],
        Terrain::River => [0.24, 0.44, 0.66],
        Terrain::Desert => lerp3([0.86, 0.76, 0.52], [0.78, 0.66, 0.44], shade),
        Terrain::Plain => {
            let north = [0.50, 0.60, 0.34];
            let south = [0.40, 0.58, 0.30];
            let c = lerp3(north, south, ((lat_t - 0.30) * 2.2).clamp(0.0, 1.0));
            lerp3(c, darker(c, 0.85), shade)
        }
        Terrain::Steppe => lerp3([0.74, 0.70, 0.48], [0.66, 0.64, 0.42], shade),
        Terrain::Forest => lerp3([0.16, 0.36, 0.17], [0.22, 0.42, 0.20], shade),
        Terrain::Hill => lerp3([0.58, 0.50, 0.34], [0.50, 0.44, 0.30], shade),
        Terrain::Mountain => {
            let h = hash2(x, y, 99);
            if h > 0.93 {
                [0.78, 0.76, 0.74]
            } else {
                lerp3([0.42, 0.38, 0.35], [0.52, 0.48, 0.44], h)
            }
        }
    }
}

impl Game {
    pub fn new(gfx: Gfx) -> Game {
        let seed = 20240 + crate::platform::random_u32() % 1000;
        let world = World::new(seed);
        let tw = (MAP_W * TEX_SCALE) as u32;
        let th = (MAP_H * TEX_SCALE) as u32;
        let ow = (MAP_W * OVERLAY_SCALE) as u32;
        let oh = (MAP_H * OVERLAY_SCALE) as u32;
        let terrain_px = vec![0u32; (tw * th) as usize];
        let overlay_px = vec![0u32; (ow * oh) as usize];
        let terrain_tex = gfx.create_texture(tw, th, &terrain_px).expect("texture");
        let overlay_tex = gfx.create_texture(ow, oh, &overlay_px).expect("texture");
        let fog_px = vec![0u32; MAP_W * MAP_H];
        let fog_tex = gfx.create_texture(MAP_W as u32, MAP_H as u32, &fog_px).expect("texture");
        let mut g = Game {
            gfx,
            art: Art::new(),
            world,
            seed,
            terrain_px,
            terrain_tex,
            overlay_px,
            overlay_tex,
            overlay_ver: u32::MAX,
            minimap_px: vec![0; MAP_W * MAP_H],
            minimap_bmp: None,
            minimap_gen: 0,
            minimap_ver: u32::MAX,
            minimap_t: 0.0,
            minimap_fog: 0,
            fog: vec![2; MAP_W * MAP_H],
            fog_px,
            fog_tex,
            fog_t: 0.0,
            fog_ver: 0,
            cam: v2(MAP_W as f32 / 2.0, MAP_H as f32 / 2.0),
            zoom: 4.0,
            mouse: (-100.0, -100.0),
            keys: [false; 256],
            ldown: None,
            cmd_pause: false,
            last_sel: Vec::new(),
            last_orders: 0,
            mdrag: None,
            mm_drag: false,
            screen: Screen::Title,
            mode: Mode::Normal,
            sel: Vec::new(),
            sel_b: None,
            groups: vec![Vec::new(); 10],
            speed: 1,
            paused: false,
            acc: 0.0,
            show_diplo: false,
            show_help: false,
            show_chron: false,
            show_market: false,
            show_summon: false,
            show_save: false,
            discard_pending: false,
            touch: false,
            ov_scroll: 0.0,
            ov_key: 0,
            ov_max: 0.0,
            tip_until: 0.0,
            place_tap: None,
            alerts_open: false,
            msg_cut: -1.0,
            autosave_years: 0.0,
            heroic_opt: false,
            chron_scroll: 0,
            title_sel: None,
            notice: None,
            buttons: Vec::new(),
            last_click: (-10.0, None),
            clock: 0.0,
            marker: None,
            idle_cycle: 0,
            train_page: 0,
            script: Vec::new(),
            tut: None,
            auto: std::env::var("YY_AUTO").ok().and_then(|v| {
                let p: Vec<&str> = v.split(',').collect();
                Some((p.first()?.parse().ok(), p.get(1)?.parse().ok()?, p.get(2)?.to_string(), p.get(3).and_then(|z| z.parse().ok()).unwrap_or(22.0)))
            }),
        };
        g.rebuild_terrain();
        g.fit_title_camera();
        if std::env::var("YY_FULL").is_ok() {
            g.gfx.toggle_fullscreen();
        }
        g
    }

    fn fit_title_camera(&mut self) {
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        let avail_w = (w - 380.0).max(300.0);
        self.zoom = (avail_w / MAP_W as f32).min(h / MAP_H as f32) * 0.98;
        self.cam = v2(MAP_W as f32 / 2.0 + 380.0 / 2.0 / self.zoom, MAP_H as f32 / 2.0);
    }

    fn rebuild_terrain(&mut self) {
        let tw = MAP_W * TEX_SCALE;
        for y in 0..MAP_H as i32 {
            for x in 0..MAP_W as i32 {
                self.paint_tile(x, y, tw);
            }
        }
        let t = &self.terrain_tex;
        self.gfx.update_texture(t, 0, 0, t.w, t.h, &self.terrain_px);
        self.overlay_ver = u32::MAX;
        self.minimap_ver = u32::MAX;
    }

    fn paint_tile(&mut self, x: i32, y: i32, tw: usize) {
        let s = TEX_SCALE as i32;
        for py in 0..s {
            for px in 0..s {
                let gx = x * s + px;
                let gy = y * s + py;
                let c = paint_aoe_terrain_pixel(&self.world.map.terrain, &self.world.map.shade, x, y, px, py, s);
                self.terrain_px[gy as usize * tw + gx as usize] = rgba(c, 1.0);
            }
        }
    }

    fn update_dirty_terrain(&mut self) {
        if self.world.terrain_dirty.is_empty() {
            return;
        }
        let tw = MAP_W * TEX_SCALE;
        let dirty = std::mem::take(&mut self.world.terrain_dirty);
        for &(x, y) in dirty.iter() {
            self.paint_tile(x, y, tw);
            let s = TEX_SCALE;
            let mut buf = vec![0u32; s * s];
            for py in 0..s {
                for px in 0..s {
                    buf[py * s + px] = self.terrain_px[(y as usize * s + py) * tw + x as usize * s + px];
                }
            }
            self.gfx.update_texture(&self.terrain_tex, (x as usize * s) as u32, (y as usize * s) as u32, s as u32, s as u32, &buf);
        }
        self.minimap_ver = u32::MAX;
    }

    fn update_overlay(&mut self) {
        if self.overlay_ver == self.world.territory_version {
            return;
        }
        self.overlay_ver = self.world.territory_version;
        let w = &self.world;
        let os = OVERLAY_SCALE;
        let ow = MAP_W * os;
        let owner_at = |nx: i32, ny: i32| -> u8 {
            let cx = nx.clamp(0, MAP_W as i32 - 1);
            let cy = ny.clamp(0, MAP_H as i32 - 1);
            let ni = Map::idx(cx, cy);
            if w.map.terrain[ni] == Terrain::Sea {
                NONE
            } else {
                w.owner[ni]
            }
        };
        for y in 0..MAP_H as i32 {
            for x in 0..MAP_W as i32 {
                let o = owner_at(x, y);
                for sy in 0..os {
                    for sx in 0..os {
                        let fx = x as f32 + (sx as f32 + 0.5) / os as f32 - 0.5;
                        let fy = y as f32 + (sy as f32 + 0.5) / os as f32 - 0.5;
                        let x0 = fx.floor() as i32;
                        let y0 = fy.floor() as i32;
                        let tx = fx - x0 as f32;
                        let ty = fy - y0 as f32;
                        let ux = tx * tx * (3.0 - 2.0 * tx);
                        let uy = ty * ty * (3.0 - 2.0 * ty);

                        let o00 = owner_at(x0, y0);
                        let o10 = owner_at(x0 + 1, y0);
                        let o01 = owner_at(x0, y0 + 1);
                        let o11 = owner_at(x0 + 1, y0 + 1);

                        // 最も近い有効な領主を決定
                        let dom = if o != NONE {
                            o
                        } else if o00 != NONE {
                            o00
                        } else if o10 != NONE {
                            o10
                        } else if o01 != NONE {
                            o01
                        } else {
                            o11
                        };
                        if dom == NONE {
                            self.overlay_px[(y as usize * os + sy) * ow + x as usize * os + sx] = 0;
                            continue;
                        }
                        let w00 = if o00 == dom { 1.0 } else { 0.0 };
                        let w10 = if o10 == dom { 1.0 } else { 0.0 };
                        let w01 = if o01 == dom { 1.0 } else { 0.0 };
                        let w11 = if o11 == dom { 1.0 } else { 0.0 };
                        let wt = (w00 * (1.0 - ux) + w10 * ux) * (1.0 - uy) + (w01 * (1.0 - ux) + w11 * ux) * uy;

                        let c = w.nations[dom as usize].color;
                        let px = if wt < 0.36 {
                            0
                        } else if wt < 0.60 {
                            rgba([c[0].min(1.0), c[1].min(1.0), c[2].min(1.0)], 0.86)
                        } else if wt < 0.80 {
                            rgba(c, 0.30)
                        } else {
                            rgba(c, 0.065)
                        };
                        self.overlay_px[(y as usize * os + sy) * ow + x as usize * os + sx] = px;
                    }
                }
            }
        }
        self.gfx.update_texture(&self.overlay_tex, 0, 0, ow as u32, (MAP_H * os) as u32, &self.overlay_px);
    }

    fn update_minimap(&mut self, dt: f32) {
        self.minimap_t -= dt;
        let gen_changed = self.minimap_gen != self.gfx.d2d_gen;
        let unchanged = self.minimap_ver == self.world.territory_version && self.minimap_fog == self.fog_ver;
        if !gen_changed && (self.minimap_t > 0.0 || unchanged) {
            return;
        }
        self.minimap_t = 1.0;
        self.minimap_ver = self.world.territory_version;
        self.minimap_fog = self.fog_ver;
        let fogged = self.fog_active();
        let w = &self.world;
        for y in 0..MAP_H as i32 {
            for x in 0..MAP_W as i32 {
                let i = Map::idx(x, y);
                let mut c = base_color(w.map.terrain[i], x, y, w.map.shade[i]);
                let o = w.owner[i];
                if o != NONE {
                    c = lerp3(c, w.nations[o as usize].color, 0.55);
                }
                if fogged {
                    match self.fog[i] {
                        0 => c = [0.06, 0.06, 0.07],
                        1 => c = darker(c, 0.55),
                        _ => {}
                    }
                }
                self.minimap_px[i] = bgra(c);
            }
        }
        if gen_changed || self.minimap_bmp.is_none() {
            self.minimap_bmp = self.gfx.create_bitmap(MAP_W as u32, MAP_H as u32, &self.minimap_px);
            self.minimap_gen = self.gfx.d2d_gen;
        } else if let Some(b) = &self.minimap_bmp {
            self.gfx.update_bitmap(b, MAP_W as u32, &self.minimap_px);
        }
    }

    // ------------------------------------------------------------ 視界（戦場の霧）
    /// 霧を表示・適用するか（タイトル画面と終局後は全域を見せる）
    fn fog_active(&self) -> bool {
        self.screen == Screen::Playing && self.world.game_over.is_none()
    }

    fn reset_fog(&mut self) {
        self.fog.fill(0);
        // 諸侯の都の所在は天下に知られている
        let homes: Vec<V2> = self.world.nations.iter().filter(|n| n.alive).map(|n| n.home).collect();
        for h in homes {
            let (x, y) = h.tile();
            Self::reveal(&mut self.fog, x, y, 3, 1);
        }
        self.fog_t = 0.0;
        self.update_fog(0.0);
    }

    fn reveal(fog: &mut [u8], cx: i32, cy: i32, r: i32, level: u8) {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy > r * r + r {
                    continue;
                }
                let (x, y) = (cx + dx, cy + dy);
                if Map::inb(x, y) {
                    let f = &mut fog[Map::idx(x, y)];
                    *f = (*f).max(level);
                }
            }
        }
    }

    fn update_fog(&mut self, dt: f32) {
        self.fog_t -= dt;
        if self.fog_t > 0.0 {
            return;
        }
        self.fog_t = 0.15;
        for f in self.fog.iter_mut() {
            if *f == 2 {
                *f = 1;
            }
        }
        let pl = self.world.player;
        for u in self.world.units.iter() {
            if u.on_map() && u.nation as usize == pl {
                let (x, y) = u.pos.tile();
                Self::reveal(&mut self.fog, x, y, u.kind.vision(), 2);
            }
        }
        for b in self.world.blds.iter() {
            if b.alive && b.nation as usize == pl {
                let (x, y) = b.center().tile();
                let r = if b.kind.city() { b.kind.radius() as i32 + 3 } else { 5 };
                Self::reveal(&mut self.fog, x, y, r, 2);
            }
        }
        let fog_c = [0.04, 0.05, 0.07];
        for (px, &f) in self.fog_px.iter_mut().zip(self.fog.iter()) {
            *px = match f {
                0 => rgba(fog_c, 1.0),
                1 => rgba(fog_c, 0.5),
                _ => 0,
            };
        }
        self.gfx.update_texture(&self.fog_tex, 0, 0, MAP_W as u32, MAP_H as u32, &self.fog_px);
        self.fog_ver = self.fog_ver.wrapping_add(1);
    }

    /// その地点が今見えているか
    fn seen(&self, p: V2) -> bool {
        if !self.fog_active() {
            return true;
        }
        let (x, y) = p.tile();
        Map::inb(x, y) && self.fog[Map::idx(x, y)] == 2
    }

    /// その地点を一度でも見たか
    fn explored(&self, p: V2) -> bool {
        if !self.fog_active() {
            return true;
        }
        let (x, y) = p.tile();
        Map::inb(x, y) && self.fog[Map::idx(x, y)] >= 1
    }

    // ------------------------------------------------------------ 座標
    fn screen_to_world(&self, sx: f32, sy: f32) -> V2 {
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        v2(self.cam.x + (sx - w / 2.0) / self.zoom, self.cam.y + (sy - h / 2.0) / self.zoom)
    }
    fn world_to_screen(&self, p: V2) -> (f32, f32) {
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        ((p.x - self.cam.x) * self.zoom + w / 2.0, (p.y - self.cam.y) * self.zoom + h / 2.0)
    }
    fn clamp_cam(&mut self) {
        self.cam.x = self.cam.x.clamp(0.0, MAP_W as f32);
        self.cam.y = self.cam.y.clamp(0.0, MAP_H as f32);
    }
    fn mm_rect(&self) -> (f32, f32, f32, f32) {
        let h = self.gfx.height as f32;
        (2.0, h - MM_H - 2.0, 2.0 + MM_W, h - 2.0)
    }
    /// 上部バーの高さ（スマートフォンでは指で押しやすいよう高くする）
    fn top_h(&self) -> f32 {
        if self.gfx.compact {
            mobile::TOP_H_COMPACT
        } else {
            TOP_H
        }
    }
    fn over_ui(&self, x: f32, y: f32) -> bool {
        if self.show_save {
            return true;
        }
        if self.screen == Screen::Title {
            return x > self.gfx.width as f32 - 380.0;
        }
        if let Some(t) = &self.tut {
            let (x0, y0, x1, y1) = t.panel;
            if x >= x0 && x <= x1 && y >= y0 && y <= y1 {
                return true;
            }
        }
        if self.show_diplo || self.show_help || self.show_chron || self.show_market || self.show_summon || self.show_save || self.world.game_over.is_some() {
            return true;
        }
        if self.peace_pending() {
            return true;
        }
        y < self.top_h() || y > self.gfx.height as f32 - BOT_H
    }

    fn notice(&mut self, s: &str) {
        if !s.is_empty() {
            self.notice = Some((s.to_string(), 3.0));
        }
    }

    // ------------------------------------------------------------ 指揮中の一時停止
    /// 自国の部隊を新たに選ぶと時間を止め、命令を出すと再開する
    fn update_cmd_pause(&mut self) {
        let pl = self.world.player;
        let own = self.sel.iter().any(|&h| self.world.unit(h).map_or(false, |u| u.nation as usize == pl));
        if self.sel != self.last_sel {
            // 戦死などで減っただけなら選び直しとはみなさない
            let added = self.sel.iter().any(|h| !self.last_sel.contains(h));
            if added {
                self.cmd_pause = own;
            }
            self.last_sel = self.sel.clone();
        }
        if !own {
            self.cmd_pause = false;
        }
        if self.world.player_orders != self.last_orders {
            self.last_orders = self.world.player_orders;
            self.cmd_pause = false;
        }
    }

    /// プレイヤーの返答を待つ和睦の申し入れがあるか（返答するまで時間を止める）
    fn peace_pending(&self) -> bool {
        self.screen == Screen::Playing && self.world.game_over.is_none() && !self.world.peace_asks.is_empty()
    }

    /// 時間を止めているか（部隊の選択中・範囲選択のドラッグ中）
    fn cmd_hold(&self) -> bool {
        let dragging = self.ldown.map_or(false, |(sx, sy)| (sx - self.mouse.0).abs() > 4.0 || (sy - self.mouse.1).abs() > 4.0);
        self.cmd_pause || dragging
    }

    // ------------------------------------------------------------ フレーム
    /// true を返すと終了
    pub fn frame(&mut self, evs: &[Ev], dt: f32, focused: bool) -> bool {
        // ファイルから読み込んだ記録・保存の結果（デスクトップ版）
        if let Some(data) = crate::platform::take_loaded() {
            self.apply_loaded(&data);
        }
        for n in crate::platform::take_notes() {
            self.notify(&n);
        }
        self.clock += dt;
        // OS からのマウス座標（実ピクセル）を UI 座標系へ
        let ui = self.gfx.ui;
        let mut evs: Vec<Ev> = evs
            .iter()
            .map(|e| match *e {
                Ev::MouseMove(x, y) => Ev::MouseMove(x / ui, y / ui),
                Ev::Down(b, x, y) => Ev::Down(b, x / ui, y / ui),
                Ev::Up(b, x, y) => Ev::Up(b, x / ui, y / ui),
                o => o,
            })
            .collect();
        while !self.script.is_empty() && self.script[0].0 <= self.clock {
            evs.push(self.script.remove(0).1);
        }
        for e in evs {
            if self.handle_event(e) {
                return true;
            }
        }
        if self.screen == Screen::Playing {
            self.scroll(dt, focused);
            self.update_cmd_pause();
            self.world.prune_peace_asks();
            if !self.paused && !self.cmd_hold() && self.world.game_over.is_none() && !self.show_help && !self.peace_pending() {
                self.acc += dt * self.speed as f32;
                let mut steps = 0;
                while self.acc >= DT && steps < 16 {
                    self.world.step();
                    self.acc -= DT;
                    steps += 1;
                    // 和睦の申し入れが届いたら、返答するまで時間を止める
                    if self.peace_pending() {
                        self.acc = 0.0;
                        break;
                    }
                }
                if steps >= 16 {
                    self.acc = 0.0;
                }
            }
            self.autosave_tick();
            self.update_fog(dt);
            self.update_tutorial(dt);
            let pl = self.world.player;
            let sel: Vec<Handle> = self.sel.iter().copied().filter(|&h| self.world.unit(h).map_or(false, |u| u.nation as usize == pl || self.seen(u.pos))).collect();
            self.sel = sel;
            if let Some(b) = self.sel_b {
                if self.world.bld(b).is_none() {
                    self.sel_b = None;
                }
            }
        }
        if let Some((_, t)) = &mut self.notice {
            *t -= dt;
            if *t <= 0.0 {
                self.notice = None;
            }
        }
        if let Some(m) = &mut self.marker {
            m.1 -= dt;
            if m.1 <= 0.0 {
                self.marker = None;
            }
        }
        if let Some((nation, secs, _, z)) = self.auto.clone() {
            if self.screen == Screen::Title {
                if let Some(step) = std::env::var("YY_TUT").ok().and_then(|v| v.parse::<usize>().ok()) {
                    self.start_tutorial();
                    self.tut_enter(step);
                    self.zoom = z;
                    if let Some(k) = std::env::var("YY_PRESS").ok().and_then(|v| u32::from_str_radix(&v, 16).ok()) {
                        self.script = vec![(self.clock + 0.5, Ev::Key(k, true)), (self.clock + 0.55, Ev::Key(k, false))];
                    }
                }
            }
            if self.screen == Screen::Title {
                if nation.is_none() {
                    // タイトル画面のまま国を選んでおく（YY_SEL=国番号）
                    if let Some(s) = std::env::var("YY_SEL").ok().and_then(|v| v.parse().ok()) {
                        self.title_sel = Some(s);
                    }
                    if std::env::var("YY_HEROIC").is_ok() {
                        self.heroic_opt = true;
                    }
                }
                if let Some(n) = nation {
                    self.title_sel = Some(n);
                    self.do_act(Act::Start);
                    // 撮影用：英霊モード（YY_HEROIC=1）
                    if std::env::var("YY_HEROIC").is_ok() {
                        self.world.heroic = true;
                    }
                    // 撮影用：自国も AI に任せる（YY_AIPLAY=1）
                    self.world.ai_player = std::env::var("YY_AIPLAY").is_ok();
                    // 撮影用：指定の年（西暦）から始める（YY_YEAR=1230 など）
                    if let Some(y) = std::env::var("YY_YEAR").ok().and_then(|v| v.parse::<f32>().ok()) {
                        self.world.years = y - START_AD;
                    }
                    // 撮影用：指定の国と交戦させ、和睦を申し入れさせる（YY_PEACE=国番号）
                    if let Some(o) = std::env::var("YY_PEACE").ok().and_then(|v| v.parse::<usize>().ok()) {
                        let pl = self.world.player;
                        self.world.declare_war(pl, o, true);
                        self.world.war_start[pl][o] -= 12.0;
                        self.world.propose_peace(o, pl, 8.0, t!("連合の解散", "the coalition disbanded", "联军解散"));
                    }
                    // 撮影用：民を1人選び、都の近くに田を3つ予約させる（YY_QUEUE=1）
                    if std::env::var("YY_QUEUE").is_ok() {
                        let pl = self.world.player;
                        let home = self.world.nations[pl].home;
                        let wk = (0..self.world.units.len()).find(|&i| self.world.units[i].alive && self.world.units[i].nation as usize == pl && self.world.units[i].kind == UnitKind::Worker);
                        if let Some(i) = wk {
                            let h = self.world.uh(i);
                            let mut spots: Vec<(i32, i32)> = Vec::new();
                            for r in 4..10 {
                                for dy in -r..=r {
                                    for dx in -r..=r {
                                        let (tx, ty) = (home.x as i32 + dx, home.y as i32 + dy);
                                        if spots.len() < 3 && self.world.can_place(pl, BKind::Farm, tx, ty).is_ok() && !spots.iter().any(|&(x, y)| (x - tx).abs() < 4 && (y - ty).abs() < 4) {
                                            spots.push((tx, ty));
                                        }
                                    }
                                }
                            }
                            for (tx, ty) in spots {
                                if let Ok(b) = self.world.place_building(pl, BKind::Farm, tx, ty) {
                                    self.world.queue_build(&[h], b);
                                }
                            }
                            self.set_sel(vec![h]);
                        }
                    }
                    self.speed = std::env::var("YY_SPEED").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
                    self.zoom = z;
                    // 撮影直前にキーを押す（例：72 で歴史を開く）
                    // YY_PRESS は「47,48」のように複数のキーを0.5秒おきに押せる
                    if let Ok(v) = std::env::var("YY_PRESS") {
                        let at = std::env::var("YY_PRESS_AT").ok().and_then(|v| v.parse::<f32>().ok()).unwrap_or(secs - 0.6);
                        let t = at.max(self.clock + 0.3);
                        for (i, k) in v.split(',').filter_map(|s| u32::from_str_radix(s.trim(), 16).ok()).enumerate() {
                            let tk = t + i as f32 * 0.5;
                            self.script.push((tk, Ev::Key(k, true)));
                            self.script.push((tk + 0.05, Ev::Key(k, false)));
                        }
                    }
                    if std::env::var("YY_DEMO").is_ok() {
                        self.speed = 1;
                        let (cx, cy) = (self.gfx.width as f32 / 2.0, self.gfx.height as f32 / 2.0);
                        let c = self.clock;
                        self.script = vec![
                            (c + 0.3, Ev::MouseMove(cx, cy)),
                            (c + 0.4, Ev::Down(0, cx, cy)),
                            (c + 0.45, Ev::Up(0, cx, cy)),
                            (c + 0.6, Ev::Key(0x51, true)),
                            (c + 0.65, Ev::Key(0x51, false)),
                            (c + 0.9, Ev::Down(0, cx - 160.0, cy - 160.0)),
                            (c + 0.95, Ev::MouseMove(cx + 160.0, cy + 160.0)),
                            (c + 1.0, Ev::Up(0, cx + 160.0, cy + 160.0)),
                            (c + 1.2, Ev::Key(0x51, true)),
                            (c + 1.25, Ev::Key(0x51, false)),
                            (c + 1.4, Ev::MouseMove(cx + 140.0, cy + 80.0)),
                            (c + 1.6, Ev::Down(0, cx + 140.0, cy + 80.0)),
                            (c + 1.65, Ev::Up(0, cx + 140.0, cy + 80.0)),
                            (c + 1.8, Ev::Key(0x57, true)),
                            (c + 1.85, Ev::Key(0x57, false)),
                            (c + 2.0, Ev::MouseMove(cx - 150.0, cy + 60.0)),
                            (c + 2.4, Ev::Key(0x71, true)),
                            (c + 2.45, Ev::Key(0x71, false)),
                        ];
                    }
                }
            }
            if let Some(n) = nation {
                let _ = n;
                // 自国の兵に追従
                if self.clock > 1.0 {
                    let pl = self.world.player;
                    if let Some(u) = self.world.units.iter().find(|u| u.on_map() && u.nation as usize == pl && u.kind.military()) {
                        let _ = u;
                    }
                }
            }
            if self.clock >= secs {
                // 撮影時のカーソル位置（YY_MOUSE="x,y"、画面ピクセル）：ツールチップの確認用
                if let Some((mx, my)) = std::env::var("YY_MOUSE").ok().and_then(|v| {
                    let mut it = v.split(',').filter_map(|s| s.trim().parse::<f32>().ok());
                    Some((it.next()?, it.next()?))
                }) {
                    self.mouse = (mx / self.gfx.ui, my / self.gfx.ui);
                }
                self.update_dirty_terrain();
                self.update_overlay();
                self.render_nopresent(dt);
                let path = self.auto.as_ref().unwrap().2.clone();
                let _ = self.gfx.save_backbuffer(&path);
                return true;
            }
        }
        self.update_dirty_terrain();
        self.update_overlay();
        self.render(dt);
        false
    }

    fn scroll(&mut self, dt: f32, focused: bool) {
        let sp = 900.0 / self.zoom * dt;
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        let (mx, my) = self.mouse;
        let mut d = v2(0.0, 0.0);
        if self.keys[VK_LEFT as usize] {
            d.x -= 1.0;
        }
        if self.keys[VK_RIGHT as usize] {
            d.x += 1.0;
        }
        if self.keys[VK_UP as usize] {
            d.y -= 1.0;
        }
        if self.keys[VK_DOWN as usize] {
            d.y += 1.0;
        }
        if focused && !self.touch && self.mdrag.is_none() && self.ldown.is_none() {
            if mx <= 3.0 && mx >= 0.0 {
                d.x -= 1.0;
            }
            if mx >= w - 4.0 && mx <= w {
                d.x += 1.0;
            }
            if my <= 3.0 && my >= 0.0 {
                d.y -= 1.0;
            }
            if my >= h - 4.0 && my <= h {
                d.y += 1.0;
            }
        }
        self.cam = self.cam + d * sp;
        self.clamp_cam();
    }

    fn handle_event(&mut self, e: Ev) -> bool {
        match e {
            Ev::Resize(w, h) => {
                self.gfx.resize(w, h);
                self.clamp_cam();
                if self.screen == Screen::Title {
                    self.fit_title_camera();
                }
            }
            Ev::MouseMove(x, y) => {
                self.mouse = (x, y);
                if let Some((sx, sy, c0)) = self.mdrag {
                    self.cam = c0 - v2(x - sx, y - sy) * (1.0 / self.zoom);
                    self.clamp_cam();
                }
                if self.mm_drag {
                    self.minimap_jump(x, y);
                }
            }
            Ev::Compact(c) => {
                self.gfx.set_compact(c);
                self.clamp_cam();
                if self.screen == Screen::Title {
                    self.fit_title_camera();
                }
            }
            Ev::Touch(t) => {
                self.touch = t;
                if t {
                    self.tip_until = self.clock + 2.5;
                }
            }
            Ev::Scroll(dy) => {
                self.ov_scroll = (self.ov_scroll - dy / self.gfx.ui).clamp(0.0, self.ov_max);
            }
            Ev::Wheel(d) if self.ov_max > 0.0 && self.overlay_open() => {
                self.ov_scroll = (self.ov_scroll - d * 60.0).clamp(0.0, self.ov_max);
            }
            Ev::Wheel(d) => {
                if self.screen == Screen::Playing && !self.over_ui(self.mouse.0, self.mouse.1) {
                    let before = self.screen_to_world(self.mouse.0, self.mouse.1);
                    self.zoom = (self.zoom * 1.15f32.powf(d)).clamp(1.2, 64.0);
                    let after = self.screen_to_world(self.mouse.0, self.mouse.1);
                    self.cam = self.cam + (before - after);
                    self.clamp_cam();
                }
            }
            Ev::Down(b, x, y) => {
                self.mouse = (x, y);
                match b {
                    0 => {
                        if let Some(a) = self.button_at(x, y) {
                            self.do_act(a);
                            return false;
                        }
                        if self.screen == Screen::Title {
                            self.title_click(x, y);
                            return false;
                        }
                        let (mx0, my0, mx1, my1) = self.mm_rect();
                        if x >= mx0 && x <= mx1 && y >= my0 && y <= my1 && !self.show_diplo && !self.show_help && !self.show_chron && !self.show_market && !self.show_summon && !self.show_save {
                            self.mm_drag = true;
                            self.minimap_jump(x, y);
                            return false;
                        }
                        if self.over_ui(x, y) {
                            return false;
                        }
                        match self.mode {
                            Mode::Place(_) if self.touch && self.place_tap.map_or(true, |(px, py)| (px - x).abs() > 22.0 || (py - y).abs() > 22.0) => {
                                // タッチ：1回目のタップは置き場所を見せるだけ
                                self.place_tap = Some((x, y));
                                self.notice(t!("同じ場所をもう一度タップすると建てる", "Tap the same spot again to build", "再次点击同一位置即可建造"));
                            }
                            Mode::Place(k) => {
                                self.place_tap = None;
                                // Ctrl を押しながら置くと建設の予約（続けて置ける）
                                let queue = self.keys[VK_CONTROL as usize];
                                self.try_place(k, queue);
                                if !self.keys[VK_SHIFT as usize] && !queue {
                                    self.mode = Mode::Normal;
                                }
                            }
                            Mode::PlantForest => {
                                let (tx, ty) = self.placement_tile();
                                let workers = self.own_sel();
                                match self.world.plant_forest(self.world.player, &workers, tx, ty) {
                                    Ok(()) => {
                                        self.mode = Mode::Normal;
                                        self.cmd_pause = false;
                                        self.notice(t!("苗を植えました。30秒で森になります", "Planted trees. The forest will mature in 30 seconds.", "已种下树苗。30秒后长成森林。"));
                                    }
                                    Err(reason) => self.notice(reason),
                                }
                            }
                            Mode::AttackMove => {
                                let p = self.screen_to_world(x, y);
                                let hs = self.own_sel();
                                self.world.order_move(&hs, p, true);
                                self.marker = Some((p, 0.6, true));
                                self.mode = Mode::Normal;
                                if let Some(t) = &mut self.tut {
                                    t.moved = true;
                                }
                            }
                            Mode::Normal => self.ldown = Some((x, y)),
                        }
                    }
                    1 => {
                        if self.mode != Mode::Normal {
                            self.mode = Mode::Normal;
                            return false;
                        }
                        if self.screen != Screen::Playing {
                            return false;
                        }
                        let (mx0, my0, mx1, my1) = self.mm_rect();
                        if x >= mx0 && x <= mx1 && y >= my0 && y <= my1 && !self.show_diplo {
                            let p = v2((x - mx0) / MM_W * MAP_W as f32, (y - my0) / MM_H * MAP_H as f32);
                            self.right_click_world(p);
                            return false;
                        }
                        if !self.over_ui(x, y) {
                            let p = self.screen_to_world(x, y);
                            self.right_click_world(p);
                        }
                    }
                    _ => {
                        if self.screen == Screen::Playing {
                            self.mdrag = Some((x, y, self.cam));
                        }
                    }
                }
            }
            Ev::Up(b, x, y) => {
                self.mouse = (x, y);
                match b {
                    0 => {
                        self.mm_drag = false;
                        if let Some((sx, sy)) = self.ldown.take() {
                            // タッチ：自国の部隊を選んでいるとき、自国の兵・民以外をタップすれば命令（右クリックと同じ）
                            let tap = (sx - x).abs() < 12.0 && (sy - y).abs() < 12.0;
                            if self.touch && tap && self.screen == Screen::Playing && !self.own_sel().is_empty() {
                                let wp = self.screen_to_world(x, y);
                                let own_unit = self.unit_at(wp).and_then(|h| self.world.unit(h)).map_or(false, |u| u.nation as usize == self.world.player);
                                if !own_unit {
                                    self.right_click_world(wp);
                                    return false;
                                }
                            }
                            self.select_release(sx, sy, x, y);
                        }
                    }
                    2 => self.mdrag = None,
                    _ => {}
                }
            }
            Ev::Key(k, down) => {
                if (k as usize) < 256 {
                    self.keys[k as usize] = down;
                }
                if down {
                    return self.key_down(k);
                }
            }
            Ev::Char(_) => {}
        }
        false
    }

    fn minimap_jump(&mut self, x: f32, y: f32) {
        let (mx0, my0, _, _) = self.mm_rect();
        self.cam = v2(((x - mx0) / MM_W * MAP_W as f32).clamp(0.0, MAP_W as f32), ((y - my0) / MM_H * MAP_H as f32).clamp(0.0, MAP_H as f32));
    }

    fn key_down(&mut self, k: u32) -> bool {
        if std::mem::take(&mut self.discard_pending) && k == VK_ESCAPE {
            return false;
        }
        let ctrl = self.keys[VK_CONTROL as usize];
        match k {
            VK_F5 => {
                self.do_act(Act::SaveMenu);
                return false;
            }
            VK_F11 => {
                self.do_act(Act::Fullscreen);
                return false;
            }
            VK_ADD | VK_OEM_PLUS if ctrl => {
                self.do_act(Act::Scale(1));
                return false;
            }
            VK_SUB | VK_OEM_MINUS if ctrl => {
                self.do_act(Act::Scale(-1));
                return false;
            }
            _ => {}
        }
        if self.screen == Screen::Title {
            if k == VK_ESCAPE {
                if self.show_save {
                    self.show_save = false;
                    return false;
                }
                return true;
            }
            if k == VK_SPACE || k == 0x0D {
                if self.title_sel.is_some() {
                    self.do_act(Act::Start);
                }
            }
            return false;
        }
        // ボタンのホットキー
        if !ctrl {
            let mut hit = None;
            for b in self.buttons.iter() {
                if b.key == k && b.enabled {
                    hit = Some(b.act);
                    break;
                }
            }
            if let Some(a) = hit {
                self.do_act(a);
                return false;
            }
        }
        match k {
            VK_ESCAPE => {
                if self.mode != Mode::Normal {
                    self.mode = Mode::Normal;
                } else if self.world.banner.is_some() && !self.overlay_open() {
                    // 出来事の告知を閉じる
                    self.world.dismiss_banner();
                } else if self.show_diplo || self.show_help || self.show_chron || self.show_market || self.show_summon || self.show_save {
                    self.show_summon = false;
                    self.show_diplo = false;
                    self.show_help = false;
                    self.show_chron = false;
                    self.show_market = false;
                    self.show_save = false;
                } else {
                    self.clear_sel();
                }
            }
            VK_SPACE => self.paused = !self.paused,
            VK_F1 => self.show_help = !self.show_help,
            VK_F2 => self.show_diplo = !self.show_diplo,
            VK_F3 => self.do_act(Act::Chron),
            VK_F4 => self.do_act(Act::Market),
            VK_HOME => self.do_act(Act::Home),
            VK_ADD | VK_OEM_PLUS => self.speed = (self.speed * 2).min(8),
            VK_SUB | VK_OEM_MINUS => self.speed = (self.speed / 2).max(1),
            VK_OEM_PERIOD => self.do_act(Act::IdleWorker),
            VK_DELETE => self.do_act(Act::Demolish),
            VK_TAB | VK_BACK => {}
            0x30..=0x39 => {
                let g = (k - 0x30) as usize;
                if ctrl {
                    self.groups[g] = self.own_sel();
                    self.notice(&tf!("部隊{}に登録", "Saved as group {}", "编为第{}队", g));
                } else {
                    let hs: Vec<Handle> = self.groups[g].iter().copied().filter(|&h| self.world.unit(h).is_some()).collect();
                    if !hs.is_empty() {
                        let double = self.last_click.0 > self.clock - 0.4 && self.last_click.1 == Some(hs[0]);
                        self.set_sel(hs.clone());
                        if double {
                            let p = self.world.unit(hs[0]).unwrap().pos;
                            self.cam = p;
                        }
                        self.last_click = (self.clock, Some(hs[0]));
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn clear_sel(&mut self) {
        self.discard_pending = false;
        for h in self.sel.drain(..) {
            if let Some(u) = self.world.unit_mut(h) {
                u.selected = false;
            }
        }
        self.sel_b = None;
    }

    fn set_sel(&mut self, hs: Vec<Handle>) {
        self.clear_sel();
        for &h in hs.iter() {
            if let Some(u) = self.world.unit_mut(h) {
                u.selected = true;
            }
        }
        self.sel = hs;
    }

    fn own_sel(&self) -> Vec<Handle> {
        let p = self.world.player;
        self.sel.iter().copied().filter(|&h| self.world.unit(h).map_or(false, |u| u.nation as usize == p)).collect()
    }

    fn all_workers(&self) -> Vec<Handle> {
        let p = self.world.player;
        (0..self.world.units.len())
            .filter(|&i| {
                let u = &self.world.units[i];
                u.on_map() && u.nation as usize == p && u.kind == UnitKind::Worker
            })
            .map(|i| self.world.uh(i))
            .collect()
    }

    /// 兵の数の表示（傭兵がいれば「＋傭兵の数」を添える。傭兵は上限の外）
    fn soldier_text(&self, n: u32, cap: u32) -> String {
        let pl = self.world.player;
        let m = self.world.units.iter().filter(|u| u.on_map() && u.nation as usize == pl && u.merc_until > 0.0).count();
        if m > 0 {
            format!("{}/{}+{}", n, cap, m)
        } else {
            format!("{}/{}", n, cap)
        }
    }

    fn all_military(&self) -> Vec<Handle> {
        let p = self.world.player;
        (0..self.world.units.len())
            .filter(|&i| {
                let u = &self.world.units[i];
                u.on_map() && u.nation as usize == p && u.kind.military()
            })
            .map(|i| self.world.uh(i))
            .collect()
    }

    fn upgrade_handles(&mut self, mut hs: Vec<Handle>) {
        let mut err = None;
        let mut ok = 0;
        hs.sort_by(|a, b| self.world.unit(*a).unwrap().power.partial_cmp(&self.world.unit(*b).unwrap().power).unwrap());
        for h in hs {
            match self.world.upgrade_unit(h) {
                Ok(()) => ok += 1,
                Err(e) => err = Some(e),
            }
        }
        if ok == 0 {
            if let Some(e) = err {
                self.notice(e);
            }
        } else {
            self.notice(&tf!("{}部隊の能力を高めた", "Raised the power of {} units", "提升了{}个单位的能力", ok));
        }
    }

    /// 民（選択民または全民）を空いている田・森・銅山へバランス配分する
    fn order_workers_balanced(&mut self, hs: &[Handle], skip_builders: bool) {
        let pl = self.world.player;
        let mut gathering = [0usize; 3];
        for u in self.world.units.iter() {
            if u.on_map() && u.nation as usize == pl && u.kind == UnitKind::Worker && !hs.iter().any(|&h| self.world.unit(h).map_or(false, |x| std::ptr::eq(x, u))) {
                if let Order::Gather { src, .. } = u.order {
                    gathering[src.res()] += 1;
                }
            }
        }
        let mut assigned = 0;
        let mut built = 0;
        let res = self.world.nations[pl].res;
        let mut want = [0.44f32, 0.33, 0.23];
        if self.world.nations[pl].upkeep > self.world.nations[pl].income[0] * 0.55 {
            want[0] += 0.18;
        }
        for k in 0..3 {
            if res[k] > 700.0 {
                want[k] *= 0.45;
            }
        }
        let sumw: f32 = want.iter().sum();

        for &h in hs {
            let Some(u) = self.world.unit(h) else { continue };
            if u.kind != UnitKind::Worker {
                continue;
            }
            if skip_builders && matches!(u.order, Order::Build(_)) {
                continue;
            }
            if let Order::Build(bh) = u.order {
                if self.world.bld(bh).map_or(false, |b| !b.done()) && self.world.bld_builders(bh) <= 1 {
                    continue;
                }
            }
            let p = u.pos;
            // 作業者のいない未完成の建物があれば最優先で建設を再開させる
            if let Some(bh) = self.world.unfinished_bld(pl, p) {
                self.world.order_build(&[h], bh);
                built += 1;
                assigned += 1;
                continue;
            }
            let total = (gathering[0] + gathering[1] + gathering[2] + 1) as f32;
            let mut best_k = 0;
            let mut best_def = f32::MIN;
            for k in 0..3 {
                let def = want[k] / sumw * total - gathering[k] as f32;
                if def > best_def {
                    best_def = def;
                    best_k = k;
                }
            }
            let src = match best_k {
                0 => self.world.free_farm(pl, p).map(Src::Farm),
                2 => self.world.nearest_mine(p, 28.0).map(Src::Mine),
                _ => self.world.nearest_forest(p, 22).map(|(x, y)| Src::Forest(x, y)),
            }
            .or_else(|| self.world.free_farm(pl, p).map(Src::Farm))
            .or_else(|| self.world.nearest_forest(p, 28).map(|(x, y)| Src::Forest(x, y)))
            .or_else(|| self.world.nearest_mine(p, 36.0).map(Src::Mine));
            if let Some(s) = src {
                gathering[s.res()] += 1;
                self.world.order_gather(&[h], s);
                assigned += 1;
            }
        }
        if assigned > 0 {
            if built > 0 {
                self.notice(&tf!("未完成の建物へ民{}体を派遣し、計{}体を自動配分した", "Sent {} workers to unfinished buildings; {} assigned in all", "派{}名民前往未完工建筑，共自动分配{}名", built, assigned));
            } else {
                self.notice(&tf!("民{}体を田・森・鉱山へ自動配分した", "Assigned {} workers to fields, forests and mines", "已将{}名民分配到田、森林、铁矿", assigned));
            }
        } else {
            self.notice(t!("配分できる民や資源が近くにない", "No workers or resources nearby to assign", "附近没有可分配的民或资源"));
        }
    }

    /// 指定した未完成建物へ、近くの民を1体派遣して建設を再開・加速させる
    fn dispatch_worker_to_build(&mut self, bh: Handle) {
        let pl = self.world.player;
        let Some(b) = self.world.bld(bh) else { return };
        if b.nation as usize != pl || b.done() {
            return;
        }
        let c = b.center();
        let bname = b.kind.name();
        let mut best: Option<(f32, Handle)> = None;
        for (i, u) in self.world.units.iter().enumerate() {
            if !u.on_map() || u.nation as usize != pl || u.kind != UnitKind::Worker {
                continue;
            }
            if u.order == Order::Build(bh) {
                continue;
            }
            let pen = match u.order {
                Order::Idle => 0.0,
                Order::Build(_) => 100.0,
                _ => 8.0,
            };
            let score = u.pos.dist(c) + pen;
            if best.map_or(true, |bs| score < bs.0) {
                best = Some((score, self.world.uh(i)));
            }
        }
        if let Some((_, wh)) = best {
            self.world.order_build(&[wh], bh);
            self.marker = Some((c, 0.6, false));
            self.notice(&tf!("民を{}の建設へ派遣した", "Sent workers to build the {}", "已派民去建造{}", bname));
        } else if self.world.bld_builders(bh) > 0 {
            self.notice(&tf!("すでに民が{}を建設中", "Workers are already building the {}", "已有民在建造{}", bname));
        } else {
            self.notice(t!("派遣できる民がいない", "No workers to send", "没有可派遣的民"));
        }
    }

    /// 民に一斉伐採（最寄りの森）または一斉採掘（最寄りの銅山）を命じる
    fn order_workers_resource(&mut self, hs: &[Handle], wood: bool) {
        let mut assigned = 0;
        let home = self.world.nations[self.world.player].home;
        for &h in hs {
            let Some(u) = self.world.unit(h) else { continue };
            if u.kind != UnitKind::Worker || matches!(u.order, Order::Build(_)) {
                continue;
            }
            let p = u.pos;
            let src = if wood {
                self.world
                    .nearest_forest(p, 28)
                    .or_else(|| self.world.nearest_forest(home, 36))
                    .map(|(x, y)| Src::Forest(x, y))
            } else {
                self.world
                    .nearest_mine(p, 35.0)
                    .or_else(|| self.world.nearest_mine(home, 45.0))
                    .map(Src::Mine)
            };
            if let Some(s) = src {
                self.world.order_gather(&[h], s);
                assigned += 1;
            }
        }
        if assigned > 0 {
            let kind_str = if wood { t!("森の伐採", "cut wood", "伐木") } else { t!("鉱山の採掘", "mine iron", "采铁") };
            self.notice(&tf!("民{}体に{}を命じた", "Ordered {} workers to {}", "命令{}名民{}", assigned, kind_str));
        } else {
            self.notice(if wood { t!("近くに伐採できる森がない", "No forest nearby to cut", "附近没有可砍伐的森林") } else { t!("近くに採掘できる鉱山がない", "No mine nearby", "附近没有可开采的铁矿") });
        }
    }

    /// 軍事ユニットに領内防衛・侵入者迎撃・城邑集結を命じる
    fn order_army_defend(&mut self, hs: &[Handle]) {
        if hs.is_empty() {
            self.notice(t!("軍事ユニットがいない", "No soldiers", "没有军事单位"));
            return;
        }
        let pl = self.world.player;
        let home = self.world.nations[pl].home;
        let mut intruder: Option<V2> = None;
        for u in self.world.units.iter() {
            if !u.on_map() || !self.world.at_war(pl, u.nation as usize) {
                continue;
            }
            let (x, y) = u.pos.tile();
            if Map::inb(x, y) && self.world.owner[Map::idx(x, y)] as usize == pl {
                if intruder.map_or(true, |p| p.dist(home) > u.pos.dist(home)) {
                    intruder = Some(u.pos);
                }
            }
        }
        if let Some(p) = intruder {
            self.world.order_move(hs, p, true);
            self.marker = Some((p, 0.6, true));
            self.notice(&tf!("{}部隊に領内侵入者の迎撃を命じた", "Ordered {} units to intercept intruders", "命令{}个单位迎击入侵者", hs.len()));
            if let Some(t) = &mut self.tut {
                t.moved = true;
            }
            return;
        }
        let cities: Vec<V2> = self
            .world
            .blds
            .iter()
            .filter(|b| b.alive && b.nation as usize == pl && b.kind.city() && b.done())
            .map(|b| b.center())
            .collect();
        let rally = if cities.is_empty() {
            home
        } else {
            // 各部隊から最も近い自国城邑へ帰還・防衛配置
            for (k, &h) in hs.iter().enumerate() {
                if let Some(u) = self.world.unit(h) {
                    let up = u.pos;
                    let c = cities.iter().copied().min_by(|a, b| a.dist(up).partial_cmp(&b.dist(up)).unwrap()).unwrap_or(home);
                    let ang = (k as f32) * 1.1;
                    let goal = c + v2(ang.cos() * 2.2, ang.sin() * 2.2);
                    self.world.order_move(&[h], goal, true);
                }
            }
            cities[0]
        };
        self.marker = Some((rally, 0.6, false));
        self.notice(&tf!("{}部隊に街の防衛集結を命じた", "Ordered {} units to gather and defend the towns", "命令{}个单位集结防守城镇", hs.len()));
        if let Some(t) = &mut self.tut {
            t.moved = true;
        }
    }

    /// 全軍に総進撃（交戦国の最寄り城邑・敵兵への攻撃移動＋進軍モード）を命じる
    fn order_all_army_attack(&mut self) {
        let hs = self.all_military();
        if hs.is_empty() {
            self.notice(t!("軍事ユニットがいない", "No soldiers", "没有军事单位"));
            return;
        }
        self.set_sel(hs.clone());
        let pl = self.world.player;
        let home = self.world.nations[pl].home;
        // 交戦中の敵城邑または敵部隊を探す
        let mut best_target: Option<(f32, V2)> = None;
        for b in self.world.blds.iter() {
            if b.alive && self.world.at_war(pl, b.nation as usize) && b.kind.city() {
                let d = b.center().dist(home);
                if best_target.map_or(true, |x| d < x.0) {
                    best_target = Some((d, b.center()));
                }
            }
        }
        if best_target.is_none() {
            for u in self.world.units.iter() {
                if u.on_map() && self.world.at_war(pl, u.nation as usize) {
                    let d = u.pos.dist(home);
                    if best_target.map_or(true, |x| d < x.0) {
                        best_target = Some((d, u.pos));
                    }
                }
            }
        }
        if let Some((_, goal)) = best_target {
            self.world.order_move(&hs, goal, true);
            self.marker = Some((goal, 0.6, true));
            self.mode = Mode::AttackMove;
            self.notice(&tf!("全軍{}部隊に敵陣への総進撃を命じた（地図クリックで目標変更可）", "All {} units ordered to advance on the enemy (click the map to change the target)", "全军{}个单位向敌阵总攻（点击地图可改变目标）", hs.len()));
            if let Some(t) = &mut self.tut {
                t.moved = true;
            }
        } else {
            self.mode = Mode::AttackMove;
            self.notice(&tf!("全軍{}部隊を選択：進軍先を地図上でクリックしてください", "All {} units selected: click the map to choose where to march", "已选全军{}个单位：请在地图上点击进军目标", hs.len()));
        }
    }

    fn unit_at(&self, p: V2) -> Option<Handle> {
        let mut best: Option<(f32, Handle)> = None;
        for (i, u) in self.world.units.iter().enumerate() {
            if !u.on_map() {
                continue;
            }
            if u.nation as usize != self.world.player && !self.seen(u.pos) {
                continue;
            }
            let r: f32 = if u.kind == UnitKind::Worker { 0.45 } else if u.kind.naval() { 1.6 } else { 0.55 };
            // タッチは指の太さぶん広く拾う（画面上で約 18 単位）
            let r = if self.touch { r.max(18.0 / self.zoom) } else { r };
            let d = u.pos.dist(p);
            if d < r && best.map_or(true, |b| d < b.0) {
                best = Some((d, self.world.uh(i)));
            }
        }
        best.map(|b| b.1)
    }

    fn bld_at(&self, p: V2) -> Option<Handle> {
        let (x, y) = p.tile();
        if Map::inb(x, y) {
            let b = self.world.bmap[Map::idx(x, y)];
            if b != 0 && self.explored(p) {
                return Some(self.world.bh(b as usize - 1));
            }
        }
        let mut best: Option<(f32, Handle)> = None;
        for (i, b) in self.world.blds.iter().enumerate() {
            if !b.alive || !self.explored(b.center()) {
                continue;
            }
            let d = b.dist_to_footprint(p);
            if d < 0.45 && best.map_or(true, |bs| d < bs.0) {
                best = Some((d, self.world.bh(i)));
            }
        }
        best.map(|bs| bs.1)
    }

    fn select_release(&mut self, sx: f32, sy: f32, x: f32, y: f32) {
        let p = self.world.player;
        if (sx - x).abs() < 5.0 && (sy - y).abs() < 5.0 {
            let wp = self.screen_to_world(x, y);
            if let Some(h) = self.unit_at(wp) {
                let shift = self.keys[VK_SHIFT as usize];
                let double = self.last_click.0 > self.clock - 0.35 && self.last_click.1 == Some(h);
                self.last_click = (self.clock, Some(h));
                let u = self.world.unit(h).unwrap();
                if double && u.nation as usize == p {
                    // 画面内の同種ユニットを全選択
                    let kind = u.kind;
                    let (w, hh) = (self.gfx.width as f32, self.gfx.height as f32);
                    let a = self.screen_to_world(0.0, self.top_h());
                    let b = self.screen_to_world(w, hh - BOT_H);
                    let hs: Vec<Handle> = (0..self.world.units.len())
                        .filter(|&i| {
                            let u = &self.world.units[i];
                            u.on_map() && u.nation as usize == p && u.kind == kind && u.pos.x >= a.x && u.pos.x <= b.x && u.pos.y >= a.y && u.pos.y <= b.y
                        })
                        .map(|i| self.world.uh(i))
                        .collect();
                    self.set_sel(hs);
                } else if shift && u.nation as usize == p {
                    let mut hs = self.own_sel();
                    if let Some(k) = hs.iter().position(|&x| x == h) {
                        hs.remove(k);
                    } else {
                        hs.push(h);
                    }
                    self.set_sel(hs);
                } else {
                    self.set_sel(vec![h]);
                }
            } else if let Some(b) = self.bld_at(wp) {
                self.clear_sel();
                self.sel_b = Some(b);
            } else {
                self.clear_sel();
            }
        } else {
            let a = self.screen_to_world(sx.min(x), sy.min(y));
            let b = self.screen_to_world(sx.max(x), sy.max(y));
            let hs: Vec<Handle> = (0..self.world.units.len())
                .filter(|&i| {
                    let u = &self.world.units[i];
                    u.on_map() && u.nation as usize == p && u.pos.x >= a.x && u.pos.x <= b.x && u.pos.y >= a.y && u.pos.y <= b.y
                })
                .map(|i| self.world.uh(i))
                .collect();
            if !hs.is_empty() {
                let mut hs = hs;
                if self.keys[VK_SHIFT as usize] {
                    for h in self.own_sel() {
                        if !hs.contains(&h) {
                            hs.push(h);
                        }
                    }
                }
                self.set_sel(hs);
            } else if !self.keys[VK_SHIFT as usize] {
                self.clear_sel();
            }
        }
    }

    fn right_click_world(&mut self, p: V2) {
        self.discard_pending = false;
        let pl = self.world.player;
        // ユニット未選択時：未完成建物の右クリックなら民を派遣、建物選択中なら集合地点設定
        if self.sel.is_empty() {
            if let Some(bh) = self.bld_at(p) {
                if let Some(b) = self.world.bld(bh) {
                    if b.nation as usize == pl && !b.done() {
                        self.dispatch_worker_to_build(bh);
                        return;
                    }
                }
            }
            if let Some(bh) = self.sel_b {
                if let Some(b) = self.world.bld_mut(bh) {
                    if b.nation as usize == pl && b.kind != BKind::Farm {
                        b.rally = Some(p);
                        self.marker = Some((p, 0.6, false));
                    }
                }
            }
            return;
        }
        let hs = self.own_sel();
        if hs.is_empty() {
            return;
        }
        let workers: Vec<Handle> = hs.iter().copied().filter(|&h| self.world.unit(h).map_or(false, |u| u.kind == UnitKind::Worker)).collect();
        let soldiers: Vec<Handle> = hs.iter().copied().filter(|&h| self.world.unit(h).map_or(false, |u| u.kind.military())).collect();
        if !soldiers.is_empty() {
            if let Some(t) = &mut self.tut {
                t.moved = true;
            }
        }
        let ships: Vec<Handle> = hs.iter().copied().filter(|&h| self.world.unit(h).map_or(false, |u| u.kind.naval())).collect();
        let land: Vec<Handle> = hs.iter().copied().filter(|&h| self.world.unit(h).map_or(false, |u| !u.kind.naval())).collect();
        // 海の向こうへの命令：船があれば自動で渡る。無ければ知らせる
        let across = land.iter().any(|&h| self.world.unit(h).map_or(false, |u| !self.world.same_land(u.pos, p))) && self.world.passable_at(p);
        if across && self.unit_at(p).map_or(true, |t| self.world.unit(t).map_or(true, |u| !u.kind.naval())) {
            if self.world.has_ships(pl) {
                self.notice(t!("海の向こうへは船で渡る：兵は近くの岸で待ち、手の空いた船が往復して運ぶ", "Across the sea: the troops wait on the nearest shore, and free ships ferry them back and forth", "渡海：士兵在最近的岸边等候，空闲的船只会往返运送"));
            } else if self.world.can_build_ships(pl) {
                self.notice(t!("海を渡るには船が要る（民で海辺に港を建て、港で船を造る）", "Crossing the sea needs ships (build a harbor on the coast with workers, then build ships there)", "渡海需要船（用民在海边建港口，再在港口造船）"));
            } else {
                self.notice(t!("海を渡るには船が要る（イングランドは港でコグ船を造り、フランスは交易画面でガレー船を雇う）", "Crossing the sea needs ships (England builds cogs at a Harbor; France hires galleys on the Trade screen)", "渡海需要船（英格兰在港口建造柯克船，法兰西在贸易界面雇佣桨帆船）"));
            }
        }
        // 自国の船：陸の兵を乗せる
        if let Some(t) = self.unit_at(p) {
            let tu = self.world.unit(t).unwrap();
            if tu.nation as usize == pl && tu.kind.naval() && !land.is_empty() {
                // 選んでいる船があれば、それとこの船とに分けて乗せる（どの船に乗るかは問わない）
                let mut pool = ships.clone();
                if !pool.contains(&t) {
                    pool.push(t);
                }
                let left = self.world.order_board_many(&land, &pool);
                self.marker = Some((p, 0.6, false));
                if left == land.len() {
                    self.notice(t!("船は満員（1隻に6人まで）", "The ships are full (6 troops each)", "船已满（每船最多6人）"));
                } else if left > 0 {
                    self.notice(&tf!("船が足りず、{}人は乗れない（1隻に6人まで）", "Not enough room: {} troops cannot board (6 per ship)", "船位不足，{}人无法登船（每船6人）", left));
                }
                return;
            }
        }
        // 船で陸を右クリック：その近くの岸へ向かい、兵を降ろす
        if !ships.is_empty() && self.world.passable_at(p) && self.unit_at(p).map_or(true, |t| self.world.unit(t).map_or(true, |u| u.nation as usize == pl)) && self.bld_at(p).map_or(true, |b| self.world.bld(b).map_or(true, |b| b.nation as usize == pl)) {
            let loaded: Vec<Handle> = ships.iter().copied().filter(|&h| self.world.unit(h).map_or(false, |u| !u.cargo.is_empty())).collect();
            if !loaded.is_empty() {
                self.world.order_unload(&loaded, p);
                self.marker = Some((p, 0.6, true));
            } else if let Some(q) = self.world.shore_water_near(p, 6) {
                self.world.order_move(&ships, q, false);
                self.marker = Some((q, 0.6, false));
            } else {
                self.notice(t!("船は陸に上がれない", "Ships cannot go on land", "船无法上岸"));
            }
            if !land.is_empty() {
                self.world.order_move(&land, p, false);
            }
            return;
        }
        // 敵ユニット
        if let Some(t) = self.unit_at(p) {
            let tn = self.world.unit(t).unwrap().nation as usize;
            if tn != pl {
                if self.world.at_war(pl, tn) {
                    self.world.order_attack(&hs, Target::Unit(t));
                    self.marker = Some((p, 0.6, true));
                } else {
                    self.notice(&tf!("{} とは交戦していない（外交 F2 で宣戦できる）", "Not at war with {} (declare war in Diplomacy, F2)", "未与{}交战（可在外交 F2 中宣战）", crate::lang::tx(self.world.nations[tn].name)));
                    self.world.order_move(&hs, p, false);
                }
                return;
            }
        }
        if let Some(bh) = self.bld_at(p) {
            let b = self.world.bld(bh).unwrap();
            let (bn, kind, done, bc) = (b.nation as usize, b.kind, b.done(), b.center());
            if bn != pl {
                if self.world.at_war(pl, bn) {
                    self.world.order_attack(&hs, Target::Bld(bh));
                    self.marker = Some((p, 0.6, true));
                } else {
                    self.notice(&tf!("{} とは交戦していない（外交 F2 で宣戦できる）", "Not at war with {} (declare war in Diplomacy, F2)", "未与{}交战（可在外交 F2 中宣战）", crate::lang::tx(self.world.nations[bn].name)));
                }
                return;
            }
            if !done {
                if !workers.is_empty() {
                    self.world.order_build(&workers, bh);
                    self.marker = Some((bc, 0.6, false));
                    self.notice(&tf!("{}の建設へ民{}体を向かわせた", "Sent {1} workers to build the {0}", "派{1}名民去建造{0}", kind.name(), workers.len()));
                } else if self.world.bld_builders(bh) == 0 {
                    self.dispatch_worker_to_build(bh);
                }
                if !soldiers.is_empty() {
                    self.world.order_move(&soldiers, p, false);
                }
                return;
            }
            if kind == BKind::Farm && !workers.is_empty() {
                self.world.order_gather(&workers[..1], Src::Farm(bh));
                if workers.len() > 1 {
                    self.world.order_move(&workers[1..], p, false);
                }
                self.world.order_move(&soldiers, p, false);
                return;
            }
        }
        let (tx, ty) = p.tile();
        if !workers.is_empty() && Map::inb(tx, ty) {
            if self.world.map.terrain[Map::idx(tx, ty)] == Terrain::Forest {
                self.world.order_gather(&workers, Src::Forest(tx, ty));
                self.world.order_move(&soldiers, p, false);
                self.marker = Some((p, 0.6, false));
                return;
            }
            if let Some(m) = self.world.map.mines.iter().position(|m| m.x == tx && m.y == ty && m.amount > 0.0) {
                self.world.order_gather(&workers, Src::Mine(m));
                self.world.order_move(&soldiers, p, false);
                self.marker = Some((p, 0.6, false));
                return;
            }
        }
        self.world.order_move(&hs, p, false);
        self.marker = Some((p, 0.6, false));
    }

    fn placement_tile(&self) -> (i32, i32) {
        let p = self.screen_to_world(self.mouse.0, self.mouse.1);
        ((p.x - 0.5).round() as i32, (p.y - 0.5).round() as i32)
    }

    /// 建物を置く。queue なら建設の予約（選んだ民は今の建設を終えてから取りかかる）
    fn try_place(&mut self, k: BKind, queue: bool) {
        let (tx, ty) = self.placement_tile();
        let pl = self.world.player;
        match self.world.place_building(pl, k, tx, ty) {
            Ok(bh) => {
                let ws: Vec<Handle> = self.own_sel().into_iter().filter(|&h| self.world.unit(h).map_or(false, |u| u.kind == UnitKind::Worker)).collect();
                if queue {
                    self.world.queue_build(&ws, bh);
                } else {
                    self.world.order_build(&ws, bh);
                }
            }
            Err(e) => self.notice(e),
        }
    }

    fn title_click(&mut self, x: f32, y: f32) {
        let p = self.screen_to_world(x, y);
        let mut best: Option<(f32, usize)> = None;
        for (i, n) in self.world.nations.iter().enumerate() {
            if !n.alive {
                continue;
            }
            let d = n.home.dist(p);
            if d < 10.0 && best.map_or(true, |b| d < b.0) {
                best = Some((d, i));
            }
        }
        if let Some((_, i)) = best {
            self.title_sel = Some(i);
        }
    }

    fn button_at(&self, x: f32, y: f32) -> Option<Act> {
        for b in self.buttons.iter().rev() {
            if x >= b.x0 && x <= b.x1 && y >= b.y0 && y <= b.y1 {
                return Some(if b.enabled { b.act } else { Act::Noop });
            }
        }
        // タッチは指先がずれやすいので、すぐ近くのボタンも拾う
        if self.touch {
            let near = self
                .buttons
                .iter()
                .rev()
                .map(|b| ((b.x0 - x).max(x - b.x1).max(0.0).hypot((b.y0 - y).max(y - b.y1).max(0.0)), b))
                .filter(|(d, _)| *d <= 10.0)
                .min_by(|a, b| a.0.total_cmp(&b.0));
            if let Some((_, b)) = near {
                return Some(if b.enabled { b.act } else { Act::Noop });
            }
        }
        None
    }

    fn do_act(&mut self, a: Act) {
        let discard_pending = std::mem::take(&mut self.discard_pending);
        crate::platform::crumb(format!("操作 {:?} 選択{}", a, self.sel.len()));
        // 部隊への指示（移動・攻撃以外）でも時間を再開する
        if matches!(a, Act::Upgrade | Act::Convert | Act::Stop | Act::WorkersAuto | Act::ArmyDefend | Act::Demolish) {
            self.cmd_pause = false;
        }
        let pl = self.world.player;
        match a {
            Act::Noop => {}
            Act::Start => {
                // 押した場所に出るボタンの説明を、始めた直後に出さない
                self.tip_until = 0.0;
                if let Some(n) = self.title_sel {
                    self.world.player = n;
                    self.world.heroic = self.heroic_opt;
                    self.screen = Screen::Playing;
                    self.zoom = 22.0;
                    self.cam = self.world.nations[n].home;
                    self.reset_fog();
                    self.autosave_years = self.world.years;
                    let name = crate::lang::tx(self.world.nations[n].name);
                    self.world.push_msg(tf!("1337年、百年にわたる戦が始まる。あなたは {} の王となった。", "1337: a war that will last a hundred years begins. You are now the King of {}.", "1337年，长达百年的战争开始了。你成为了{}的国王。", name), [1.0, 0.9, 0.6]);
                    self.world.push_msg(t!("F1でヘルプ、F2で外交画面を開く", "F1 opens Help, F2 opens Diplomacy", "F1 打开帮助，F2 打开外交画面").into(), [0.8, 0.8, 0.8]);
                }
            }
            Act::Tutorial => self.start_tutorial(),
            Act::TutNext => self.tut_next(),
            Act::TutQuit => self.end_tutorial(),
            Act::Scale(d) => {
                self.gfx.adjust_scale(d);
                if self.screen == Screen::Title {
                    self.fit_title_camera();
                }
                self.notice(&tf!("文字の大きさ ×{:.2}", "Text size ×{:.2}", "文字大小 ×{:.2}", self.gfx.ui));
            }
            Act::Fullscreen => self.gfx.toggle_fullscreen(),
            #[cfg(target_arch = "wasm32")]
            Act::RefreshCache => unsafe { crate::platform::js::js_refresh_cache() },
            Act::Lang => {
                let l = crate::lang::lang().next();
                crate::lang::set_lang(l);
                crate::platform::save_lang(l);
                self.gfx.reload_fonts();
            }
            Act::DismissBanner => self.world.dismiss_banner(),
            Act::DismissAllBanners => self.world.dismiss_all_banners(),
            Act::ToggleAlerts => self.alerts_open = !self.alerts_open,
            Act::ClearMsgs => self.msg_cut = self.world.time,
            Act::Cancel => {
                if self.mode != Mode::Normal {
                    self.mode = Mode::Normal;
                    self.place_tap = None;
                } else if self.overlay_open() {
                    // 開いている一覧を閉じる（Esc と同じ）
                    self.show_summon = false;
                    self.show_diplo = false;
                    self.show_help = false;
                    self.show_chron = false;
                    self.show_market = false;
                    self.show_save = false;
                } else {
                    self.clear_sel();
                }
            }
            Act::SaveMenu => {
                self.show_save = !self.show_save;
                if self.show_save {
                    self.show_diplo = false;
                    self.show_help = false;
                    self.show_chron = false;
                    self.show_market = false;
                    self.show_summon = false;
                }
            }
            Act::SaveSlot(s) => self.save_slot(s, false),
            Act::LoadSlot(s) => {
                self.notice(t!("記録を読み込んでいる…", "Loading the save...", "正在读取存档…"));
                crate::platform::load_slot(s);
            }
            Act::ExportSlot(s) => crate::platform::export_slot(s),
            Act::ImportFile => crate::platform::import_file(),
            Act::Restart => {
                self.tut = None;
                self.seed = self.seed.wrapping_add(17);
                self.world = World::new(self.seed);
                self.rebuild_terrain();
                self.screen = Screen::Title;
                self.title_sel = None;
                self.sel.clear();
                self.sel_b = None;
                self.show_diplo = false;
                self.show_help = false;
                self.show_chron = false;
                self.show_market = false;
                self.show_save = false;
                self.show_summon = false;
                self.paused = false;
                self.speed = 1;
                self.fit_title_camera();
            }
            Act::Train(k) => {
                if let Some(bh) = self.sel_b {
                    if let Err(e) = self.world.enqueue(bh, k) {
                        self.notice(e);
                    }
                }
            }
            Act::TrainPage => self.train_page += 1,
            Act::BoardAll => {
                let own = self.own_sel();
                let ships: Vec<Handle> = own.iter().copied().filter(|&h| self.world.unit(h).map_or(false, |u| u.kind.naval())).collect();
                let land: Vec<Handle> = own.iter().copied().filter(|&h| self.world.unit(h).map_or(false, |u| !u.kind.naval())).collect();
                if !ships.is_empty() && !land.is_empty() {
                    let left = self.world.order_board_many(&land, &ships);
                    self.cmd_pause = false;
                    if left > 0 {
                        self.notice(&tf!("船が足りず、{}人は乗れない（1隻に6人まで）", "Not enough room: {} troops cannot board (6 per ship)", "船位不足，{}人无法登船（每船6人）", left));
                    }
                }
            }
            Act::Raze => {}
            Act::Unload => {
                let ships: Vec<Handle> = self.own_sel().into_iter().filter(|&h| self.world.unit(h).map_or(false, |u| u.kind.naval())).collect();
                let mut any = false;
                for h in ships {
                    if self.world.unit(h).map_or(false, |u| !u.cargo.is_empty()) {
                        if self.world.unload_here(h, None) {
                            any = true;
                        } else {
                            self.notice(t!("岸の近くでなければ降ろせない（陸を右クリックすると、その近くの岸へ向かって降ろす）", "Troops can only land near the shore (right-click land to sail there and land them)", "必须靠岸才能卸下（右键点击陆地，船会驶往附近的岸边卸下士兵）"));
                        }
                    }
                }
                if any {
                    self.cmd_pause = false;
                }
            }
            Act::Place(k) => {
                let cost = k.cost();
                let r = self.world.nations[pl].res;
                if r[0] < cost[0] || r[1] < cost[1] || r[2] < cost[2] {
                    self.notice(t!("資源が足りない", "Not enough resources", "资源不足"));
                } else {
                    self.mode = Mode::Place(k);
                    self.place_tap = None;
                }
            }
            Act::Upgrade => {
                let hs = self.own_sel();
                self.upgrade_handles(hs);
            }
            Act::Convert => {
                let mut err = None;
                for h in self.own_sel() {
                    if let Err(e) = self.world.convert_unit(h) {
                        err = Some(e);
                    }
                }
                if let Some(e) = err {
                    self.notice(e);
                }
            }
            Act::Stop => {
                for h in self.own_sel() {
                    if let Some(u) = self.world.unit_mut(h) {
                        u.order = Order::Idle;
                        u.resume = None;
                        u.path.clear();
                        u.build_queue.clear();
                    }
                }
            }
            Act::AttackMove => {
                if !self.own_sel().is_empty() {
                    self.mode = Mode::AttackMove;
                }
            }
            Act::SelectAllWorkers => {
                let hs = self.all_workers();
                if hs.is_empty() {
                    self.notice(t!("民がいない", "No workers", "没有民"));
                } else {
                    self.notice(&tf!("民全体（{}体）を選択した", "Selected all workers ({})", "已选择全部民（{}名）", hs.len()));
                    self.set_sel(hs);
                }
            }
            Act::SelectAllMilitary => {
                let hs = self.all_military();
                if hs.is_empty() {
                    self.notice(t!("軍事ユニットがいない", "No soldiers", "没有军事单位"));
                } else {
                    self.notice(&tf!("全軍事ユニット（{}体）を選択した", "Selected all soldiers ({})", "已选择全部军事单位（{}个）", hs.len()));
                    self.set_sel(hs);
                }
            }
            Act::SelectKind(kind) => {
                let hs: Vec<Handle> = (0..self.world.units.len())
                    .filter(|&i| {
                        let u = &self.world.units[i];
                        u.on_map() && u.nation as usize == pl && u.kind == kind
                    })
                    .map(|i| self.world.uh(i))
                    .collect();
                if hs.is_empty() {
                    self.notice(&tf!("{}はいない", "No {} units", "没有{}", kind.name()));
                } else {
                    self.notice(&tf!("{}全体（{}体）を選択した", "Selected all {} units ({})", "已选择全部{}（{}个）", kind.name(), hs.len()));
                    self.set_sel(hs);
                }
            }
            Act::FilterKind(kind) => {
                let hs: Vec<Handle> = self
                    .own_sel()
                    .into_iter()
                    .filter(|&h| self.world.unit(h).map_or(false, |u| u.kind == kind))
                    .collect();
                if !hs.is_empty() {
                    self.set_sel(hs);
                }
            }
            Act::WorkersAuto => {
                let hs = self.own_sel();
                self.order_workers_balanced(&hs, false);
            }
            Act::AllWorkersAuto => {
                let hs = self.all_workers();
                self.order_workers_balanced(&hs, true);
            }
            Act::AllWorkersWood => {
                let hs = self.all_workers();
                self.order_workers_resource(&hs, true);
            }
            Act::AllWorkersMine => {
                let hs = self.all_workers();
                self.order_workers_resource(&hs, false);
            }
            Act::AllWorkersUpgrade => {
                let hs = self.all_workers();
                if hs.is_empty() {
                    self.notice(t!("民がいない", "No workers", "没有民"));
                } else {
                    self.upgrade_handles(hs);
                }
            }
            Act::ArmyDefend => {
                let hs: Vec<Handle> = self.own_sel().into_iter().filter(|&h| self.world.unit(h).map_or(false, |u| u.kind.military())).collect();
                self.order_army_defend(&hs);
            }
            Act::AllArmyAttack => {
                self.order_all_army_attack();
            }
            Act::AllArmyDefend => {
                let hs = self.all_military();
                self.order_army_defend(&hs);
            }
            Act::AllArmyUpgrade => {
                let hs = self.all_military();
                if hs.is_empty() {
                    self.notice(t!("軍事ユニットがいない", "No soldiers", "没有军事单位"));
                } else {
                    self.upgrade_handles(hs);
                }
            }
            Act::Speed(s) => {
                if s == 0 {
                    self.paused = !self.paused;
                } else {
                    self.speed = s;
                    self.paused = false;
                }
            }
            Act::Diplo => {
                self.show_diplo = !self.show_diplo;
                self.show_help = false;
                self.show_chron = false;
                self.show_market = false;
                self.show_save = false;
            }
            Act::Help => {
                self.show_help = !self.show_help;
                self.show_diplo = false;
                self.show_chron = false;
                self.show_market = false;
                self.show_save = false;
            }
            Act::Chron => {
                self.show_chron = !self.show_chron;
                self.chron_scroll = 0;
                self.show_diplo = false;
                self.show_help = false;
                self.show_market = false;
                self.show_save = false;
            }
            Act::Market => {
                self.show_market = !self.show_market;
                self.show_diplo = false;
                self.show_help = false;
                self.show_chron = false;
            }
            Act::Tax(t) => {
                self.world.nations[pl].tax = t;
                self.notice(&tf!("税率を{}にした", "Tax set to {}", "税率改为{}", tax_name(t as usize)));
            }
            Act::Buy(k) => match self.world.buy_resource(pl, k) {
                Ok(()) => self.notice(&tf!("市場で{}を{}買った", "Bought {1} {0} at the market", "在市场购入{0}{1}", res_name(k), MARKET_LOT as i32)),
                Err(e) => self.notice(e),
            },
            Act::Hire(t) => match self.world.hire_mercs(pl, t) {
                Ok((hs, bi)) => {
                    // 現れた場所が分かるよう、交易画面を閉じて傭兵の所へ地図を動かし、選んでおく（すぐ命令できる）
                    let place = self.world.city_place(bi);
                    let c = self.world.blds[bi].center();
                    self.show_market = false;
                    self.cam = c;
                    self.clamp_cam();
                    self.marker = Some((c, 1.5, false));
                    if !hs.is_empty() {
                        self.set_sel(hs);
                    }
                    self.notice(&tf!("{}の傭兵が{}に現れた", "{} mercenaries appeared at {}", "{}佣兵出现在{}", crate::lang::tx(TRIBES[t].name), place));
                }
                Err(e) => self.notice(e),
            },
            Act::ChronPage(d) => {
                self.chron_scroll = (self.chron_scroll as i32 + d).max(0) as usize;
            }
            Act::ToggleHeroic => {
                self.heroic_opt = !self.heroic_opt;
            }
            Act::Summon => {
                {
                    self.show_summon = !self.show_summon;
                    self.show_diplo = false;
                    self.show_help = false;
                    self.show_chron = false;
                    self.show_market = false;
                    self.show_save = false;
                }
            }
            Act::SummonHero(k) => match self.world.demand_ransom(k) {
                Ok(a) => self.notice(&tf!("身代金 金貨{:.0}を受け取った", "Received a ransom of {:.0} gold", "收到赎金 金币{:.0}", a)),
                Err(e) => self.notice(e),
            },
            Act::CloseOverlay => {
                self.show_summon = false;
                self.show_diplo = false;
                self.show_help = false;
                self.show_chron = false;
                self.show_market = false;
                self.show_save = false;
            }
            Act::War(o) => {
                if self.world.truce[pl][o] > self.world.years {
                    let r = (self.world.truce[pl][o] - self.world.years).ceil() as i32;
                    self.notice(&tf!("休戦期間中（残り{}年）", "Truce in effect ({} years left)", "休战期内（剩余{}年）", r));
                } else {
                    self.world.declare_war(pl, o, false);
                }
            }
            Act::Peace(o) => {
                let w = &mut self.world;
                if w.coalition_locked(pl, o) {
                    self.notice(t!("連合軍の討伐中は和睦できない", "Cannot make peace while a coalition campaign is on", "联军讨伐期间无法议和"));
                    return;
                }
                let dur = w.years - w.war_start[pl][o];
                let accept = w.peace_offer[o] || (dur > 5.0 && (w.nations[o].strength < w.nations[pl].strength * 1.1 || w.rng.chance(0.25)));
                if accept {
                    w.make_peace(pl, o, 10.0);
                } else {
                    let t = tf!("{} は和睦を拒否した", "{} refused peace", "{}拒绝议和", crate::lang::tx(w.nations[o].name));
                    self.notice(&t);
                }
            }
            Act::PeaceAnswer(accept) => self.world.answer_peace(accept),
            Act::Home => {
                if let Some(c) = self.world.nations[pl].capital.and_then(|h| self.world.bld(h)) {
                    self.cam = c.center();
                }
            }
            Act::Discard => {
                let hs = self.own_sel();
                self.discard_pending = !hs.is_empty() && hs.iter().all(|&h| self.world.can_discard_unit(pl, h));
            }
            Act::ConfirmDiscard => {
                let hs = self.own_sel();
                if discard_pending && self.world.discard_units(pl, &hs) {
                    self.clear_sel();
                    self.cmd_pause = false;
                    self.notice(&tf!("ユニットを{}体破棄しました", "Removed {} units", "已移除{}个单位", hs.len()));
                }
            }
            Act::PlantForest => {
                self.mode = Mode::PlantForest;
                self.notice(t!("植林する場所を選ぶ", "Choose where to plant trees", "请选择植树地点"));
            }
            Act::Repair => {
                if let Some(h) = self.sel_b {
                    if self.world.repair_building(pl, h) {
                        self.notice(t!("建物を補修しました", "Building repaired", "建筑已修复"));
                    }
                }
            }
            Act::Demolish => {
                if let Some(bh) = self.sel_b {
                    if let Some(b) = self.world.bld(bh) {
                        if b.nation as usize == pl && (!b.kind.city() || !b.done()) {
                            let i = bh.idx as usize;
                            self.world.remove_building(i);
                            self.sel_b = None;
                        }
                    }
                }
            }
            Act::ResumeBuild => {
                if let Some(bh) = self.sel_b {
                    self.dispatch_worker_to_build(bh);
                }
            }
            Act::IdleWorker => {
                let idle: Vec<usize> = (0..self.world.units.len())
                    .filter(|&i| {
                        let u = &self.world.units[i];
                        u.on_map() && u.nation as usize == pl && u.kind == UnitKind::Worker && u.order == Order::Idle
                    })
                    .collect();
                if idle.is_empty() {
                    self.notice(t!("待機中の民はいない", "No idle workers", "没有闲置的民"));
                } else {
                    self.idle_cycle = (self.idle_cycle + 1) % idle.len();
                    let i = idle[self.idle_cycle];
                    let h = self.world.uh(i);
                    self.cam = self.world.units[i].pos;
                    self.set_sel(vec![h]);
                }
            }
        }
    }

    // ------------------------------------------------------------ 描画
    fn render(&mut self, dt: f32) {
        self.render_nopresent(dt);
        self.gfx.present();
    }

    fn render_nopresent(&mut self, dt: f32) {
        self.update_minimap(dt);
        self.gfx.begin([0.07, 0.06, 0.05, 1.0]);
        self.render_world();
        if self.gfx.d2d_begin() {
            self.art.ensure(&self.gfx);
            self.buttons.clear();
            self.render_world_labels();
            if self.screen == Screen::Playing {
                self.render_hero_labels();
                self.render_build_queue();
            }
            // スマートフォン向け画面では、縦に収まらない一覧は指でなぞってスクロールする
            let key = self.overlay_key();
            if key != self.ov_key {
                self.ov_key = key;
                self.ov_scroll = 0.0;
            }
            self.ov_max = 0.0;
            if self.screen == Screen::Title {
                self.render_title();
                if self.show_save {
                    self.scrolled(Self::render_save);
                }
            } else {
                self.render_hud();
                if self.show_diplo {
                    self.scrolled(Self::render_diplo);
                }
                if self.show_help {
                    self.scrolled(Self::render_help);
                }
                if self.show_chron {
                    self.scrolled(Self::render_chron);
                }
                if self.show_market {
                    self.scrolled(Self::render_market);
                }
                if self.show_summon {
                    self.scrolled(Self::render_summon);
                }
                if self.show_save {
                    self.scrolled(Self::render_save);
                }
                if self.peace_pending() {
                    self.render_peace_ask();
                }
                if self.world.game_over.is_some() {
                    self.render_gameover();
                }
                self.render_tutorial();
                self.render_tooltip();
            }
            self.gfx.d2d_end();
        }
    }

    fn render_world(&mut self) {
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        let z = self.zoom;
        let sx = 2.0 * z / w;
        let sy = -2.0 * z / h;
        self.gfx.set_xform(sx, sy, -self.cam.x * sx, -self.cam.y * sy);
        let place_tile = self.placement_tile();
        let fog_on = self.fog_active();
        let pl = self.world.player;
        let fog = &self.fog;
        let fog_at = move |p: V2| -> u8 {
            if !fog_on {
                return 2;
            }
            let (x, y) = p.tile();
            if Map::inb(x, y) {
                fog[Map::idx(x, y)]
            } else {
                0
            }
        };
        let g = &mut self.gfx;
        let mw = MAP_W as f32;
        let mh = MAP_H as f32;
        crate::art::draw_map_frame(g, mw, mh);
        g.draw_textured(&self.terrain_tex, 0.0, 0.0, mw, mh, [1.0; 4], true);
        g.draw_textured(&self.overlay_tex, 0.0, 0.0, mw, mh, [1.0; 4], true);
        let wd = &self.world;
        let view0 = v2(self.cam.x - w / 2.0 / z - 2.0, self.cam.y - h / 2.0 / z - 2.0);
        let view1 = v2(self.cam.x + w / 2.0 / z + 2.0, self.cam.y + h / 2.0 / z + 2.0);
        let inview = |p: V2| p.x >= view0.x && p.x <= view1.x && p.y >= view0.y && p.y <= view1.y;
        let px = 1.0 / z; // 1ピクセル
        // 森林の樹木・山岳の立体表現
        crate::art::draw_world_terrain_props(
            g,
            &wd.map,
            view0.x.floor() as i32,
            view0.y.floor() as i32,
            view1.x.ceil() as i32,
            view1.y.ceil() as i32,
            z,
            &fog_at,
        );
        // 銅山
        for m in wd.map.mines.iter() {
            if m.amount <= 0.0 {
                continue;
            }
            let c = v2(m.x as f32 + 0.5, m.y as f32 + 0.5);
            if !inview(c) || fog_at(c) == 0 {
                continue;
            }
            crate::art::draw_world_mine(g, c, m.amount, px);
        }
        // 建物
        for (bi, b) in wd.blds.iter().enumerate() {
            if !b.alive {
                continue;
            }
            let c = b.center();
            if !inview(c) || fog_at(c) == 0 {
                continue;
            }
            let nc = wd.nations[b.nation as usize].color;
            let (x0, y0, x1, y1) = (b.tx as f32, b.ty as f32, b.tx as f32 + 2.0, b.ty as f32 + 2.0);
            if wd.descent.is_some() && b.nation as usize == HEAVEN && b.kind.city() {
                let dr = if b.kind == BKind::Capital { 1.58 } else { 1.32 };
                g.circle(c.x, c.y + 0.08, dr, 24, [1.0, 0.92, 0.45, 0.25]);
                g.ring(c.x, c.y + 0.08, dr, (2.2 * px).max(0.06), 24, [1.0, 0.90, 0.42, 0.90]);
            }
            crate::art::draw_world_building(g, b, nc, px);
            let sel = self.sel_b == Some(wd.bh(bi));
            if sel {
                g.rect_outline(x0 - 0.32, y0 - 0.32, x1 + 0.32, y1 + 0.32, (2.2 * px).max(0.05), [1.0, 0.92, 0.55, 0.95]);
                if let Some(r) = b.rally {
                    g.line((c.x, c.y), (r.x, r.y), 1.5 * px, [1.0, 0.92, 0.55, 0.65]);
                    g.rect(r.x - 0.05, r.y - 0.65, r.x + 0.05, r.y, [0.92, 0.85, 0.65, 1.0]);
                    g.tri((r.x + 0.05, r.y - 0.65), (r.x + 0.48, r.y - 0.48), (r.x + 0.05, r.y - 0.32), c4(nc, 1.0));
                }
            }
        }
        // 隊商（商人・西域の商人・部族）
        for c in wd.trade.caravans.iter() {
            if !inview(c.pos) || fog_at(c.pos) != 2 {
                continue;
            }
            crate::game::market::draw_caravan(g, c, px);
        }
        // ユニット
        let era = wd.era;
        for u in wd.units.iter() {
            if !u.on_map() || !inview(u.pos) || (u.nation as usize != pl && fog_at(u.pos) != 2) {
                continue;
            }
            let nc = wd.nations[u.nation as usize].color;
            let ue = if (u.tribe as usize) < TRIBES.len() && TRIBES[u.tribe as usize].nomad { Era::Empire } else { era };
            if u.hero != NONE {
                g.circle(u.pos.x, u.pos.y + 0.15, 0.75, 16, [1.0, 0.85, 0.35, 0.35]);
                g.ring(u.pos.x, u.pos.y + 0.15, 0.75, (2.0 * px).max(0.05), 16, [1.0, 0.85, 0.35, 0.95]);
            } else if u.divine {
                g.circle(u.pos.x, u.pos.y + 0.12, 0.56, 16, [1.0, 0.94, 0.55, 0.28]);
                g.ring(u.pos.x, u.pos.y + 0.12, 0.56, (1.8 * px).max(0.045), 16, [1.0, 0.92, 0.48, 0.92]);
            }
            crate::art::draw_world_unit(g, u, nc, ue, &RES_COLORS, px);
        }
        // 矢
        for s in wd.shots.iter() {
            let t = 1.0 - s.t / 0.3;
            let p = s.a + (s.b - s.a) * t.clamp(0.0, 1.0);
            if fog_at(p) != 2 {
                continue;
            }
            let d = (s.b - s.a).norm();
            g.line((p.x - d.x * 0.34, p.y - d.y * 0.34), (p.x, p.y), (2.2 * px).max(0.045), [0.85, 0.72, 0.45, 0.95]);
            g.circle(p.x, p.y, (1.8 * px).max(0.04), 5, [1.0, 0.95, 0.65, 1.0]);
        }
        if fog_on {
            g.draw_textured(&self.fog_tex, 0.0, 0.0, mw, mh, [1.0; 4], true);
        }
        // 移動目標の印
        if let Some((p, t, atk)) = self.marker {
            let r = 0.2 + t * 0.6;
            let c = if atk { [1.0, 0.3, 0.2, t / 0.6] } else { [0.4, 1.0, 0.4, t / 0.6] };
            g.ring(p.x, p.y, r, 2.0 * px, 16, c);
        }
        if z < 2.2 {
            for &i in wd.saplings.keys() {
                let x = (i % MAP_W) as f32 + 0.5;
                let y = (i / MAP_W) as f32 + 0.5;
                if fog_at(v2(x, y)) == 2 {
                    g.circle(x, y, 0.24, 6, [0.33, 0.66, 0.25, 1.0]);
                }
            }
        }
        if self.mode == Mode::PlantForest {
            let (tx, ty) = place_tile;
            let ok = wd.can_plant_forest(wd.player, &self.sel, tx, ty).is_ok();
            let color = if ok { [0.3, 1.0, 0.3, 0.45] } else { [1.0, 0.25, 0.2, 0.45] };
            let (x, y) = (tx as f32, ty as f32);
            g.rect(x, y, x + 1.0, y + 1.0, color);
        }
        // 建設予定地
        if let Mode::Place(k) = self.mode {
            let (tx, ty) = place_tile;
            let ok = wd.can_place(wd.player, k, tx, ty).is_ok();
            let c = if ok { [0.3, 1.0, 0.3, 0.45] } else { [1.0, 0.25, 0.2, 0.45] };
            g.rect(tx as f32, ty as f32, tx as f32 + 2.0, ty as f32 + 2.0, c);
            if k.city() {
                g.ring(tx as f32 + 1.0, ty as f32 + 1.0, k.radius() as f32, 2.0 * px, 48, [1.0, 1.0, 1.0, 0.5]);
            }
        }
        g.flush();
        // 画面座標での選択矩形
        if let Some((sx, sy)) = self.ldown {
            let (mx, my) = self.mouse;
            if (sx - mx).abs() > 4.0 || (sy - my).abs() > 4.0 {
                self.gfx.set_screen_xform();
                let g = &mut self.gfx;
                g.rect(sx.min(mx), sy.min(my), sx.max(mx), sy.max(my), [0.4, 1.0, 0.4, 0.12]);
                g.rect_outline(sx.min(mx), sy.min(my), sx.max(mx), sy.max(my), 1.5, [0.4, 1.0, 0.4, 0.9]);
                g.flush();
            }
        }
    }

    /// 選んだ民の建設の順番（建設中＝1、予約＝2, 3, …）を建設地に示す
    fn render_build_queue(&mut self) {
        let pl = self.world.player;
        let mut order: Vec<(Handle, usize)> = Vec::new();
        for h in self.sel.iter() {
            let Some(u) = self.world.unit(*h) else { continue };
            if u.nation as usize != pl || u.kind != UnitKind::Worker || u.build_queue.is_empty() {
                continue;
            }
            let cur = if let Order::Build(b) = u.order { Some(b) } else { None };
            for (k, &b) in cur.iter().chain(u.build_queue.iter()).enumerate() {
                match order.iter_mut().find(|x| x.0 == b) {
                    Some(x) => x.1 = x.1.min(k + 1),
                    None => order.push((b, k + 1)),
                }
            }
        }
        for (b, k) in order {
            let Some(bd) = self.world.bld(b) else { continue };
            if bd.done() {
                continue;
            }
            let (sx, sy) = self.world_to_screen(bd.center());
            let r = 11.0;
            self.gfx.fill_rect(sx - r, sy - r - 18.0, sx + r, sy + r - 18.0, [0.12, 0.10, 0.06, 0.85]);
            self.gfx.fill_rect(sx - r, sy + r - 20.0, sx + r, sy + r - 18.0, [1.0, 0.8, 0.35, 1.0]);
            let s = format!("{}", k);
            let tw = self.gfx.measure(&s, Font::Normal);
            self.gfx.text(&s, sx - tw / 2.0, sy - r - 18.0, tw + 4.0, 2.0 * r, Font::Normal, [1.0, 0.88, 0.5, 1.0]);
        }
    }

    fn render_world_labels(&mut self) {
        let z = self.zoom;
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        let fog_on = self.fog_active();
        let pl = self.world.player;

        enum WorldItem {
            Planting { sx: f32, sy: f32, size: f32, stage: usize },
            Mountain { sx: f32, sy: f32, r: f32, flip_x: bool, fogged: bool },
            Forest { sx: f32, sy: f32, r: f32, flip_x: bool, fogged: bool },
            Mine { sx: f32, sy: f32, r: f32, fogged: bool },
            Building {
                sx: f32,
                sy: f32,
                r: f32,
                kind: BKind,
                origin: usize,
                nc: [f32; 3],
                done: bool,
                progress: f32,
                hp_frac: Option<f32>,
                fogged: bool,
                divine: bool,
            },
            Unit {
                sx: f32,
                sy: f32,
                kind: UnitKind,
                nation: usize,
                ut: u8,
                pw: f32,
                nc: [f32; 3],
                carry: Option<usize>,
                flip_x: bool,
                hp_frac: Option<f32>,
                hero: u8,
                divine: bool,
            },
        }

        if z >= 2.2 {
            let mut items: Vec<(f32, WorldItem)> = Vec::new();
            let view0 = v2(self.cam.x - w / 2.0 / z - 2.5, self.cam.y - h / 2.0 / z - 2.5);
            let view1 = v2(self.cam.x + w / 2.0 / z + 2.5, self.cam.y + h / 2.0 / z + 2.5);
            let x0 = (view0.x.floor() as i32).max(0);
            let y0 = (view0.y.floor() as i32).max(0);
            let x1 = (view1.x.ceil() as i32).min(MAP_W as i32 - 1);
            let y1 = (view1.y.ceil() as i32).min(MAP_H as i32 - 1);

            for (&i, &ready_at) in &self.world.saplings {
                let x = (i % MAP_W) as f32 + 0.5;
                let y = (i / MAP_W) as f32 + 0.5;
                if x < view0.x || x > view1.x || y < view0.y || y > view1.y || (fog_on && self.fog[i] != 2) {
                    continue;
                }
                let elapsed = FOREST_GROWTH_TIME - (ready_at - self.world.time);
                let stage = ((elapsed * 3.0 / FOREST_GROWTH_TIME) as usize).min(2);
                let size = (z * [0.66, 1.0, 1.44][stage]).clamp(4.0, 72.0);
                let (sx, sy) = self.world_to_screen(v2(x, y));
                items.push((sy, WorldItem::Planting { sx, sy, size, stage }));
            }

            // 山岳・森林のAI生成スプライト
            let thin = z < 8.5;
            for ty in y0..=y1 {
                for tx in x0..=x1 {
                    let idx = Map::idx(tx, ty);
                    let t = self.world.map.terrain[idx];
                    if !matches!(t, Terrain::Mountain | Terrain::Forest) {
                        continue;
                    }
                    if z < 5.5 {
                        if ((tx * 3 + ty * 5) & 3) != 0 {
                            continue;
                        }
                    } else if thin && ((tx + ty) & 1) != 0 {
                        continue;
                    }
                    let f = if fog_on { self.fog[idx] } else { 2 };
                    if f == 0 {
                        continue;
                    }
                    let fogged = f == 1;
                    let h1 = crate::rng::hash2(tx, ty, 31);
                    let h2 = crate::rng::hash2(tx, ty, 47);
                    let flip_x = h1 > 0.5;
                    let ox = (h1 - 0.5) * 0.28;
                    let oy = (h2 - 0.5) * 0.24;
                    let wp = v2(tx as f32 + 0.5 + ox, ty as f32 + 0.5 + oy);
                    let (sx, sy) = self.world_to_screen(wp);
                    if sx < -64.0 || sy < -64.0 || sx > w + 64.0 || sy > h + 64.0 {
                        continue;
                    }
                    match t {
                        Terrain::Mountain => {
                            let mult = if z < 5.5 { 1.65 } else if thin { 1.25 } else { 0.92 + 0.22 * h2 };
                            let r = (z * mult).clamp(5.5, 44.0);
                            items.push((sy + r * 0.25, WorldItem::Mountain { sx, sy, r, flip_x, fogged }));
                        }
                        Terrain::Forest => {
                            let mult = if z < 5.5 { 1.45 } else if thin { 1.12 } else { 0.80 + 0.20 * h2 };
                            let r = (z * mult).clamp(5.0, 38.0);
                            items.push((sy + r * 0.22, WorldItem::Forest { sx, sy, r, flip_x, fogged }));
                        }
                        _ => {}
                    }
                }
            }

            // 銅山のAI生成スプライト
            for m in self.world.map.mines.iter() {
                if m.amount <= 0.0 {
                    continue;
                }
                let wp = v2(m.x as f32 + 0.5, m.y as f32 + 0.5);
                if !self.explored(wp) {
                    continue;
                }
                let (sx, sy) = self.world_to_screen(wp);
                if sx < -64.0 || sy < -64.0 || sx > w + 64.0 || sy > h + 64.0 {
                    continue;
                }
                let fogged = fog_on && !self.seen(wp);
                let s = 0.78 + 0.24 * (m.amount / 2500.0).clamp(0.2, 1.0);
                let r = (z * s).clamp(6.5, 38.0);
                items.push((sy + r * 0.28, WorldItem::Mine { sx, sy, r, fogged }));
            }

            // 建物（都・邑・兵営・田）のAI生成スプライト
            for (bi, b) in self.world.blds.iter().enumerate() {
                if !b.alive {
                    continue;
                }
                let c = b.center();
                if !self.explored(c) {
                    continue;
                }
                let (sx, sy) = self.world_to_screen(c);
                if sx < -96.0 || sy < -96.0 || sx > w + 96.0 || sy > h + 96.0 {
                    continue;
                }
                let fogged = fog_on && !self.seen(c);
                let nc = self.world.nations[b.nation as usize].color;
                let mult = match b.kind {
                    BKind::Capital => if z < 5.5 { 2.25 } else { 1.62 },
                    BKind::Town => if z < 5.5 { 1.75 } else { 1.38 },
                    BKind::Barracks => 1.25,
                    BKind::Farm => 1.16,
                    BKind::Fort => 1.3,
                    BKind::Port => 1.3,
                    BKind::Village => 1.2,
                };
                let r = (z * mult).clamp(9.0, 68.0);
                let sel = self.sel_b == Some(self.world.bh(bi));
                let hp_frac = if b.done() && (b.hp < b.kind.max_hp() - 0.5 || sel) {
                    Some((b.hp / b.kind.max_hp()).clamp(0.0, 1.0))
                } else {
                    None
                };
                let sort_y = if b.kind == BKind::Farm { sy - r * 0.2 } else { sy + r * 0.45 };
                let divine = self.world.descent.is_some() && b.nation as usize == HEAVEN && b.kind.city();
                items.push((
                    sort_y,
                    WorldItem::Building {
                        sx,
                        sy,
                        r,
                        kind: b.kind,
                        origin: b.origin as usize,
                        nc,
                        done: b.done(),
                        progress: b.progress,
                        hp_frac,
                        fogged,
                        divine,
                    },
                ));
            }

            // ユニットの高精細AI生成スプライト（マップや建物に対して適切なサイズに縮小）
            if z >= 6.5 && self.screen == Screen::Playing {
                for u in self.world.units.iter() {
                    if !u.on_map() || (u.nation as usize != pl && !self.seen(u.pos)) {
                        continue;
                    }
                    let (sx, sy) = self.world_to_screen(u.pos);
                    if sx < -48.0 || sy < -48.0 || sx > w + 48.0 || sy > h + 48.0 {
                        continue;
                    }
                    let nc = self.world.nations[u.nation as usize].color;
                    let carry = if u.carry > 0.5 { Some(u.carry_kind) } else { None };
                    let flip_x = u.dir.x < -0.05;
                    let mhp = u.max_hp();
                    let hp_frac = if u.hp < mhp - 0.5 || u.selected {
                        Some((u.hp / mhp).clamp(0.0, 1.0))
                    } else {
                        None
                    };
                    items.push((
                        sy + 2.0,
                        WorldItem::Unit {
                            sx,
                            sy,
                            kind: u.kind,
                            nation: u.nation as usize,
                            ut: u.ut,
                            pw: u.power,
                            nc,
                            carry,
                            flip_x,
                            hp_frac,
                            hero: u.hero,
                            divine: u.divine,
                        },
                    ));
                }
            }

            items.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

            for (_, item) in items {
                match item {
                    WorldItem::Mountain { sx, sy, r, flip_x, fogged } => {
                        if let Some(bmp) = self.art.mountain_sprite(flip_x, fogged) {
                            self.gfx.draw_bitmap(bmp, sx - r, sy - r * 1.18, sx + r, sy + r * 0.82);
                        }
                    }
                    WorldItem::Planting { sx, sy, size, stage } => {
                        if let Some(bmp) = self.art.planting_sprite(stage) {
                            self.gfx.draw_bitmap(bmp, sx - size * 0.5, sy - size * 0.9, sx + size * 0.5, sy + size * 0.1);
                        }
                    }
                    WorldItem::Forest { sx, sy, r, flip_x, fogged } => {
                        if let Some(bmp) = self.art.forest_sprite(flip_x, fogged) {
                            self.gfx.draw_bitmap(bmp, sx - r, sy - r * 1.12, sx + r, sy + r * 0.88);
                        }
                    }
                    WorldItem::Mine { sx, sy, r, fogged } => {
                        if let Some(bmp) = self.art.mine_sprite(fogged) {
                            self.gfx.draw_bitmap(bmp, sx - r, sy - r * 1.05, sx + r, sy + r * 0.95);
                        }
                    }
                    WorldItem::Building { sx, sy, r, kind, origin, nc, done, progress, hp_frac, fogged, divine } => {
                        let alpha = if done { 1.0 } else { 0.65 };
                        let b_bmp = if divine {
                            self.art.divine_bld_sprite(kind, fogged)
                        } else {
                            self.art.national_building(origin, kind, fogged)
                        };
                        if let Some(bmp) = b_bmp {
                            self.gfx.draw_bitmap_alpha(bmp, sx - r, sy - r * 1.08, sx + r, sy + r * 0.92, alpha);
                        }
                        // 建物右下の国色紋章バッジ（どの国の城邑・兵営か一目で判別可能に）
                        if kind != BKind::Farm && z >= 7.0 {
                            let br = (r * 0.16).clamp(3.2, 6.5);
                            let bx = sx + r * 0.56;
                            let by = sy + r * 0.48;
                            let k = if fogged { 0.55 } else { 1.0 };
                            self.gfx.fill_circle(bx, by, br + 1.2, [0.06, 0.05, 0.04, 0.92]);
                            if divine {
                                self.gfx.fill_circle(bx, by, br, [1.0 * k, 0.90 * k, 0.42 * k, 1.0]);
                                self.gfx.stroke_circle(bx, by, br + 0.6, 1.3, [0.95 * k, 0.98 * k, 1.0 * k, 0.98]);
                            } else {
                                self.gfx.fill_circle(bx, by, br, [nc[0] * k, nc[1] * k, nc[2] * k, 1.0]);
                                self.gfx.stroke_circle(bx, by, br + 0.5, 1.1, [0.95 * k, 0.82 * k, 0.38 * k, 0.95]);
                            }
                        }
                        if !done {
                            let bw = (r * 1.4).max(16.0);
                            let by = sy + r * 0.72;
                            self.gfx.fill_round(sx - bw * 0.5, by, sx + bw * 0.5, by + 3.5, 1.5, [0.08, 0.06, 0.04, 0.68]);
                            if progress > 0.01 {
                                self.gfx.fill_round(sx - bw * 0.5, by, sx - bw * 0.5 + bw * progress, by + 3.5, 1.5, [0.35, 0.78, 1.0, 0.95]);
                            }
                            self.gfx.stroke_round(sx - bw * 0.5, by, sx + bw * 0.5, by + 3.5, 1.5, 0.8, [0.78, 0.64, 0.34, 0.75]);
                        } else if let Some(f) = hp_frac {
                            let bw = (r * 1.4).max(16.0);
                            let by = sy - r * 1.02;
                            self.gfx.fill_round(sx - bw * 0.5, by, sx + bw * 0.5, by + 3.5, 1.5, [0.08, 0.06, 0.04, 0.75]);
                            self.gfx.fill_round(sx - bw * 0.5, by, sx - bw * 0.5 + bw * f, by + 3.5, 1.5, [1.0 - f, 0.22 + 0.75 * f, 0.15, 0.95]);
                        }
                    }
                    WorldItem::Unit { sx, sy, kind, nation, ut, pw, nc, carry, flip_x, hp_frac, hero, divine } => {
                        let is_hero = (hero as usize) < HEROES.len();
                        let scale = if is_hero {
                            if kind == UnitKind::Cavalry { 0.88 } else { 0.78 }
                        } else if divine {
                            if kind == UnitKind::Cavalry { 0.82 } else { 0.70 }
                        } else if kind == UnitKind::Cavalry {
                            0.76
                        } else {
                            0.62
                        };
                        let max_r = if is_hero { 27.0 } else if divine { 24.0 } else { 22.0 };
                        let r = (z * scale * (1.0 + 0.04 * (pw - 1.0).min(4.0))).clamp(7.5, max_r);
                        let u_bmp = if is_hero {
                            self.art.period_hero(hero as usize, flip_x)
                        } else if divine {
                            self.art.divine_unit_sprite(kind, flip_x)
                        } else {
                            self.art.national_unit(nation, ut, flip_x)
                        };
                        if let Some(bmp) = u_bmp {
                            self.gfx.draw_bitmap(bmp, sx - r, sy - r * 1.35, sx + r, sy + r * 0.65);
                        }
                        // 足元右下の国色小紋章（天の軍は神々しい白金紋章）
                        let pr = (r * 0.19).clamp(2.2, 3.8);
                        let px0 = sx + r * 0.42;
                        let py0 = sy + r * 0.32;
                        self.gfx.fill_circle(px0, py0, pr + 0.9, [0.08, 0.06, 0.04, 0.92]);
                        if divine {
                            self.gfx.fill_circle(px0, py0, pr, [1.0, 0.92, 0.48, 1.0]);
                            self.gfx.stroke_circle(px0, py0, pr + 0.5, 0.9, [0.92, 0.98, 1.0, 0.95]);
                        } else {
                            self.gfx.fill_circle(px0, py0, pr, c4(nc, 1.0));
                            self.gfx.stroke_circle(px0, py0, pr + 0.4, 0.8, [0.95, 0.82, 0.42, 0.9]);
                        }
                        // 資源運搬中はミニ資源アイコンを左上に表示
                        if let Some(ck) = carry {
                            if z >= 11.0 {
                                let cr = (r * 0.38).clamp(3.8, 6.2);
                                let cx = sx - r * 0.55;
                                let cy = sy - r * 0.65;
                                self.gfx.fill_circle(cx, cy, cr + 0.8, [0.08, 0.06, 0.04, 0.85]);
                                if let Some(rbmp) = &self.art.res_icons[ck] {
                                    self.gfx.draw_bitmap(rbmp, cx - cr, cy - cr, cx + cr, cy + cr);
                                }
                            }
                        }
                        // 強化済みユニットは金の星（ランクピップ）を右下に表示
                        if pw > 1.01 && z >= 12.0 {
                            let stars = ((pw - 1.0) / 0.5).round().clamp(1.0, 6.0) as i32;
                            for s in 0..stars {
                                let px = sx + r * 0.44 - s as f32 * 3.8;
                                let py = sy + r * 0.48;
                                self.gfx.fill_circle(px, py, 2.3, [0.12, 0.08, 0.04, 0.95]);
                                self.gfx.fill_circle(px, py, 1.5, [1.0, 0.88, 0.32, 1.0]);
                            }
                        }
                        if let Some(f) = hp_frac {
                            let bw = (r * 1.4).max(11.0);
                            let by = sy - r * 1.40;
                            self.gfx.fill_rect(sx - bw * 0.5, by, sx + bw * 0.5, by + 2.8, [0.0, 0.0, 0.0, 0.8]);
                            self.gfx.fill_rect(sx - bw * 0.5, by, sx - bw * 0.5 + bw * f, by + 2.8, [1.0 - f, 0.25 + 0.7 * f, 0.15, 1.0]);
                        }
                    }
                }
            }
        }
        // 城邑名（Age of Empires 風の銘板バナー付き）
        let mut labels = Vec::new();
        for (bi, b) in self.world.blds.iter().enumerate() {
            if !b.alive || !b.kind.city() || !self.explored(b.center()) {
                continue;
            }
            let (sx, sy) = self.world_to_screen(b.center());
            if sx < -100.0 || sy < -50.0 || sx > w + 100.0 || sy > h + 50.0 {
                continue;
            }
            let n = &self.world.nations[b.nation as usize];
            let cap = b.kind == BKind::Capital;
            if z < 6.0 && !cap {
                continue;
            }
            let divine = self.world.descent.is_some() && b.nation as usize == HEAVEN;
            let name = if divine {
                t!("天の裁き", "Judgement", "天之审判").to_string()
            } else {
                self.world.town_name(bi)
            };
            let col = if divine { [1.0, 0.88, 0.38] } else { n.color };
            labels.push((sx, sy, name, cap, col, divine));
        }
        for (sx, sy, name, cap, c, divine) in labels {
            let f = if cap && z >= 9.0 { Font::BigCenter } else { Font::NormalCenter };
            let off = if z >= 9.0 { 1.58 * z + 14.0 } else { 1.25 * z + 6.0 };
            let ty = sy - off - 14.0;
            if z >= 8.0 {
                let bw = (self.gfx.measure(&name, if cap && z >= 9.0 { Font::Big } else { Font::Normal }) + 16.0).max(48.0);
                let bh = if cap { 24.0 } else { 20.0 };
                let bx0 = sx - bw * 0.5;
                let by0 = ty + (28.0 - bh) * 0.5;
                self.gfx.fill_round(bx0, by0, bx0 + bw, by0 + bh, 4.0, [0.08, 0.06, 0.04, 0.85]);
                self.gfx.stroke_round(bx0, by0, bx0 + bw, by0 + bh, 4.0, if divine { 1.6 } else { 1.2 }, [c[0] * 0.7 + 0.25, c[1] * 0.7 + 0.25, c[2] * 0.7 + 0.2, 0.95]);
            }
            let col = [c[0] * 0.45 + 0.55, c[1] * 0.45 + 0.55, c[2] * 0.45 + 0.55, 1.0];
            self.gfx.text_shadow(&name, sx - 110.0, ty, 220.0, 28.0, f, col);
        }
    }

    fn act_cmd_icon(&self, act: Act) -> Option<CmdIcon> {
        match act {
            Act::Train(ut) => Some(match udef(ut).class {
                UnitKind::Worker => CmdIcon::Worker,
                UnitKind::Infantry => CmdIcon::Infantry,
                UnitKind::Archer => CmdIcon::Archer,
                UnitKind::Cavalry => CmdIcon::Cavalry,
                UnitKind::Ship => CmdIcon::Ship,
            }),
            Act::SelectAllWorkers | Act::SelectKind(UnitKind::Worker) | Act::FilterKind(UnitKind::Worker) | Act::ResumeBuild => Some(CmdIcon::Worker),
            Act::SelectKind(UnitKind::Infantry) | Act::FilterKind(UnitKind::Infantry) => Some(CmdIcon::Infantry),
            Act::SelectKind(UnitKind::Archer) | Act::FilterKind(UnitKind::Archer) => Some(CmdIcon::Archer),
            Act::SelectKind(UnitKind::Cavalry) | Act::FilterKind(UnitKind::Cavalry) => Some(CmdIcon::Cavalry),
            Act::SelectKind(UnitKind::Ship) | Act::FilterKind(UnitKind::Ship) | Act::Unload => Some(CmdIcon::Ship),
            Act::Place(BKind::Port) => Some(CmdIcon::Port),
            Act::Raze => Some(CmdIcon::Demolish),
            Act::SelectAllMilitary => Some(CmdIcon::AllArmy),
            Act::WorkersAuto | Act::AllWorkersAuto => Some(CmdIcon::GatherAuto),
            Act::AllWorkersWood => Some(CmdIcon::GatherWood),
            Act::AllWorkersMine => Some(CmdIcon::GatherBronze),
            Act::Place(BKind::Farm) => Some(CmdIcon::Farm),
            Act::Place(BKind::Barracks) => Some(CmdIcon::Barracks),
            Act::Place(BKind::Town) | Act::Place(BKind::Village) => Some(CmdIcon::Town),
            Act::Place(BKind::Capital) => Some(CmdIcon::Capital),
            Act::Place(BKind::Fort) => Some(CmdIcon::Fort),
            Act::Upgrade | Act::AllWorkersUpgrade | Act::AllArmyUpgrade => Some(CmdIcon::Upgrade),
            Act::Convert => Some(CmdIcon::Convert),
            Act::Stop => Some(CmdIcon::Stop),
            Act::AttackMove | Act::AllArmyAttack => Some(CmdIcon::AttackMove),
            Act::ArmyDefend | Act::AllArmyDefend => Some(CmdIcon::Defend),
            Act::Demolish => Some(CmdIcon::Demolish),
            Act::Diplo => Some(CmdIcon::Diplo),
            Act::Help => Some(CmdIcon::Help),
            _ => None,
        }
    }

    fn add_button(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, label: &str, sub: &str, key: u32, enabled: bool, act: Act, tip: &str) {
        let (mx, my) = self.mouse;
        let hover = mx >= x0 && mx <= x1 && my >= y0 && my <= y1;
        let btn_bmp = if !enabled {
            self.art.btn_dis.as_ref()
        } else if hover {
            self.art.btn_hover.as_ref()
        } else {
            self.art.btn_norm.as_ref()
        };
        if let Some(bmp) = btn_bmp {
            self.gfx.draw_bitmap(bmp, x0, y0, x1, y1);
        } else {
            let bg = if !enabled {
                [0.18, 0.17, 0.15, 0.9]
            } else if hover {
                [0.45, 0.36, 0.20, 0.95]
            } else {
                [0.30, 0.24, 0.15, 0.95]
            };
            self.gfx.fill_round(x0, y0, x1, y1, 4.0, bg);
            self.gfx.stroke_round(x0, y0, x1, y1, 4.0, 1.0, [0.75, 0.62, 0.35, if enabled { 0.9 } else { 0.4 }]);
        }

        let bh = y1 - y0;
        let mut tx0 = x0;
        if let Some(ci) = self.act_cmd_icon(act) {
            let ibmp = match act {
                Act::Train(ut) => self.art.national_unit(self.world.player, ut, false),
                Act::Place(k) => self.art.national_building(self.world.player, k, false),
                _ => self.art.cmd_icons[ci as usize].as_ref(),
            };
            if let Some(ibmp) = ibmp {
                if bh >= 36.0 && !(self.gfx.compact && y1 <= self.top_h()) {
                    let isz = (bh - 10.0).min(34.0);
                    let iy = y0 + (bh - isz) * 0.5;
                    self.gfx.draw_bitmap(ibmp, x0 + 3.0, iy, x0 + 3.0 + isz, iy + isz);
                    if !enabled {
                        self.gfx.fill_rect(x0 + 3.0, iy, x0 + 3.0 + isz, iy + isz, [0.05, 0.05, 0.05, 0.55]);
                    }
                    tx0 = x0 + isz + 4.0;
                } else if x1 - x0 >= 68.0 {
                    let isz = (bh - 6.0).max(14.0);
                    self.gfx.draw_bitmap(ibmp, x0 + 4.0, y0 + 3.0, x0 + 4.0 + isz, y0 + 3.0 + isz);
                    tx0 = x0 + isz + 4.0;
                }
            }
        }

        let tc = if enabled { [1.0, 0.95, 0.85, 1.0] } else { [0.55, 0.52, 0.48, 1.0] };
        let tw = (x1 - tx0).max(10.0);
        if sub.is_empty() {
            self.gfx.text_shadow(label, tx0, y0, tw, bh, Font::NormalCenter, tc);
        } else {
            self.gfx.text_shadow(label, tx0 + 2.0, y0 + 2.0, tw + 24.0, bh * 0.52, Font::Normal, tc);
            // 小見出しが入りきらなければ、アイコンの下まで左へ寄せる
            let sw = self.gfx.measure(sub, Font::Small);
            let sx = (x1 - 4.0 - sw).min(tx0 + 2.0).max(x0 + 4.0);
            self.gfx.text(sub, sx, y0 + bh * 0.53, x1 - sx, bh * 0.45, Font::Small, [tc[0] * 0.92, tc[1] * 0.86, tc[2] * 0.68, 1.0]);
        }
        if key >= 0x41 && key <= 0x5A {
            let ks = ((key as u8) as char).to_string();
            self.gfx.fill_round(x0 + 2.0, y0 + 2.0, x0 + 15.0, y0 + 15.0, 2.0, [0.08, 0.06, 0.04, 0.84]);
            self.gfx.text(&ks, x0 + 4.0, y0 + 1.0, 11.0, 14.0, Font::Small, [1.0, 0.88, 0.45, 0.98]);
        }
        if hover && !tip.is_empty() {
            self.notice = None;
        }
        self.buttons.push(Button { x0, y0, x1, y1, label: label.to_string(), key, enabled, act, tip: tip.to_string() });
    }

    fn cost_str(c: [f32; 3]) -> String {
        let cnt = c.iter().filter(|&&v| v > 0.0).count();
        let sep = if cnt >= 3 && crate::lang::lang() != crate::lang::Lang::En { "" } else { " " };
        let mut s = String::new();
        for k in 0..3 {
            if c[k] > 0.0 {
                if !s.is_empty() {
                    s.push_str(sep);
                }
                s.push_str(&format!("{}{}", res_short(k), c[k] as i32));
            }
        }
        s
    }

    fn render_title(&mut self) {
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        // 国都マーカーの強調
        let hov = {
            let p = self.screen_to_world(self.mouse.0, self.mouse.1);
            let mut best: Option<(f32, usize)> = None;
            for (i, n) in self.world.nations.iter().enumerate() {
                if n.alive && n.home.dist(p) < 10.0 && best.map_or(true, |b| n.home.dist(p) < b.0) {
                    best = Some((n.home.dist(p), i));
                }
            }
            best.map(|b| b.1)
        };
        for (i, n) in self.world.nations.iter().enumerate() {
            if !n.alive {
                continue;
            }
            let (sx, sy) = self.world_to_screen(n.home);
            let seven = NATIONS[i].major;
            if Some(i) == self.title_sel || Some(i) == hov {
                self.gfx.fill_circle(sx, sy, 15.0, [1.0, 0.92, 0.55, 0.38]);
                self.gfx.stroke_circle(sx, sy, 15.0, 2.0, [1.0, 0.88, 0.40, 0.9]);
            }
            if seven {
                self.gfx.fill_circle(sx, sy, 4.5, [1.0, 0.85, 0.3, 1.0]);
            }
        }
        let px = w - 380.0;
        self.art.draw_stone_panel(&self.gfx, px, 0.0, w, h);
        if let Some(scene) = &self.art.title_scene {
            self.gfx.draw_bitmap(scene, px + 4.0, 4.0, w - 4.0, 308.0);
            self.gfx.fill_rect(px + 4.0, 4.0, w - 4.0, 308.0, [0.025, 0.03, 0.05, 0.64]);
        }

        self.art.draw_corner_ornaments(&self.gfx, px + 4.0, 4.0, w - 4.0, h - 4.0, 18.0);
        // 英語の題字は長いので、扁額を広げた飾りを使う
        let wide = crate::lang::lang() == crate::lang::Lang::En;
        let crest = if wide { &self.art.title_crest_wide } else { &self.art.title_crest };
        if let Some(crest) = crest {
            self.gfx.draw_bitmap(crest, px + 20.0, 10.0, w - 20.0, 126.0);
        }
        let title = t!("英仏百年戦争", "Hundred Years' War", "英法百年战争");
        let plaque = if wide { crate::art::TITLE_PLAQUE_W_WIDE } else { crate::art::TITLE_PLAQUE_W };
        // 扁額（両端に少し余白）に収まらなければ一回り小さい書体で書く
        let tf = if self.gfx.measure(title, Font::Title) <= plaque - 8.0 { Font::Title } else { Font::TitleMid };
        self.gfx.text_shadow(title, px, 19.0, 380.0, 56.0, tf, [1.0, 0.86, 0.42, 1.0]);
        self.gfx.text_shadow(t!("— 二つの王冠 —", "— Two Crowns —", "— 两顶王冠 —"), px, 76.0, 380.0, 28.0, Font::BigCenter, [0.96, 0.90, 0.78, 1.0]);
        self.gfx.text("1337 — 1453  /  FRANCE & ENGLAND", px, 110.0, 380.0, 18.0, Font::NormalCenter, [0.58, 0.73, 0.77, 1.0]);

        let compact = self.gfx.compact;
        if !compact || self.title_sel.is_none() {
        self.art.draw_inset_box(&self.gfx, px + 18.0, 132.0, w - 18.0, 306.0);
        self.gfx.text(
            t!("1337年、フランス王フィリップ6世はギュイエンヌを没収し、イングランド王エドワード3世はフランス王位を求めた。諸侯は寝返り、スコットランドは北から攻め、黒死病が両国を襲う。\n\n地図上の都（ロンドンかパリ）をクリックして、率いる王国を選べ。1453年が暮れても決着がつかなければ、ジャンヌ・ダルクの亡霊が両国を滅ぼす。", "1337: Philip VI confiscates Guyenne and Edward III claims the French crown. Lords change sides, the Scots raid from the north and the Black Death strikes.\n\nClick London or Paris to choose your kingdom. If the war is still undecided after 1453, the ghost of Joan of Arc destroys both.", "1337年，法王腓力六世没收吉耶讷，英王爱德华三世要求法兰西王位。诸侯倒戈，苏格兰自北来攻，黑死病席卷两国。\n\n点击地图上的都城（伦敦或巴黎），选择你要率领的王国。若1453年过后仍未分出胜负，贞德的亡灵将毁灭两国。"),
            px + 28.0,
            140.0,
            324.0,
            162.0,
            Font::Normal,
            [0.90, 0.86, 0.78, 1.0],
        );
        }
        if self.title_sel.is_none() && !compact {
            if let Some(scene) = &self.art.title_scene {
                self.gfx.draw_bitmap(scene, px + 20.0, 322.0, w - 20.0, 526.0);
                self.gfx.fill_rect(px + 20.0, 482.0, w - 20.0, 526.0, [0.03, 0.035, 0.05, 0.82]);
                self.gfx.text(t!("二つの王冠、一つの王位", "Two crowns. One throne.", "两顶王冠，一个王位"), px + 26.0, 488.0, 328.0, 28.0, Font::BigCenter, [0.91, 0.82, 0.62, 1.0]);
                self.art.draw_corner_ornaments(&self.gfx, px + 19.0, 321.0, w - 19.0, 527.0, 12.0);
            }
        }
        if let Some(i) = self.title_sel {
            let n = &self.world.nations[i];
            let d = &NATIONS[i];
            // スマートフォンでは説明文の欄を隠して、国の情報をその位置に詰める
            let oy = if compact { 132.0 - 316.0 } else { 0.0 };
            self.art.draw_inset_box(&self.gfx, px + 18.0, oy + 316.0, w - 18.0, oy + 522.0);
            self.art.draw_corner_ornaments(&self.gfx, px + 20.0, oy + 318.0, w - 20.0, oy + 520.0, 12.0);
            if let Some(cap_ic) = self.art.national_building(self.title_sel.unwrap_or(self.world.player), BKind::Capital, false) {
                self.gfx.draw_bitmap(cap_ic, px + 30.0, oy + 328.0, px + 78.0, oy + 376.0);
            }
            self.gfx.fill_rect(px + 86.0, oy + 336.0, px + 104.0, oy + 368.0, c4(n.color, 1.0));
            self.gfx.stroke_rect(px + 86.0, oy + 336.0, px + 104.0, oy + 368.0, 1.2, [0.92, 0.78, 0.40, 1.0]);
            self.gfx.text_shadow(crate::lang::tx(n.name), px + 112.0, oy + 332.0, 230.0, 40.0, Font::Big, [1.0, 0.95, 0.85, 1.0]);
            self.gfx.text(crate::lang::tx(d.desc), px + 32.0, oy + 382.0, 316.0, 44.0, Font::Normal, [0.88, 0.84, 0.76, 1.0]);
            let s = tf!("生産力：×{:.2}{}", "Production: ×{:.2}{}", "生产力：×{:.2}{}", d.bias, if d.major { t!("（大国）", " (great power)", "（大国）") } else { "" });
            self.gfx.text(&s, px + 32.0, oy + 426.0, 316.0, 20.0, Font::Normal, [0.98, 0.86, 0.48, 1.0]);
            let cities = self.world.blds.iter().filter(|b| b.alive && b.nation as usize == i && b.kind.city()).count();
            let s = tf!("街：{}　王家：{}", "Towns: {}   House: {}", "城镇：{}　王室：{}", cities, crate::lang::tx(INITIAL_HOUSE[i]));
            self.gfx.text(&s, px + 32.0, oy + 448.0, 316.0, 20.0, Font::Normal, [0.88, 0.84, 0.76, 1.0]);
            let diff = if i == FRANCE {
                t!("勝利：1453年のうちにイングランドを大陸から追い出す", "Victory: drive England off the Continent within 1453", "胜利：1453年之内将英格兰逐出大陆")
            } else {
                t!("勝利：1453年のうちにフランスのすべての街を取る", "Victory: take every French town within 1453", "胜利：1453年之内夺取法兰西全部城镇")
            };
            self.gfx.text(diff, px + 32.0, oy + 470.0, 316.0, 20.0, Font::Normal, [0.88, 0.84, 0.76, 1.0]);
            // 史実に基づく国の特徴（説明はツールチップ）
            let trs: Vec<&Trait> = all_traits_of(i).collect();
            if !trs.is_empty() {
                let names: Vec<String> = trs
                    .iter()
                    .map(|t| if t.from_ad > START_AD as i32 { tf!("{}（{}年〜）", "{} (from {})", "{}（{}年起）", crate::lang::tx(t.name), t.from_ad) } else if t.to_ad < END_AD as i32 { tf!("{}（〜{}年）", "{} (until {})", "{}（至{}年）", crate::lang::tx(t.name), t.to_ad) } else { crate::lang::tx(t.name).to_string() })
                    .collect();
                let s = tf!("特徴：{}", "Traits: {}", "特点：{}", names.join(sep()));
                self.gfx.text(&s, px + 32.0, oy + 492.0, 316.0, 22.0, Font::Normal, [0.70, 0.92, 1.0, 1.0]);
                let tip: Vec<String> = trs.iter().map(|t| format!("【{}】{}", crate::lang::tx(t.name), crate::lang::tx(t.desc))).collect();
                self.buttons.push(Button {
                    x0: px + 32.0,
                    y0: oy + 490.0,
                    x1: w - 30.0,
                    y1: oy + 514.0,
                    label: String::new(),
                    key: 0,
                    enabled: true,
                    act: Act::Noop,
                    tip: tip.join("\n"),
                });
            }
            let sy = if compact { (oy + 528.0).min(h - 138.0) } else { 530.0 };
            self.add_button(px + 50.0, sy, w - 50.0, sy + if compact { 40.0 } else { 46.0 }, t!("この王国で戦を始める", "Begin the war", "以此王国开战"), "", 0, true, Act::Start, "");
        }
        if compact {
            self.render_title_buttons_compact(px);
            return;
        }
        let ty = (h - 128.0).max(592.0);
        // 記録から続ける（空きがあれば独立した行、狭ければ英霊モードの行と並べる）
        let cy = ty - 108.0;
        let heroic_x0 = if self.title_sel.is_some() && cy < 590.0 {
            let mid = px + 24.0 + (w - 48.0 - px) * 0.5;
            self.add_button(mid + 4.0, ty - 54.0, w - 24.0, ty - 12.0, t!("続きから F5", "Continue F5", "继续 F5"), "", 0, true, Act::SaveMenu, t!("記録した対局を再開する", "Resume a saved game", "继续已保存的对局"));
            Some(mid - 4.0)
        } else {
            self.add_button(px + 24.0, cy, w - 24.0, cy + 42.0, t!("続きから（記録を読み込む） F5", "Continue (load a save) F5", "继续（读取存档） F5"), "", 0, true, Act::SaveMenu, t!("記録した対局を再開する。書き出したファイルからも読み込める", "Resume a saved game. Exported files can be loaded too", "继续已保存的对局。也可读取导出的文件"));
            None
        };
        let hx1 = heroic_x0.unwrap_or(w - 24.0);
        let _ = hx1;
        self.add_button(px + 24.0, ty, w - 24.0, ty + 48.0, t!("はじめての方は：チュートリアル", "New here? Tutorial", "初次游玩：教程"), "", 0, true, Act::Tutorial, t!("フランス王として、操作と遊び方を1段ずつ学ぶ", "Learn the controls and the game step by step as King of France", "以法兰西国王身份逐步学习操作与玩法"));
        let ty = ty + 60.0;
        self.add_button(px + 24.0, ty, px + 70.0, ty + 40.0, "A−", "", 0, true, Act::Scale(-1), t!("文字と画面部品を小さくする（Ctrl＋−）", "Make text and interface smaller (Ctrl+−)", "缩小文字与界面（Ctrl＋−）"));
        self.add_button(px + 74.0, ty, px + 120.0, ty + 40.0, "A＋", "", 0, true, Act::Scale(1), t!("文字と画面部品を大きくする（Ctrl＋＋）", "Make text and interface larger (Ctrl++)", "放大文字与界面（Ctrl＋＋）"));
        let fl = if self.gfx.is_fullscreen() { t!("ウィンドウ F11", "Window F11", "窗口 F11") } else { t!("全画面 F11", "Fullscreen F11", "全屏 F11") };
        // 表示言語（いまの言語の名前を出し、押すと次の言語へ）
        let ll = format!("{} ▸", crate::lang::lang().native());
        self.add_button(px + 124.0, ty, px + 222.0, ty + 40.0, &ll, "", 0, true, Act::Lang, "表示言語を切り替える / Change language / 切换语言（日本語 → English → 中文）");
        self.add_button(px + 226.0, ty, w - 24.0, ty + 40.0, fl, "", 0, true, Act::Fullscreen, t!("全画面表示を切り替える", "Toggle fullscreen", "切换全屏显示"));
        // タイトル画面のツールチップ
        let (mx, my) = self.mouse;
        let tip = self.buttons.iter().find(|b| mx >= b.x0 && mx <= b.x1 && my >= b.y0 && my <= b.y1 && !b.tip.is_empty()).map(|b| b.tip.clone());
        if let Some(t) = tip.filter(|_| !self.show_save && !(self.touch && self.clock > self.tip_until)) {
            // 説明の行数に合わせて上へ伸ばす
            let x0 = px + 24.0;
            let th = self.gfx.measure_h(&t, Font::Small, w - x0 - 40.0) + 10.0;
            // 英霊モードの切り替えボタンの上に出す
            let y1 = ty - 124.0;
            self.art.draw_inset_box(&self.gfx, x0, y1 - th - 4.0, w - 24.0, y1);
            self.gfx.text(&t, x0 + 8.0, y1 - th, w - x0 - 40.0, th, Font::Small, [0.95, 0.92, 0.85, 1.0]);
        }
    }

    fn render_hud(&mut self) {
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        let pl = self.world.player;
        if self.gfx.compact {
            self.render_topbar_compact();
        } else {
        // ---- 上部バー（Age of Empires 風石彫パネル＋資源アイコン）
        self.art.draw_stone_panel(&self.gfx, 0.0, 0.0, w, self.top_h());
        let n = &self.world.nations[pl];
        let mut x = 8.0;
        let res = n.res;
        let inc = n.income;
        let upkeep = n.upkeep;
        let res_icons = [ResIcon::Food, ResIcon::Wood, ResIcon::Bronze];
        let (n_workers, w_cap, n_soldiers, m_cap) = (n.workers, n.worker_cap(), n.soldiers, n.mil_cap());
        for k in 0..3 {
            if let Some(ic) = &self.art.res_icons[res_icons[k] as usize] {
                self.gfx.draw_bitmap(ic, x, 4.0, x + 28.0, 32.0);
            }
            let rate = if k == 0 { inc[k] - upkeep } else { inc[k] };
            let s = format!("{}", res[k] as i32);
            self.gfx.text_shadow(&s, x + 31.0, 7.0, 72.0, 22.0, Font::Big, RES_COLORS[k]);
            let rs = tf!("{:+.1}/秒", "{:+.1}/s", "{:+.1}/秒", rate);
            let rc = if rate < 0.0 { [1.0, 0.42, 0.32, 1.0] } else { [0.80, 0.78, 0.70, 1.0] };
            self.gfx.text(&rs, x + 88.0, 11.0, 56.0, 18.0, Font::Small, rc);
            x += 148.0;
        }
        // 民全体・全軍をワンクリック選択できるトップバーアイコンボタン
        let (mx, my) = self.mouse;
        let w_hov = mx >= x && mx <= x + 88.0 && my >= 4.0 && my <= 32.0;
        self.gfx.fill_round(x, 4.0, x + 88.0, 32.0, 4.0, if w_hov { [0.35, 0.28, 0.16, 0.9] } else { [0.16, 0.13, 0.09, 0.75] });
        self.gfx.stroke_round(x, 4.0, x + 88.0, 32.0, 4.0, 1.0, [0.75, 0.62, 0.34, 0.85]);
        if let Some(ic) = &self.art.res_icons[ResIcon::Worker as usize] {
            self.gfx.draw_bitmap(ic, x + 3.0, 5.0, x + 29.0, 31.0);
        }
        let s = format!("{}/{}", n_workers, w_cap);
        self.gfx.text_shadow(&s, x + 32.0, 8.0, 54.0, 22.0, Font::Normal, [0.95, 0.92, 0.85, 1.0]);
        self.buttons.push(Button {
            x0: x,
            y0: 4.0,
            x1: x + 88.0,
            y1: 32.0,
            label: String::new(),
            key: 0,
            enabled: true,
            act: Act::SelectAllWorkers,
            tip: t!("民（生産ユニット）全体を選択する", "Select all workers", "选择全部民（生产单位）").to_string(),
        });
        x += 94.0;

        let s_hov = mx >= x && mx <= x + 88.0 && my >= 4.0 && my <= 32.0;
        self.gfx.fill_round(x, 4.0, x + 88.0, 32.0, 4.0, if s_hov { [0.35, 0.28, 0.16, 0.9] } else { [0.16, 0.13, 0.09, 0.75] });
        self.gfx.stroke_round(x, 4.0, x + 88.0, 32.0, 4.0, 1.0, [0.75, 0.62, 0.34, 0.85]);
        if let Some(ic) = &self.art.res_icons[ResIcon::Soldier as usize] {
            self.gfx.draw_bitmap(ic, x + 3.0, 5.0, x + 29.0, 31.0);
        }
        let s = self.soldier_text(n_soldiers, m_cap);
        self.gfx.text_shadow(&s, x + 32.0, 8.0, 54.0, 22.0, Font::Normal, [0.95, 0.92, 0.85, 1.0]);
        self.buttons.push(Button {
            x0: x,
            y0: 4.0,
            x1: x + 88.0,
            y1: 32.0,
            label: String::new(),
            key: 0,
            enabled: true,
            act: Act::SelectAllMilitary,
            tip: t!("軍事ユニット全体（全軍）を選択する", "Select all soldiers", "选择全部军事单位（全军）").to_string(),
        });
        x += 96.0;

        if let Some(ic) = &self.art.res_icons[ResIcon::Upkeep as usize] {
            self.gfx.draw_bitmap(ic, x, 5.0, x + 26.0, 31.0);
        }
        let s = tf!("-{:.1}/秒", "-{:.1}/s", "-{:.1}/秒", upkeep);
        self.gfx.text_shadow(&s, x + 28.0, 8.0, 88.0, 22.0, Font::Normal, [0.98, 0.68, 0.52, 1.0]);
        x += 112.0;
        // 金貨（商人からの税収）
        let (coin, coin_inc) = (self.world.nations[pl].coin, self.world.nations[pl].coin_income);
        self.gfx.fill_circle(x + 13.0, 18.0, 11.0, [0.86, 0.70, 0.30, 1.0]);
        self.gfx.stroke_circle(x + 13.0, 18.0, 11.0, 1.2, [0.45, 0.32, 0.12, 1.0]);
        self.gfx.fill_rect(x + 9.0, 14.0, x + 17.0, 22.0, [0.30, 0.22, 0.10, 1.0]);
        let s = format!("{}", coin as i32);
        self.gfx.text_shadow(&s, x + 28.0, 7.0, 64.0, 22.0, Font::Big, [1.0, 0.86, 0.45, 1.0]);
        let s = tf!("{:+.1}/秒", "{:+.1}/s", "{:+.1}/秒", coin_inc);
        self.gfx.text(&s, x + 80.0, 11.0, 56.0, 18.0, Font::Small, [0.80, 0.78, 0.70, 1.0]);
        self.buttons.push(Button {
            x0: x,
            y0: 4.0,
            x1: x + 136.0,
            y1: 32.0,
            label: String::new(),
            key: 0,
            enabled: true,
            act: Act::Market,
            tip: t!("金貨：領内を通る商人・遠方の商人からの税、村の略奪、身代金。交易画面（F4）で資源の購入や傭兵・ガレー船の雇用に、外交で身代金に使う", "Gold: taxes from merchants passing through your land, village plunder and ransoms. Spend it on the Trade screen (F4) on resources, mercenaries and galleys, or on ransoms", "金币：途经领内的商人所缴的税、劫掠村庄与赎金所得。可在贸易界面（F4）购买资源、雇佣佣兵与桨帆船，或支付赎金").to_string(),
        });

        // 速度・メニュー
        let bx = w - 424.0;
        let ex = (bx - 248.0).max(x + 150.0);
        // 幅に収まるいちばん詳しい書き方を選ぶ（言語で文字幅が違うので測って決める）
        let yb = self.world.year_ad();
        let spring = self.world.era == Era::Rise;
        let long = tf!("{}年　{}", "AD {}  {}", "{}年　{}", yb, if spring { t!("エドワード戦争", "Edwardian War", "爱德华战争") } else { t!("ランカスター戦争", "Lancastrian War", "兰开斯特战争") });
        let mid = tf!("{}年 {}", "{} {}", "{}年 {}", yb, if spring { t!("前期", "Early", "前期") } else { t!("後期", "Late", "后期") });
        let short = tf!("{}年", "AD {}", "{}年", yb);
        let room = bx - ex - 6.0;
        let (s, ef) = [(long, Font::Big), (mid, Font::Big), (short.clone(), Font::Big), (short.clone(), Font::Normal), (short, Font::Small)]
            .into_iter()
            .find(|(s, f)| self.gfx.measure(s, *f) <= room)
            .unwrap_or((tf!("{}", "{}", "{}", yb), Font::Small));
        if let Some(ic) = &self.art.res_icons[ResIcon::Era as usize] {
            self.gfx.draw_bitmap(ic, ex - 22.0, 8.0, ex - 2.0, 28.0);
        }
        self.gfx.text_shadow(&s, ex, if ef == Font::Big { 7.0 } else { 9.0 }, (bx - ex - 4.0).max(60.0), 24.0, ef, [1.0, 0.9, 0.6, 1.0]);
        let paused = self.paused;
        let sp = self.speed;
        self.add_button(bx, 5.0, bx + 36.0, 31.0, if paused { "▶" } else { "❚❚" }, "", 0, true, Act::Speed(0), "");
        for (k, s) in [1u32, 2, 4, 8].iter().enumerate() {
            let x0 = bx + 39.0 + k as f32 * 35.0;
            let lbl = format!("×{}", s);
            self.add_button(x0, 5.0, x0 + 33.0, 31.0, &lbl, "", 0, true, Act::Speed(*s), "");
            if sp == *s && !paused {
                self.gfx.stroke_round(x0, 5.0, x0 + 33.0, 31.0, 3.0, 2.0, [1.0, 0.9, 0.4, 1.0]);
            }
        }
        self.add_button(w - 242.0, 5.0, w - 166.0, 31.0, t!("交易 F4", "Trade F4", "贸易 F4"), "", 0, true, Act::Market, t!("商人の税・市場・傭兵・ガレー船・遠方の商人", "Merchant taxes, market, mercenaries, galleys, foreign merchants", "商人税、市场、佣兵、桨帆船、远方商人"));
        self.add_button(w - 162.0, 5.0, w - 84.0, 31.0, t!("外交 F2", "Diplo F2", "外交 F2"), "", 0, true, Act::Diplo, "");
        self.add_button(w - 80.0, 5.0, w - 4.0, 31.0, t!("ヘルプ F1", "Help F1", "帮助 F1"), "", 0, true, Act::Help, "");
        // 記録（上部バーに余地が無いので、その下の右端に置く）
        self.add_button(w - 80.0, self.top_h() + 4.0, w - 4.0, self.top_h() + 30.0, t!("記録 F5", "Save F5", "存档 F5"), "", 0, true, Act::SaveMenu, t!("記録（セーブ・ロード）。10年ごとに自動でも記録する", "Save and load. The game also saves automatically every 10 years", "保存与读取。每10年也会自动保存"));
        }
        let paused = self.paused;
        if paused {
            self.gfx.text_shadow(t!("一時停止中（Space）", "Paused (Space)", "已暂停（Space）"), w / 2.0 - 120.0, self.top_h() + 60.0, 240.0, 30.0, Font::BigCenter, [1.0, 1.0, 0.8, 1.0]);
        } else if let Some(left) = self.world.descent_left() {
            let s = tf!("乙女の亡霊：天の軍勢が両国を滅ぼすまで あと{}年", "The Maid's ghost: {} years until the hosts of Heaven destroy both kingdoms", "少女的亡灵：距天之军势毁灭两国还有{}年", left.ceil() as i32);
            self.gfx.text_shadow(&s, w / 2.0 - 320.0, self.top_h() + 60.0, 640.0, 30.0, Font::BigCenter, [1.0, 0.75, 0.35, 1.0]);
        } else if self.cmd_hold() {
            self.gfx.text_shadow(t!("指揮中（時は止まっている。命令を出すか選択を解くと進む）", "Commanding: time stopped (give an order or deselect)", "指挥中（时间已停止。下达命令或取消选择后继续）"), w / 2.0 - 320.0, self.top_h() + 60.0, 640.0, 30.0, Font::BigCenter, [0.8, 0.95, 1.0, 1.0]);
        }

        // ---- 連合情報
        // 右上の「記録 F5」ボタンの下から並べる（重ならないように）。タッチ向けの画面では記録は上部バーにある
        let mut cy = self.top_h() + if self.gfx.compact { 6.0 } else { 36.0 };
        let years = self.world.years;
        // 反〇〇同盟の一覧：（文面, 自国が標的・参加しているか）
        let coal: Vec<(String, bool)> = self
            .world
            .coalitions
            .iter()
            .map(|c| {
                let me = if c.target == pl {
                    t!("（標的：自国！）", " (target: YOU!)", "（目标：我国！）")
                } else if c.members.contains(&pl) {
                    t!("（参加中）", " (you joined)", "（参与中）")
                } else {
                    ""
                };
                let s = tf!("【反{}同盟】隣国{}か国 残り{}年{}", "[Anti-{} League] {} neighbours, {} yrs{}", "【反{}同盟】邻国{}国 剩余{}年{}", crate::lang::tx(self.world.nations[c.target].name), c.members.len(), (c.end - years).ceil() as i32, me);
                (s, c.target == pl || c.members.contains(&pl))
            })
            .collect();
        let compact = self.gfx.compact;
        let ag_shown = self.world.era == Era::Rise && self.world.nations[pl].aggression > 20.0;
        let others = coal.iter().filter(|c| !c.1).count();
        // 畳んでいる間は、自国が標的・参加している同盟だけを出し、ほかは件数の目印にまとめる（押すと開く）
        let open = self.alerts_open;
        let lw = if compact { 472.0f32.min(w * 0.62) } else { 472.0 };
        let list_y0 = cy;
        let bh = if compact { 34.0 } else { 26.0 };
        if !open && (others > 0 || (compact && ag_shown && coal.is_empty())) {
            let me_target = self.world.coalitions.iter().any(|c| c.target == pl);
            let lbl = if coal.is_empty() {
                tf!("隣国の警戒 {:.0} ▾", "Alarm {:.0} ▾", "邻国警戒 {:.0} ▾", self.world.nations[pl].aggression)
            } else if compact && me_target {
                tf!("反自国同盟 ({}) ▾", "League vs YOU ({}) ▾", "反我国同盟 ({}) ▾", coal.len())
            } else {
                tf!("ほかの反〇〇同盟 {} ▾", "Other leagues {} ▾", "其他反〇〇同盟 {} ▾", others)
            };
            let bw = self.gfx.measure(&lbl, Font::Normal) + 28.0;
            self.add_button(w - 6.0 - bw, cy, w - 6.0, cy + bh, &lbl, "", 0, true, Act::ToggleAlerts, t!("各地の反〇〇同盟の一覧を開く", "Show the list of leagues", "展开各地反〇〇同盟的列表"));
            if compact && me_target {
                self.gfx.stroke_round(w - 6.0 - bw, cy, w - 6.0, cy + bh, 3.0, 2.0, [1.0, 0.35, 0.25, 1.0]);
            }
            cy += bh + 2.0;
        }
        // 開いていればすべて、畳んでいれば自国に関わるものだけ（タッチ向けの畳んだ画面では目印だけ）
        let shown: Vec<&(String, bool)> = coal.iter().filter(|c| open || (c.1 && !compact)).collect();
        for (s, mine) in shown.iter().map(|c| (&c.0, c.1)) {
            let bg = if mine { [0.50, 0.08, 0.06, 0.92] } else { [0.38, 0.10, 0.08, 0.90] };
            self.gfx.fill_round(w - lw, cy, w - 6.0, cy + 23.0, 3.0, bg);
            self.gfx.stroke_round(w - lw, cy, w - 6.0, cy + 23.0, 3.0, 1.2, [0.92, 0.72, 0.34, 0.9]);
            self.gfx.text_shadow(s, w - lw + 8.0, cy + 2.0, lw - 12.0, 20.0, Font::Normal, [1.0, 0.88, 0.52, 1.0]);
            cy += 25.0;
        }
        // 隣国の警戒度
        if (open || !compact) && self.world.era == Era::Rise {
            let ag = self.world.nations[pl].aggression;
            if ag > 20.0 {
                let s = tf!("隣国の警戒 {:.0}/100", "Neighbours' alarm {:.0}/100", "邻国警戒 {:.0}/100", ag);
                let col = if ag > 75.0 { [1.0, 0.4, 0.3, 1.0] } else { [0.95, 0.8, 0.5, 1.0] };
                self.gfx.text_shadow(&s, w - 220.0, cy + 2.0, 214.0, 20.0, Font::Normal, col);
                cy += 24.0;
            }
        }
        if open && !coal.is_empty() {
            // 開いた一覧は、押すと畳む
            let label = t!("▴ 畳む", "▴ Hide", "▴ 收起");
            self.add_button(w - 96.0, cy + 2.0, w - 6.0, cy + 2.0 + bh, label, "", 0, true, Act::ToggleAlerts, t!("一覧を畳む（自国に関わる同盟だけを残す）", "Fold the list (keep only leagues involving you)", "收起列表（只保留与我国相关的同盟）"));
            if compact {
                self.buttons.push(Button { x0: w - lw, y0: list_y0, x1: w - 6.0, y1: cy, label: String::new(), key: 0, enabled: true, act: Act::ToggleAlerts, tip: String::new() });
            }
        }
        // 告知（画面の中央上）が同盟の一覧の下に潜らないよう、一覧の下端を覚えておく
        let right_list_bottom = cy;

        // ---- メッセージ
        let now = self.world.time;
        let mut y = self.top_h() + 8.0;
        let cut = self.msg_cut;
        let msgs: Vec<(String, [f32; 3], f32)> = self.world.msgs.iter().rev().take(if self.gfx.compact { 4 } else { 7 }).filter(|m| now - m.time < 25.0 && m.time > cut).map(|m| (m.text.clone(), m.color, now - m.time)).collect();
        let any_msg = !msgs.is_empty();
        for (t, c, age) in msgs.into_iter().rev() {
            let a = if age > 20.0 { (25.0 - age) / 5.0 } else { 1.0 };
            let mw = if self.gfx.compact { w * 0.5 } else { w * 0.55 };
            let mh = self.gfx.measure_h(&t, Font::Normal, mw).clamp(20.0, 40.0);
            self.gfx.text_shadow(&t, 10.0, y, mw, mh, Font::Normal, [c[0], c[1], c[2], a]);
            y += mh;
        }
        if any_msg {
            // お知らせを消す ✕（記録は歴史・外交画面に残る）
            let (bs, bh) = if self.gfx.compact { (36.0, 32.0) } else { (28.0, 24.0) };
            self.add_button(10.0, y + 2.0, 10.0 + bs, y + 2.0 + bh, "✕", "", 0, true, Act::ClearMsgs, t!("いま出ているお知らせを消す", "Clear the messages shown now", "清除当前显示的消息"));
        }

        // ---- イベントバナー
        if let Some(b) = &self.world.banner {
            let a = b.t.min(1.0);
            let (title, text) = (b.title.clone(), b.text.clone());
            let bw = 760.0f32.min(w - 40.0);
            let x0 = (w - bw) / 2.0;
            let mut y0 = self.top_h() + if self.gfx.compact { 40.0 } else { 90.0 };
            // 右上の同盟の一覧と横に重なるなら、その下へずらす
            if x0 + bw > w - 472.0 && right_list_bottom > y0 - 4.0 {
                y0 = right_list_bottom + 6.0;
            }
            let th = self.gfx.measure_h(&text, Font::NormalCenter, bw - 40.0).max(24.0);
            let bh = 66.0 + th + if self.gfx.compact { 34.0 } else { 14.0 };
            self.art.draw_stone_panel(&self.gfx, x0, y0, x0 + bw, y0 + bh);
            self.art.draw_corner_ornaments(&self.gfx, x0 + 3.0, y0 + 3.0, x0 + bw - 3.0, y0 + bh - 3.0, 14.0);
            self.gfx.text_shadow(&title, x0, y0 + 8.0, bw, 44.0, Font::Title, [1.0, 0.85, 0.4, a]);
            self.gfx.text_shadow(&text, x0 + 20.0, y0 + 58.0, bw - 40.0, th + 6.0, Font::NormalCenter, [1.0, 0.95, 0.85, a]);
            if self.gfx.compact {
                // タッチ向けの画面：告知を押すと閉じる（時が止まっている間も消せるように）
                self.gfx.text_shadow(t!("タップで閉じる", "Tap to close", "点击关闭"), x0, y0 + bh - 28.0, bw, 22.0, Font::NormalCenter, [0.85, 0.80, 0.70, a]);
                self.buttons.push(Button { x0, y0, x1: x0 + bw, y1: y0 + bh, label: String::new(), key: 0, enabled: true, act: Act::DismissBanner, tip: String::new() });
            }
            // 閉じる ✕（Esc でも閉じる）。待っている告知があれば、まとめて閉じるボタンも出す
            let waiting = self.world.banners_waiting();
            let cs = if self.gfx.compact { 38.0 } else { 30.0 };
            let tip = if waiting > 0 {
                tf!("この告知を閉じて次へ（Esc）。あと{}件", "Close this notice and show the next (Esc). {} more", "关闭此公告并显示下一条（Esc）。还有{}条", waiting)
            } else {
                t!("この告知を閉じる（Esc）", "Close this notice (Esc)", "关闭此公告（Esc）").to_string()
            };
            self.add_button(x0 + bw - cs - 8.0, y0 + 8.0, x0 + bw - 8.0, y0 + 8.0 + cs, "✕", "", 0, true, Act::DismissBanner, &tip);
            if waiting > 0 {
                let lbl = tf!("すべて閉じる（{}件）", "Close all ({})", "全部关闭（{}条）", waiting + 1);
                let lw = self.gfx.measure(&lbl, Font::Normal) + 24.0;
                self.add_button(x0 + bw - cs - 14.0 - lw, y0 + 8.0, x0 + bw - cs - 14.0, y0 + 8.0 + cs, &lbl, "", 0, true, Act::DismissAllBanners, t!("待っている告知もまとめて閉じる（内容は記録欄と歴史に残る）", "Close every waiting notice too (they stay in the log and History)", "将等待中的公告一并关闭（内容保留在记录栏与历史中）"));
            }
        }

        // ---- 下部パネル（Age of Empires 風石彫レリーフ＆3区画フレーム）
        let py = h - BOT_H;
        self.art.draw_stone_panel(&self.gfx, 0.0, py, w, h);

        // ミニマップ枠
        let (mx0, my0, mx1, my1) = self.mm_rect();
        self.gfx.fill_rect(mx0 - 2.0, my0 - 2.0, mx1 + 2.0, my1 + 2.0, [0.05, 0.04, 0.03, 1.0]);
        if let Some(b) = &self.minimap_bmp {
            self.gfx.draw_bitmap(b, mx0, my0, mx1, my1);
        }
        let sxs = MM_W / MAP_W as f32;
        let sys = MM_H / MAP_H as f32;
        let dots: Vec<(f32, f32, [f32; 3], bool)> = self
            .world
            .units
            .iter()
            .filter(|u| u.on_map() && u.kind.military() && (u.nation as usize == pl || self.seen(u.pos)))
            .map(|u| (mx0 + u.pos.x * sxs, my0 + u.pos.y * sys, self.world.nations[u.nation as usize].color, u.nation as usize == pl))
            .collect();
        let cdots: Vec<(f32, f32, [f32; 4])> = self
            .world
            .trade
            .caravans
            .iter()
            .filter(|c| self.seen(c.pos))
            .map(|c| (mx0 + c.pos.x * sxs, my0 + c.pos.y * sys, crate::game::market::caravan_color(c.kind)))
            .collect();
        for (x, y, c) in cdots {
            self.gfx.fill_rect(x - 1.0, y - 1.0, x + 1.0, y + 1.0, c);
        }
        for (x, y, c, mine) in dots {
            let s = if mine { 1.6 } else { 1.0 };
            self.gfx.fill_rect(x - s, y - s, x + s, y + s, if mine { [1.0, 1.0, 1.0, 1.0] } else { c4(darker(c, 0.6), 1.0) });
        }
        let a = self.screen_to_world(0.0, self.top_h());
        let b = self.screen_to_world(w, h - BOT_H);
        self.gfx.stroke_rect(
            (mx0 + a.x * sxs).max(mx0),
            (my0 + a.y * sys).max(my0),
            (mx0 + b.x * sxs).min(mx1),
            (my0 + b.y * sys).min(my1),
            1.2,
            [1.0, 0.96, 0.80, 0.92],
        );
        self.gfx.stroke_rect(mx0, my0, mx1, my1, 2.0, [0.78, 0.64, 0.34, 1.0]);
        self.art.draw_corner_ornaments(&self.gfx, mx0, my0, mx1, my1, 10.0);

        // 中央の情報欄ボックス
        let ix = MM_W + 12.0;
        let iw = w - ix - 394.0;
        self.art.draw_inset_box(&self.gfx, ix, py + 6.0, ix + iw, h - 4.0);
        self.art.draw_corner_ornaments(&self.gfx, ix + 2.0, py + 8.0, ix + iw - 2.0, h - 6.0, 11.0);
        self.render_info(ix + 12.0, py + 12.0, iw - 24.0);
        // 英雄と捕虜の画面（何も選んでいないとき）
        if self.sel.is_empty() && self.sel_b.is_none() {
            let mine = self.world.captives.iter().filter(|c| c.captor == pl).count();
            let sub = if mine > 0 { tf!("捕虜{}人", "{} captive(s)", "俘虏{}人", mine) } else { t!("身代金", "ransom", "赎金").to_string() };
            self.add_button(ix + iw - 176.0, h - 58.0, ix + iw - 14.0, h - 14.0, t!("英雄と捕虜", "Heroes", "英雄与俘虏"), &sub, 0, true, Act::Summon, t!("史実の英雄の一覧と、捕らえた英雄の身代金の請求（H）", "The historical heroes, and ransom demands for captured heroes (H)", "史实英雄一览，以及索要被俘英雄的赎金（H）"));
        }

        // スマートフォン：選択や建設場所選びを指で解く ✕
        if self.gfx.compact && (self.mode != Mode::Normal || !self.sel.is_empty() || self.sel_b.is_some()) {
            let tip = if self.mode != Mode::Normal { t!("取り消す", "Cancel", "取消") } else { t!("選択を解く", "Deselect", "取消选择") };
            self.add_button(ix + iw - 46.0, py + 10.0, ix + iw - 8.0, py + 48.0, "✕", "", 0, true, Act::Cancel, tip);
        }

        // 右側のコマンド欄ボックス
        let cx0 = w - 388.0;
        self.art.draw_inset_box(&self.gfx, cx0, py + 6.0, w - 4.0, h - 4.0);
        self.art.draw_corner_ornaments(&self.gfx, cx0 + 2.0, py + 8.0, w - 6.0, h - 6.0, 11.0);
        if !self.show_diplo && !self.show_help && !self.show_chron && !self.show_market && !self.show_summon && !self.show_save {
            self.render_commands(w - 382.0, py + 12.0);
        }

        if let Some((t, life)) = &self.notice {
            let a = life.min(1.0);
            let t = t.clone();
            self.gfx.text_shadow(&t, w / 2.0 - 300.0, py - 34.0, 600.0, 26.0, Font::BigCenter, [1.0, 0.85, 0.5, a]);
        }
    }

    /// ボタンのツールチップ（すべての画面部品の上に描く。上端で切れるならカーソルの下へ）
    fn render_tooltip(&mut self) {
        if self.touch && self.clock > self.tip_until {
            return;
        }
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        let (mx, my) = self.mouse;
        // 後から置いたボタン（上に重なる画面）を優先
        let mut tip = None;
        for b in self.buttons.iter().rev() {
            if mx >= b.x0 && mx <= b.x1 && my >= b.y0 && my <= b.y1 {
                if b.label == BLOCK_LABEL {
                    break;
                }
                if !b.tip.is_empty() {
                    tip = Some(b.tip.clone());
                    break;
                }
            }
        }
        let Some(t) = tip else { return };
        let tw = 340.0f32.min(w - 8.0);
        let th = self.gfx.measure_h(&t, Font::Small, tw - 16.0) + 14.0;
        let x0 = (mx - tw * 0.5).clamp(4.0, (w - tw - 4.0).max(4.0));
        let y0 = if my - th - 10.0 >= 2.0 { my - th - 10.0 } else { (my + 24.0).min(h - th - 2.0) };
        self.art.draw_inset_box(&self.gfx, x0, y0, x0 + tw, y0 + th);
        self.gfx.text(&t, x0 + 8.0, y0 + 6.0, tw - 16.0, th - 8.0, Font::Small, [0.95, 0.92, 0.85, 1.0]);
    }

    fn order_text(&self, u: &Unit) -> &'static str {
        match u.order {
            Order::Idle => t!("待機中", "Idle", "待命"),
            Order::Move(_) => t!("移動中", "Moving", "移动中"),
            Order::AttackMove(_) => t!("進軍中", "Advancing", "进军中"),
            Order::Attack(_) => t!("交戦中", "Fighting", "交战中"),
            Order::Gather { src, phase } => match (src.res(), phase) {
                (_, 2) => t!("運搬中", "Carrying", "搬运中"),
                (0, _) => t!("農耕中", "Farming", "耕作中"),
                (1, _) => t!("伐採中", "Cutting wood", "伐木中"),
                _ => t!("採掘中", "Mining", "采矿中"),
            },
            Order::Build(_) => t!("建設中", "Building", "建造中"),
            Order::Board(_) => t!("乗船へ", "Boarding", "登船中"),
            Order::Unload(_) => t!("上陸へ", "Sailing to land", "前往登陆"),
            Order::Ferry(..) if u.aboard.is_some() => t!("船で海を渡っている", "Crossing the sea", "乘船渡海中"),
            Order::Ferry(..) => t!("岸で船を待つ", "Waiting for a ship", "在岸边等船"),
            Order::Shuttle(..) => t!("往復輸送中", "Ferrying troops", "往返运输中"),
        }
    }

    fn render_info(&mut self, x: f32, y: f32, wd: f32) {
        let pl = self.world.player;
        let cap = self.world.power_cap();
        let tc = [0.95, 0.92, 0.85, 1.0];
        if self.sel.len() == 1 {
            let u = self.world.unit(self.sel[0]).unwrap();
            let n = &self.world.nations[u.nation as usize];
            let nc = n.color;
            // Age of Empires 風の大型全身ユニット肖像カード＋体力バー
            let px0 = x + 2.0;
            let py0 = y + 2.0;
            let psz = 88.0;
            self.art.draw_inset_box(&self.gfx, px0 - 2.0, py0 - 2.0, px0 + psz + 2.0, py0 + psz + 2.0);
            self.gfx.fill_circle(px0 + psz * 0.5, py0 + psz * 0.74, psz * 0.34, c4(nc, 0.42));
            self.gfx.stroke_circle(px0 + psz * 0.5, py0 + psz * 0.74, psz * 0.34, 1.5, c4(nc, 0.9));
            let sprite_bmp = if (u.hero as usize) < HEROES.len() {
                self.art.period_hero(u.hero as usize, false)
            } else if u.divine {
                self.art.divine_unit_sprite(u.kind, false).or_else(|| self.art.national_unit(u.nation as usize, u.ut, false))
            } else {
                self.art.national_unit(u.nation as usize, u.ut, false)
            };
            if let Some(bmp) = sprite_bmp {
                self.gfx.draw_bitmap(bmp, px0 + 2.0, py0 + 2.0, px0 + psz - 2.0, py0 + psz - 2.0);
            }
            // 肖像下のHPバー
            let f = (u.hp / u.max_hp()).clamp(0.0, 1.0);
            self.gfx.fill_rect(px0, py0 + psz + 4.0, px0 + psz, py0 + psz + 11.0, [0.08, 0.06, 0.05, 0.95]);
            self.gfx.fill_rect(px0 + 1.0, py0 + psz + 5.0, px0 + 1.0 + (psz - 2.0) * f, py0 + psz + 10.0, [1.0 - f, 0.25 + 0.7 * f, 0.15, 1.0]);
            self.gfx.stroke_rect(px0, py0 + psz + 4.0, px0 + psz, py0 + psz + 11.0, 1.0, [0.75, 0.62, 0.34, 0.9]);

            let tx = x + 102.0;
            let tw = (wd - 102.0).max(100.0);
            let title = if (u.hero as usize) < HEROES.len() {
                let d = &HEROES[u.hero as usize];
                tf!("英雄・{}（{}）【{}】", "Hero: {} ({}) [{}]", "英雄・{}（{}）【{}】", crate::lang::tx(d.name), crate::lang::tx(d.era), crate::lang::tx(n.name))
            } else if u.divine {
                tf!("天の軍勢・{}", "Heavenly host: {}", "天之军势・{}", crate::lang::tx(u.def().name))
            } else if (u.tribe as usize) < TRIBES.len() {
                let td = &TRIBES[u.tribe as usize];
                if u.merc_until > 0.0 {
                    tf!("{}の傭兵（{}）【{}】残り{}年", "{} mercenary ({}) [{}] {} yrs left", "{}佣兵（{}）【{}】剩余{}年", crate::lang::tx(td.name), crate::lang::tx(u.def().name), crate::lang::tx(n.name), (u.merc_until - self.world.years).ceil() as i32)
                } else {
                    tf!("{}の{}", "{} {}", "{}{}", crate::lang::tx(td.name), crate::lang::tx(u.def().name))
                }
            } else {
                format!("{}　【{}】", crate::lang::tx(u.def().name), crate::lang::tx(n.name))
            };
            let lines = tf!(
                "兵力 {:.0}/{:.0}　　能力 {:.1}（時代上限 {:.1}）\n状態：{}{}",
                "HP {:.0}/{:.0}    Power {:.1} (era limit {:.1})\nStatus: {}{}",
                "兵力 {:.0}/{:.0}　　能力 {:.1}（时代上限 {:.1}）\n状态：{}{}",
                u.hp.max(0.0),
                u.max_hp(),
                u.power,
                cap,
                self.order_text(u),
                if u.carry > 0.5 { tf!("（{} {:.0}）", " ({} {:.0})", "（{} {:.0}）", res_name(u.carry_kind), u.carry) } else { String::new() }
            );
            let desc = if (u.hero as usize) < HEROES.len() {
                crate::lang::tx(HEROES[u.hero as usize].desc)
            } else if u.divine {
                crate::lang::tx(u.def().desc)
            } else {
                crate::lang::tx(u.def().desc)
            };
            let lines = if u.kind.naval() { format!("{}\n{}", lines, tf!("乗船 {}/{}", "Aboard {}/{}", "载员 {}/{}", u.cargo.len(), SHIP_CAP)) } else { lines };
            self.gfx.text_shadow(&title, tx, y + 2.0, tw, 28.0, Font::Big, tc);
            self.gfx.text(&lines, tx, y + 34.0, tw, 52.0, Font::Normal, tc);
            self.gfx.text(desc, x + 2.0, y + 106.0, wd, 38.0, Font::Small, [0.80, 0.76, 0.68, 1.0]);
        } else if self.sel.len() > 1 {
            let mut cnt = [0usize; 5];
            let mut pw = 0.0;
            let mut hp = 0.0;
            let mut mhp = 0.0;
            for &h in self.sel.iter() {
                let u = self.world.unit(h).unwrap();
                cnt[u.kind as usize] += 1;
                pw += u.power;
                hp += u.hp;
                mhp += u.max_hp();
            }
            let title = tf!("{} 部隊を編制選択中", "{} units selected", "已选择{}个单位", self.sel.len());
            self.gfx.text_shadow(&title, x, y, wd, 26.0, Font::Big, tc);
            let kinds = [UnitKind::Worker, UnitKind::Infantry, UnitKind::Archer, UnitKind::Cavalry, UnitKind::Ship];
            let (mx, my) = self.mouse;
            let mut ix = x;
            for k in 0..5 {
                if cnt[k] > 0 {
                    let bx0 = ix;
                    let by0 = y + 28.0;
                    let bx1 = ix + 86.0;
                    let by1 = y + 72.0;
                    let hov = mx >= bx0 && mx <= bx1 && my >= by0 && my <= by1;
                    self.gfx.fill_round(bx0, by0, bx1, by1, 4.0, if hov { [0.10, 0.29, 0.36, 0.98] } else { [0.055, 0.13, 0.18, 0.96] });
                    self.gfx.stroke_round(bx0, by0, bx1, by1, 4.0, 1.2, [0.40, 0.56, 0.59, 0.9]);
                    if let Some(bmp) = self.art.national_unit(pl, native_ut(pl, kinds[k]), false) {
                        self.gfx.draw_bitmap(bmp, bx0 + 3.0, by0 + 3.0, bx0 + 41.0, by1 - 3.0);
                    }
                    let ks = format!("×{}", cnt[k]);
                    self.gfx.text_shadow(&ks, bx0 + 44.0, by0 + 10.0, 40.0, 24.0, Font::Big, tc);
                    self.buttons.push(Button {
                        x0: bx0,
                        y0: by0,
                        x1: bx1,
                        y1: by1,
                        label: String::new(),
                        key: 0,
                        enabled: true,
                        act: Act::FilterKind(kinds[k]),
                        tip: tf!("{}（{}体）：クリックでこの兵種のみに絞り込む", "{} ({}): click to keep only this type", "{}（{}个）：点击只保留该兵种", kinds[k].name(), cnt[k]),
                    });
                    ix += 94.0;
                }
            }
            let s = tf!("総能力 {:.1}（平均 {:.2}）　　総兵力 {:.0}/{:.0}", "Total power {:.1} (avg {:.2})    Total HP {:.0}/{:.0}", "总能力 {:.1}（平均 {:.2}）　　总兵力 {:.0}/{:.0}", pw, pw / self.sel.len() as f32, hp, mhp);
            self.gfx.text(&s, x, y + 76.0, wd, 30.0, Font::Normal, tc);
            let f = (hp / mhp.max(1.0)).clamp(0.0, 1.0);
            let bw = wd.min(320.0);
            self.gfx.fill_rect(x, y + 106.0, x + bw, y + 114.0, [0.08, 0.06, 0.05, 0.95]);
            self.gfx.fill_rect(x + 1.0, y + 107.0, x + 1.0 + (bw - 2.0) * f, y + 113.0, [1.0 - f, 0.25 + 0.7 * f, 0.15, 1.0]);
            self.gfx.stroke_rect(x, y + 106.0, x + bw, y + 114.0, 1.0, [0.75, 0.62, 0.34, 0.9]);
        } else if let Some(bh) = self.sel_b {
            let b = self.world.bld(bh).unwrap();
            let n = &self.world.nations[b.nation as usize];
            let nc = n.color;
            let is_divine_city = self.world.descent.is_some() && b.nation as usize == HEAVEN && b.kind.city();
            let px0 = x + 2.0;
            let py0 = y + 4.0;
            let psz = 68.0;
            self.gfx.fill_rect(px0 - 2.0, py0 - 2.0, px0 + psz + 2.0, py0 + psz + 2.0, if is_divine_city { [0.95, 0.85, 0.45, 0.95] } else { c4(nc, 0.9) });
            let bld_bmp = if is_divine_city {
                self.art.divine_bld_sprite(b.kind, false).or_else(|| self.art.national_building(b.origin as usize, b.kind, false))
            } else {
                self.art.national_building(b.origin as usize, b.kind, false)
            };
            if let Some(bmp) = bld_bmp {
                self.gfx.draw_bitmap(bmp, px0, py0, px0 + psz, py0 + psz);
            }
            let f = (b.hp / b.kind.max_hp()).clamp(0.0, 1.0);
            self.gfx.fill_rect(px0, py0 + psz + 4.0, px0 + psz, py0 + psz + 11.0, [0.08, 0.06, 0.05, 0.95]);
            self.gfx.fill_rect(px0 + 1.0, py0 + psz + 5.0, px0 + 1.0 + (psz - 2.0) * f, py0 + psz + 10.0, [1.0 - f, 0.25 + 0.7 * f, 0.15, 1.0]);
            self.gfx.stroke_rect(px0, py0 + psz + 4.0, px0 + psz, py0 + psz + 11.0, 1.0, [0.75, 0.62, 0.34, 0.9]);

            let tx = x + 84.0;
            let tw = (wd - 84.0).max(100.0);
            let title = if is_divine_city {
                tf!("天の{}　【{}】", "Heaven's {}  [{}]", "天之{}　【{}】", b.kind.name(), crate::lang::tx(n.name))
            } else {
                format!("{}　【{}】", b.kind.name(), crate::lang::tx(n.name))
            };
            let mut s = tf!("耐久 {:.0}/{:.0}", "HP {:.0}/{:.0}", "耐久 {:.0}/{:.0}", b.hp.max(0.0), b.kind.max_hp());
            if !b.done() {
                let builders = self.world.bld_builders(bh);
                if builders > 0 {
                    s.push_str(&tf!("　（建設中 {:.0}%・作業民{}体）", "  (building {:.0}%, {} workers)", "　（建造中 {:.0}%・施工民{}名）", b.progress * 100.0, builders));
                } else {
                    s.push_str(&tf!("　（建設中断 {:.0}%・右の「建設を再開」で民を派遣）", "  (paused at {:.0}%: press Resume building to send workers)", "　（建造中断 {:.0}%・点右侧“继续建造”派民）", b.progress * 100.0));
                }
            }
            let desc = if is_divine_city {
                t!("天の軍勢に焼かれようとしている街。", "A town about to be burned by the hosts of Heaven.", "即将被天之军势焚毁的城镇。")
            } else {
                match b.kind {
                    BKind::Capital => t!("王国の都。民を訓練し、資源の運び先となる。失うと都を移し、すべての街を失うと王国は滅ぶ。", "The royal capital. Trains workers and receives resources. If it falls the court moves; lose every town and the kingdom falls.", "王国之都。训练民，接收资源。失守则迁都，失去全部城镇则王国灭亡。"),
                    BKind::Town => t!("最初からある主要な街。領土を広げ、民を訓練し、資源の運び先となる。攻め落とすか、諸侯の寝返りで持ち主が変わる。", "One of the great towns that exist from the start. Expands territory, trains workers and receives resources. It changes hands by siege or when its lord changes sides.", "一开始就存在的主要城镇。扩张领土，训练民，接收资源。可通过攻城或诸侯倒戈易主。"),
                    BKind::Village => t!("民が建てる村。糧を生み、資源の運び先となり、民の上限を1、兵の上限を3上げる。敵に壊されると略奪される。", "A village built by workers. Produces food, receives resources and raises the worker limit by 1 and soldier limit by 3. If an enemy destroys it, it is plundered.", "由民建造的村庄。产粮，接收资源，使民的上限+1、兵的上限+3。被敌人摧毁时会遭劫掠。"),
                    BKind::Barracks => t!("自国の兵を訓練する。年代が進むと新しい兵（大砲・常備軍）が加わる。", "Trains your kingdom's troops. New troops (cannon, standing army) appear as the years pass.", "训练本国兵种。随着年代推进会加入新兵种（大炮、常备军）。"),
                    BKind::Port => t!("船を造る。船は兵6人を乗せて海を渡り、遠くから矢を射る。", "Builds ships. A ship carries 6 troops across the sea and shoots from range.", "建造船只。船可载6名士兵渡海，并能远程射击。"),
                    BKind::Farm => t!("民が耕すと糧を産む。枯れることはない。", "Produces food when a worker farms it. Never runs out.", "民耕作后产粮，永不枯竭。"),
                    BKind::Fort => t!("近づく敵兵を射る（射程9）。数は街の数で決まる上限まで。", "Shoots enemy soldiers that come near (range 9). How many you may build depends on your number of towns.", "射击靠近的敌兵（射程9）。数量上限取决于城镇数。"),
                }
            };
            let queue: Vec<(u8, f32)> = b.queue.iter().enumerate().map(|(i, &k)| (k, if i == 0 { b.queue_t / udef(k).time } else { 0.0 })).collect();
            self.gfx.text_shadow(&title, tx, y, tw, 28.0, Font::Big, tc);
            self.gfx.text(&s, tx, y + 30.0, tw, 24.0, Font::Normal, tc);
            self.gfx.text(desc, tx, y + 56.0, tw, 40.0, Font::Small, [0.80, 0.76, 0.68, 1.0]);
            for (i, (k, f)) in queue.iter().enumerate() {
                let qx = tx + i as f32 * 44.0;
                let qy = y + 98.0;
                self.gfx.fill_round(qx, qy, qx + 40.0, qy + 40.0, 3.0, [0.055, 0.13, 0.18, 1.0]);
                if let Some(ubmp) = self.art.national_unit(b.nation as usize, *k, false) {
                    self.gfx.draw_bitmap(ubmp, qx + 2.0, qy + 2.0, qx + 38.0, qy + 38.0);
                }
                if *f > 0.0 {
                    self.gfx.fill_rect(qx + 2.0, qy + 34.0, qx + 2.0 + 36.0 * f, qy + 38.0, [0.35, 0.9, 0.4, 1.0]);
                }
            }
        } else {
            let n = &self.world.nations[pl];
            let nc = n.color;
            let is_divine = self.world.descent.is_some() && pl == HEAVEN;
            let px0 = x + 2.0;
            let py0 = y + 4.0;
            let psz = 68.0;
            self.gfx.fill_rect(px0 - 2.0, py0 - 2.0, px0 + psz + 2.0, py0 + psz + 2.0, if is_divine { [0.95, 0.85, 0.45, 0.95] } else { c4(nc, 0.9) });
            let cap_bmp = if is_divine {
                self.art.divine_bld_sprite(BKind::Capital, false).or_else(|| self.art.national_building(pl, BKind::Capital, false))
            } else {
                self.art.national_building(pl, BKind::Capital, false)
            };
            if let Some(bmp) = cap_bmp {
                self.gfx.draw_bitmap(bmp, px0, py0, px0 + psz, py0 + psz);
            }
            let tx = x + 84.0;
            let tw = (wd - 84.0).max(100.0);
            let title = if self.gfx.compact {
                tf!("{} の国勢", "State of {}", "{}国势", crate::lang::tx(n.name))
            } else {
                tf!("{} の国勢（右のボタンで民全体・全軍へ命令可）", "State of {} (the buttons on the right command all workers or all soldiers)", "{}国势（右侧按钮可命令全体民或全军）", crate::lang::tx(n.name))
            };
            let wars: Vec<&str> = (0..NATIONS.len()).filter(|&o| self.world.at_war(pl, o)).map(|o| crate::lang::tx(self.world.nations[o].name)).collect();
            let s = tf!(
                "街 {}　村 {}　領土 {}　国力 {:.0}　生産 ×{:.2}　時代 ×{:.2}　上限 {:.1}\n交戦中：{}",
                "Towns {}  Villages {}  Land {}  Strength {:.0}  Production ×{:.2}  Era ×{:.2}  Limit {:.1}\nAt war with: {}",
                "城镇 {}　村庄 {}　领土 {}　国力 {:.0}　生产 ×{:.2}　时代 ×{:.2}　上限 {:.1}\n交战中：{}",
                n.cities,
                n.villages,
                n.tiles,
                n.strength,
                n.bias,
                self.world.era_mult(),
                cap,
                if wars.is_empty() {
                    t!("なし", "none", "无").to_string()
                } else if wars.len() > 8 {
                    format!("{}{}", wars[..8].join(sep()), tf!("ほか{}国", " +{} more", "等{}国", wars.len() - 8))
                } else {
                    wars.join(sep())
                }
            );
            self.gfx.text_shadow(&title, tx, y, tw, 26.0, Font::Big, tc);
            self.gfx.text(&s, tx, y + 28.0, tw, 48.0, Font::Normal, tc);

            // 自国の各兵種アイコンボタン（クリックでその兵種全体を選択）
            let mut cnt = [0usize; 5];
            for u in self.world.units.iter() {
                if u.on_map() && u.nation as usize == pl {
                    cnt[u.kind as usize] += 1;
                }
            }
            let kinds = [UnitKind::Worker, UnitKind::Infantry, UnitKind::Archer, UnitKind::Cavalry, UnitKind::Ship];
            let (mx, my) = self.mouse;
            let mut ix = tx;
            for k in 0..5 {
                if k == 4 && cnt[4] == 0 {
                    continue;
                }
                let bx0 = ix;
                let by0 = y + 82.0;
                let bx1 = ix + 78.0;
                let by1 = y + 122.0;
                let hov = mx >= bx0 && mx <= bx1 && my >= by0 && my <= by1;
                self.gfx.fill_round(bx0, by0, bx1, by1, 4.0, if hov { [0.10, 0.29, 0.36, 0.98] } else { [0.055, 0.13, 0.18, 0.96] });
                self.gfx.stroke_round(bx0, by0, bx1, by1, 4.0, 1.1, [0.40, 0.56, 0.59, 0.85]);
                if let Some(bmp) = self.art.national_unit(pl, native_ut(pl, kinds[k]), false) {
                    self.gfx.draw_bitmap(bmp, bx0 + 3.0, by0 + 3.0, bx0 + 37.0, by1 - 3.0);
                }
                let ks = format!("×{}", cnt[k]);
                self.gfx.text_shadow(&ks, bx0 + 40.0, by0 + 8.0, 36.0, 22.0, Font::Normal, tc);
                self.buttons.push(Button {
                    x0: bx0,
                    y0: by0,
                    x1: bx1,
                    y1: by1,
                    label: String::new(),
                    key: 0,
                    enabled: cnt[k] > 0,
                    act: Act::SelectKind(kinds[k]),
                    tip: tf!("{}全体（{}体）を選択する", "Select all {} ({})", "选择全部{}（{}个）", kinds[k].name(), cnt[k]),
                });
                ix += 84.0;
            }
        }
    }

    fn render_commands(&mut self, x: f32, y: f32) {
        let pl = self.world.player;
        let _era = self.world.era;
        let (bw, bh, gap) = (122.0, 46.0, 4.0);
        let pos = |i: usize| -> (f32, f32, f32, f32) {
            let c = (i % 3) as f32;
            let r = (i / 3) as f32;
            let x0 = x + c * (bw + gap);
            let y0 = y + r * (bh + gap);
            (x0, y0, x0 + bw, y0 + bh)
        };
        let res = self.world.nations[pl].res;
        let afford = |c: [f32; 3]| res[0] >= c[0] && res[1] >= c[1] && res[2] >= c[2];
        let own = self.own_sel();
        let (n_workers, n_soldiers) = (self.world.nations[pl].workers, self.world.nations[pl].soldiers);
        // 全軍への命令は傭兵も含む（訓練だけは傭兵を除く）
        let n_army = self.all_military().len();
        let n_trainable = n_soldiers;
        let utility_y = y + 3.0 * (bh + gap);
        let utility_bottom = utility_y + 36.0;
        if !own.is_empty() {
            let can_discard = own.iter().all(|&h| self.world.can_discard_unit(pl, h));
            let (discard_x, _, discard_right, _) = pos(2);
            if self.discard_pending && can_discard {
                let label = tf!("{}体を破棄する", "Remove {} units", "移除{}个单位", own.len());
                self.add_button(x, utility_y, discard_x - gap, utility_bottom, &label, "", 0, true, Act::ConfirmDiscard,
                    t!("選択中の自国ユニットを完全に取り除きます。資源は戻りません。", "Permanently remove your selected units. No resources are refunded.", "永久移除所选己方单位，不返还资源。"));
                self.add_button(discard_x, utility_y, discard_right, utility_bottom, t!("取消", "Cancel", "取消"), "", 0, true, Act::Noop,
                    t!("ユニットの破棄を取り消す", "Cancel unit removal", "取消移除单位"));
            } else {
                let tip = if can_discard {
                    t!("選択中の自国ユニットを破棄する（確認あり）", "Remove your selected units after confirmation", "确认后移除所选己方单位")
                } else {
                    t!("乗員がいる船は、先に乗員を降ろしてください", "Unload passengers before removing a ship", "移除船只前请先让乘员下船")
                };
                self.add_button(discard_x, utility_y, discard_right, utility_bottom, t!("ユニット破棄", "Remove units", "移除单位"), "", 0, can_discard, Act::Discard, tip);
            }
            let any_worker = own.iter().any(|&h| self.world.unit(h).unwrap().kind == UnitKind::Worker);
            let any_mil = own.iter().any(|&h| self.world.unit(h).unwrap().kind.military());
            if any_worker && !(self.discard_pending && can_discard) {
                let (_, _, plant_right, _) = pos(1);
                let label = tf!("植林（{}）", "Plant trees ({})", "植树（{}）", Self::cost_str(PLANT_FOREST_COST));
                self.add_button(x, utility_y, plant_right, utility_bottom, &label, "", 0, afford(PLANT_FOREST_COST), Act::PlantForest,
                    t!("選んだ民から6マス以内の自国領内の平地・草原に植える。30秒で木材を採れる森になる。", "Plant on empty plains or steppe in your territory within 6 tiles of a selected worker. The forest can be harvested after 30 seconds.", "在所选的民周围6格内的己方空旷平地或草原上植树。30秒后可伐木。"));
            }
            let mut i = 0;
            if any_worker {
                for (k, key, tip) in [
                    (BKind::Farm, 'Q', t!("田：民が耕すと糧を産む。自国領内に建設。", "Field: produces food when farmed. Build inside your territory.", "田：民耕作后产粮。建于己方领土内。")),
                    (BKind::Barracks, 'W', t!("兵営：軍事ユニットを訓練する。自国領内に建設。", "Barracks: trains soldiers. Build inside your territory.", "兵营：训练军事单位。建于己方领土内。")),
                    (BKind::Village, 'E', t!("村：糧を生み、資源の運び先になり、民の上限を1、兵の上限を3上げる。自国領内で、街や村から6マス以上離す。敵に壊されると略奪される（街は新しく築けない）。", "Village: produces food, receives resources and raises worker limit by 1 and soldier limit by 3. Build in your territory, 6+ tiles from any town or village. An enemy who destroys it plunders it. (Towns cannot be founded.)", "村庄：产粮、接收资源，民上限+1、兵上限+3。建于己方领土内，距城镇或村庄6格以上。被敌人摧毁时会遭劫掠（无法新建城镇）。")),
                ] {
                    let (x0, y0, x1, y1) = pos(i);
                    let c = k.cost();
                    self.add_button(x0, y0, x1, y1, k.name(), &Self::cost_str(c), key as u32, afford(c), Act::Place(k), tip);
                    i += 1;
                }
            }
            // 鍛錬
            let u0 = self.world.unit(own[0]).unwrap();
            let u0_kind = u0.kind;
            let up_cost = self.world.upgrade_cost(u0);
            let label = if any_mil && !any_worker { t!("訓練", "Train", "训练") } else if any_worker && !any_mil { t!("教化", "Educate", "教化") } else { t!("鍛錬", "Train", "锻炼") };
            let (x0, y0, x1, y1) = pos(i);
            self.add_button(x0, y0, x1, y1, label, &Self::cost_str(up_cost), 'T' as u32, true, Act::Upgrade, t!("選択部隊の能力を +0.5 高める（自国領内のみ）。兵は戦闘力と兵力、民は採集量が上がる。費用は現在の能力に比例し、糧・木・鉄のうち蓄えの多いものから優先して払う。", "Raises the selected units' power by +0.5 (inside your territory only). Soldiers gain strength and HP, workers gather more. The cost grows with power and is paid mostly from whichever of food, wood and iron you have most of.", "使所选单位能力+0.5（仅限己方领土内）。士兵提升战斗力与兵力，民提升采集量。费用与当前能力成正比，优先从粮、木、铁中储量较多者支付。"));
            i += 1;
            let (x0, y0, x1, y1) = pos(i);
            let (cl, ct) = if any_mil && !any_worker {
                (t!("解散→民", "Disband", "解散→民"), t!("兵を解散して民にする（能力は2/3に目減り）。", "Disband soldiers into workers (power drops to 2/3).", "将士兵解散为民（能力降为2/3）。"))
            } else if any_worker && !any_mil {
                (t!("徴兵→歩兵", "Draft", "征兵→步兵"), t!("民を徴兵して歩兵にする（能力は2/3に目減り）。", "Draft workers as infantry (power drops to 2/3).", "征民为步兵（能力降为2/3）。"))
            } else {
                (t!("兵⇔民", "Convert", "兵⇔民"), t!("兵は民に、民は歩兵に転換する（能力は2/3に目減り）。", "Soldiers become workers and workers become infantry (power drops to 2/3).", "士兵转为民，民转为步兵（能力降为2/3）。"))
            };
            let ship_sel: Vec<Handle> = own.iter().copied().filter(|&h| self.world.unit(h).map_or(false, |u| u.kind.naval())).collect();
            if !ship_sel.is_empty() {
                let cargo: usize = ship_sel.iter().filter_map(|&h| self.world.unit(h)).map(|u| u.cargo.len()).sum();
                let cap = ship_sel.len() * SHIP_CAP;
                self.add_button(x0, y0, x1, y1, t!("上陸", "Land troops", "登陆"), &tf!("乗船 {}/{}", "aboard {}/{}", "载员 {}/{}", cargo, cap), 'C' as u32, cargo > 0, Act::Unload, t!("船の兵をいまいる岸に降ろす。\n兵を乗せるには、兵を選んで岸辺の船を右クリック。船を選んで陸を右クリックすると、その近くの岸へ向かって兵を降ろす。", "Land the troops aboard at the current shore.\nTo embark, select troops and right-click a ship by the shore. Select a ship and right-click land to sail there and land the troops.", "让船上士兵在当前岸边登陆。\n选中士兵后右键点击岸边的船即可登船。选中船后右键点击陆地，船会驶往附近岸边并让士兵登陆。"));
            } else {
                self.add_button(x0, y0, x1, y1, cl, t!("能力×2/3", "power ×2/3", "能力×2/3"), 'C' as u32, true, Act::Convert, ct);
            }
            i += 1;
            let (x0, y0, x1, y1) = pos(i);
            if any_worker && !any_mil {
                // 民だけを選んでいるときは、停止の代わりに要塞
                let n = &self.world.nations[pl];
                let (nf, cap) = (self.world.fort_count(pl), n.fort_cap());
                let c = BKind::Fort.cost();
                let tip = tf!(
                    "要塞：近づく敵兵を射る（射程{:.0}）。自国領内に建設。数は街3つごとに1つ、最大6まで（いま{}／{}）。余った資源の使い道に。",
                    "Fort: shoots enemy soldiers that come near (range {:.0}). Build inside your territory. One per three towns, six at most (now {}/{}). A good use for surplus resources.",
                    "要塞：射击靠近的敌兵（射程{:.0}）。建于己方领土内。每三座城镇可建一座，最多6座（现有{}／{}）。可消耗多余资源。",
                    FORT_RANGE, nf, cap
                );
                let lbl = tf!("要塞 {}/{}", "Fort {}/{}", "要塞 {}/{}", nf, cap);
                self.add_button(x0, y0, x1, y1, &lbl, &Self::cost_str(c), 'R' as u32, afford(c) && nf < cap, Act::Place(BKind::Fort), &tip);
            } else {
                self.add_button(x0, y0, x1, y1, t!("停止", "Stop", "停止"), "", 'S' as u32, true, Act::Stop, t!("選択中の部隊を停止して待機させる。", "Stop the selected units and make them wait.", "让所选单位停止待命。"));
            }
            i += 1;

            if any_worker && !any_mil {
                // 下段：選択民および民全体への一括命令（船を造れる国は港）
                let (x0, y0, x1, y1) = pos(6);
                if self.world.can_build_ships(pl) {
                    let c = BKind::Port.cost();
                    self.add_button(x0, y0, x1, y1, BKind::Port.name(), &Self::cost_str(c), 'F' as u32, afford(c), Act::Place(BKind::Port), t!("港：船を造る。海から2マス以内の自国領に建設。船は兵6人を乗せて海を渡る。", "Harbor: builds ships. Build in your territory within 2 tiles of the sea. A ship carries 6 troops across the sea.", "港口：建造船只。建于离海2格以内的己方领土。船可载6名士兵渡海。"));
                } else {
                    self.add_button(x0, y0, x1, y1, t!("自動採集", "Auto gather", "自动采集"), t!("田・森・鉱山へ", "auto-assign", "田・森・矿"), 'F' as u32, true, Act::WorkersAuto, t!("選択中の民を、空いている田・森・鉱山へバランスよく自動配分する。", "Spread the selected workers evenly over free fields, forests and mines.", "将所选的民均衡分配到空闲的田、森林、矿山。"));
                }
                let all_sel = own.len() >= n_workers as usize;
                let (x0, y0, x1, y1) = pos(7);
                if all_sel {
                    self.add_button(x0, y0, x1, y1, t!("全民・伐採", "All: wood", "全民・伐木"), t!("最寄りの森へ", "nearest forest", "去最近的森林"), 'G' as u32, true, Act::AllWorkersWood, t!("すべての民（建設中を除く）に最寄りの森の伐採を命じる。", "Order every worker (except builders) to cut the nearest forest.", "命令全部民（建造中除外）砍伐最近的森林。"));
                } else {
                    self.add_button(x0, y0, x1, y1, t!("全民を選択", "All workers", "选择全民"), &tf!("全{}体", "all {}", "全{}名", n_workers), 'G' as u32, n_workers > 0, Act::SelectAllWorkers, t!("自国のすべての民を選択する。", "Select all your workers.", "选择我国全部民。"));
                }
                let (x0, y0, x1, y1) = pos(8);
                if all_sel {
                    self.add_button(x0, y0, x1, y1, t!("全民・採掘", "All: iron", "全民・采矿"), t!("最寄りの鉱山", "nearest mine", "最近的铁矿"), 'V' as u32, true, Act::AllWorkersMine, t!("すべての民（建設中を除く）に最寄りの鉱山の採掘を命じる。", "Order every worker (except builders) to mine the nearest mine.", "命令全部民（建造中除外）开采最近的铁矿。"));
                } else {
                    self.add_button(x0, y0, x1, y1, t!("全民・自動", "All: auto", "全民・自动"), t!("民全体を配分", "assign everyone", "分配全部民"), 'V' as u32, n_workers > 0, Act::AllWorkersAuto, t!("自国のすべての民（建設中を除く）を田・森・鉱山へ自動でバランス配分する。", "Spread all your workers (except builders) evenly over fields, forests and mines.", "将我国全部民（建造中除外）均衡分配到田、森林、铁矿。"));
                }
            } else if any_mil && !any_worker {
                let (x0, y0, x1, y1) = pos(i);
                self.add_button(x0, y0, x1, y1, t!("進軍", "Attack-move", "进军"), t!("攻撃移動", "fight on the way", "攻击移动"), 'A' as u32, true, Act::AttackMove, t!("クリックした地点へ、途中の敵と戦いながら進む。", "March to the clicked point, fighting enemies on the way.", "向点击的地点进军，沿途与敌交战。"));
                i += 1;
                let (x0, y0, x1, y1) = pos(i);
                self.add_button(x0, y0, x1, y1, t!("防衛・帰還", "Defend", "防卫・归还"), t!("領内迎撃・集結", "guard cities", "领内迎击・集结"), 'D' as u32, true, Act::ArmyDefend, t!("領内の侵入者を迎え撃ち、平時は最寄りの自国城邑へ帰還・防衛配置する。", "Intercept intruders in your land; in peacetime return to the nearest city and guard it.", "迎击领内入侵者，平时返回最近的己方城邑防守。"));
                i += 1;
                let (x0, y0, x1, y1) = pos(i);
                let n_ship = own.iter().filter(|&&h| self.world.unit(h).map_or(false, |u| u.kind.naval())).count();
                let n_land = own.len() - n_ship;
                if n_ship > 0 && n_land > 0 {
                    // 兵と船を一緒に選んでいれば、まとめて乗せる
                    self.add_button(x0, y0, x1, y1, t!("まとめて乗船", "Board all", "全部登船"), &tf!("兵{}・船{}", "{} troops, {} ships", "兵{}・船{}", n_land, n_ship), 'F' as u32, true, Act::BoardAll, t!("選んだ兵を、選んだ船に空きのある近いものから乗せる（1隻に6人）。船は兵の近くの岸へ迎えに来る。選んだ船を右クリックしても同じ", "Put the selected troops aboard the selected ships, nearest ship with room first (6 each). The ships come to the shore near the troops. Right-clicking one of the selected ships does the same", "将所选士兵分别装上所选船只，优先选择有空位的近船（每船6人）。船会驶到士兵附近的岸边迎接。右键点击所选船只亦同"));
                } else {
                    self.add_button(x0, y0, x1, y1, t!("同兵種選択", "Same type", "选择同兵种"), u0_kind.name(), 'F' as u32, true, Act::SelectKind(u0_kind), t!("自国の同じ兵種のユニットをすべて選択する。", "Select all your units of the same type.", "选择我国所有同兵种单位。"));
                }
                // 下段：全軍（軍事ユニット全体）への命令
                let (x0, y0, x1, y1) = pos(6);
                self.add_button(x0, y0, x1, y1, t!("全軍を選択", "All soldiers", "选择全军"), &tf!("全{}体", "all {}", "全{}个", n_army), 'G' as u32, n_army > 0, Act::SelectAllMilitary, t!("自国のすべての軍事ユニットを選択する。", "Select all your soldiers.", "选择我国全部军事单位。"));
                let (x0, y0, x1, y1) = pos(7);
                self.add_button(x0, y0, x1, y1, t!("全軍・進撃", "All: attack", "全军・进击"), t!("全軍で敵陣へ", "whole army", "全军攻敌"), 'Q' as u32, n_army > 0, Act::AllArmyAttack, t!("全軍事ユニットを選択し、交戦国の城邑・敵部隊へ一斉進撃（または地図クリックで進軍先指定）する。", "Select every soldier and advance on enemy cities and armies (or click the map to choose where).", "选择全部军事单位，向交战国城邑、敌军全面进击（或点击地图指定目标）。"));
                let (x0, y0, x1, y1) = pos(8);
                self.add_button(x0, y0, x1, y1, t!("全軍・防衛", "All: defend", "全军・防卫"), t!("全軍で防衛", "whole army", "全军防守"), 'W' as u32, n_army > 0, Act::AllArmyDefend, t!("全軍事ユニットに領内侵入者の迎撃および城邑への防衛帰還を命じる。", "Order every soldier to intercept intruders and return to defend the cities.", "命令全部军事单位迎击入侵者并返回城邑防守。"));
            } else {
                let (x0, y0, x1, y1) = pos(i);
                self.add_button(x0, y0, x1, y1, t!("進軍", "Attack-move", "进军"), t!("攻撃移動", "fight on the way", "攻击移动"), 'A' as u32, true, Act::AttackMove, t!("クリックした地点へ、途中の敵と戦いながら進む。", "March to the clicked point, fighting enemies on the way.", "向点击的地点进军，沿途与敌交战。"));
                let (x0, y0, x1, y1) = pos(7);
                let n_ship = own.iter().filter(|&&h| self.world.unit(h).map_or(false, |u| u.kind.naval())).count();
                let n_land = own.len() - n_ship;
                if n_ship > 0 && n_land > 0 {
                    // 民・兵と船を一緒に選んでいれば、まとめて乗せる
                    self.add_button(x0, y0, x1, y1, t!("まとめて乗船", "Board all", "全部登船"), &tf!("{}人・船{}", "{} units, {} ships", "{}人・船{}", n_land, n_ship), 'G' as u32, true, Act::BoardAll, t!("選んだ民・兵を、選んだ船に空きのある近いものから乗せる（1隻に6人）。船は近くの岸へ迎えに来る。選んだ船を右クリックしても同じ", "Put the selected workers and troops aboard the selected ships, nearest ship with room first (6 each). The ships come to the nearby shore. Right-clicking one of the selected ships does the same", "将所选的民与士兵分别装上所选船只，优先选择有空位的近船（每船6人）。船会驶到附近岸边迎接。右键点击所选船只亦同"));
                } else {
                    self.add_button(x0, y0, x1, y1, t!("全民を選択", "All workers", "选择全民"), &tf!("全{}体", "all {}", "全{}名", n_workers), 'G' as u32, n_workers > 0, Act::SelectAllWorkers, t!("自国のすべての民を選択する。", "Select all your workers.", "选择我国全部民。"));
                }
                let (x0, y0, x1, y1) = pos(8);
                self.add_button(x0, y0, x1, y1, t!("全軍を選択", "All soldiers", "选择全军"), &tf!("全{}体", "all {}", "全{}个", n_army), 'V' as u32, n_army > 0, Act::SelectAllMilitary, t!("自国のすべての軍事ユニットを選択する。", "Select all your soldiers.", "选择我国全部军事单位。"));
            }
        } else if let Some(bh) = self.sel_b {
            let b = self.world.bld(bh).unwrap();
            if b.nation as usize != pl {
                return;
            }
            let (kind, done) = (b.kind, b.done());
            if let Some(cost) = self.world.repair_cost(pl, bh) {
                let (repair_x, _, _, _) = pos(1);
                let (_, _, repair_right, _) = pos(2);
                let label = tf!("補修（{}）", "Repair ({})", "修复（{}）", Self::cost_str(cost));
                self.add_button(repair_x, utility_y, repair_right, utility_bottom, &label, "", 0, afford(cost), Act::Repair,
                    t!("失った耐久10につき木材1（端数切り上げ）で耐久を全回復する。", "Restore all HP for 1 wood per 10 missing HP, rounded up.", "每损失10点耐久消耗1木材（向上取整），恢复全部耐久。"));
            }
            let mut i = 0;
            if done {
                let b = self.world.bld(bh).unwrap();
                let list = self.world.trainable_at(b);
                let keys = ['Q', 'W', 'E', 'R', 'A', 'S', 'D', 'F'];
                // 8種より多ければ頁を分ける（7種＋「次へ」）
                let per = if list.len() > 8 { 7 } else { 8 };
                let pages = (list.len() + per - 1) / per.max(1);
                let page = if pages > 0 { self.train_page % pages } else { 0 };
                for (slot, &(ut, origin)) in list.iter().skip(page * per).take(per).enumerate() {
                    let d = udef(ut);
                    let (x0, y0, x1, y1) = pos(i);
                    let c = self.world.train_cost(pl, ut);
                    let from = if origin != pl { tf!("\n服属：{}の兵（その国の城邑を領しているので作れる）", "\nSubject: a {} unit (available because you hold one of its cities)", "\n臣属：{}的兵种（因占有该国城邑而可训练）", crate::lang::tx(NATIONS[origin].name)) } else { String::new() };
                    let stats = if d.class == UnitKind::Worker {
                        String::new()
                    } else {
                        tf!("\n兵力{:.0}　攻撃{:.0}　射程{:.1}　速さ{:.1}", "\nHP {:.0}   Attack {:.0}   Range {:.1}   Speed {:.1}", "\n兵力{:.0}　攻击{:.0}　射程{:.1}　速度{:.1}", d.hp, d.atk, d.range, d.speed)
                    };
                    let tip = tf!(
                        "{}：{}{}{}\n訓練時間 {:.0}秒　維持費 {}",
                        "{}: {}{}{}\nTraining time {:.0}s   Upkeep {}",
                        "{}：{}{}{}\n训练时间 {:.0}秒　维持费 {}",
                        crate::lang::tx(d.name),
                        crate::lang::tx(d.desc),
                        stats,
                        from,
                        d.time,
                        if d.class.military() { t!("糧0.07/秒×能力", "0.07 food/s × power", "粮0.07/秒×能力") } else { t!("なし", "none", "无") }
                    );
                    let label = if origin != pl { format!("{}*", crate::lang::tx(d.name)) } else { crate::lang::tx(d.name).to_string() };
                    self.add_button(x0, y0, x1, y1, &label, &Self::cost_str(c), keys[slot] as u32, afford(c), Act::Train(ut), &tip);
                    i += 1;
                }
                if pages > 1 {
                    let (x0, y0, x1, y1) = pos(7);
                    self.add_button(x0, y0, x1, y1, t!("次の兵種 ▶", "More units ▶", "更多兵种 ▶"), &format!("{}/{}", page + 1, pages), 'G' as u32, true, Act::TrainPage, t!("作れる兵種の次の頁", "Next page of available units", "可训练兵种的下一页"));
                }
            } else {
                let (x0, y0, x1, y1) = pos(0);
                self.add_button(
                    x0,
                    y0,
                    x1,
                    y1,
                    t!("建設を再開", "Resume building", "继续建造"),
                    t!("民を派遣", "send workers", "派遣民"),
                    'Q' as u32,
                    n_workers > 0,
                    Act::ResumeBuild,
                    t!("近くの民をこの建物の建設へ派遣し、中断した建設を再開（または作業人数を追加）する（Q）", "Send nearby workers to build this, resuming a paused build or adding hands (Q)", "派附近的民来建造此建筑，继续中断的工程或增派人手（Q）"),
                );
            }
            if !kind.city() || !done {
                let (x0, y0, x1, y1) = pos(8);
                self.add_button(x0, y0, x1, y1, t!("取り壊す", "Demolish", "拆除"), "", 0, true, Act::Demolish, t!("この建物を取り壊す（Delete）", "Demolish this building (Delete)", "拆除此建筑（Delete）"));
            }
        } else {
            // 未選択時：民全体・軍事ユニット全体に対する全体命令パネル
            let (x0, y0, x1, y1) = pos(0);
            self.add_button(x0, y0, x1, y1, t!("全民・自動", "All: auto", "全民・自动"), t!("田・森・鉄へ", "auto-assign", "田・森・铁"), 'Q' as u32, n_workers > 0, Act::AllWorkersAuto, t!("自国のすべての民（建設中を除く）を、空いている田・近くの森・鉱山へバランスよく自動配分する。", "Spread all your workers (except builders) evenly over free fields and nearby forests and mines.", "将我国全部民（建造中除外）均衡分配到空闲的田、附近的森林与铁矿。"));
            let (x0, y0, x1, y1) = pos(1);
            self.add_button(x0, y0, x1, y1, t!("全民・伐採", "All: wood", "全民・伐木"), t!("最寄りの森へ", "nearest forest", "去最近的森林"), 'W' as u32, n_workers > 0, Act::AllWorkersWood, t!("自国のすべての民（建設中を除く）に、最寄りの森の伐採を命じる。", "Order all your workers (except builders) to cut the nearest forest.", "命令我国全部民（建造中除外）砍伐最近的森林。"));
            let (x0, y0, x1, y1) = pos(2);
            self.add_button(x0, y0, x1, y1, t!("全民・採掘", "All: iron", "全民・采矿"), t!("最寄りの鉱山", "nearest mine", "最近的铁矿"), 'E' as u32, n_workers > 0, Act::AllWorkersMine, t!("自国のすべての民（建設中を除く）に、最寄りの鉱山の採掘を命じる。", "Order all your workers (except builders) to mine the nearest mine.", "命令我国全部民（建造中除外）开采最近的铁矿。"));

            let (x0, y0, x1, y1) = pos(3);
            self.add_button(x0, y0, x1, y1, t!("全民・教化", "All: educate", "全民・教化"), t!("能力+0.5", "power +0.5", "能力+0.5"), 'T' as u32, n_workers > 0, Act::AllWorkersUpgrade, t!("自国領内の民を、能力の低い順に資源が続く限り一斉に教化する。", "Educate the workers in your territory, weakest first, as far as resources last.", "按能力由低到高，在资源允许范围内一并教化己方领土内的民。"));
            let (x0, y0, x1, y1) = pos(4);
            self.add_button(x0, y0, x1, y1, t!("全民を選択", "All workers", "选择全民"), &tf!("全{}体", "all {}", "全{}名", n_workers), 'G' as u32, n_workers > 0, Act::SelectAllWorkers, t!("自国のすべての民を選択する（建設や一斉移動を行えます）。", "Select all your workers (to build or move them together).", "选择我国全部民（可进行建造或统一移动）。"));
            let (x0, y0, x1, y1) = pos(5);
            self.add_button(x0, y0, x1, y1, t!("全軍を選択", "All soldiers", "选择全军"), &tf!("全{}体", "all {}", "全{}个", n_army), 'V' as u32, n_army > 0, Act::SelectAllMilitary, t!("自国のすべての軍事ユニットを選択する。", "Select all your soldiers.", "选择我国全部军事单位。"));

            let (x0, y0, x1, y1) = pos(6);
            self.add_button(x0, y0, x1, y1, t!("全軍・進撃", "All: attack", "全军・进击"), t!("敵城・目標へ", "enemy cities", "敌城・目标"), 'A' as u32, n_army > 0, Act::AllArmyAttack, t!("全軍事ユニットを選択し、交戦国の最寄り城邑・敵部隊へ総進撃（または地図クリックで進軍先指定）する。", "Select every soldier and advance on the nearest enemy city and armies (or click the map to choose where).", "选择全部军事单位，向交战国最近的城邑、敌军总攻（或点击地图指定目标）。"));
            let (x0, y0, x1, y1) = pos(7);
            self.add_button(x0, y0, x1, y1, t!("全軍・防衛", "All: defend", "全军・防卫"), t!("領内迎撃・帰還", "guard cities", "领内迎击・归还"), 'D' as u32, n_army > 0, Act::AllArmyDefend, t!("全軍事ユニットに、領内の侵入者の迎撃または自国城邑への防衛帰還を命じる。", "Order every soldier to intercept intruders or return to defend your cities.", "命令全部军事单位迎击入侵者或返回己方城邑防守。"));
            let (x0, y0, x1, y1) = pos(8);
            self.add_button(x0, y0, x1, y1, t!("全軍・訓練", "All: train", "全军・训练"), t!("能力+0.5", "power +0.5", "能力+0.5"), 'R' as u32, n_trainable > 0, Act::AllArmyUpgrade, t!("自国領内の軍事ユニットを、能力の低い順に資源が続く限り一斉に訓練する。", "Train the soldiers in your territory, weakest first, as far as resources last.", "按能力由低到高，在资源允许范围内一并训练己方领土内的军事单位。"));
        }
    }

    fn render_diplo(&mut self) {
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        let pl = self.world.player;
        let pw = 960.0f32.min(w - 20.0);
        let ph = 620.0f32.min(h - 60.0);
        let x0 = (w - pw) / 2.0;
        let y0 = (h - ph) / 2.0;
        self.gfx.fill_rect(0.0, 0.0, w, h, [0.0, 0.0, 0.0, 0.45]);
        self.art.draw_stone_panel(&self.gfx, x0, y0, x0 + pw, y0 + ph);
        self.art.draw_corner_ornaments(&self.gfx, x0 + 3.0, y0 + 3.0, x0 + pw - 3.0, y0 + ph - 3.0, 16.0);
        if let Some(ic) = &self.art.cmd_icons[CmdIcon::Diplo as usize] {
            self.gfx.draw_bitmap(ic, x0 + 16.0, y0 + 8.0, x0 + 48.0, y0 + 40.0);
        }
        self.gfx.text_shadow(t!("諸侯外交", "Diplomacy", "诸侯外交"), x0 + 56.0, y0 + 10.0, 200.0, 34.0, Font::Big, [1.0, 0.85, 0.5, 1.0]);
        let my = self.world.nations[pl].strength;
        let s = tf!("自国の国力 {:.0}　（国力は兵の能力・街・村・民・領土から算出）", "Our strength {:.0}  (from soldiers' power, cities, workers and land)", "我国国力 {:.0}　（国力由兵力、城邑、民、领土计算）", my);
        self.gfx.text(&s, x0 + 160.0, y0 + 16.0, pw - 370.0, 24.0, Font::Normal, [0.85, 0.82, 0.75, 1.0]);
        self.add_button(x0 + pw - 90.0, y0 + 10.0, x0 + pw - 14.0, y0 + 40.0, t!("閉じる", "Close", "关闭"), "", 0, true, Act::CloseOverlay, "");
        self.add_button(x0 + pw - 196.0, y0 + 10.0, x0 + pw - 98.0, y0 + 40.0, t!("歴史 F3", "History F3", "历史 F3"), "", 0, true, Act::Chron, t!("英雄・諸子百家・政治指導者、反乱・簒奪の記録と各国の君主家", "Records of heroes, philosophers, statesmen, revolts and usurpations, and each state's ruling house", "英雄、诸子百家、政治家、叛乱与篡位的记录及各国君主家"));
        self.art.draw_inset_box(&self.gfx, x0 + 10.0, y0 + 48.0, x0 + pw - 10.0, y0 + ph - 10.0);
        let home = self.world.nations[pl].home;
        let mut list: Vec<usize> = (0..NATIONS.len()).filter(|&i| i != pl && self.world.nations[i].alive).collect();
        list.sort_by(|&a, &b| self.world.nations[a].home.dist(home).partial_cmp(&self.world.nations[b].home.dist(home)).unwrap());
        let cols = 2;
        let colw = (pw - 30.0) / cols as f32;
        let rowh = 25.0;
        let rows_per = ((ph - 70.0) / rowh) as usize;
        for (k, &o) in list.iter().enumerate() {
            let c = k / rows_per;
            if c >= cols {
                break;
            }
            let r = k % rows_per;
            let rx = x0 + 15.0 + c as f32 * colw;
            let ry = y0 + 54.0 + r as f32 * rowh;
            let n = &self.world.nations[o];
            let war = self.world.at_war(pl, o);
            let truce = self.world.truce[pl][o] - self.world.years;
            let locked = self.world.coalition_locked(pl, o);
            let offer = self.world.peace_offer[o];
            if r % 2 == 0 {
                self.gfx.fill_rect(rx, ry, rx + colw - 8.0, ry + rowh, [1.0, 1.0, 1.0, 0.04]);
            }
            self.gfx.fill_rect(rx + 4.0, ry + 5.0, rx + 18.0, ry + 19.0, c4(n.color, 1.0));
            let name = format!("{}{}", crate::lang::tx(n.name), if NATIONS[o].major { "★" } else { "" });
            self.gfx.text(&name, rx + 24.0, ry + 3.0, 100.0, 20.0, Font::Normal, [0.95, 0.92, 0.85, 1.0]);
            let st = tf!("国力{:.0} 城{} 距離{:.0}", "Str {:.0} Cities {} Dist {:.0}", "国力{:.0} 城{} 距离{:.0}", n.strength, n.cities, n.home.dist(home));
            self.gfx.text(&st, rx + 126.0, ry + 5.0, 146.0, 20.0, Font::Small, [0.75, 0.72, 0.66, 1.0]);
            let (rel, rc) = if war {
                if locked {
                    (t!("交戦中（連合）", "At war (coalition)", "交战中（联军）"), [1.0, 0.35, 0.3, 1.0])
                } else if offer {
                    (t!("和睦の申し入れ", "Peace offered", "议和请求"), [0.6, 0.9, 1.0, 1.0])
                } else {
                    (t!("交戦中", "Fighting", "交战中"), [1.0, 0.45, 0.35, 1.0])
                }
            } else if truce > 0.0 {
                (t!("休戦中", "Truce", "休战中"), [0.7, 0.9, 0.7, 1.0])
            } else {
                (t!("平和", "Peace", "和平"), [0.75, 0.75, 0.7, 1.0])
            };
            let rel = if !war && truce > 0.0 { tf!("休戦 残{}年", "Truce, {} yrs", "休战 剩{}年", truce.ceil() as i32) } else { rel.to_string() };
            self.gfx.text(&rel, rx + 274.0, ry + 4.0, colw - 364.0, 20.0, Font::Small, rc);
            let bx = rx + colw - 82.0;
            if war {
                let lbl = if offer { t!("受諾", "Accept", "接受") } else { t!("和睦", "Peace", "议和") };
                self.add_button(bx, ry + 2.0, bx + 70.0, ry + rowh - 2.0, lbl, "", 0, !locked, Act::Peace(o), "");
            } else {
                self.add_button(bx, ry + 2.0, bx + 70.0, ry + rowh - 2.0, t!("宣戦", "War", "宣战"), "", 0, truce <= 0.0, Act::War(o), "");
            }
        }
    }

    /// 和睦の申し入れ：受けるか退けるかを選ぶ（返答するまで時間は止まる）
    fn render_peace_ask(&mut self) {
        let Some(q) = self.world.peace_asks.front().cloned() else { return };
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        let pl = self.world.player;
        let o = q.from;
        let pw = 520.0f32.min(w - 20.0);
        let ph = 250.0;
        let x0 = (w - pw) / 2.0;
        let y0 = (h - ph) / 2.0 - 40.0;
        self.gfx.fill_rect(0.0, 0.0, w, h, [0.0, 0.0, 0.0, 0.35]);
        self.art.draw_stone_panel(&self.gfx, x0, y0, x0 + pw, y0 + ph);
        self.art.draw_corner_ornaments(&self.gfx, x0 + 3.0, y0 + 3.0, x0 + pw - 3.0, y0 + ph - 3.0, 16.0);
        if let Some(ic) = &self.art.cmd_icons[CmdIcon::Diplo as usize] {
            self.gfx.draw_bitmap(ic, x0 + 16.0, y0 + 8.0, x0 + 48.0, y0 + 40.0);
        }
        if let Some((hero, amount)) = q.ransom {
            self.gfx.text_shadow(t!("身代金の請求", "Ransom demanded", "索要赎金"), x0 + 56.0, y0 + 10.0, 260.0, 34.0, Font::Big, [1.0, 0.85, 0.5, 1.0]);
            self.art.draw_inset_box(&self.gfx, x0 + 12.0, y0 + 48.0, x0 + pw - 12.0, y0 + ph - 62.0);
            let n = &self.world.nations[o];
            self.gfx.fill_rect(x0 + 24.0, y0 + 62.0, x0 + 40.0, y0 + 78.0, c4(n.color, 1.0));
            let s = tf!("{0} が {1} の身代金として金貨{2:.0}を求めてきた", "{0} demands {2:.0} gold as ransom for {1}", "{0}索要金币{2:.0}作为{1}的赎金", crate::lang::tx(n.name), crate::lang::tx(HEROES[hero].name), amount);
            self.gfx.text(&s, x0 + 48.0, y0 + 58.0, pw - 70.0, 48.0, Font::Normal, [0.95, 0.92, 0.85, 1.0]);
            let coin = self.world.nations[pl].coin;
            let s = tf!("自国の金貨 {:.0}
払えば英雄は自国の街に戻る（史実の没年までは）。払わなければ捕虜のまま。", "Our gold: {:.0}
Pay and the hero returns to one of our towns (until their historical death). Refuse and they stay a captive.", "我国金币 {:.0}
支付后英雄将回到我国城镇（直至史实卒年）。拒付则仍为俘虏。", coin);
            self.gfx.text(&s, x0 + 24.0, y0 + 110.0, pw - 48.0, 76.0, Font::Small, [0.82, 0.8, 0.74, 1.0]);
            let bw = (pw - 60.0) / 2.0;
            let by = y0 + ph - 52.0;
            let lbl = tf!("払う（金貨{:.0}）", "Pay ({:.0} gold)", "支付（金币{:.0}）", amount);
            self.add_button(x0 + 20.0, by, x0 + 20.0 + bw, by + 38.0, &lbl, "", 'Y' as u32, coin >= amount, Act::PeaceAnswer(true), t!("身代金を払って英雄を取り戻す（Y）", "Pay the ransom and get the hero back (Y)", "支付赎金赎回英雄（Y）"));
            self.add_button(x0 + 40.0 + bw, by, x0 + 40.0 + 2.0 * bw, by + 38.0, t!("払わない", "Refuse", "拒付"), "", 'N' as u32, true, Act::PeaceAnswer(false), t!("身代金を払わない（N）", "Refuse to pay (N)", "拒付赎金（N）"));
            return;
        }
        self.gfx.text_shadow(t!("和睦の申し入れ", "Peace offered", "议和请求"), x0 + 56.0, y0 + 10.0, 260.0, 34.0, Font::Big, [1.0, 0.85, 0.5, 1.0]);
        let rest = self.world.peace_asks.len();
        if rest > 1 {
            let s = tf!("ほか{}件", "{} more", "另有{}件", rest - 1);
            self.gfx.text(&s, x0 + pw - 110.0, y0 + 18.0, 96.0, 20.0, Font::Small, [0.8, 0.78, 0.7, 1.0]);
        }
        self.art.draw_inset_box(&self.gfx, x0 + 12.0, y0 + 48.0, x0 + pw - 12.0, y0 + ph - 62.0);
        let n = &self.world.nations[o];
        self.gfx.fill_rect(x0 + 24.0, y0 + 62.0, x0 + 40.0, y0 + 78.0, c4(n.color, 1.0));
        let s = tf!("{} が和睦を申し入れてきた（{}）", "{} offers peace ({})", "{}提出议和（{}）", crate::lang::tx(n.name), q.why);
        self.gfx.text(&s, x0 + 48.0, y0 + 58.0, pw - 70.0, 24.0, Font::Normal, [0.95, 0.92, 0.85, 1.0]);
        let dur = (self.world.years - self.world.war_start[pl][o]).max(0.0);
        let s = tf!(
            "交戦{}年　国力：自国 {:.0}　{} {:.0}\n受諾すると{}年間休戦し、その間は互いに宣戦できない。\n退けると交戦を続ける。",
            "At war for {} years.  Strength: us {:.0}, {} {:.0}\nAccepting brings a {}-year truce in which neither side may declare war.\nRefusing continues the war.",
            "交战{}年　国力：我国 {:.0}　{} {:.0}\n接受则休战{}年，其间双方均不能宣战。\n拒绝则继续交战。",
            dur.floor() as i32,
            self.world.nations[pl].strength,
            crate::lang::tx(n.name),
            n.strength,
            q.years as i32
        );
        self.gfx.text(&s, x0 + 24.0, y0 + 88.0, pw - 48.0, 96.0, Font::Small, [0.82, 0.8, 0.74, 1.0]);
        let bw = (pw - 60.0) / 2.0;
        let by = y0 + ph - 52.0;
        let lbl = tf!("受諾（休戦{}年）", "Accept ({}-year truce)", "接受（休战{}年）", q.years as i32);
        self.add_button(x0 + 20.0, by, x0 + 20.0 + bw, by + 38.0, &lbl, "", 'Y' as u32, true, Act::PeaceAnswer(true), t!("和睦して休戦する（Y）", "Make peace and agree a truce (Y)", "议和并休战（Y）"));
        self.add_button(x0 + 40.0 + bw, by, x0 + 40.0 + 2.0 * bw, by + 38.0, t!("拒否（交戦を続ける）", "Refuse (keep fighting)", "拒绝（继续交战）"), "", 'N' as u32, true, Act::PeaceAnswer(false), t!("申し入れを退けて戦を続ける（N）", "Refuse the offer and keep fighting (N)", "拒绝请求，继续作战（N）"));
    }

    fn render_help(&mut self) {
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        let pw = 1180.0f32.min(w - 20.0);
        let ph = (h - 20.0).max(300.0);
        let x0 = (w - pw) / 2.0;
        let y0 = (h - ph) / 2.0;
        self.gfx.fill_rect(0.0, 0.0, w, h, [0.0, 0.0, 0.0, 0.45]);
        self.art.draw_stone_panel(&self.gfx, x0, y0, x0 + pw, y0 + ph);
        self.art.draw_corner_ornaments(&self.gfx, x0 + 3.0, y0 + 3.0, x0 + pw - 3.0, y0 + ph - 3.0, 16.0);
        if let Some(ic) = &self.art.cmd_icons[CmdIcon::Help as usize] {
            self.gfx.draw_bitmap(ic, x0 + 16.0, y0 + 8.0, x0 + 48.0, y0 + 40.0);
        }
        self.gfx.text_shadow(t!("兵法・遊び方", "How to Play", "兵法・玩法"), x0 + 56.0, y0 + 10.0, 220.0, 34.0, Font::Big, [1.0, 0.85, 0.5, 1.0]);
        self.add_button(x0 + pw - 90.0, y0 + 10.0, x0 + pw - 14.0, y0 + 40.0, t!("閉じる", "Close", "关闭"), "", 0, true, Act::CloseOverlay, "");
        #[cfg(target_arch = "wasm32")]
        self.add_button(x0 + pw - 310.0, y0 + 10.0, x0 + pw - 100.0, y0 + 40.0,
            t!("キャッシュを更新して再読み込み", "Refresh cache and reload", "刷新缓存并重新加载"),
            "", 0, true, Act::RefreshCache, "");
        self.art.draw_inset_box(&self.gfx, x0 + 14.0, y0 + 46.0, x0 + pw - 14.0, y0 + ph - 14.0);
        let text = t!(
            "■ 目的　1337年、百年戦争が始まる。フランスなら、1453年（史実で戦が終わった年）が暮れるまでに、大陸（ブリテン島の外）からイングランドの街をすべて取り除けば勝利。イングランドなら、フランスの街をすべて取れば勝利。どちらも果たせなければ、ジャンヌ・ダルクの亡霊が天使の軍勢を引き連れて現れ、5年のうちに両国を滅ぼす（敗北）。時間は1年＝30秒（1415年のランカスター朝の戦争からは39秒）。\n\n■ 街と村　主要な街は最初からあり、新しく築けない。街は攻め落とすか、諸侯の寝返り・同盟で持ち主が変わる。民は「村」（E）を自国領内に建てられる。村は糧を生み、資源の運び先になり、民の上限を1、兵の上限を3上げる。\n■ 略奪　敵の村を攻めて壊すと略奪になり、糧・木・鉄・金貨を奪える（イングランドは騎行で多く奪う）。スコットランド軍や大傭兵団も村を荒らす。\n■ 諸侯　街にはそれぞれ治める諸侯がいる（フランドル、ブルターニュ、ナバラ王、ブルゴーニュ公、ガスコーニュ諸侯、パリ市民、シャンパーニュの諸都市、ウェールズなど）。史実の年に諸侯が寝返ったり同盟したりすると、その街はもう一方の王に付く（たとえば1419年のブルゴーニュ公の同盟、1435年のアラスの和約）。ジャンヌ・ダルクがいれば1429年にシャンパーニュの街が王太子に城門を開く。\n\n■ 資源　糧（田）・木（森）・鉄（鉱山）。民が街や村へ運ぶ。街と村からも税が入る。金貨は商人の税・略奪・身代金で得る。\n■ ユニット　数には上限がある（街と村の数で増える）。「訓練／教化」で一体ごとの能力を高められる。兵は糧を食べる（維持費）。\n　歩兵（槍）＞騎兵＞弓兵＞歩兵 の相性。イングランドのロングボウ兵は射程が長く騎兵に強い。フランス騎士の突撃は重い。1375年からは大砲、1445年からは勅令隊、1448年からはフラン・アルシェが加わる。\n■ 英雄　エドワード黒太子、デュ・ゲクラン、ヘンリー5世、ジャンヌ・ダルク、タルボットなど、実在の英雄が史実の年に現れる。英雄のそばの兵は攻撃が上がる。英雄は史実の没年を越えて生きられない。\n■ 身代金　討たれた英雄は捕らえられる。捕らえた英雄は「英雄と捕虜」（H）から相手国に身代金を請求できる。自国の英雄が捕らえられると、相手国が身代金を求めてくる。ジャンヌ・ダルクだけは身代金で救えない。\n■ 船と海　海峡を渡るには船が要る。イングランドは港（海辺）でコグ船を造る。フランスは船を造れず、交易画面（F4）でジェノヴァ（1369年からはカスティーリャも）のガレー船を雇う。海の向こうへ移動・攻撃を命じれば、兵は岸で待ち、船が自動で運ぶ（1隻に6人）。\n\n■ 外交　F2で外交画面。両国は最初から戦っている。休戦（ブレティニー、ルランゲムなど）の申し入れは受けるか退けるか選べる。\n■ スコットランド　国ではないが、史実の時期（ネヴィルズ・クロス、オッターバーン、ホミルドン・ヒルなど）にイングランドの北へ攻め込む。1419年と1424年にはフランスに兵を送る。\n■ 黒死病・反乱　疫病（1348年）や反乱（ジャックリー、ワット・タイラー）で民と兵を失う。\n■ 要塞　民だけを選ぶと「要塞（R）」で築ける。数は「1＋街の数÷3」まで（最大6）。\n■ 交易（F4）　商人は領内を通るだけで金貨を納める。遠方の商人は羊毛の道・ワインの道・イタリア商人・ハンザ商人の先に来る。金貨で資源を買い、傭兵を雇える。\n■ 指揮中は時が止まる　自国の部隊を選ぶと時間が止まり、命令を出すと進む。\n■ 戦場の霧　自国の民・兵・建物の周りしか見えない。\n\n■ 操作\n　左クリック：選択　ドラッグ：範囲選択　ダブルクリック：同種を全選択　Shift：追加選択\n　右クリック：移動／攻撃／採集／建設の手伝い／乗船（兵で船を）／上陸（船で陸を）　建物選択中の右クリック：集合地点\n　A：進軍　S：停止　T：訓練・教化　C：兵⇔民（船は上陸）　Q/W/E…：建設・訓練　H：英雄と捕虜\n　Ctrl を押しながら建物を置く：建設の予約\n　Ctrl+数字：部隊登録　数字：部隊呼び出し　.：待機中の民　Home：都へ\n　ホイール：拡大縮小　中ボタンドラッグ・矢印キー・画面端：スクロール　ミニマップクリック：移動\n　Space：一時停止　＋／－：速度　Esc：告知を閉じる・取り消し　F5：記録　F11：全画面　Ctrl＋＋／Ctrl＋−：文字の大きさ",
            "■ Goal  The Hundred Years' War begins in 1337. As France, win by removing every English town from the Continent (outside Great Britain) before the end of 1453, the year the war historically ended. As England, win by taking every French town. If neither happens, the ghost of Joan of Arc appears with a host of angels and destroys both kingdoms within 5 years (defeat). One year = 30 s (39 s from the Lancastrian War of 1415).\n\n■ Towns and villages  The great towns exist from the start and cannot be founded. Towns change hands by siege, or when their lords change sides or make alliances. Workers can build Villages (E) inside your territory: they produce food, receive resources and raise the worker limit by 1 and soldier limit by 3.\n■ Plunder  Attack and destroy an enemy village to plunder it for food, wood, iron and gold (England takes more on the chevauchée). The Scots and the Great Companies ravage villages too.\n■ Lords  Every town has a lord (Flanders, Brittany, the King of Navarre, the Duke of Burgundy, the Gascon lords, the citizens of Paris, the towns of Champagne, Wales...). When a lord changes sides or makes an alliance in the historical year, those towns go over to the other king (for example Burgundy's alliance with England in 1419 and the Congress of Arras in 1435). If Joan of Arc is alive in 1429, the towns of Champagne open their gates to the Dauphin.\n\n■ Resources  Food (fields), wood (forests), iron (mines). Workers carry them to towns and villages, which also pay taxes. Gold comes from merchant taxes, plunder and ransoms.\n■ Units  There is a limit (raised by towns and villages). Train / Educate raises each unit's power. Soldiers eat food (upkeep).\n  Counters: infantry (spears) > cavalry > archers > infantry. English longbowmen shoot far and are deadly to cavalry; the French knights' charge hits hardest. Cannon arrive in 1375, the Compagnies d'ordonnance in 1445 and the francs-archers in 1448.\n■ Heroes  Real heroes — the Black Prince, Du Guesclin, Henry V, Joan of Arc, Talbot and more — appear in their historical years. Troops near a hero attack harder. No hero lives past their historical death.\n■ Ransom  A defeated hero is captured. Demand a ransom for your captives from the enemy on the Heroes screen (H). If one of your heroes is captured, the enemy demands a ransom from you. Only Joan of Arc cannot be ransomed.\n■ Ships and the sea  Crossing the Channel needs ships. England builds cogs at a Harbor on the coast. France cannot build ships and hires Genoese galleys (and Castilian ones from 1369) on the Trade screen (F4). Order troops across the sea and they wait on the shore while ships ferry them automatically (6 per ship).\n\n■ Diplomacy  F2 opens Diplomacy. The two kingdoms are at war from the start. You may accept or refuse historical truces (Brétigny, Leulinghem...).\n■ Scotland  Not a kingdom in this game, but in its historical years (Neville's Cross, Otterburn, Homildon Hill...) it invades the north of England, and in 1419 and 1424 it sends troops to France.\n■ Plague and revolt  The Black Death (1348) and revolts (the Jacquerie, Wat Tyler) cost workers and soldiers.\n■ Forts  With only workers selected, press Fort (R). You may have 1 + towns ÷ 3 of them (6 at most).\n■ Trade (F4)  Merchants pay gold just by passing through your land. Foreign merchants come by the Wool Road, the Wine Road, the Italian and the Hanseatic routes. Gold buys resources and mercenaries.\n■ Time stops while commanding  Selecting your own units stops time; it resumes when you give an order.\n■ Fog of war  You see only around your own workers, soldiers and buildings.\n\n■ Controls\n  Left click: select   Drag: box select   Double click: select all of a type   Shift: add to selection\n  Right click: move / attack / gather / help build / board (troops on a ship) / land (ship on land)   Right click with a building selected: rally point\n  A: attack-move   S: stop   T: train / educate   C: soldier ⇔ worker (ships: land troops)   Q/W/E...: build / train   H: heroes & captives\n  Hold Ctrl while placing buildings: reserve builds\n  Ctrl+number: save group   Number: recall group   .: idle worker   Home: go to capital\n  Wheel: zoom   Middle drag / arrow keys / screen edge: scroll   Minimap click: jump\n  Space: pause   + / -: speed   Esc: close notice / cancel   F5: save / load   F11: fullscreen   Ctrl++ / Ctrl+−: text size",
            "■ 目标　百年战争始于1337年。若为法兰西，须在1453年（史实中战争结束之年）年底之前，将英格兰的城镇全部逐出大陆（大不列颠岛以外）即获胜。若为英格兰，夺取法兰西全部城镇即获胜。若双方都未能达成，贞德的亡灵将率领天使大军现身，5年之内毁灭两国（败北）。1年＝30秒（1415年兰开斯特王朝战争起为39秒）。\n\n■ 城镇与村庄　主要城镇一开始就存在，无法新建。城镇可通过攻城，或因诸侯倒戈、结盟而易主。民可在己方领土内建造“村庄”（E）：产粮、接收资源，并使民的上限+1、兵的上限+3。\n■ 劫掠　攻击并摧毁敌方村庄即为劫掠，可夺取粮、木、铁与金币（英格兰在骑行中夺得更多）。苏格兰军与大雇佣兵团也会蹂躏村庄。\n■ 诸侯　每座城镇都有其领主（佛兰德斯、布列塔尼、纳瓦拉国王、勃艮第公爵、加斯科涅诸侯、巴黎市民、香槟诸城、威尔士等）。诸侯在史实年份倒戈或结盟时，其城镇便归附另一位国王（例如1419年勃艮第公爵与英格兰结盟、1435年阿拉斯和约）。若贞德在世，1429年香槟诸城会向王太子打开城门。\n\n■ 资源　粮（田）・木（森林）・铁（矿山）。民将其运往城镇或村庄，城镇与村庄也会缴税。金币来自商人税、劫掠与赎金。\n■ 单位　数量有上限（随城镇与村庄增加）。“训练／教化”可提升每个单位的能力。士兵会消耗粮（维持费）。\n　步兵（枪）＞骑兵＞弓兵＞步兵 的克制关系。英格兰长弓兵射程远、克制骑兵；法兰西骑士的冲锋最为沉重。1375年起有大炮，1445年起有敕令骑士团，1448年起有自由弓兵。\n■ 英雄　黑太子、迪盖克兰、亨利五世、贞德、塔尔博特等真实英雄会在史实年份登场。英雄身边的士兵攻击提升。英雄无法活过史实卒年。\n■ 赎金　被击败的英雄会被俘。可在“英雄与俘虏”（H）界面向敌国索要赎金。己方英雄被俘时，敌国会索要赎金。唯有贞德无法用赎金赎回。\n■ 船与海　渡过海峡需要船。英格兰可在海边的港口建造柯克船。法兰西无法造船，需在贸易界面（F4）雇佣热那亚（1369年起还有卡斯蒂利亚）的桨帆船。命令部队前往海对岸，士兵会在岸边等待，船只自动运送（每船6人）。\n\n■ 外交　F2打开外交界面。两国一开始就处于交战状态。可选择接受或拒绝史实中的休战（布雷蒂尼、勒兰冈等）。\n■ 苏格兰　本作中不作为国家，但会在史实时期（内维尔十字、奥特本、霍米尔顿山等）入侵英格兰北部，并于1419年与1424年派兵援助法兰西。\n■ 黑死病与叛乱　瘟疫（1348年）与叛乱（扎克雷起义、瓦特·泰勒起义）会使民与士兵减少。\n■ 要塞　只选中民时按“要塞（R）”即可修建。数量上限为“1＋城镇数÷3”（最多6座）。\n■ 贸易（F4）　商人仅途经领内即缴纳金币。远方商人经羊毛之路、葡萄酒之路、意大利商人与汉萨商人之路而来。金币可购买资源与雇佣佣兵。\n■ 指挥中时间停止　选中己方部队时时间停止，下达命令后继续。\n■ 战争迷雾　只能看到己方民、士兵与建筑周围。\n\n■ 操作\n　左键：选择　拖动：框选　双击：选择全部同类　Shift：追加选择\n　右键：移动／攻击／采集／协助建造／登船（士兵点船）／登陆（船点陆地）　选中建筑时右键：集结点\n　A：进军　S：停止　T：训练・教化　C：兵⇔民（船为登陆）　Q/W/E…：建造・训练　H：英雄与俘虏\n　按住 Ctrl 放置建筑：预约建造\n　Ctrl+数字：编队　数字：调出编队　.：空闲的民　Home：回到都城\n　滚轮：缩放　中键拖动・方向键・屏幕边缘：滚动　点击小地图：移动\n　Space：暂停　＋／－：速度　Esc：关闭通知・取消　F5：存档　F11：全屏　Ctrl＋＋／Ctrl＋−：文字大小"
        );
        // 入りきらなければ小さい字にする
        let f = if self.gfx.measure_h(text, Font::Normal, pw - 48.0) > ph - 64.0 { Font::Small } else { Font::Normal };
        self.gfx.text(text, x0 + 24.0, y0 + 52.0, pw - 48.0, ph - 64.0, f, [0.92, 0.9, 0.84, 1.0]);
    }

    fn render_gameover(&mut self) {
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        let Some((win, text)) = self.world.game_over.clone() else { return };
        self.gfx.fill_rect(0.0, 0.0, w, h, [0.0, 0.0, 0.0, 0.55]);
        let pw = 640.0;
        let x0 = (w - pw) / 2.0;
        let y0 = h / 2.0 - 140.0;
        self.art.draw_stone_panel(&self.gfx, x0, y0, x0 + pw, y0 + 260.0);
        self.art.draw_corner_ornaments(&self.gfx, x0 + 3.0, y0 + 3.0, x0 + pw - 3.0, y0 + 257.0, 16.0);
        let title = if win {
            t!("勝利", "Victory", "胜利")
        } else if self.world.descent.is_some() {
            t!("乙女の裁き", "The Maid's Judgement", "少女的审判")
        } else {
            t!("敗北", "Defeat", "败北")
        };
        self.gfx.text_shadow(title, x0, y0 + 20.0, pw, 60.0, Font::Title, if win { [1.0, 0.85, 0.4, 1.0] } else { [0.9, 0.4, 0.35, 1.0] });
        self.gfx.text(&text, x0 + 20.0, y0 + 100.0, pw - 40.0, 60.0, Font::BigCenter, [0.95, 0.92, 0.85, 1.0]);
        self.add_button(x0 + pw / 2.0 - 210.0, y0 + 180.0, x0 + pw / 2.0 - 10.0, y0 + 228.0, t!("新しい戦へ", "New game", "新的战争"), "", 0, true, Act::Restart, "");
        self.add_button(x0 + pw / 2.0 + 10.0, y0 + 180.0, x0 + pw / 2.0 + 210.0, y0 + 228.0, t!("記録から再開 F5", "Load a save F5", "读取存档 F5"), "", 0, true, Act::SaveMenu, "");
    }
}
