//! 英雄の一覧と捕虜（身代金の請求）の画面、地図上の英雄の名札

use super::*;

impl Game {
    pub(super) fn render_summon(&mut self) {
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        let pl = self.world.player;
        let pw = 1120.0f32.min(w - 20.0);
        let ph = 680.0f32.min(h - 30.0);
        let x0 = (w - pw) / 2.0;
        let y0 = (h - ph) / 2.0;
        self.gfx.fill_rect(0.0, 0.0, w, h, [0.0, 0.0, 0.0, 0.5]);
        self.art.draw_stone_panel(&self.gfx, x0, y0, x0 + pw, y0 + ph);
        self.art.draw_corner_ornaments(&self.gfx, x0 + 3.0, y0 + 3.0, x0 + pw - 3.0, y0 + ph - 3.0, 16.0);
        self.gfx.text_shadow(t!("英雄と捕虜", "Heroes & Captives", "英雄与俘虏"), x0 + 24.0, y0 + 10.0, 260.0, 34.0, Font::Big, [1.0, 0.85, 0.5, 1.0]);
        self.add_button(x0 + pw - 90.0, y0 + 10.0, x0 + pw - 14.0, y0 + 40.0, t!("閉じる", "Close", "关闭"), "", 0, true, Act::CloseOverlay, "");
        let gold = [1.0, 0.85, 0.5, 1.0];
        let tc = [0.95, 0.92, 0.85, 1.0];
        let dim = [0.72, 0.69, 0.62, 1.0];
        let head = t!("英雄は史実の年に現れ、没年を越えて生きられない。討たれた英雄は捕らえられ、身代金を請求できる（ジャンヌ・ダルクを除く）。英雄のそばの兵は攻撃が上がる。", "Heroes appear in their historical years and cannot outlive their historical deaths. A defeated hero is captured and can be ransomed (except Joan of Arc). Troops near a hero attack harder.", "英雄在史实年份登场，无法活过史实卒年。被击败的英雄会被俘，可索要赎金（贞德除外）。英雄身边的士兵攻击提升。");
        self.gfx.text(head, x0 + 290.0, y0 + 8.0, pw - 400.0, 38.0, Font::Small, tc);

        self.art.draw_inset_box(&self.gfx, x0 + 10.0, y0 + 48.0, x0 + pw - 10.0, y0 + ph - 10.0);
        let half = (HEROES.len() + 1) / 2;
        let colw = (pw - 30.0) / 2.0;
        let rowh = ((ph - 70.0) / half as f32).min(44.0);
        for (k, d) in HEROES.iter().enumerate() {
            let c = k / half;
            let r = k % half;
            let rx = x0 + 15.0 + c as f32 * colw;
            let ry = y0 + 54.0 + r as f32 * rowh;
            if r % 2 == 0 {
                self.gfx.fill_rect(rx, ry, rx + colw - 8.0, ry + rowh, [1.0, 1.0, 1.0, 0.04]);
            }
            let nc = self.world.nations[d.nation].color;
            self.gfx.fill_rect(rx + 4.0, ry + 6.0, rx + 10.0, ry + rowh - 6.0, c4(nc, 1.0));
            let tx = rx + 16.0;
            let ny = ry + (rowh - 38.0).max(0.0) * 0.5;
            self.gfx.text(crate::lang::tx(d.name), tx, ny, 300.0, 20.0, Font::Normal, if d.nation == pl { gold } else { tc });
            self.gfx.text(crate::lang::tx(d.era), tx, ny + 19.0, 300.0, 17.0, Font::Small, dim);
            let st = self.world.hero_status(k);
            let cap = self.world.captives.iter().find(|c| c.hero == k).cloned();
            let st = match &cap {
                Some(c) => tf!("{}の捕虜", "held by {}", "被{}俘虏", crate::lang::tx(self.world.nations[c.captor].name)),
                None => st.to_string(),
            };
            let scol = if cap.is_some() { [1.0, 0.6, 0.5, 1.0] } else { [0.7, 0.92, 1.0, 1.0] };
            self.gfx.text(&st, tx + 300.0, ry + (rowh - 20.0) * 0.5, colw - 420.0, 20.0, Font::Small, scol);
            if let Some(c) = cap.filter(|c| c.captor == pl) {
                let amount = self.world.ransom_price(k);
                let owner = d.nation;
                let can = self.world.nations[owner].coin >= amount;
                let tip = tf!("{0}\n{1}に金貨{2:.0}の身代金を求める（相手に金貨があれば払って英雄を取り戻す）", "{0}\nDemand a ransom of {2:.0} gold from {1} (if they have the gold they pay and the hero goes home)", "{0}\n向{1}索要金币{2:.0}的赎金（对方有金币便会付款赎回英雄）", crate::lang::tx(d.desc), crate::lang::tx(self.world.nations[owner].name), amount);
                let _ = c;
                self.add_button(rx + colw - 112.0, ry + 3.0, rx + colw - 14.0, ry + rowh - 3.0, t!("身代金を請求", "Ransom", "索要赎金"), "", 0, can, Act::SummonHero(k), &tip);
            } else {
                self.buttons.push(Button { x0: rx, y0: ry, x1: rx + colw - 120.0, y1: ry + rowh, label: String::new(), key: 0, enabled: true, act: Act::Noop, tip: crate::lang::tx(d.desc).to_string() });
            }
        }
    }

    /// 英雄の名札（地図上）
    pub(super) fn render_hero_labels(&mut self) {
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        let pl = self.world.player;
        let z = self.zoom;
        let mut labels = Vec::new();
        for u in self.world.units.iter() {
            if !u.on_map() || (u.nation as usize != pl && !self.seen(u.pos)) {
                continue;
            }
            let name = if (u.hero as usize) < HEROES.len() {
                crate::lang::tx(HEROES[u.hero as usize].name)
            } else if u.ut == UT_GHOST {
                crate::lang::tx(udef(UT_GHOST).name)
            } else {
                continue;
            };
            let (sx, sy) = self.world_to_screen(u.pos);
            if sx < -80.0 || sy < -40.0 || sx > w + 80.0 || sy > h + 40.0 {
                continue;
            }
            labels.push((sx, sy, name, self.world.nations[u.nation as usize].color));
        }
        for (sx, sy, name, c) in labels {
            let ty = sy - (z * 1.3).clamp(14.0, 34.0) - 22.0;
            let tw = (self.gfx.measure(name, Font::Small) + 14.0).max(60.0);
            self.gfx.fill_round(sx - tw / 2.0, ty, sx + tw / 2.0, ty + 20.0, 4.0, [0.10, 0.07, 0.02, 0.85]);
            self.gfx.stroke_round(sx - tw / 2.0, ty, sx + tw / 2.0, ty + 20.0, 4.0, 1.4, c4(c, 0.95));
            self.gfx.text(name, sx - tw / 2.0, ty + 2.0, tw, 18.0, Font::NormalCenter, [1.0, 0.92, 0.6, 1.0]);
        }
    }
}
