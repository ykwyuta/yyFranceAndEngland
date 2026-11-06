//! ジャンヌ・ダルクの亡霊：1453年（史実で百年戦争が終わった年）が暮れても決着がつかなければ、
//! ルーアンで火刑に処された乙女の亡霊が天の軍勢を引き連れて現れ、戦をやめなかった両国を滅ぼす（強制終了）

use super::*;

/// 天の軍の仮の枠（国としては扱わない。どの国とも戦う）
pub const HEAVEN: usize = NATIONS.len() + 1;
/// 天の軍勢が両国を滅ぼし尽くすまでの年数
pub const DESCENT_YEARS: f32 = 5.0;
/// 亡霊が現れる地（ジャンヌが火刑に処されたルーアン）
pub const ROUEN: (f32, f32) = (1.10, 49.44);
/// 天使の能力（誰も傷つけられない）
const DIVINE_POWER: f32 = 1.0;
/// 同時にいる天使の上限
const DIVINE_MAX: usize = 70;

pub struct Descent {
    /// 現れた年（経過年）
    pub start: f32,
    /// 現れた時点で両国が持っていた街の数
    pub total: usize,
    pub taken: usize,
}

impl World {
    /// 残り年数（両国が滅びるまで）
    pub fn descent_left(&self) -> Option<f32> {
        self.descent.as_ref().map(|d| (d.start + DESCENT_YEARS - self.years).max(0.0))
    }

    /// 亡霊が現れる地点
    fn ghost_home(&self) -> V2 {
        let (x, y) = ll_to_tile(ROUEN.0, ROUEN.1);
        let p = v2(x, y);
        if self.passable_at(p) {
            return p;
        }
        self.land_near(p, 6).first().copied().unwrap_or(p)
    }

    /// 1454年：ジャンヌ・ダルクの亡霊が天の軍勢を連れて現れる
    pub(super) fn begin_descent(&mut self) {
        self.coalitions.clear();
        self.peace_asks.clear();
        let total = self.blds.iter().filter(|b| b.alive && b.kind.city() && b.done() && (b.nation as usize) < NATIONS.len()).count();
        self.descent = Some(Descent { start: self.years, total, taken: 0 });
        self.nations[HEAVEN].home = self.ghost_home();
        // 亡霊と天使
        let home = self.nations[HEAVEN].home;
        let gh = self.spawn_unit(HEAVEN, UT_GHOST, home, DIVINE_POWER);
        if let Some(u) = self.unit_mut(gh) {
            u.divine = true;
        }
        self.spawn_angels(home, 12);
        let text = t!("1453年は暮れ、百年の戦はなお終わらない。ルーアンで灰となった乙女の亡霊が、天使の軍勢を引き連れて現れた。「主の名において、この戦を終わらせる」。天の軍勢には人の剣も矢も届かず、5年のうちにイングランドもフランスも滅ぼされる。", "1453 has ended, and still the war of a hundred years goes on. The ghost of the Maid, burned to ashes at Rouen, has appeared at the head of a host of angels: \"In the name of God, this war shall end.\" No sword or arrow can touch the hosts of Heaven; within 5 years both England and France will be destroyed.", "1453年已经过去，百年的战争仍未结束。在鲁昂化为灰烬的少女亡灵，率领天使大军现身：“以主之名，结束这场战争。”人间的刀剑弓矢无法触及天之军势，5年之内，英格兰与法兰西都将灭亡。").to_string();
        self.push_msg(t!("ジャンヌ・ダルクの亡霊が天使の軍勢を連れて現れた", "The ghost of Joan of Arc has appeared with a host of angels", "贞德的亡灵率领天使大军现身").to_string(), [1.0, 0.92, 0.6]);
        self.banner = None;
        self.banner_queue.clear();
        self.queue_banner(Banner { title: t!("乙女の亡霊", "The Ghost of the Maid", "少女的亡灵").to_string(), text, t: 12.0 });
    }

