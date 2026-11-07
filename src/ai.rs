//! コンピュータ国家の思考

use crate::geo::{Map, ENGLAND, NATIONS};
use crate::rng::Rng;
use crate::world::*;

pub struct AiState {
    pub rng: Rng,
    pub(crate) think_t: f32,
    pub(crate) war_t: f32,
    pub(crate) attack_t: f32,
    /// 好戦性（国ごとの性格）
    pub aggr: f32,
}

impl AiState {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let aggr = rng.range(0.6, 1.4);
        let think_t = rng.range(0.0, 1.0);
        AiState { rng, think_t, war_t: 10.0 + think_t * 5.0, attack_t: 0.0, aggr }
    }
}

pub fn update_all(w: &mut World, dt: f32) {
    let mut ais = std::mem::take(&mut w.ai);
    for n in 0..ais.len() {
        if (n == w.player && !w.ai_player) || !w.nations[n].alive {
            continue;
        }
        let a = &mut ais[n];
        a.think_t -= dt;
        a.war_t -= dt;
        a.attack_t -= dt;
        if a.think_t <= 0.0 {
            a.think_t = 1.0 + a.rng.f32() * 0.4;
            think(w, n, a);
        }
    }
    w.ai = ais;
}

fn find_spot(w: &World, n: usize, kind: BKind, center: V2, rmin: f32, rmax: f32, rng: &mut Rng) -> Option<(i32, i32)> {
    for k in 0..60 {
        let ang = rng.range(0.0, std::f32::consts::TAU);
        // 見つからなければ範囲を広げる
        let d = rng.range(rmin, rmax + (k / 20) as f32 * 3.0);
        let tx = (center.x + ang.cos() * d) as i32 - 1;
        let ty = (center.y + ang.sin() * d) as i32 - 1;
        if w.can_place(n, kind, tx, ty).is_ok() {
            return Some((tx, ty));
        }
    }
    None
}

