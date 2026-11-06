//! 史実の英雄：エドワード黒太子、デュ・ゲクラン、ジャンヌ・ダルクなど、百年戦争に実在した将軍・王がユニットとして現れる。
//! 英雄は史実の年に現れ、史実の没年を越えて生きることはない。
//! 戦場で討たれた英雄は捕らえられ、捕らえた国は相手国に身代金を請求できる（ジャンヌ・ダルクだけは身代金で解放されない）。

use super::*;

/// 英雄のそばで戦う兵が奮い立つ範囲（タイル）
pub const HERO_AURA_R: f32 = 7.0;

pub struct HeroDef {
    pub name: &'static str,
    /// 国・活躍した年代（表示用）
    pub era: &'static str,
    pub nation: usize,
    /// 戦場に現れる年
    pub from_ad: i32,
    /// 史実の没年（この年を越えて生きることはない）
    pub death_ad: i32,
    /// 兵種（UNIT_DEFS の番号）
    pub ut: u8,
    /// 能力
    pub power: f32,
    /// 身代金（金貨。0なら身代金では解放されない）
    pub ransom: f32,
    /// まわりの兵の攻撃の上がり幅
    pub aura: f32,
    /// 現れる街（TOWNS の名前。自国のものでなければ都）
    pub town: &'static str,
    pub desc: &'static str,
}

impl HeroDef {
    pub fn kind(&self) -> UnitKind {
        udef(self.ut).class
    }
}

macro_rules! hd {
    ($n:expr, $e:expr, $nat:ident, $f:expr, $d:expr, $u:expr, $p:expr, $r:expr, $a:expr, $t:expr, $desc:expr) => {
        HeroDef { name: $n, era: $e, nation: $nat, from_ad: $f, death_ad: $d, ut: $u, power: $p, ransom: $r, aura: $a, town: $t, desc: $desc }
    };
}

/// ジャンヌ・ダルク（身代金を取れない）
pub const JOAN: usize = 13;

