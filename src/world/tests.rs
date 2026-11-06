//! 世界のシミュレーションのテスト

use super::*;

fn town(w: &World, name: &str) -> usize {
    w.town_bld(town_id(name)).unwrap_or_else(|| panic!("{} が無い", name))
}

/// 年表は昇順で、国の番号と君主家の表が国家データと揃っている
#[test]
fn history_table() {
    assert_eq!(INITIAL_HOUSE.len(), NATIONS.len());
    for w in HISTORY.windows(2) {
        assert!(w[0].ad <= w[1].ad, "{} の後に {}", w[0].title, w[1].title);
    }
    for e in HISTORY {
        assert!(e.nation < NATIONS.len());
        assert!(e.ad >= START_AD as i32 && e.ad <= END_AD as i32);
        for fx in e.fx {
            if let Fx::Cede(name, _) = fx {
                town_id(name);
            }
            if let Fx::DefectIfHero(_, _, h) = fx {
                assert_eq!(HEROES[*h].name, "ジャンヌ・ダルク");
            }
        }
    }
    assert_eq!(HEROES[JOAN].name, "ジャンヌ・ダルク");
    assert_eq!(HEROES[JOAN].ransom, 0.0);
    for h in HEROES {
        assert!(h.from_ad < h.death_ad, "{}", h.name);
        town_id(h.town);
    }
}

/// すべての街が置かれ、それぞれの持ち主の国に属する。都はロンドンとパリ
#[test]
fn towns_placed() {
    let w = World::new(12345);
    for (k, t) in TOWNS.iter().enumerate() {
        let bi = w.town_bld(k).unwrap_or_else(|| panic!("{} が置けない", t.name));
        assert_eq!(w.blds[bi].nation as usize, t.owner, "{}", t.name);
        let (x, y) = ll_to_tile(t.lon, t.lat);
        assert!(w.blds[bi].center().dist(v2(x, y)) < 6.0, "{} が遠くに置かれた", t.name);
    }
    assert_eq!(w.blds[w.nations[ENGLAND].capital.unwrap().idx as usize].town as usize, town_id("ロンドン"));
    assert_eq!(w.blds[w.nations[FRANCE].capital.unwrap().idx as usize].town as usize, town_id("パリ"));
    assert!(w.war[ENGLAND][FRANCE], "最初から戦になっていない");
}

/// ブリテン島と大陸は海で隔てられ、大陸の街どうしは陸でつながる
#[test]
fn geography() {
    let w = World::new(12345);
    let c = |n: &str| w.blds[town(&w, n)].center();
    assert!(!w.same_land(c("ロンドン"), c("パリ")), "ブリテン島が大陸とつながっている");
    assert!(w.same_land(c("ロンドン"), c("ヨーク")) && w.same_land(c("ロンドン"), c("ハーレック")) && w.same_land(c("ロンドン"), c("ベリック")));
    for n in ["ボルドー", "カレー", "ブレスト", "ヘント", "リヨン", "バイヨンヌ", "ニーム", "シェルブール"] {
        assert!(w.same_land(c("パリ"), c(n)), "{} がパリと陸でつながらない", n);
    }
    // 陸路で行ける
    let mut pf = crate::path::PathFinder::new();
    let path = pf.find(&w.cost, c("パリ").tile(), c("ボルドー").tile(), 3, 400000, crate::path::UNIT_WEIGHT);
    assert!(!path.is_empty(), "パリからボルドーへ行けない");
    assert_eq!(w.continental_towns(ENGLAND), 4);
}

/// 街は築けないが、村は自国領内に建てられる
#[test]
fn villages_not_towns() {
    let mut w = World::new(12345);
    let n = FRANCE;
    w.nations[n].res = [9999.0; 3];
    let home = w.nations[n].home;
    let mut built = None;
    'o: for r in 5..12 {
        for dy in -r..=r {
            for dx in -r..=r {
                let (tx, ty) = (home.x as i32 + dx, home.y as i32 + dy);
                assert!(w.can_place(n, BKind::Town, tx, ty).is_err());
                if let Ok(h) = w.place_building(n, BKind::Village, tx, ty) {
                    built = Some(h);
                    break 'o;
                }
            }
        }
    }
    let v = built.expect("村を建てられない");
    let cap0 = w.nations[n].worker_cap();
    w.bld_mut(v).unwrap().progress = 1.0;
    w.update_stats();
    assert_eq!(w.nations[n].worker_cap(), cap0 + 1, "村で上限が上がらない");
    assert!(w.nearest_dropsite(n, w.bld(v).unwrap().center()).is_some());
}