fn think(w: &mut World, n: usize, a: &mut AiState) {
    let res = w.nations[n].res;
    let home = w.nations[n].home;
    let mut workers: Vec<(Handle, Order, V2)> = Vec::new();
    let mut soldiers: Vec<(Handle, Order, V2, f32)> = Vec::new();
    let mut ships: Vec<(Handle, Order, V2, usize)> = Vec::new();
    let mut gathering = [0usize; 3];
    for (i, u) in w.units.iter().enumerate() {
        if !u.on_map() || u.nation as usize != n {
            continue;
        }
        let h = Handle { idx: i as u32, gen: u.gen };
        if u.kind.naval() {
            ships.push((h, u.order, u.pos, u.cargo.len()));
        } else if u.kind.military() {
            soldiers.push((h, u.order, u.pos, u.power));
        } else {
            if let Order::Gather { src, .. } = u.order {
                gathering[src.res()] += 1;
            }
            workers.push((h, u.order, u.pos));
        }
    }
    let mut cities = Vec::new();
    let mut barracks = Vec::new();
    let mut ports = Vec::new();
    let mut unfinished = Vec::new();
    let mut farms = 0;
    let mut villages = 0;
    for (i, b) in w.blds.iter().enumerate() {
        if !b.alive || b.nation as usize != n {
            continue;
        }
        let h = Handle { idx: i as u32, gen: b.gen };
        if !b.done() {
            unfinished.push(h);
        }
        match b.kind {
            BKind::Capital | BKind::Town => {
                if b.done() {
                    cities.push(h)
                }
            }
            BKind::Barracks => {
                if b.done() {
                    barracks.push(h)
                }
            }
            BKind::Farm => farms += 1,
            BKind::Fort => {}
            BKind::Port => ports.push(h),
            BKind::Village => villages += 1,
        }
    }
    if cities.is_empty() {
        return;
    }
    let nat = &w.nations[n];
    let (wc, mc) = (nat.worker_cap(), nat.mil_cap());
    let (nw, nm) = (nat.workers + nat.q_workers, nat.soldiers + nat.q_soldiers);
    let at_war_any = (0..NATIONS.len()).any(|o| w.at_war(n, o));
    w.ai_trade(n, &mut a.rng);

    // ---- 民の訓練
    if nw < wc && res[0] >= 50.0 + if at_war_any { 40.0 } else { 0.0 } {
        let free: Vec<Handle> = cities.iter().copied().filter(|&c| w.bld(c).map_or(false, |b| b.queue.is_empty())).collect();
        // 民の少ない陸地の街を優先する
        let pick = free.iter().copied().min_by_key(|&c| {
            let cc = w.bld(c).unwrap().center();
            let count = workers.iter().filter(|x| w.same_land(x.2, cc)).count();
            // 英軍は大陸の現地生産を厚くする（大陸：本土がおよそ2：1）。
            count * if n == ENGLAND && w.same_land(cc, home) { 2 } else { 1 }
        });
        if let Some(c) = pick {
            let _ = w.enqueue(c, UT_WORKER);
        }
    }

    // ---- 未完成の建物に建設者を
    for &bh in unfinished.iter() {
        let builders = workers.iter().filter(|x| x.1 == Order::Build(bh)).count();
        if builders == 0 {
            let bc = w.bld(bh).unwrap().center();
            let mut best: Option<(f32, usize)> = None;
            for (k, x) in workers.iter().enumerate() {
                if matches!(x.1, Order::Build(_)) || !w.same_land(x.2, bc) {
                    continue;
                }
                let d = x.2.dist(bc);
                if best.map_or(true, |b| d < b.0) {
                    best = Some((d, k));
                }
            }
            if let Some((_, k)) = best {
                w.order_build(&[workers[k].0], bh);
                workers[k].1 = Order::Build(bh);
            }
        }
    }

    // ---- 兵営：戦時は働き手の数に合わせて最大六つまで増やす
    let res = w.nations[n].res;
    if res[1] >= 150.0 && res[2] >= 30.0 {
        let mut regions: Vec<(u32, V2)> = Vec::new();
        for &c in cities.iter() {
            let cc = w.bld(c).unwrap().center();
            let r = w.land_region_at(cc);
            if !regions.iter().any(|x| x.0 == r) {
                regions.push((r, cc));
            }
        }
        if n == ENGLAND {
            regions.sort_by_key(|&(r, _)| r == w.land_region_at(home));
        }
        for (r, cc) in regions.into_iter().take(3) {
            let count = w.blds.iter().filter(|b| b.alive && b.nation as usize == n && b.kind == BKind::Barracks && w.land_region_at(b.center()) == r).count();
            let workforce = workers.iter().filter(|s| w.land_region_at(s.2) == r).count();
            let want = if at_war_any { (1 + workforce / 6).clamp(2, 6) } else { 1 };
            if count >= want {
                continue;
            }
            // その陸地の民がいなければ建てない
            if !workers.iter().any(|x| w.land_region_at(x.2) == r) {
                continue;
            }
            let center = if r == w.land_region_at(home) { home } else { cc };
            if let Some((tx, ty)) = find_spot(w, n, BKind::Barracks, center, 3.0, 7.0, &mut a.rng) {
                if let Ok(bh) = w.place_building(n, BKind::Barracks, tx, ty) {
                    let bc = w.bld(bh).unwrap().center();
                    if let Some(x) = workers.iter_mut().filter(|x| !matches!(x.1, Order::Build(_)) && w.same_land(x.2, bc)).min_by(|p, q| p.2.dist(bc).partial_cmp(&q.2.dist(bc)).unwrap()) {
                        w.order_build(&[x.0], bh);
                        x.1 = Order::Build(bh);
                    }
                }
            }
            break;
        }
    }

    // ---- 資源配分
    let total = workers.len().max(1) as f32;
    let mut want = [0.45, 0.32, 0.23];
    let res = w.nations[n].res;
    for k in 0..3 {
        if res[k] > 700.0 {
            want[k] *= 0.4;
        }
    }
    if w.nations[n].upkeep > w.nations[n].income[0] * 0.5 {
        want[0] += 0.15;
    }
    if res[0] < 200.0 {
        want[0] += 0.3;
    }
    let sumw: f32 = want.iter().sum();
    // 田が足りなければ作る
    let food_target = (total * want[0] / sumw).round() as usize;
    if farms < food_target.max(1) && res[1] >= 60.0 && unfinished.len() < 3 {
        // 働き手のいる街の近くに田を集め、遠方の無人の街への建設を避ける。
        let c = cities.iter().copied().max_by_key(|&c| {
            let cc = w.bld(c).unwrap().center();
            let local_workers = workers.iter().filter(|x| x.2.dist(cc) < 14.0).count();
            let local_farms = w.blds.iter().filter(|b| b.alive && b.nation as usize == n && b.kind == BKind::Farm && b.center().dist(cc) < 14.0).count();
            (local_workers as f32 * want[0] / sumw - local_farms as f32).round() as i32
        }).unwrap();
        let cc = w.bld(c).unwrap().center();
        if let Some((tx, ty)) = find_spot(w, n, BKind::Farm, cc, 2.5, 6.0, &mut a.rng) {
            if let Ok(bh) = w.place_building(n, BKind::Farm, tx, ty) {
                let bc = w.bld(bh).unwrap().center();
                let mut best: Option<(f32, usize)> = None;
                for (k, x) in workers.iter().enumerate() {
                    if matches!(x.1, Order::Build(_)) {
                        continue;
                    }
                    let d = x.2.dist(bc);
                    if best.map_or(true, |b| d < b.0) {
                        best = Some((d, k));
                    }
                }
                if let Some((_, k)) = best {
                    w.order_build(&[workers[k].0], bh);
                    workers[k].1 = Order::Build(bh);
                }
            }
        }
    }
    // 過剰な資源の採集者も配分を見直す。荷を運んでいる民は引き戻さない。
    // 一度に二人までにして、配分の揺れと移動の浪費を抑える。
    let mut reassigned = 0;
    for x in workers.iter_mut() {
        if reassigned >= 2 {
            break;
        }
        if let Order::Gather { src, .. } = x.1 {
            let k = src.res();
            if gathering[k] as f32 > want[k] / sumw * total + 2.0 && w.unit(x.0).map_or(false, |u| u.carry < 0.1) {
                gathering[k] = gathering[k].saturating_sub(1);
                x.1 = Order::Idle;
                reassigned += 1;
            }
        }
    }
    for x in workers.iter() {
        if x.1 != Order::Idle {
            continue;
        }
        // 不足している資源
        let mut best_k = 0;
        let mut best_def = f32::MIN;
        for k in 0..3 {
            let def = want[k] / sumw * total - gathering[k] as f32;
            if def > best_def {
                best_def = def;
                best_k = k;
            }
        }
        let p = x.2;
        let src = match best_k {
            0 => w.free_farm(n, p).map(Src::Farm),
            2 => w.nearest_mine_for(Some(n), p, 22.0).map(Src::Mine),
            _ => None,
        }
        .or_else(|| w.nearest_forest_for(Some(n), p, 16).map(|(x, y)| Src::Forest(x, y)))
        .or_else(|| w.nearest_mine_for(Some(n), p, 30.0).map(Src::Mine));
        if let Some(s) = src {
            gathering[s.res()] += 1;
            w.order_gather(&[x.0], s);
        }
    }

    // ---- 要塞：資源が余っているか戦時なら、城邑の近くに築く
    let res = w.nations[n].res;
    let fort_c = BKind::Fort.cost();
    let rich = res[0] >= fort_c[0] * 2.0 && res[1] >= fort_c[1] * 2.0 && res[2] >= fort_c[2] * 2.0;
    let can_pay = res[0] >= fort_c[0] && res[1] >= fort_c[1] && res[2] >= fort_c[2];
    let forts_building = unfinished.iter().any(|&h| w.bld(h).map_or(false, |b| b.kind == BKind::Fort));
    if can_pay && (rich || at_war_any) && !forts_building && w.fort_count(n) < w.nations[n].fort_cap() && a.rng.chance(0.15) {
        let c = cities[a.rng.below(cities.len() as u32) as usize];
        let cc = w.bld(c).unwrap().center();
        if let Some((tx, ty)) = find_spot(w, n, BKind::Fort, cc, 4.0, 8.0, &mut a.rng) {
            if let Ok(bh) = w.place_building(n, BKind::Fort, tx, ty) {
                if let Some(x) = workers.iter_mut().find(|x| !matches!(x.1, Order::Build(_))) {
                    w.order_build(&[x.0], bh);
                    x.1 = Order::Build(bh);
                }
            }
        }
    }

    // ---- 村を建てる（街一つにつき二つまで）
    let res = w.nations[n].res;
    let max_villages = (cities.len() * 2).min(24);
    let vill_building = unfinished.iter().any(|&h| w.bld(h).map_or(false, |b| b.kind == BKind::Village));
    if villages < max_villages && !vill_building && res[1] >= 160.0 && res[0] >= 80.0 && a.rng.chance(0.35) {
        let c = cities[a.rng.below(cities.len() as u32) as usize];
        let cc = w.bld(c).unwrap().center();
        if let Some((tx, ty)) = find_spot(w, n, BKind::Village, cc, 5.0, 10.0, &mut a.rng) {
            if let Ok(bh) = w.place_building(n, BKind::Village, tx, ty) {
                let bc = w.bld(bh).unwrap().center();
                if let Some(x) = workers.iter_mut().filter(|x| !matches!(x.1, Order::Build(_)) && w.same_land(x.2, bc)).min_by(|p, q| p.2.dist(bc).partial_cmp(&q.2.dist(bc)).unwrap()) {
                    w.order_build(&[x.0], bh);
                    x.1 = Order::Build(bh);
                }
            }
        }
    }

    // ---- 軍の訓練
    // 英軍の限られた糧・鉄は、先に大陸の兵営へ回す。
    if n == ENGLAND {
        barracks.sort_by_key(|&b| w.same_land(w.bld(b).unwrap().center(), home));
    }
    // 空いている兵営をすべて使う。支払い後の残高と予約兵数を毎回確認する。
    for &b in &barracks {
        let nat = &w.nations[n];
        let res = nat.res;
        let food_ok = res[0] > 350.0 || nat.income[0] > nat.upkeep * 1.3 + 0.3;
        if nat.soldiers + nat.q_soldiers >= mc || res[0] < 100.0 || !food_ok {
            break;
        }
        if w.bld(b).map_or(false, |x| x.queue.len() < 2) {
            // 作れる兵から選ぶ（自国の兵を多めに、強い兵ほど少し多めに）
            let list = w.bld(b).map(|x| w.trainable_at(x)).unwrap_or_default();
            if !list.is_empty() {
                let wt: Vec<f32> = list.iter().enumerate().map(|(k, &(ut, o))| {
                    let levy = if n == ENGLAND && matches!(udef(ut).class, UnitKind::Infantry | UnitKind::Archer) { 1.8 } else { 1.0 };
                    (1.0 + k as f32 * 0.25) * if o == n { levy } else { 0.6 }
                }).collect();
                let tot: f32 = wt.iter().sum();
                let mut x = a.rng.f32() * tot;
                let mut pick = list[0].0;
                for (k, &(u, _)) in list.iter().enumerate() {
                    pick = u;
                    x -= wt[k];
                    if x <= 0.0 {
                        break;
                    }
                }
                if w.enqueue(b, pick).is_err() {
                    let _ = w.enqueue(b, list[0].0);
                }
            }
        }
    }

    // ---- 港と船（海の向こうの敵を攻めるとき）
    let sea_target = naval_target(w, n);
    if sea_target.is_some() && w.can_build_ships(n) {
        let res = w.nations[n].res;
        let port_c = BKind::Port.cost();
        let port_site = w.blds.iter().any(|b| b.alive && b.nation as usize == n && b.kind == BKind::Port);
        if !port_site && res[1] >= port_c[1] && res[0] >= port_c[0] && res[2] >= port_c[2] {
            'outer: for &c in cities.iter() {
                let cc = w.bld(c).unwrap().center();
                if w.shore_water_near(cc, 9).is_none() {
                    continue;
                }
                for _ in 0..3 {
                    if let Some((tx, ty)) = find_spot(w, n, BKind::Port, cc, 2.0, 9.0, &mut a.rng) {
                        if let Ok(bh) = w.place_building(n, BKind::Port, tx, ty) {
                            if let Some(x) = workers.iter_mut().find(|x| !matches!(x.1, Order::Build(_))) {
                                w.order_build(&[x.0], bh);
                                x.1 = Order::Build(bh);
                            }
                            break 'outer;
                        }
                    }
                }
            }
        }
        let want_ships = ((soldiers.len() + SHIP_CAP - 1) / SHIP_CAP).clamp(2, 8);
        let queued: usize = ports.iter().filter_map(|&p| w.bld(p)).map(|b| b.queue.len()).sum();
        if ships.len() + queued < want_ships && nm < mc {
            if let Some(&p) = ports.first() {
                let list = w.bld(p).map(|x| w.trainable_at(x)).unwrap_or_default();
                if let Some(&(u, _)) = list.first() {
                    let _ = w.enqueue(p, u);
                }
            }
        }
    }

    // ---- 能力の鍛錬（上限に達したら質で補う）
    let res = w.nations[n].res;
    if res[0] > 300.0 && res[2] > 150.0 && (nm >= mc || res[0] > 700.0) {
        // 蓄えが多ければ、一度に何人も鍛える
        let k = if res[0] > 3000.0 && res[2] > 600.0 { 4 } else { 1 };
        let mut ss: Vec<&(Handle, Order, V2, f32)> = soldiers.iter().collect();
        ss.sort_by(|x, y| x.3.partial_cmp(&y.3).unwrap());
        for s in ss.into_iter().take(k) {
            let _ = w.upgrade_unit(s.0);
        }
    }
    if res[0] > 350.0 && res[1] > 250.0 && nw >= wc {
        if let Some(x) = workers.iter().min_by(|x, y| w.unit(x.0).unwrap().power.partial_cmp(&w.unit(y.0).unwrap().power).unwrap()) {
            let _ = w.upgrade_unit(x.0);
        }
    }

    // ---- 外交
    if a.war_t <= 0.0 {
        a.war_t = 4.0 + a.rng.f32() * 2.0;
        diplomacy(w, n, a, soldiers.len());
    }

    // ---- 軍の指揮
    let _ = (&ships, sea_target);
    let soldiers: Vec<_> = soldiers.into_iter().filter(|s| !matches!(s.1, Order::Board(_) | Order::Ferry(..))).collect();
    command_army(w, n, a, &soldiers);
}