pub const HEROES: &[HeroDef] = &[
    // ---- イングランド
    hd!("エドワード3世", "イングランド王・1337〜1377年", ENGLAND, 1338, 1377, UT_KNIGHT_E, 3.0, 1200.0, 0.2, "ロンドン", "フランス王位を求めて百年戦争を始めた王。クレシーで大勝し、カレーを囲んだ"),
    hd!("エドワード黒太子", "イングランド・1346〜1376年", ENGLAND, 1346, 1376, UT_KNIGHT_E, 3.6, 900.0, 0.25, "ボルドー", "十六歳でクレシーに戦い、ポワティエでフランス王ジャン2世を捕らえた。黒い鎧の王太子"),
    hd!("ランカスター公ヘンリー", "イングランド・1345〜1361年", ENGLAND, 1345, 1361, UT_KNIGHT_E, 3.0, 700.0, 0.15, "ボルドー", "ヘンリー・オブ・グロスモント。ガスコーニュとブルターニュで勝ち続けた名将。黒死病に倒れた"),
    hd!("ジョン・シャンドス", "イングランド・1346〜1370年", ENGLAND, 1346, 1370, UT_KNIGHT_E, 2.8, 450.0, 0.15, "ボルドー", "黒太子の右腕。オーレーでデュ・ゲクランを捕らえ、リュサックの橋で討たれた"),
    hd!("ロバート・ノールズ", "イングランド・1351〜1407年", ENGLAND, 1351, 1407, UT_MOUNTED_ARCHER, 2.6, 350.0, 0.12, "ブレスト", "傭兵隊長から身を起こし、騎行でパリの城門まで迫った"),
    hd!("ジョン・オブ・ゴーント", "イングランド・1369〜1399年", ENGLAND, 1369, 1399, UT_KNIGHT_E, 2.6, 800.0, 0.12, "ボルドー", "エドワード3世の子、ランカスター公。カレーからボルドーへの大騎行を率いた"),
    hd!("ヘンリー・パーシー（ホットスパー）", "イングランド・1385〜1403年", ENGLAND, 1385, 1403, UT_KNIGHT_E, 2.8, 450.0, 0.15, "ニューカッスル", "スコットランドとの境を守った北の猛将。オッターバーンで捕らえられ、王に背いて討たれた"),
    hd!("ヘンリー5世", "イングランド王・1413〜1422年", ENGLAND, 1415, 1422, UT_KNIGHT_E, 4.0, 1500.0, 0.3, "サウサンプトン", "アジャンクールで大勝し、ノルマンディーを征し、トロワ条約でフランス王の後継者となった"),
    hd!("ベッドフォード公ジョン", "イングランド・1422〜1435年", ENGLAND, 1422, 1435, UT_KNIGHT_E, 3.0, 900.0, 0.15, "ルーアン", "幼いヘンリー6世に代わってフランスを治めた摂政。ヴェルヌイユで勝った"),
    hd!("ソールズベリー伯トマス・モンタギュー", "イングランド・1415〜1428年", ENGLAND, 1415, 1428, UT_KNIGHT_E, 3.0, 500.0, 0.15, "ルーアン", "攻城の名手。オルレアンの包囲のさなか、砲弾に倒れた"),
    hd!("ジョン・タルボット", "イングランド・1427〜1453年", ENGLAND, 1427, 1453, UT_KNIGHT_E, 3.4, 600.0, 0.2, "ルーアン", "「イングランドのアキレス」。パテーで捕らえられ、八十に近くしてカスティヨンで討たれた"),
    hd!("ジョン・ファストルフ", "イングランド・1415〜1459年", ENGLAND, 1415, 1459, UT_LONGBOW, 2.6, 400.0, 0.15, "ルーアン", "ニシンの戦いで補給の荷車を守り抜いた老練の騎士"),
    // ---- フランス
    hd!("フィリップ6世", "フランス王・1337〜1350年", FRANCE, 1337, 1350, UT_KNIGHT_F, 2.6, 1500.0, 0.15, "パリ", "ヴァロワ朝の初代の王。ギュイエンヌを没収して戦を始め、クレシーで敗れた"),
    hd!("ジャンヌ・ダルク", "フランス・1429〜1431年", FRANCE, 1429, 1431, UT_KNIGHT_F, 3.5, 0.0, 0.45, "ヴォークルール", "神の声を聞いたロレーヌの乙女。オルレアンを解放し、王をランスで戴冠させた。身代金では救えない"),
    hd!("ジャン2世", "フランス王・1350〜1364年", FRANCE, 1350, 1364, UT_KNIGHT_F, 2.8, 1500.0, 0.15, "パリ", "善良王。ポワティエで黒太子に捕らえられ、三百万エキュの身代金を求められた"),
    hd!("ベルトラン・デュ・ゲクラン", "フランス・1357〜1380年", FRANCE, 1357, 1380, UT_KNIGHT_F, 3.6, 600.0, 0.25, "レンヌ", "ブルターニュの小貴族から大元帥へ。会戦を避けて城を一つずつ取り返した"),
    hd!("オリヴィエ・ド・クリッソン", "フランス・1370〜1407年", FRANCE, 1370, 1407, UT_KNIGHT_F, 3.0, 500.0, 0.15, "ナント", "「屠殺者」の異名を持つブルターニュの大元帥。デュ・ゲクランの後を継いだ"),
    hd!("ジャン・ド・ヴィエンヌ", "フランス・1373〜1396年", FRANCE, 1373, 1396, UT_CROSSBOW, 2.6, 400.0, 0.12, "アルフルール", "フランスの提督。ガレー船でイングランドの海岸を襲い、ニコポリスで斃れた"),
    hd!("ブーシコー", "フランス・1391〜1421年", FRANCE, 1391, 1421, UT_KNIGHT_F, 2.8, 500.0, 0.12, "パリ", "騎士道の鑑と讃えられた元帥。アジャンクールで捕らえられ、イングランドで世を去った"),
    hd!("バカン伯ジョン・ステュアート", "フランス（スコットランド人）・1419〜1424年", FRANCE, 1419, 1424, UT_SCOTS, 3.0, 400.0, 0.2, "トゥール", "スコットランド軍を率いてフランスに渡り、ボージェで勝って大元帥となった。ヴェルヌイユで斃れた"),
    hd!("ラ・イール", "フランス・1418〜1443年", FRANCE, 1418, 1443, UT_KNIGHT_F, 3.0, 400.0, 0.2, "オルレアン", "「怒り」の名を持つ荒武者。ジャンヌとともにオルレアンとパテーで戦った"),
    hd!("ジャン・ド・デュノワ", "フランス・1427〜1468年", FRANCE, 1427, 1468, UT_KNIGHT_F, 3.2, 600.0, 0.2, "オルレアン", "オルレアンの私生児。オルレアンを守り抜き、ノルマンディーとギュイエンヌを取り戻した"),
    hd!("アルテュール・ド・リッシュモン", "フランス・1425〜1458年", FRANCE, 1425, 1458, UT_KNIGHT_F, 3.2, 700.0, 0.2, "ナント", "ブルターニュ公の弟、大元帥。パリを取り返し、フォルミニーで勝ちを決めた"),
    hd!("ジャン・ポトン・ド・ザントライユ", "フランス・1418〜1461年", FRANCE, 1418, 1461, UT_KNIGHT_F, 2.8, 350.0, 0.15, "ブールジュ", "ラ・イールの戦友。パテーでタルボットを捕らえた"),
    hd!("ジャン・ビュロー", "フランス・1436〜1463年", FRANCE, 1436, 1463, UT_ROYAL_GUNS, 3.0, 400.0, 0.2, "パリ", "王の砲兵長官。弟ガスパールとともに大砲でノルマンディーの城を次々と落とし、カスティヨンで勝った"),
];