/// 敵の村を壊すと略奪になり、資源と金貨を奪える
#[test]
fn plunder_village() {
    let mut w = World::new(12345);
    let bdx = w.blds[town(&w, "ボルドー")].center();
    // ベルジュラックの近くにフランスの村
    let berg = w.blds[town(&w, "ベルジュラック")].center();
    let mut vill = None;
    'o: for r in 4..12 {
        for dy in -r..=r {
            for dx in -r..=r {
                if w.can_place(FRANCE, BKind::Village, berg.x as i32 + dx, berg.y as i32 + dy).is_ok() {
                    vill = Some(w.spawn_building(FRANCE, BKind::Village, berg.x as i32 + dx, berg.y as i32 + dy, true));
                    break 'o;
                }
            }
        }
    }
    let vill = vill.expect("村が置けない");
    let _ = bdx;
    let r0 = w.nations[ENGLAND].res;
    let c0 = w.nations[ENGLAND].coin;
    let vi = vill.idx as usize;
    w.damage_bld(vi, 99999.0, ENGLAND);
    assert!(w.bld(vill).is_none(), "村が残っている");
    assert!(w.nations[ENGLAND].res[0] > r0[0] + 100.0, "糧を奪えていない");
    assert!(w.nations[ENGLAND].coin > c0, "金貨を奪えていない");
}

/// 諸侯の寝返り：1419年のブルゴーニュ公の同盟でブルゴーニュの街がイングランドに付き、1435年のアラスの和約で戻る
#[test]
fn lords_defect() {
    let mut w = World::new(12345);
    w.player = ENGLAND;
    w.ai_player = true;
    let dijon = town(&w, "ディジョン");
    let ghent = town(&w, "ヘント");
    assert!(w.defect(L_BURGUNDY, ENGLAND).is_some());
    assert!(w.defect(L_FLANDERS, ENGLAND).is_some());
    assert_eq!(w.blds[dijon].nation as usize, ENGLAND);
    assert_eq!(w.blds[ghent].nation as usize, ENGLAND);
    // もう付いている諸侯は動かない
    assert!(w.defect(L_BURGUNDY, ENGLAND).is_none());
    // パリが寝返ると、フランスは都を移す
    assert!(w.defect(L_PARIS, ENGLAND).is_some());
    let cap = w.nations[FRANCE].capital.expect("フランスの都が無い");
    assert_ne!(cap.idx as usize, town(&w, "パリ"));
    assert!(w.nations[FRANCE].alive);
    w.defect(L_PARIS, FRANCE);
    assert_eq!(w.blds[town(&w, "パリ")].nation as usize, FRANCE);
}

/// 史実の年表どおりに諸侯が動く（年を飛ばして確かめる）
#[test]
fn history_runs() {
    let mut w = World::new(777);
    w.player = FRANCE;
    w.ai_player = true;
    w.hist_skip_to(1340.5 - START_AD);
    assert_eq!(w.blds[town(&w, "ヘント")].nation as usize, ENGLAND, "1340年にフランドルがイングランドに付いていない");
    w.hist_skip_to(1420.5 - START_AD);
    assert_eq!(w.blds[town(&w, "ディジョン")].nation as usize, ENGLAND, "ブルゴーニュが寝返っていない");
    assert_eq!(w.blds[town(&w, "パリ")].nation as usize, ENGLAND, "トロワ条約でパリが渡っていない");
    w.hist_skip_to(1436.5 - START_AD);
    assert_eq!(w.blds[town(&w, "ディジョン")].nation as usize, FRANCE, "アラスの和約で戻っていない");
    assert_eq!(w.blds[town(&w, "パリ")].nation as usize, FRANCE, "パリが戻っていない");
    assert_eq!(w.nations[ENGLAND].house, "ランカスター家");
}

/// 英雄は史実の年に現れ、没年を越えて生きない。討たれると捕虜になり、身代金で戻る。ジャンヌは身代金で救えない
#[test]
fn heroes_and_ransom() {
    let mut w = World::new(4242);
    w.player = ENGLAND;
    w.hist_skip_to(1357.2 - START_AD);
    w.hero_year = 0;
    w.yearly_heroes();
    let dg = HEROES.iter().position(|h| h.name == "ベルトラン・デュ・ゲクラン").unwrap();
    assert!(w.hero_on_field(dg), "デュ・ゲクランが現れない");
    // 討たれて捕虜に
    let i = (0..w.units.len()).find(|&i| w.units[i].alive && w.units[i].hero as usize == dg).unwrap();
    w.damage_unit(i, 99999.0, None, ENGLAND);
    assert!(!w.hero_on_field(dg));
    assert!(w.captives.iter().any(|c| c.hero == dg && c.captor == ENGLAND));
    // フランスに金貨があれば身代金を払う
    w.nations[FRANCE].coin = 0.0;
    assert!(w.demand_ransom(dg).is_err());
    w.nations[FRANCE].coin = 5000.0;
    let e0 = w.nations[ENGLAND].coin;
    let paid = w.demand_ransom(dg).expect("身代金を取れない");
    assert!(w.nations[ENGLAND].coin >= e0 + paid - 0.1);
    assert!(w.hero_on_field(dg), "身代金を払っても戻らない");
    // 没年を越えて生きない
    w.hist_skip_to(1380.2 - START_AD);
    w.hero_year = 0;
    w.yearly_heroes();
    assert!(!w.hero_on_field(dg), "デュ・ゲクランが没年を越えて生きている");
    // ジャンヌ
    w.hist_skip_to(1429.2 - START_AD);
    w.hero_year = 0;
    w.yearly_heroes();
    assert!(w.hero_on_field(JOAN), "ジャンヌが現れない");
    let j = (0..w.units.len()).find(|&i| w.units[i].alive && w.units[i].hero as usize == JOAN).unwrap();
    w.damage_unit(j, 99999.0, None, ENGLAND);
    assert!(!w.hero_on_field(JOAN));
    assert!(!w.captives.iter().any(|c| c.hero == JOAN), "ジャンヌが身代金の対象になっている");
}