fn diplomacy(w: &mut World, n: usize, a: &mut AiState, nsold: usize) {
    let my = w.nations[n].strength;
    let years = w.years;
    // 和睦
    for e in 0..NATIONS.len() {
        if !w.at_war(n, e) || w.coalition_locked(n, e) {
            continue;
        }
        let dur = years - w.war_start[n][e];
        let theirs = w.nations[e].strength;
        if dur > 15.0 && my < theirs * 0.55 && a.rng.chance(0.3) {
            if e == w.player && !w.ai_player {
                if !w.peace_offer[n] && w.offer_cd[n] <= years {
                    w.peace_offer[n] = true;
                    w.propose_peace(n, e, 10.0, t!("戦の長期化", "a long war", "战事旷日持久"));
                }
            } else {
                w.make_peace(n, e, 10.0);
            }
        }
    }
    // 宣戦
    let wars = (0..NATIONS.len()).filter(|&o| w.at_war(n, o)).count();
    if wars > 0 || nsold < 3 {
        return;
    }
    if !a.rng.chance(0.6 * a.aggr) {
        return;
    }
    let home = w.nations[n].home;
    let mut best: Option<(f32, usize)> = None;
    for o in 0..NATIONS.len() {
        let on = &w.nations[o];
        if o == n || !on.alive || w.truce[n][o] > years || w.coalition_locked(n, o) || (o == w.player && w.protect_player) {
            continue;
        }
        let d = on.home.dist(home);
        // 百年戦争：休戦が明ければ、ためらわずに戦を再開する
        {
            let score = on.strength + d * 2.0;
            if best.map_or(true, |b| score < b.0) {
                best = Some((score, o));
            }
        }
    }
    if let Some((_, o)) = best {
        w.declare_war(n, o, false);
    }
}

