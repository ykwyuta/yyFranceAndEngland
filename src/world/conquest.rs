//! 街と村の行方：村の略奪、街の名前、諸侯の寝返り・同盟による街の離脱、遷都

use super::*;

impl World {
    /// 最初からある街の番号に対応する建物（いま残っていれば）
    pub fn town_bld(&self, town: usize) -> Option<usize> {
        self.blds.iter().position(|b| b.alive && b.kind.city() && b.town as usize == town)
    }

    /// 街の名前（村なら最寄りの街の名を冠する）
    pub fn town_name(&self, bi: usize) -> String {
        let b = &self.blds[bi];
        if let Some(t) = TOWNS.get(b.town as usize) {
            return tx(t.name).to_string();
        }
        let c = b.center();
        let near = self
            .blds
            .iter()
            .filter(|x| x.alive && x.kind.city() && (x.town as usize) < TOWNS.len())
            .min_by(|x, y| x.center().dist(c).partial_cmp(&y.center().dist(c)).unwrap());
        match near {
            Some(x) => tf!("{}近くの{}", "the {1} near {0}", "{}附近的{}", tx(TOWNS[x.town as usize].name), b.kind.name()),
            None => b.kind.name().to_string(),
        }
    }

    /// 村の略奪：壊した国が蓄えを奪う（部族・傭兵団・天の軍勢なら焼かれるだけ）
    pub(super) fn plunder_village(&mut self, bi: usize, an: usize) {
        let vn = self.blds[bi].nation as usize;
        let place = self.town_name(bi);
        let m = self.era_mult();
        let mut loot = [140.0 * m, 60.0 * m, 25.0 * m];
        let mut coin = 30.0 * m;
        if an < NATIONS.len() {
            let k = self.nations[an].tm.loot;
            for r in loot.iter_mut() {
                *r *= k;
            }
            coin *= k;
        }
        // 村の持ち主も、蓄えの一部を失う
        if vn < NATIONS.len() {
            let n = &mut self.nations[vn];
            for r in n.res.iter_mut() {
                *r *= 0.95;
            }
        }
        self.remove_building(bi);
        if an < NATIONS.len() {
            let n = &mut self.nations[an];
            for k in 0..3 {
                n.res[k] += loot[k];
                n.earned[k] += loot[k];
            }
            n.coin += coin;
            n.aggression += 3.0;
        }
        let p = self.player;
        if an == p {
            let t = tf!("{}を略奪した（糧{:.0}・木{:.0}・鉄{:.0}・金貨{:.0}を奪った）", "Plundered {} (took food {:.0}, wood {:.0}, iron {:.0}, gold {:.0})", "劫掠了{}（夺得粮{:.0}、木{:.0}、铁{:.0}、金币{:.0}）", place, loot[0], loot[1], loot[2], coin);
            self.push_msg(t, [1.0, 0.8, 0.4]);
        } else if vn == p {
            let who = if an < self.nations.len() { tx(self.nations[an].name) } else { "?" };
            let t = tf!("{}が{}を略奪し、焼き払った", "{} plundered and burned {}", "{}劫掠并焚毁了{}", who, place);
            self.push_msg(t, [1.0, 0.4, 0.3]);
        }
        self.territory_dirty = true;
    }

    /// 街が持ち主を変えたとき、その街のまわりの村も従う
    pub(super) fn villages_follow(&mut self, bi: usize, old: usize, new: usize) {
        let c = self.blds[bi].center();
        let r = self.blds[bi].kind.radius().max(9) as f32 + 2.0;
        for i in 0..self.blds.len() {
            let b = &self.blds[i];
            if !b.alive || b.nation as usize != old || b.kind == BKind::Village && b.center().dist(c) > r {
                continue;
            }
            if b.kind == BKind::Village {
                // 他の自国の街のほうが近い村はそのまま
                let d = b.center().dist(c);
                let closer = self.blds.iter().enumerate().any(|(j, x)| j != bi && x.alive && x.kind.city() && x.nation as usize == old && x.center().dist(b.center()) < d);
                if !closer {
                    let b = &mut self.blds[i];
                    b.nation = new as u8;
                    b.queue.clear();
                }
            }
        }
        self.territory_dirty = true;
    }