/// 勝敗：大陸からイングランドを追い出せばフランスの勝ち、フランスの街をすべて取ればイングランドの勝ち
#[test]
fn victory_conditions() {
    let mut w = World::new(12345);
    w.player = FRANCE;
    for n in ["ボルドー", "バイヨンヌ", "モン＝ド＝マルサン", "アブヴィル"] {
        let bi = town(&w, n);
        w.capture_city(bi, FRANCE);
    }
    assert_eq!(w.continental_towns(ENGLAND), 0);
    w.check_victory();
    assert!(matches!(&w.game_over, Some((true, _))), "フランスが勝っていない");

    let mut w = World::new(12345);
    w.player = ENGLAND;
    let fr: Vec<usize> = (0..w.blds.len()).filter(|&i| w.blds[i].alive && w.blds[i].kind.city() && w.blds[i].nation as usize == FRANCE).collect();
    for bi in fr {
        if w.blds[bi].alive && w.blds[bi].nation as usize == FRANCE {
            w.capture_city(bi, ENGLAND);
        }
    }
    assert!(!w.nations[FRANCE].alive);
    assert!(matches!(&w.game_over, Some((true, _))), "イングランドが勝っていない");
}

/// 1453年が暮れても決着がつかなければ、ジャンヌの亡霊が両国を滅ぼす
#[test]
fn ghost_of_joan() {
    let mut w = World::new(12345);
    w.player = FRANCE;
    w.ai_player = true;
    w.years = END_AD - START_AD + 1.0;
    w.yearly();
    assert!(w.descent.is_some(), "亡霊が現れない");
    assert!(w.units.iter().any(|u| u.alive && u.ut == UT_GHOST));
    // 天使は傷つかない
    let a = (0..w.units.len()).find(|&i| w.units[i].alive && w.units[i].ut == UT_ANGEL).unwrap();
    let hp = w.units[a].hp;
    w.damage_unit(a, 9999.0, None, FRANCE);
    assert_eq!(w.units[a].hp, hp);
    for _ in 0..6 {
        w.years += 1.0;
        w.yearly();
        if w.game_over.is_some() {
            break;
        }
    }
    assert!(!w.nations[ENGLAND].alive && !w.nations[FRANCE].alive, "両国が滅びていない");
    assert!(matches!(&w.game_over, Some((false, _))));
}

/// イングランドはコグ船を造れ、フランスは港を築けないがガレー船を雇える
#[test]
fn ships_and_galleys() {
    let mut w = World::new(12345);
    assert!(w.can_build_ships(ENGLAND) && !w.can_build_ships(FRANCE));
    let galley = TRIBES.iter().position(|t| t.name == "ジェノヴァのガレー船").unwrap();
    assert!(w.can_hire(ENGLAND, galley).is_err());
    w.nations[FRANCE].coin = 2000.0;
    let (hs, _) = w.hire_mercs(FRANCE, galley).expect("ガレー船を雇えない");
    assert_eq!(hs.len(), 2);
    for h in hs.iter() {
        let u = w.unit(*h).unwrap();
        assert!(u.kind.naval() && w.passable_l(u.pos, true), "ガレー船が海の上にいない");
    }
    // フランスの兵を船でイングランドへ
    assert!(w.has_ships(FRANCE));
    let rouen = w.blds[town(&w, "ルーアン")].center();
    let land = w.land_near(rouen, 4);
    let mut troops = Vec::new();
    for k in 0..4 {
        troops.push(w.spawn_unit(FRANCE, UT_KNIGHT_F, land[(k + 8) % land.len()], 1.0));
    }
    // ヘースティングズの浜（街の弓の届かない所）
    let (hx, hy) = ll_to_tile(0.40, 50.95);
    let dover = w.land_near(v2(hx, hy), 5)[0];
    w.order_move(&troops, dover, false);
    for _ in 0..(160.0 / DT) as usize {
        w.step();
        if troops.iter().all(|&h| w.unit(h).map_or(true, |u| u.aboard.is_none() && w.same_land(u.pos, dover))) {
            break;
        }
    }
    let across = troops.iter().filter(|&&h| w.unit(h).map_or(false, |u| u.aboard.is_none() && w.same_land(u.pos, dover))).count();
    assert!(across >= 4, "フランスの兵が海峡を渡れない（{}人）", across);
}