fn command_army(w: &mut World, n: usize, a: &mut AiState, soldiers: &[(Handle, Order, V2, f32)]) {
    if soldiers.is_empty() {
        return;
    }
    // 陸地ごとに部隊を分ける（ブリテン島と大陸）
    let mut groups: Vec<(u32, Vec<(Handle, Order, V2, f32)>)> = Vec::new();
    for s in soldiers.iter() {
        let r = w.land_region_at(s.2);
        match groups.iter_mut().find(|g| g.0 == r) {
            Some(g) => g.1.push(*s),
            None => groups.push((r, vec![*s])),
        }
    }
    let at_war: Vec<usize> = (0..NATIONS.len()).filter(|&o| w.at_war(n, o)).collect();
    let mut attacked = false;
    for (reg, group) in groups.iter() {
        let reg = *reg;
        // その陸地の自国の街
        let own: Vec<V2> = w.blds.iter().filter(|b| b.alive && b.nation as usize == n && b.kind.city() && b.done() && w.land_region_at(b.center()) == reg).map(|b| b.center()).collect();
        let center = {
            let mut c = v2(0.0, 0.0);
            for s in group.iter() {
                c = c + s.2;
            }
            c * (1.0 / group.len() as f32)
        };
        let base = own.iter().copied().min_by(|p, q| p.dist(center).partial_cmp(&q.dist(center)).unwrap()).unwrap_or(center);
        // 自国領の侵入者（部族・天の軍も）
        let mut intruder: Option<V2> = None;
        for u in w.units.iter() {
            if !u.on_map() || !w.at_war(n, u.nation as usize) || w.land_region_at(u.pos) != reg {
                continue;
            }
            let (x, y) = u.pos.tile();
            if Map::inb(x, y) && w.owner[Map::idx(x, y)] as usize == n && intruder.map_or(true, |p| p.dist(base) > u.pos.dist(base)) {
                intruder = Some(u.pos);
            }
        }
        let mut defenders = Vec::new();
        if let Some(p) = intruder {
            let mut available: Vec<_> = group.iter().filter(|s| s.1 == Order::Idle || (matches!(s.1, Order::AttackMove(_)) && s.2.dist(p) < 40.0)).collect();
            if n == ENGLAND {
                available.sort_by(|a, b| a.2.dist(p).total_cmp(&b.2.dist(p)));
                let enemies = w.units.iter().filter(|u| u.on_map() && w.at_war(n, u.nation as usize) && u.pos.dist(p) < 12.0).count();
                available.truncate((enemies * 2).clamp(4, 24));
            }
            defenders = available.iter().map(|s| s.0).collect();
            if !defenders.is_empty() {
                w.order_move(&defenders, p, true);
            }
            if n != ENGLAND {
                continue;
            }
        }
        let idle: Vec<Handle> = group.iter().filter(|s| s.1 == Order::Idle && !defenders.contains(&s.0)).map(|s| s.0).collect();
        if at_war.is_empty() {
            // 平時：遠くの兵は最寄りの街へ戻る
            let far: Vec<Handle> = group.iter().filter(|s| s.1 == Order::Idle && s.2.dist(base) > 12.0).map(|s| s.0).collect();
            if !far.is_empty() {
                let g = base + v2(a.rng.range(-3.0, 3.0), a.rng.range(2.0, 4.0));
                w.order_move(&far, g, false);
            }
            continue;
        }
        if a.attack_t > 0.0 {
            continue;
        }
        // 初動は6〜16人でまとめて出撃。後続は全軍の待機を待たず、
        // 軍の規模に応じた4〜10人の増援として送る。
        let advancing = group.iter().any(|s| matches!(s.1, Order::AttackMove(_) | Order::Attack(_)));
        // 英軍の大陸遠征は現地で補充し、さらに大きな部隊で攻勢を続ける。
        let expedition = n == ENGLAND && reg != w.land_region_at(w.nations[n].home);
        let need = if expedition {
            if advancing { (group.len() / 6).clamp(8, 16) } else { (group.len() / 2).clamp(12, 24) }
        } else if advancing { (group.len() / 8).clamp(4, 10) } else { group.len().clamp(6, 16) };
        if idle.len() < need {
            continue;
        }
        // 目標：同じ陸地の敵の街、ときに村（騎行）。無ければ海の向こうの街へ（船があれば渡る）
        let raid = a.rng.chance(if n == ENGLAND { 0.35 } else { 0.2 });
        let mut best: Option<(f32, V2)> = None;
        let mut over_sea: Option<(f32, V2)> = None;
        for b in w.blds.iter() {
            if !b.alive || !b.done() || !at_war.contains(&(b.nation as usize)) || (b.nation as usize) >= NATIONS.len() {
                continue;
            }
            let want = if raid { b.kind == BKind::Village || b.kind.city() } else { b.kind.city() };
            if !want {
                continue;
            }
            let c = b.center();
            let mut d = c.dist(center);
            if raid && b.kind == BKind::Village {
                d *= 0.5;
            }
            if w.land_region_at(c) == reg {
                if best.map_or(true, |x| d < x.0) {
                    best = Some((d, c));
                }
            } else if b.kind.city() && over_sea.map_or(true, |x| d < x.0) {
                over_sea = Some((d, c));
            }
        }
        let goal = match (best, over_sea) {
            (Some((_, p)), _) => Some(p),
            (None, Some((_, p))) if w.has_ships(n) => Some(p),
            _ => None,
        };
        if let Some(p) = goal {
            w.order_move(&idle, p, true);
            attacked = true;
        }
    }
    if attacked {
        a.attack_t = 4.0 / a.aggr.clamp(0.6, 1.4);
    }
}