    /// 天使を出す
    fn spawn_angels(&mut self, at: V2, n: usize) {
        let have = self.units.iter().filter(|u| u.alive && u.divine).count();
        let spots = self.land_near(at, 3);
        for k in 0..n.min(DIVINE_MAX.saturating_sub(have)) {
            let Some(&sp) = spots.get(k % spots.len().max(1)) else { break };
            let h = self.spawn_unit(HEAVEN, UT_ANGEL, sp, DIVINE_POWER);
            self.unit_mut(h).unwrap().divine = true;
        }
    }

    /// 天の軍勢が街を落とした：街は灰となる
    pub(super) fn smite_town(&mut self, bi: usize) {
        let old = self.blds[bi].nation as usize;
        let place = self.town_name(bi);
        let c = self.blds[bi].center();
        let was_cap = self.blds[bi].kind == BKind::Capital;
        self.remove_building(bi);
        // まわりの村も焼かれる
        for i in 0..self.blds.len() {
            if self.blds[i].alive && self.blds[i].nation as usize == old && self.blds[i].center().dist(c) < 10.0 && !self.blds[i].kind.city() {
                self.remove_building(i);
            }
        }
        if old == self.player {
            self.push_msg(tf!("{}は天の軍勢に焼き払われた", "{} has been burned by the hosts of Heaven", "{}被天之军势焚毁", place), [1.0, 0.5, 0.35]);
        }
        if let Some(d) = &mut self.descent {
            d.taken += 1;
        }
        if old < NATIONS.len() {
            if was_cap {
                self.move_capital(old, c);
            }
            if !self.blds.iter().any(|b| b.alive && b.nation as usize == old && b.kind.city() && b.done()) {
                self.eliminate(old, HEAVEN);
            }
        }
        self.territory_dirty = true;
    }

    /// 毎年：予定どおりに街を灰にし、天使を次の街へ向かわせる
    pub(super) fn yearly_descent(&mut self) {
        let Some(d) = &self.descent else { return };
        let p = ((self.years - d.start) / DESCENT_YEARS).clamp(0.0, 1.0);
        let (total, taken) = (d.total, d.taken);
        let home = self.nations[HEAVEN].home;
        // 亡霊の地から近い順
        let mut rest: Vec<(f32, usize)> = self
            .blds
            .iter()
            .enumerate()
            .filter(|(_, b)| b.alive && b.kind.city() && b.done() && (b.nation as usize) < NATIONS.len())
            .map(|(i, b)| (b.center().dist(home), i))
            .collect();
        rest.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let want = if p >= 1.0 { taken + rest.len() } else { ((total as f32 * p).ceil() as usize).max(taken) };
        let n = (want.saturating_sub(taken)).min(rest.len());
        for &(_, bi) in rest.iter().take(n) {
            if !self.blds[bi].alive {
                continue;
            }
            let c = self.blds[bi].center();
            self.smite_town(bi);
            self.spawn_angels(c, 3);
        }
        // 天使は次の街へ（海の向こうへも飛んでゆく：通れなければ岸に現れる）
        let next: Vec<V2> = rest.iter().skip(n).take(8).map(|&(_, bi)| self.blds[bi].center()).collect();
        if !next.is_empty() {
            let idle: Vec<usize> = (0..self.units.len()).filter(|&i| self.units[i].alive && self.units[i].divine && !matches!(self.units[i].order, Order::Attack(_))).collect();
            for (k, i) in idle.into_iter().enumerate() {
                let g = next[k % next.len()];
                if !self.same_land(self.units[i].pos, g) {
                    // 翼ある者に海は障りにならない
                    if let Some(&q) = self.land_near(g, 4).last() {
                        self.units[i].pos = q;
                        self.units[i].reset_nav();
                    }
                }
                let h = self.uh(i);
                self.order_move(&[h], g, true);
            }
        }
        // 5年が過ぎた：両国は滅ぶ
        if p >= 1.0 && self.game_over.is_none() {
            for n in 0..NATIONS.len() {
                if self.nations[n].alive {
                    self.eliminate(n, HEAVEN);
                }
            }
            if self.game_over.is_none() {
                let text = t!("天の軍勢がすべての街を灰にした。百年の戦はこうして終わった……", "The hosts of Heaven have burned every town to ash. So ended the war of a hundred years...", "天之军势将所有城镇化为灰烬。百年战争就此终结……").to_string();
                self.game_over = Some((false, text));
            }
        }
    }
}