    /// 都を失った国は、最も近い街に都を移す
    pub(super) fn move_capital(&mut self, n: usize, from: V2) {
        let mut best: Option<(f32, usize)> = None;
        for (i, b) in self.blds.iter().enumerate() {
            if b.alive && b.nation as usize == n && b.kind == BKind::Town && b.done() {
                let d = b.center().dist(from);
                if best.map_or(true, |x| d < x.0) {
                    best = Some((d, i));
                }
            }
        }
        match best {
            Some((_, i)) => {
                self.blds[i].kind = BKind::Capital;
                let h = self.bh(i);
                self.nations[n].capital = Some(h);
                self.nations[n].home = self.blds[i].center();
                if n == self.player {
                    let t = tf!("都を{}に移した", "The court moved to {}", "迁都至{}", self.town_name(i));
                    self.push_msg(t, [1.0, 0.8, 0.5]);
                }
            }
            None => self.nations[n].capital = None,
        }
    }

    /// 街をもう一方の王に渡す（諸侯の寝返り・同盟・条約）。渡した街の名を返す
    pub(super) fn transfer_town(&mut self, bi: usize, to: usize) -> Option<String> {
        let old = self.blds[bi].nation as usize;
        if old == to || !self.blds[bi].alive || !self.nations[to].alive {
            return None;
        }
        let was_cap = self.blds[bi].kind == BKind::Capital;
        {
            let b = &mut self.blds[bi];
            b.nation = to as u8;
            b.queue.clear();
            b.queue_t = 0.0;
            b.rally = None;
            b.kind = BKind::Town;
            if b.done() {
                b.hp = b.hp.max(b.kind.max_hp() * 0.6);
            }
        }
        self.villages_follow(bi, old, to);
        if self.nations[to].capital.and_then(|h| self.bld(h)).is_none() {
            self.blds[bi].kind = BKind::Capital;
            let h = self.bh(bi);
            self.nations[to].capital = Some(h);
            self.nations[to].home = self.blds[bi].center();
        }
        if was_cap {
            self.move_capital(old, self.blds[bi].center());
        }
        // 街の中にいた元の持ち主の兵は外へ出る（建物に重なったままにしない）
        self.territory_dirty = true;
        let name = self.town_name(bi);
        if old < NATIONS.len() && !self.blds.iter().any(|b| b.alive && b.nation as usize == old && b.kind.city() && b.done()) {
            self.eliminate(old, to);
        }
        Some(name)
    }

    /// 諸侯の寝返り・同盟：その諸侯の街のうち、もう一方の王に付いているものが to に移る
    pub(super) fn defect(&mut self, lord: u8, to: usize) -> Option<String> {
        let list: Vec<usize> = (0..self.blds.len()).filter(|&i| self.blds[i].alive && self.blds[i].kind.city() && self.blds[i].lord == lord && self.blds[i].nation as usize != to && (self.blds[i].nation as usize) < NATIONS.len()).collect();
        let mut names = Vec::new();
        for bi in list {
            if let Some(n) = self.transfer_town(bi, to) {
                names.push(n);
            }
        }
        if names.is_empty() {
            return None;
        }
        Some(tf!("{}の街（{}）が{}に付いた", "the towns of {} ({}) go over to {}", "{}的城镇（{}）归附{}", tx(LORDS[lord as usize].name), names.join(sep()), tx(self.nations[to].name)))
    }

    /// 条約などで一つの街が渡される
    pub(super) fn cede(&mut self, town: usize, to: usize) -> Option<String> {
        let bi = self.town_bld(town)?;
        if self.blds[bi].nation as usize == to || self.blds[bi].nation as usize >= NATIONS.len() {
            return None;
        }
        let name = self.transfer_town(bi, to)?;
        Some(tf!("{}が{}に渡る", "{} passes to {}", "{}归于{}", name, tx(self.nations[to].name)))
    }
}