/// 海の向こうにしかない交戦国の城邑（船で攻める目標）
fn naval_target(w: &World, n: usize) -> Option<V2> {
    let home = w.nations[n].home;
    let mut best: Option<(f32, V2)> = None;
    let mut land_target = false;
    for b in w.blds.iter() {
        if !b.alive || !b.kind.city() || !w.at_war(n, b.nation as usize) || b.nation as usize >= NATIONS.len() {
            continue;
        }
        if w.same_land(b.center(), home) {
            land_target = true;
            continue;
        }
        let d = b.center().dist(home);
        if best.map_or(true, |x| d < x.0) {
            best = Some((d, b.center()));
        }
    }
    let _ = land_target;
    best.map(|b| b.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::FRANCE;

    fn army(w: &mut World, idle: usize, advancing: usize) -> Vec<(Handle, Order, V2, f32)> {
        let pos = w.land_near(w.nations[FRANCE].home, 4)[0];
        army_at(w, FRANCE, pos, idle, advancing)
    }

    fn army_at(w: &mut World, nation: usize, pos: V2, idle: usize, advancing: usize) -> Vec<(Handle, Order, V2, f32)> {
        w.units.clear();
        let goal = w.blds.iter().find(|b| b.alive && b.nation as usize != nation && w.same_land(pos, b.center())).unwrap().center();
        (0..idle + advancing).map(|i| {
            let h = w.spawn_unit(nation, UT_INF, pos, 1.0);
            let order = if i < idle { Order::Idle } else { Order::AttackMove(goal) };
            w.unit_mut(h).unwrap().order = order;
            (h, order, pos, 1.0)
        }).collect()
    }

    #[test]
    fn main_army_launches_together() {
        let mut w = World::new(12345);
        let troops = army(&mut w, 16, 0);
        let mut ai = AiState::new(1);
        command_army(&mut w, FRANCE, &mut ai, &troops);
        assert!(troops.iter().all(|s| matches!(w.unit(s.0).unwrap().order, Order::AttackMove(_))));
        assert!(ai.attack_t > 0.0 && ai.attack_t < 8.0);
    }

    #[test]
    fn reinforcements_join_without_waiting_for_the_main_army() {
        let mut w = World::new(12345);
        let troops = army(&mut w, 6, 42);
        let mut ai = AiState::new(1);
        command_army(&mut w, FRANCE, &mut ai, &troops);
        assert!(troops[..6].iter().all(|s| matches!(w.unit(s.0).unwrap().order, Order::AttackMove(_))));
        // 進軍中の先発隊の命令は変更しない。
        assert!(troops[6..].iter().all(|s| w.unit(s.0).unwrap().order == s.1));
    }

    #[test]
    fn isolated_soldier_waits_for_reinforcements() {
        let mut w = World::new(12345);
        let troops = army(&mut w, 1, 8);
        command_army(&mut w, FRANCE, &mut AiState::new(1), &troops);
        assert_eq!(w.unit(troops[0].0).unwrap().order, Order::Idle);
    }

    #[test]
    fn small_groups_wait_to_form_a_wave() {
        let mut w = World::new(12345);
        for (idle, advancing) in [(5, 0), (3, 24)] {
            let troops = army(&mut w, idle, advancing);
            command_army(&mut w, FRANCE, &mut AiState::new(1), &troops);
            assert!(troops[..idle].iter().all(|s| w.unit(s.0).unwrap().order == Order::Idle));
        }
    }

    #[test]
    fn attack_cooldown_and_peace_still_prevent_sorties() {
        let mut w = World::new(12345);
        let troops = army(&mut w, 16, 0);
        let mut ai = AiState::new(1);
        ai.attack_t = 1.0;
        command_army(&mut w, FRANCE, &mut ai, &troops);
        assert!(troops.iter().all(|s| w.unit(s.0).unwrap().order == Order::Idle));
        ai.attack_t = 0.0;
        w.make_peace(FRANCE, ENGLAND, 3.0);
        command_army(&mut w, FRANCE, &mut ai, &troops);
        assert!(troops.iter().all(|s| !matches!(w.unit(s.0).unwrap().order, Order::AttackMove(_) | Order::Attack(_))));
        for _ in 0..100 {
            diplomacy(&mut w, FRANCE, &mut ai, 3);
        }
        assert!(!w.at_war(FRANCE, ENGLAND), "休戦中は再宣戦しない");
    }

    #[test]
    fn wartime_production_uses_each_barracks_without_exceeding_cap() {
        let mut w = World::new(12345);
        let home = w.nations[FRANCE].home;
        let mut ai = AiState::new(1);
        let mut barracks = Vec::new();
        for _ in 0..2 {
            let (x, y) = find_spot(&w, FRANCE, BKind::Barracks, home, 3.0, 10.0, &mut ai.rng).unwrap();
            barracks.push(w.spawn_building(FRANCE, BKind::Barracks, x, y, true));
        }
        w.update_stats();
        w.nations[FRANCE].res = [1000.0; 3];
        think(&mut w, FRANCE, &mut ai);
        assert!(barracks.iter().all(|&h| w.bld(h).unwrap().queue.len() == 1));
        w.nations[FRANCE].q_soldiers = w.nations[FRANCE].mil_cap() - w.nations[FRANCE].soldiers;
        think(&mut w, FRANCE, &mut ai);
        assert!(barracks.iter().all(|&h| w.bld(h).unwrap().queue.len() == 1));
    }

    #[test]
    fn english_army_keeps_advancing_while_a_small_guard_defends() {
        let mut w = World::new(12345);
        let pos = w.blds.iter().find(|b| b.alive && b.nation as usize == ENGLAND && !w.same_land(b.center(), w.nations[ENGLAND].home)).unwrap().center();
        let pos = w.land_near(pos, 4)[0];
        let troops = army_at(&mut w, ENGLAND, pos, 30, 0);
        w.spawn_unit(FRANCE, UT_INF, pos, 1.0);
        command_army(&mut w, ENGLAND, &mut AiState::new(1), &troops);
        let mut guards = 0;
        let mut advancing = 0;
        for s in &troops {
            if let Order::AttackMove(goal) = w.unit(s.0).unwrap().order {
                if goal.dist(pos) < 2.0 { guards += 1; } else { advancing += 1; }
            }
        }
        assert_eq!(guards, 4);
        assert_eq!(advancing, 26);
    }

    #[test]
    fn english_recruitment_prioritizes_continental_barracks() {
        let mut w = World::new(12345);
        let home = w.nations[ENGLAND].home;
        let continent = w.blds.iter().find(|b| b.alive && b.nation as usize == ENGLAND && !w.same_land(b.center(), home)).unwrap().center();
        let mut ai = AiState::new(1);
        let (x, y) = find_spot(&w, ENGLAND, BKind::Barracks, home, 3.0, 10.0, &mut ai.rng).unwrap();
        let home_barracks = w.spawn_building(ENGLAND, BKind::Barracks, x, y, true);
        let (x, y) = find_spot(&w, ENGLAND, BKind::Barracks, continent, 3.0, 10.0, &mut ai.rng).unwrap();
        let forward_barracks = w.spawn_building(ENGLAND, BKind::Barracks, x, y, true);
        w.update_stats();
        w.nations[ENGLAND].res = [150.0, 0.0, 200.0];
        w.nations[ENGLAND].income[0] = 100.0;
        think(&mut w, ENGLAND, &mut ai);
        assert_eq!(w.bld(forward_barracks).unwrap().queue.len(), 1);
        assert!(w.bld(home_barracks).unwrap().queue.is_empty());
    }
}