/// スコットランド軍はイングランドを襲う（フランスは襲わない）
#[test]
fn scots_raid_england() {
    let mut w = World::new(12345);
    w.player = FRANCE;
    w.ai_player = true;
    assert!(w.tribe_raid(0, true), "スコットランド軍が出ない");
    let b = w.trade.bands.last().unwrap();
    assert_eq!(b.target, ENGLAND);
    assert!(TRIBES[0].hire == 0, "スコットランドが雇えてしまう");
}

#[test]
fn long_sim() {
    let seed = std::env::var("SIM_SEED").ok().and_then(|s| s.parse().ok()).unwrap_or(12345u32);
    let mut w = World::new(seed);
    w.player = FRANCE;
    w.ai_player = true;
    let t0 = std::time::Instant::now();
    let years = std::env::var("SIM_YEARS").ok().and_then(|s| s.parse().ok()).unwrap_or(130.0f32);
    let mut next_report = 0.0;
    let mut seen_t = -1.0f32;
    let mut steps = 0usize;
    while w.years < years + 1.0 {
        steps += 1;
        w.step();
        if w.years >= next_report {
            next_report += 5.0;
            let alive: Vec<String> = (0..NATIONS.len())
                .map(|i| &w.nations[i])
                .filter(|n| n.alive)
                .map(|n| format!("{}(街{} 村{} 民{} 兵{} 力{:.0} 糧{:.0}木{:.0}鉄{:.0} 金{:.0})", n.name, n.cities, n.villages, n.workers, n.soldiers, n.strength, n.res[0], n.res[1], n.res[2], n.coin))
                .collect();
            let units = w.units.iter().filter(|u| u.alive).count();
            let ships = w.units.iter().filter(|u| u.alive && u.kind.naval()).count();
            println!("AD{} t={:.1}s units={} ships={} 大陸の英={} 捕虜={} :: {}", w.year_ad(), t0.elapsed().as_secs_f32(), units, ships, w.continental_towns(ENGLAND), w.captives.len(), alive.join(" "));
        }
        for m in w.msgs.iter().filter(|m| m.time > seen_t) {
            println!("  [msg] {}", m.text);
        }
        seen_t = w.time;
        if w.game_over.is_some() {
            println!("決着：{:?}", w.game_over);
            break;
        }
    }
    println!("elapsed {:.1}s for {} steps", t0.elapsed().as_secs_f32(), steps);
}


#[test]
#[ignore]
fn dump_map() {
    let w = World::new(12345);
    let mut out = format!("P6 {} {} 255\n", MAP_W, MAP_H).into_bytes();
    let (lx, ly) = ll_to_tile(TOWNS[0].lon, TOWNS[0].lat);
    let lr = w.land_region_at(v2(lx, ly));
    for i in 0..MAP_W * MAP_H {
        let c: [u8; 3] = match w.map.terrain[i] {
            Terrain::Sea => [20, 50, 110],
            Terrain::Lake => [60, 110, 200],
            Terrain::River => [70, 130, 220],
            Terrain::Plain => if w.land_region[i] == lr { [150, 190, 90] } else { [120, 170, 80] },
            Terrain::Steppe => [200, 190, 120],
            Terrain::Forest => [40, 100, 40],
            Terrain::Hill => [160, 140, 90],
            Terrain::Mountain => [120, 110, 100],
            Terrain::Desert => [220, 200, 140],
        };
        out.extend_from_slice(&c);
    }
    for b in w.blds.iter().filter(|b| b.alive) {
        let (x, y) = b.center().tile();
        for dy in -1..=1 {
            for dx in -1..=1 {
                let i = Map::idx(x + dx, y + dy) * 3 + out.len() - MAP_W * MAP_H * 3;
                let col = if b.nation as usize == ENGLAND { [255, 40, 40] } else { [255, 255, 255] };
                out[i..i + 3].copy_from_slice(&col);
            }
        }
    }
    std::fs::write(std::env::var("MAP_OUT").unwrap(), out).unwrap();
}

#[test]
#[ignore]
fn dump_regions() {
    let w = World::new(12345);
    for (k, t) in TOWNS.iter().enumerate() {
        let bi = w.town_bld(k).unwrap();
        let c = w.blds[bi].center();
        println!("{} {:?} reg={}", t.name, c, w.land_region_at(c));
    }
}