/// 捕らえられた英雄
#[derive(Clone, Debug)]
pub struct Captive {
    pub hero: usize,
    /// 捕らえた国
    pub captor: usize,
    /// 捕らえた年（経過年）
    pub since: f32,
    /// 次に身代金を求める年（経過年。AI の催促）
    pub next_ask: f32,
}

impl World {
    /// その英雄がいま地上にいるか
    pub fn hero_on_field(&self, h: usize) -> bool {
        self.units.iter().any(|u| u.alive && u.hero as usize == h)
    }

    /// 英雄の状態（地上・捕虜・まだ現れない・世を去った）
    pub fn hero_status(&self, h: usize) -> &'static str {
        let ad = self.year_ad();
        let d = &HEROES[h];
        if self.hero_on_field(h) {
            t!("戦場に", "in the field", "在战场")
        } else if self.captives.iter().any(|c| c.hero == h) {
            t!("捕虜", "captive", "被俘")
        } else if ad < d.from_ad {
            t!("まだ現れない", "not yet", "尚未登场")
        } else {
            t!("世を去った", "gone", "已离世")
        }
    }

    /// 英雄の加護：自国の英雄の近くでは兵の攻撃が上がる（最も強い一人だけ）
    pub fn hero_aura(&self, n: usize, p: V2) -> f32 {
        let mut best = 0.0f32;
        for u in self.units.iter() {
            if u.alive && u.nation as usize == n && (u.hero as usize) < HEROES.len() && u.aboard.is_none() && u.pos.dist(p) <= HERO_AURA_R {
                best = best.max(HEROES[u.hero as usize].aura);
            }
        }
        1.0 + best
    }

    /// 英雄を地上に出す（現れる街、なければ都）
    fn spawn_hero(&mut self, h: usize, power_mul: f32) -> bool {
        let d = &HEROES[h];
        let n = d.nation;
        if !self.nations[n].alive || self.hero_on_field(h) {
            return false;
        }
        let home = TOWNS.iter().position(|t| t.name == d.town).and_then(|t| self.town_bld(t)).filter(|&bi| self.blds[bi].nation as usize == n);
        let bi = match home {
            Some(bi) => bi,
            None => match self.nations[n].capital.and_then(|c| self.bld(c).map(|_| c.idx as usize)) {
                Some(bi) => bi,
                None => return false,
            },
        };
        let bh = self.bh(bi);
        let Some(sp) = self.spawn_point(bh) else { return false };
        let pw = (d.power * power_mul).max(1.0);
        let uh = self.spawn_unit(n, d.ut, sp, pw);
        self.unit_mut(uh).unwrap().hero = h as u8;
        true
    }

    /// 毎年：史実の年に英雄が現れ、没年が来れば世を去る。捕虜の身代金を AI が求める
    pub(super) fn yearly_heroes(&mut self) {
        let ad = self.year_ad();
        if ad == self.hero_year {
            return;
        }
        self.hero_year = ad;
        let p = self.player;
        for h in 0..HEROES.len() {
            let d = &HEROES[h];
            // 没年：戦場にいても捕虜でも世を去る
            if ad >= d.death_ad && ad < d.death_ad + 1 {
                let mut gone = false;
                for i in 0..self.units.len() {
                    if self.units[i].alive && self.units[i].hero as usize == h {
                        self.kill_unit(i);
                        gone = true;
                    }
                }
                if let Some(k) = self.captives.iter().position(|c| c.hero == h) {
                    self.captives.remove(k);
                    gone = true;
                }
                if gone {
                    let t = if h == JOAN {
                        t!("ジャンヌ・ダルクはコンピエーニュで捕らえられ、ルーアンで異端として火刑に処された（1431年）", "Joan of Arc was captured at Compiègne and burned as a heretic at Rouen (1431)", "贞德在贡比涅被俘，以异端之名在鲁昂被处以火刑（1431年）").to_string()
                    } else {
                        tf!("{}が世を去った（{}年）", "{} has died ({})", "{}去世了（{}年）", tx(d.name), d.death_ad)
                    };
                    self.push_msg(t.clone(), [0.85, 0.8, 0.95]);
                    if d.nation == p || h == JOAN {
                        self.queue_banner(Banner { title: tx(d.name).to_string(), text: t, t: 6.0 });
                    }
                }
            }
            // 登場
            if ad == d.from_ad && ad < d.death_ad && !self.captives.iter().any(|c| c.hero == h) && self.spawn_hero(h, 1.0) {
                let t = tf!("英雄・{}が{}に現れた", "The hero {} has come to the side of {}", "英雄{}投效{}", tx(d.name), tx(self.nations[d.nation].name));
                self.push_msg(t.clone(), [1.0, 0.88, 0.5]);
                if d.nation == p || h == JOAN {
                    self.queue_banner(Banner { title: tf!("英雄・{}", "Hero: {}", "英雄・{}", tx(d.name)), text: format!("{}\n{}", tx(d.desc), t), t: 7.0 });
                }
            }
        }
        // AI が捕らえた英雄の身代金を求める
        let years = self.years;
        let snapshot: Vec<Captive> = self.captives.clone();
        for c in snapshot {
            let Some(k) = self.captives.iter().position(|x| x.hero == c.hero) else { continue };
            if years < c.next_ask {
                continue;
            }
            let owner = HEROES[c.hero].nation;
            if c.captor == p && !self.ai_player {
                continue;
            }
            self.captives[k].next_ask = years + 4.0;
            let amount = self.ransom_price(c.hero);
            if owner == p && !self.ai_player {
                if !self.peace_asks.iter().any(|q| q.ransom.map_or(false, |r| r.0 == c.hero)) {
                    let why = tf!("{}の身代金", "ransom for {}", "{}的赎金", tx(HEROES[c.hero].name));
                    let t = tf!("{0}が{2}の身代金として金貨{1:.0}を求めてきた", "{0} demands {1:.0} gold as ransom for {2}", "{0}索要金币{1:.0}作为{2}的赎金", tx(self.nations[c.captor].name), amount, tx(HEROES[c.hero].name));
                    self.push_msg(t, [1.0, 0.85, 0.5]);
                    self.peace_asks.push_back(PeaceAsk { from: c.captor, years: 0.0, why, ransom: Some((c.hero, amount)) });
                }
            } else if self.nations[owner].coin >= amount {
                // AI どうし：払えれば払う
                self.pay_ransom(c.hero, amount);
            }
        }
    }

    /// 身代金の額（年代とともに少し上がる）
    pub fn ransom_price(&self, h: usize) -> f32 {
        (HEROES[h].ransom * self.era_mult()).round()
    }

    /// 英雄が討たれた：国に討たれれば捕虜になり、そうでなければ斃れる
    pub(super) fn hero_fallen(&mut self, ui: usize, by: usize) {
        let h = self.units[ui].hero as usize;
        let n = self.units[ui].nation as usize;
        self.kill_unit(ui);
        let d = &HEROES[h];
        let p = self.player;
        if by >= NATIONS.len() || by == n {
            let t = tf!("英雄・{}が討ち死にした", "The hero {} has fallen in battle", "英雄{}阵亡了", tx(d.name));
            self.push_msg(t, [1.0, 0.6, 0.5]);
            return;
        }
        if h == JOAN {
            // ジャンヌは身代金では救えない
            let t = t!("ジャンヌ・ダルクが捕らえられた。フランス王は身代金を払わず、乙女はルーアンで異端として火刑に処された……", "Joan of Arc has been captured. The King of France paid no ransom, and the Maid was burned as a heretic at Rouen...", "贞德被俘。法兰西国王没有支付赎金，少女在鲁昂以异端之名被处以火刑……").to_string();
            self.push_msg(t.clone(), [1.0, 0.5, 0.4]);
            self.queue_banner(Banner { title: t!("乙女の最期", "The Maid's End", "少女的结局").to_string(), text: t, t: 8.0 });
            self.nations[by].aggression += 10.0;
            return;
        }
        self.captives.push(Captive { hero: h, captor: by, since: self.years, next_ask: self.years + 1.0 });
        let t = tf!("英雄・{}が{}に捕らえられた（身代金 金貨{:.0}）", "The hero {} has been captured by {} (ransom: {:.0} gold)", "英雄{}被{}俘获（赎金：金币{:.0}）", tx(d.name), tx(self.nations[by].name), self.ransom_price(h));
        let col = if by == p { [0.6, 1.0, 0.6] } else if n == p { [1.0, 0.45, 0.35] } else { [0.9, 0.85, 0.7] };
        self.push_msg(t.clone(), col);
        if by == p || n == p {
            let more = if by == p {
                t!("外交画面（F2）で相手国に身代金を請求できる。", "You can demand a ransom from the enemy on the Diplomacy screen (F2).", "可在外交界面（F2）向敌国索要赎金。")
            } else {
                t!("相手国が身代金を求めてくる。払えば英雄は戻ってくる。", "The enemy will demand a ransom. Pay it and the hero returns.", "敌国会索要赎金。支付后英雄即可归来。")
            };
            self.queue_banner(Banner { title: tf!("{}の捕縛", "{} Captured", "{}被俘", tx(d.name)), text: format!("{}\n{}", t, more), t: 7.0 });
        }
    }

    /// 身代金を払い、英雄を国へ戻す
    fn pay_ransom(&mut self, h: usize, amount: f32) -> bool {
        let Some(k) = self.captives.iter().position(|c| c.hero == h) else { return false };
        let captor = self.captives[k].captor;
        let owner = HEROES[h].nation;
        if self.nations[owner].coin < amount || !self.nations[owner].alive {
            return false;
        }
        self.nations[owner].coin -= amount;
        self.nations[captor].coin += amount;
        self.nations[captor].coin_earned += amount;
        self.captives.remove(k);
        let back = self.year_ad() < HEROES[h].death_ad && self.spawn_hero(h, 0.9);
        let t = tf!("{0}が金貨{2:.0}の身代金で{1}から解き放たれた", "{0} was released by {1} for a ransom of {2:.0} gold", "{0}以金币{2:.0}的赎金从{1}获释", tx(HEROES[h].name), tx(self.nations[captor].name), amount);
        self.push_msg(t, [0.95, 0.9, 0.6]);
        let _ = back;
        true
    }

    /// プレイヤーが捕らえた英雄の身代金を相手国に求める（AI は払えれば払う）
    pub fn demand_ransom(&mut self, h: usize) -> Result<f32, &'static str> {
        let Some(c) = self.captives.iter().find(|c| c.hero == h) else { return Err("") };
        if c.captor != self.player {
            return Err("");
        }
        let owner = HEROES[h].nation;
        if !self.nations[owner].alive {
            return Err(t!("相手の国はもう無い", "That kingdom is gone", "对方国家已不存在"));
        }
        let amount = self.ransom_price(h);
        if self.nations[owner].coin < amount {
            return Err(t!("相手国は今は身代金を払えない（金貨が足りない）", "The enemy cannot pay the ransom yet (not enough gold)", "敌国目前付不起赎金（金币不足）"));
        }
        self.pay_ransom(h, amount);
        Ok(amount)
    }

    /// 相手国が求める身代金に答える（プレイヤー）
    pub(super) fn answer_ransom(&mut self, h: usize, amount: f32, accept: bool) {
        if accept {
            if !self.pay_ransom(h, amount) {
                self.push_msg(t!("金貨が足りず、身代金を払えなかった", "Not enough gold to pay the ransom", "金币不足，无法支付赎金").to_string(), [1.0, 0.6, 0.4]);
            }
        } else {
            let t = tf!("{}の身代金を払わなかった（捕虜のまま）", "Refused to pay the ransom for {} (still a captive)", "拒付{}的赎金（仍为俘虏）", tx(HEROES[h].name));
            self.push_msg(t, [1.0, 0.7, 0.5]);
        }
    }
}
