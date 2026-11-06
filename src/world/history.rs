//! 史実イベント：王・将軍・政治家の登場、諸侯の寝返りと同盟（街の離脱）、休戦と開戦、
//! 黒死病・反乱、スコットランドの侵攻
//!
//! 年表は西暦の年で並べ、その年が来たら発生する。対象国が滅亡していれば省略する。

use super::*;

/// 各国の始まりの君主家。NATIONS と同じ並び
pub const INITIAL_HOUSE: &[&str] = &["プランタジネット家", "ヴァロワ家"];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HKind {
    /// 英雄・名将・名君
    Hero,
    /// 学者・宗教者
    Sage,
    /// 政治指導者（宰相・改革者）
    Minister,
    /// 反乱・内乱
    Revolt,
    /// 簒奪・乗っ取り（君主家の交代）
    Usurp,
    /// 旧君主家の復辟
    Restore,
    /// 大敗・災厄
    Disaster,
}

impl HKind {
    pub fn label(self) -> &'static str {
        match self {
            HKind::Hero => t!("英雄", "Hero", "英雄"),
            HKind::Sage => t!("賢者", "Sage", "贤者"),
            HKind::Minister => t!("政治指導者", "Statesman", "政治家"),
            HKind::Revolt => t!("反乱", "Revolt", "叛乱"),
            HKind::Usurp => t!("簒奪", "Usurpation", "篡位"),
            HKind::Restore => t!("復辟", "Restoration", "复辟"),
            HKind::Disaster => t!("大敗", "Disaster", "大败"),
        }
    }
    /// 国にとって不利な出来事か
    pub fn bad(self) -> bool {
        matches!(self, HKind::Revolt | HKind::Usurp | HKind::Disaster)
    }
    pub fn color(self) -> [f32; 3] {
        if self.bad() {
            [1.0, 0.55, 0.45]
        } else {
            [0.75, 0.9, 1.0]
        }
    }
}

/// イベントの効果（対象は原則としてイベントの国）
#[derive(Clone, Copy, Debug)]
pub enum Fx {
    /// 人物が仕える：生産（税・採集）と兵の攻撃力を一定年数だけ高める（負なら混乱）
    Figure(&'static str, f32, f32, f32),
    /// 人物が去る
    Dismiss(&'static str),
    /// 資源を得る
    Grant(f32, f32, f32),
    /// 金貨を得る
    Gold(f32),
    /// 全兵の能力を上げる（時代上限まで）
    Train(f32),
    /// 都に兵を召集
    Levy(u32),
    /// 兵の一部が離反・戦死する
    Purge(f32),
    /// 蓄えの一部を失う
    Plunder(f32),
    /// 疫病：民と兵の一部が死ぬ（両国とも）
    Plague(f32),
    /// 君主家が変わる
    House(&'static str),
    /// a と b が休戦する
    Truce(usize, usize, f32),
    /// a が b に宣戦する
    War(usize, usize),
    /// 生産能力バイアスを掛ける（国の盛衰）
    Bias(f32),
    /// 諸侯が寝返る・同盟する：その諸侯の街が指定の国に付く
    Defect(u8, usize),
    /// 英雄が生きて戦場にいれば、諸侯の街が付く（ジャンヌのランス戴冠など）
    DefectIfHero(u8, usize, usize),
    /// 街が渡される（条約・質入れ）
    Cede(&'static str, usize),
    /// スコットランド軍がイングランドに攻め込む
    ScotsRaid,
    /// スコットランド軍がフランスに渡って加勢する（兵の数）
    ScotsArmy(u32),
}

pub struct HEvent {
    pub ad: i32,
    pub nation: usize,
    pub kind: HKind,
    pub title: &'static str,
    pub text: &'static str,
    pub fx: &'static [Fx],
}

macro_rules! ev {
    ($bc:expr, $n:expr, $k:ident, $t:expr, $x:expr, [$($fx:expr),* $(,)?]) => {
        HEvent { ad: $bc, nation: $n, kind: HKind::$k, title: $t, text: $x, fx: &[$($fx),*] }
    };
}

use Fx::*;

/// 史実年表（西暦の昇順）。tools/gendata.py が生成する
pub const HISTORY: &[HEvent] = &[
    ev!(1337, FRANCE, Minister, "ギュイエンヌの没収", "フィリップ6世は臣下エドワード3世のギュイエンヌ公領を没収した。百年にわたる戦の始まりである。", [Levy(3)]),
    ev!(1337, ENGLAND, Hero, "フランス王位の要求", "母イザベルを通じてカペー家の血を引くエドワード3世は、自らこそフランス王であると宣言した。", [Gold(200.0), Levy(3)]),
    ev!(1338, ENGLAND, Minister, "羊毛の禁輸", "エドワード3世は羊毛の輸出を止め、イングランドの羊毛なしには織機の止まるフランドルの都市を揺さぶった。", [Gold(150.0)]),
    ev!(1340, ENGLAND, Hero, "フランドルの同盟", "ヘントの指導者ヤーコプ・ファン・アルテフェルデはエドワード3世をフランス王と認め、フランドルの都市はイングランドに付いた。", [Defect(L_FLANDERS, ENGLAND)]),
    ev!(1340, FRANCE, Disaster, "スロイスの海戦", "ズウィン湾に集めたフランスの艦隊は、イングランドの弓兵に射すくめられて全滅した。", [Plunder(0.15)]),
    ev!(1341, ENGLAND, Hero, "ブルターニュ継承戦争", "ブルターニュ公の後継をめぐり、モンフォール家はイングランドと、ブロワ家はフランスと結んだ。ブレストとヴァンヌがイングランドに付く。", [Defect(L_MONTFORT, ENGLAND)]),
    ev!(1346, ENGLAND, Hero, "クレシーの戦い", "ノルマンディーを駆けたエドワード3世の軍は、クレシーでフランス騎士の突撃を長弓で打ち砕いた。", [Train(0.5)]),
    ev!(1346, ENGLAND, Revolt, "デイヴィッド2世の侵攻", "フランス王の求めに応じ、スコットランド王デイヴィッド2世が国境を越えて攻め込んだ。", [ScotsRaid]),
    ev!(1348, ENGLAND, Disaster, "黒死病", "東から来た疫病がフランスとイングランドを覆い、人々の三人に一人が死んだ。王たちも戦を休まざるを得なかった。", [Plague(0.25), Truce(ENGLAND, FRANCE, 3.0)]),
    ev!(1349, FRANCE, Minister, "フランドル伯ルイ・ド・マール", "アルテフェルデの死の後、フランドル伯ルイ・ド・マールは都市を従え、フランス王のもとに戻った。", [Defect(L_FLANDERS, FRANCE)]),
    ev!(1354, ENGLAND, Hero, "ナバラ王シャルル（悪王）", "ノルマンディーに領地を持つナバラ王シャルルはフランス王の大元帥を暗殺し、イングランドと結んだ。", [Defect(L_NAVARRE, ENGLAND)]),
    ev!(1355, ENGLAND, Hero, "黒太子の大騎行", "黒太子はボルドーから地中海の近くまで駆け、五百の村を焼いて莫大な戦利品を持ち帰った。", [Gold(250.0), Train(0.3)]),
    ev!(1356, ENGLAND, Hero, "ポワティエの戦い", "黒太子はポワティエで数に勝るフランス軍を破った。王を守る騎士たちは馬を降りて戦い、倒れていった。", [Train(0.3)]),
    ev!(1357, FRANCE, Revolt, "エティエンヌ・マルセルの反乱", "王を失った混乱の中、パリの商人頭エティエンヌ・マルセルが三部会を率いて王太子に迫った。", [Plunder(0.15), Purge(0.1)]),
    ev!(1358, FRANCE, Revolt, "ジャックリーの乱", "戦と略奪に苦しむボーヴェジの農民が領主に背き、城館を焼いた。", [Plunder(0.15), Purge(0.1)]),
    ev!(1360, FRANCE, Minister, "ブレティニー条約", "ランスの包囲に失敗したエドワード3世は和を結んだ。フランスは王の身代金三百万エキュを約した。", [Truce(ENGLAND, FRANCE, 9.0), Plunder(0.1)]),
    ev!(1361, ENGLAND, Disaster, "疫病の再来", "黒死病が再び両国を襲った。ランカスター公ヘンリーもこの年に世を去った。", [Plague(0.12)]),
    ev!(1364, FRANCE, Hero, "賢明王シャルル5世", "シャルル5世が即位した。会戦を避けて税を整え、城を一つずつ取り戻す王である。", [Figure("シャルル5世", 0.2, 0.15, 16.0)]),
    ev!(1364, FRANCE, Hero, "コシュレルの戦い", "デュ・ゲクランがコシュレルでナバラ王の軍を破り、ナバラ王はフランス王と和を結んだ。", [Defect(L_NAVARRE, FRANCE)]),
    ev!(1369, FRANCE, Hero, "ガスコーニュ諸侯の上訴", "黒太子の重い炉税に怒ったアルブレ家とアルマニャック家はパリの王に訴え、シャルル5世は再び戦を始めた。", [Defect(L_GASCON, FRANCE), War(FRANCE, ENGLAND)]),
    ev!(1372, FRANCE, Hero, "ラ・ロシェルの海戦", "フランスと結んだカスティーリャのガレー船がイングランドの艦隊を焼き、ラ・ロシェルの市民はフランス王に城門を開いた。", [Cede("ラ・ロシェル", FRANCE)]),
    ev!(1372, ENGLAND, Hero, "ブルターニュ公ジャン4世の同盟", "オーレーで勝ってブルターニュ公となったモンフォール家のジャン4世は、イングランドと同盟を結んだ。", [Defect(L_BRITTANY, ENGLAND), Defect(L_MONTFORT, ENGLAND)]),
    ev!(1373, FRANCE, Minister, "ブルターニュの諸侯", "イングランドと結んだ公に背いたブルターニュの諸侯は、デュ・ゲクランを迎え入れ、公はイングランドへ逃れた。", [Defect(L_BRITTANY, FRANCE)]),
    ev!(1375, FRANCE, Minister, "ブルッヘの休戦", "教皇の使節の仲立ちで、両国はブルッヘで休戦を結んだ。", [Truce(ENGLAND, FRANCE, 2.0)]),
    ev!(1377, ENGLAND, Minister, "リチャード2世の即位", "黒太子に続いてエドワード3世も世を去り、十歳のリチャード2世が王となった。", [Figure("リチャード2世", -0.05, -0.1, 22.0)]),
    ev!(1378, ENGLAND, Minister, "シェルブールとブレストの質入れ", "ナバラ王はシェルブールを、ブルターニュ公はブレストをイングランドに貸し与えた。シャルル5世はナバラ王のエヴルーを没収する。", [Cede("シェルブール", ENGLAND), Cede("ブレスト", ENGLAND), Cede("エヴルー", FRANCE)]),
    ev!(1380, FRANCE, Disaster, "賢明王とデュ・ゲクランの死", "シャルル5世が世を去り、十一歳のシャルル6世が王となった。叔父たちが国を争う。", [Dismiss("シャルル5世"), Plunder(0.1)]),
    ev!(1381, ENGLAND, Revolt, "ワット・タイラーの乱", "人頭税に怒った農民がロンドンに攻め入り、大法官と大蔵卿を殺した。", [Purge(0.1), Plunder(0.2)]),
    ev!(1382, FRANCE, Hero, "ローゼベーケの戦い", "フランスの騎士たちはローゼベーケでフランドルの都市の軍を打ち破り、伯の支配を取り戻した。", [Defect(L_FLANDERS, FRANCE)]),
    ev!(1384, FRANCE, Minister, "ブルゴーニュ公、フランドルを継ぐ", "ブルゴーニュ公フィリップ（豪胆公）が妻の遺産としてフランドルとアルトワを継ぎ、大諸侯となった。", [Grant(200.0, 100.0, 50.0)]),
    ev!(1385, ENGLAND, Revolt, "ジャン・ド・ヴィエンヌのスコットランド遠征", "フランスの提督ジャン・ド・ヴィエンヌが兵を率いてスコットランドに渡り、スコットランド軍とともにイングランドの北を襲った。", [ScotsRaid]),
    ev!(1388, ENGLAND, Disaster, "オッターバーンの戦い", "ダグラス伯のスコットランド軍がノーサンバランドに攻め込み、オッターバーンでホットスパーを捕らえた。", [ScotsRaid]),
    ev!(1389, FRANCE, Minister, "ルランゲムの休戦", "疲れ果てた両国はカレーの近くのルランゲムで休戦を結んだ。", [Truce(ENGLAND, FRANCE, 6.0)]),
    ev!(1392, FRANCE, Disaster, "シャルル6世の狂気", "ル・マンの森でシャルル6世は狂気に取りつかれ、従者に斬りかかった。以後、王の心は晴れたり曇ったりを繰り返す。", [Figure("シャルル6世の狂気", -0.15, -0.1, 30.0)]),
    ev!(1394, FRANCE, Minister, "シェルブールの返還", "イングランドは借りていたシェルブールをナバラ王に返し、町はフランス王に従った。", [Cede("シェルブール", FRANCE)]),
    ev!(1396, ENGLAND, Minister, "長い休戦", "リチャード2世はシャルル6世の娘イザベルを妃に迎え、二十八年の休戦が結ばれた。", [Truce(ENGLAND, FRANCE, 12.0)]),
    ev!(1397, FRANCE, Minister, "ブレストの返還", "休戦にともない、イングランドはブレストをブルターニュ公に返した。", [Cede("ブレスト", FRANCE)]),
    ev!(1399, ENGLAND, Usurp, "ランカスター家の簒奪", "追放されていたヘンリー・ボリンブルックが兵を挙げてリチャード2世を廃し、ヘンリー4世として即位した。", [House("ランカスター家"), Dismiss("リチャード2世"), Purge(0.05)]),
    ev!(1400, ENGLAND, Revolt, "オワイン・グリンドゥールの反乱", "ウェールズのオワイン・グリンドゥールがプリンス・オブ・ウェールズを名乗り、イングランドに背いた。", [Purge(0.05)]),
    ev!(1402, ENGLAND, Revolt, "ホミルドン・ヒルの戦い", "スコットランド軍が再びイングランドの北に攻め込んだ。", [ScotsRaid]),
    ev!(1404, FRANCE, Hero, "グリンドゥールとの同盟", "グリンドゥールはフランスと同盟を結び、フランスの兵がウェールズに上陸した。ウェールズの城はフランス王の側に付く。", [Defect(L_WALES, FRANCE)]),
    ev!(1407, FRANCE, Revolt, "オルレアン公の暗殺", "ブルゴーニュ公ジャン（無畏公）がパリの路上でオルレアン公ルイを殺させた。アルマニャック派とブルゴーニュ派の内乱が始まる。", [Figure("アルマニャック派とブルゴーニュ派の争い", -0.1, -0.1, 28.0)]),
    ev!(1409, ENGLAND, Hero, "ハーレックの陥落", "王太子ヘンリーがハーレック城を落とし、グリンドゥールの反乱は山に追いやられた。", [Defect(L_WALES, ENGLAND)]),
    ev!(1413, ENGLAND, Hero, "ヘンリー5世の即位", "若き王ヘンリー5世は、曾祖父エドワード3世のフランス王位の要求をふたたび掲げた。", [Figure("ヘンリー5世", 0.1, 0.15, 9.0)]),
    ev!(1415, ENGLAND, Hero, "アルフルールとアジャンクール", "ヘンリー5世はノルマンディーに上陸してアルフルールを囲み、アジャンクールでフランスの貴族の花を散らした。", [War(ENGLAND, FRANCE), Levy(6), Train(0.3)]),
    ev!(1417, ENGLAND, Revolt, "「愚かな侵攻」", "王がフランスにいる隙を突き、スコットランド軍がロクスバラとベリックに攻め寄せた。", [ScotsRaid]),
    ev!(1418, FRANCE, Revolt, "ブルゴーニュ派のパリ掌握", "ブルゴーニュ派がパリに入り、アルマニャック派を殺した。王太子シャルルはパリを逃れた。", [Purge(0.1), Plunder(0.1)]),
    ev!(1419, FRANCE, Disaster, "モントローの暗殺", "和解の会見の場で、王太子の家臣がブルゴーニュ公ジャンを斬り殺した。息子フィリップ（善良公）はイングランドと結ぶ。", [Defect(L_BURGUNDY, ENGLAND), Defect(L_FLANDERS, ENGLAND)]),
    ev!(1419, FRANCE, Hero, "スコットランド軍の来援", "「古い同盟」に応え、バカン伯の率いるスコットランドの兵がラ・ロシェルに上陸した。", [ScotsArmy(6)]),
    ev!(1420, FRANCE, Disaster, "トロワ条約", "狂気の王シャルル6世はヘンリー5世を娘婿とし、王位の後継者と定めた。パリとシャンパーニュの都市はイングランド王に従う。", [Defect(L_PARIS, ENGLAND), Defect(L_CHAMPAGNE, ENGLAND)]),
    ev!(1421, FRANCE, Hero, "ボージェの戦い", "スコットランドとフランスの軍がボージェでクラレンス公を討ち取った。", [Train(0.3)]),
    ev!(1422, ENGLAND, Disaster, "ヘンリー5世の死", "ヘンリー5世はヴァンセンヌで赤痢に倒れた。後を継いだヘンリー6世は生後九か月だった。", [Dismiss("ヘンリー5世")]),
    ev!(1422, FRANCE, Minister, "シャルル7世", "シャルル6世も世を去った。ロワールの南に拠る王太子はシャルル7世を名乗ったが、人々は「ブールジュの王」と呼んだ。", [Dismiss("シャルル6世の狂気"), Figure("シャルル7世", 0.05, 0.0, 39.0)]),
    ev!(1423, ENGLAND, Hero, "アミアン条約", "ブルターニュ公ジャン5世は、ベッドフォード公とブルゴーニュ公と同盟を結んだ。", [Defect(L_BRITTANY, ENGLAND), Defect(L_MONTFORT, ENGLAND)]),
    ev!(1424, FRANCE, Hero, "スコットランドの第二陣", "ダグラス伯がさらに六千の兵を率いてフランスに渡った。", [ScotsArmy(6)]),
    ev!(1425, FRANCE, Hero, "リッシュモンの大元帥就任", "ブルターニュ公の弟アルテュール・ド・リッシュモンがフランスの大元帥となり、ブルターニュはフランス王の側に戻った。", [Defect(L_BRITTANY, FRANCE), Defect(L_MONTFORT, FRANCE)]),
    ev!(1428, ENGLAND, Hero, "オルレアンの包囲", "イングランド軍はロワールの要オルレアンを囲んだ。この町が落ちれば、王太子の南の国は開け放たれる。", [Levy(4)]),
    ev!(1429, FRANCE, Hero, "ランスの戴冠", "ジャンヌ・ダルクはオルレアンを解き、パテーで勝ち、王太子をランスへ導いて戴冠させた。道の街々は次々と城門を開いた。", [DefectIfHero(L_CHAMPAGNE, FRANCE, 13), Train(0.3)]),
    ev!(1431, ENGLAND, Minister, "ヘンリー6世のパリ戴冠", "十歳のヘンリー6世がパリのノートルダムでフランス王として戴冠した。だがフランスの人々の心は離れていた。", [Gold(150.0)]),
    ev!(1435, FRANCE, Hero, "アラスの和約", "ブルゴーニュ公フィリップはシャルル7世と和を結び、イングランドとの同盟を捨てた。", [Defect(L_BURGUNDY, FRANCE), Defect(L_FLANDERS, FRANCE)]),
    ev!(1435, ENGLAND, Disaster, "ベッドフォード公の死", "フランスを治めた摂政ベッドフォード公がルーアンで世を去った。", [Bias(0.92)]),
    ev!(1436, FRANCE, Hero, "パリの回復", "パリの市民は城門を開いてリッシュモンの軍を迎え入れた。", [Defect(L_PARIS, FRANCE)]),
    ev!(1439, FRANCE, Minister, "恒常のタイユ", "オルレアンの三部会は、王が常備の兵を養うための税を毎年取ることを認めた。", [Gold(300.0), Bias(1.05)]),
    ev!(1440, FRANCE, Revolt, "プラグリーの乱", "王の改革を嫌う大貴族たちが王太子ルイを担いで背いた。", [Purge(0.05)]),
    ev!(1440, FRANCE, Minister, "ジャック・クール", "ブールジュの商人ジャック・クールが王の財務を預かり、地中海の交易で国庫を満たした。", [Figure("ジャック・クール", 0.2, 0.0, 11.0)]),
    ev!(1444, FRANCE, Minister, "トゥールの休戦", "ヘンリー6世はアンジューのマルグリットを妃に迎え、両国は休戦した。", [Truce(ENGLAND, FRANCE, 5.0)]),
    ev!(1445, FRANCE, Hero, "勅令隊の創設", "シャルル7世は傭兵団を解き、王が給金を払う常備の騎兵隊を置いた。", [Train(0.5)]),
    ev!(1449, ENGLAND, Disaster, "フージェールの略奪", "イングランドの傭兵隊長が休戦を破ってブルターニュのフージェールを襲い、ブルターニュはフランスと結び、戦が再び始まった。", [War(FRANCE, ENGLAND), Defect(L_BRITTANY, FRANCE), Defect(L_MONTFORT, FRANCE)]),
    ev!(1450, ENGLAND, Revolt, "ジャック・ケイドの乱", "ノルマンディーの喪失に怒ったケントの民がロンドンに攻め入った。", [Purge(0.08), Plunder(0.15)]),
    ev!(1452, ENGLAND, Hero, "ボルドーの蜂起", "三百年イングランド王に仕えてきたボルドーの市民は、フランスの重い税を嫌ってタルボットを呼び戻した。", [Defect(L_GUYENNE, ENGLAND)]),
    ev!(1453, FRANCE, Hero, "カスティヨンの戦い", "ビュロー兄弟の大砲が、突撃するタルボットの軍をなぎ倒した。この年のうちに決着をつけねばならない。", [Train(0.3)]),
    ev!(1453, ENGLAND, Disaster, "ヘンリー6世の狂気", "ヘンリー6世は祖父シャルル6世と同じく正気を失い、王国は薔薇戦争へと傾いてゆく。", [Figure("ヘンリー6世の狂気", -0.1, -0.1, 5.0)]),
];

/// 在世の人物（効果は年限つき）
pub struct Person {
    pub name: &'static str,
    pub eco: f32,
    pub mil: f32,
    pub until: f32,
}

/// 史書の記録
pub struct Chron {
    pub ad: i32,
    pub nation: usize,
    pub kind: HKind,
    pub title: &'static str,
    pub text: &'static str,
    /// 対象国が滅んでいて起こらなかった
    pub skipped: bool,
}

impl World {
    /// 年が改まるごとに史実イベントを発生させる
    pub(super) fn run_history(&mut self) {
        let ad = self.year_ad();
        // 人物の退場
        let years = self.years;
        for n in self.nations.iter_mut() {
            n.figures.retain(|f| f.until > years);
        }
        while self.hist_i < HISTORY.len() && HISTORY[self.hist_i].ad <= ad {
            let e = &HISTORY[self.hist_i];
            self.hist_i += 1;
            self.fire_event(e);
        }
        self.refresh_mods();
    }

    fn fire_event(&mut self, e: &'static HEvent) {
        let n = e.nation;
        let skipped = !self.nations[n].alive;
        self.chronicle.push(Chron { ad: e.ad, nation: n, kind: e.kind, title: e.title, text: e.text, skipped });
        if skipped {
            return;
        }
        // チュートリアル中はプレイヤーに不利な出来事を起こさない
        let shield = self.protect_player && n == self.player;
        let mut notes: Vec<String> = Vec::new();
        for fx in e.fx.iter() {
            match *fx {
                Figure(name, eco, mil, yrs) => {
                    if shield && (eco < 0.0 || mil < 0.0) {
                        continue;
                    }
                    let until = self.years + yrs;
                    let nat = &mut self.nations[n];
                    nat.figures.retain(|f| f.name != name);
                    nat.figures.push(Person { name, eco, mil, until });
                    let mut s = String::from(tx(name));
                    if eco != 0.0 {
                        s.push_str(&tf!(" 生産{:+.0}%", " production {:+.0}%", " 生产{:+.0}%", eco * 100.0));
                    }
                    if mil != 0.0 {
                        s.push_str(&tf!(" 武{:+.0}%", " military {:+.0}%", " 武{:+.0}%", mil * 100.0));
                    }
                    s.push_str(&tf!("（{}年）", " ({} years)", "（{}年）", yrs.min(END_AD - START_AD) as i32));
                    notes.push(s);
                }
                Dismiss(name) => {
                    let nat = &mut self.nations[n];
                    if nat.figures.iter().any(|f| f.name == name) {
                        nat.figures.retain(|f| f.name != name);
                        notes.push(tf!("{} が去る", "{} departs", "{}离去", tx(name)));
                    }
                }
                Grant(a, b, c) => {
                    let r = &mut self.nations[n].res;
                    r[0] += a;
                    r[1] += b;
                    r[2] += c;
                    notes.push(tf!("糧{:.0} 木{:.0} 鉄{:.0}", "food {:.0}, wood {:.0}, iron {:.0}", "粮{:.0} 木{:.0} 铁{:.0}", a, b, c));
                }
                Train(x) => {
                    let cap = self.power_cap();
                    let mut k = 0;
                    for u in self.units.iter_mut() {
                        if u.alive && u.nation as usize == n && u.kind.military() && u.power < cap {
                            let ratio = u.hp / u.max_hp();
                            u.power = (u.power + x).min(cap);
                            u.hp = ratio * u.max_hp();
                            k += 1;
                        }
                    }
                    if k > 0 {
                        notes.push(tf!("兵{}人の能力{:+.1}", "{} soldiers' power {:+.1}", "{}名士兵能力{:+.1}", k, x));
                    }
                }
                Levy(cnt) => {
                    let Some(c) = self.nations[n].capital else { continue };
                    let pw = (1.0 + self.years / (END_AD - START_AD) * 2.0).min(self.power_cap());
                    let ut = best_land_ut(n, self.year_ad());
                    let mut k = 0;
                    for _ in 0..cnt {
                        if let Some(sp) = self.spawn_point(c) {
                            self.spawn_unit(n, ut, sp, pw);
                            k += 1;
                        }
                    }
                    if k > 0 {
                        notes.push(tf!("{}{}人が馳せ参じる", "{1} {0} rally to the capital", "{}{}名赶来效命", tx(udef(ut).name), k));
                    }
                }
                Bias(f) => {
                    if shield && f < 1.0 {
                        continue;
                    }
                    let nat = &mut self.nations[n];
                    let old = nat.bias;
                    nat.bias = (nat.bias * f).clamp(0.4, 2.4);
                    notes.push(tf!("国力 ×{:.2} → ×{:.2}", "national strength ×{:.2} → ×{:.2}", "国力 ×{:.2} → ×{:.2}", old, nat.bias));
                }
                Gold(g) => {
                    self.nations[n].coin += g;
                    notes.push(tf!("金貨{:.0}", "gold {:.0}", "金币{:.0}", g));
                }
                Plague(f) => {
                    let mut k = 0;
                    for i in 0..self.units.len() {
                        let u = &self.units[i];
                        if u.alive && (u.nation as usize) < NATIONS.len() && u.hero == NONE && !u.kind.naval() && self.rng.chance(f) {
                            self.kill_unit(i);
                            k += 1;
                        }
                    }
                    if k > 0 {
                        notes.push(tf!("両国で民と兵{}人が死ぬ", "{} workers and soldiers die in both kingdoms", "两国共有{}名民与士兵死去", k));
                    }
                }
                Defect(l, to) => {
                    if let Some(s) = self.defect(l, to) {
                        notes.push(s);
                    }
                }
                DefectIfHero(l, to, h) => {
                    if self.hero_on_field(h) {
                        if let Some(s) = self.defect(l, to) {
                            notes.push(s);
                        }
                    } else {
                        notes.push(tf!("{}がいないため、街は動かなかった", "without {}, the towns did not move", "因{}不在，城镇未有动静", tx(HEROES[h].name)));
                    }
                }
                Cede(name, to) => {
                    if let Some(s) = self.cede(town_id(name), to) {
                        notes.push(s);
                    }
                }
                ScotsRaid => {
                    if self.tribe_raid(0, true) {
                        notes.push(t!("スコットランド軍が国境を越える", "the Scots cross the border", "苏格兰军越过边境").to_string());
                    }
                }
                ScotsArmy(cnt) => {
                    let Some(c) = self.nations[n].capital else { continue };
                    let bi = self.town_bld(town_id("トゥール")).filter(|&b| self.blds[b].nation as usize == n).map(|b| self.bh(b)).unwrap_or(c);
                    let pw = (1.0 + self.years / (END_AD - START_AD) * 2.0).min(self.power_cap());
                    let mut k = 0;
                    for j in 0..cnt {
                        if let Some(sp) = self.spawn_point(bi) {
                            let ut = if j % 3 == 2 { UT_ARC } else { UT_SCOTS };
                            let h = self.spawn_unit(n, ut, sp, pw);
                            let until = self.years + MERC_YEARS;
                            let u = self.unit_mut(h).unwrap();
                            u.tribe = 0;
                            u.merc_until = until;
                            k += 1;
                        }
                    }
                    if k > 0 {
                        notes.push(tf!("スコットランド兵{}人が加わる", "{} Scottish troops join", "{}名苏格兰士兵加入", k));
                    }
                }
                Purge(f) => {
                    if shield {
                        continue;
                    }
                    let mut idx: Vec<usize> = (0..self.units.len()).filter(|&i| self.units[i].alive && self.units[i].nation as usize == n && self.units[i].kind.military()).collect();
                    // 都から遠い兵ほど離反しやすい
                    let home = self.nations[n].home;
                    idx.sort_by(|&a, &b| self.units[b].pos.dist(home).partial_cmp(&self.units[a].pos.dist(home)).unwrap());
                    let k = (idx.len() as f32 * f).round() as usize;
                    for &i in idx.iter().take(k) {
                        self.kill_unit(i);
                    }
                    if k > 0 {
                        notes.push(tf!("兵{}人を失う", "{} soldiers lost", "损失{}名士兵", k));
                    }
                }
                Plunder(f) => {
                    if shield {
                        continue;
                    }
                    for r in self.nations[n].res.iter_mut() {
                        *r *= 1.0 - f;
                    }
                    notes.push(tf!("蓄えの{:.0}%を失う", "{:.0}% of stores lost", "损失{:.0}%的储备", f * 100.0));
                }
                House(h) => {
                    if self.nations[n].house != h {
                        notes.push(tf!("君主家：{} → {}", "ruling house: {} → {}", "君主家：{} → {}", tx(self.nations[n].house), tx(h)));
                        self.nations[n].house = h;
                    }
                }
                Truce(a, b, yrs) => {
                    if !self.nations[a].alive || !self.nations[b].alive || a == b {
                        continue;
                    }
                    if self.coalition_locked(a, b) {
                        continue;
                    }
                    // 交戦中のプレイヤーには休戦を受けるか選ばせる
                    if self.war[a][b] && !self.propose_peace(a, b, yrs, tx(e.title)) {
                        notes.push(tf!("{}と{}に{}年の休戦の勧め", "{} and {} are urged to a {}-year truce", "劝{}与{}休战{}年", tx(self.nations[a].name), tx(self.nations[b].name), yrs as i32));
                        continue;
                    }
                    let t = self.years + yrs;
                    self.truce[a][b] = self.truce[a][b].max(t);
                    self.truce[b][a] = self.truce[b][a].max(t);
                    notes.push(tf!("{}と{}が{}年の休戦", "{} and {} agree a {}-year truce", "{}与{}休战{}年", tx(self.nations[a].name), tx(self.nations[b].name), yrs as i32));
                }
                War(a, b) => {
                    // プレイヤーの国に勝手に宣戦させない
                    if a == self.player || (self.protect_player && b == self.player) {
                        continue;
                    }
                    if self.declare_war(a, b, true) {
                        notes.push(tf!("{}が{}に攻め込む", "{} invades {}", "{}进攻{}", tx(self.nations[a].name), tx(self.nations[b].name)));
                    }
                }
            }
        }
        self.refresh_mods();
        self.territory_dirty = true;
        let name = tx(self.nations[n].name);
        let head = tf!("【{}・{}】{}", "[{} · {}] {}", "【{}・{}】{}", name, e.kind.label(), tx(e.title));
        // 本文はバナーと史記に任せ、記録欄は見出しと効果だけにする
        let mut msg = head;
        if !notes.is_empty() {
            msg.push_str(&tf!("（{}）", " ({})", "（{}）", notes.join(t!("、", "; ", "；"))));
        }
        self.push_msg(msg, e.kind.color());
        // 大きな出来事はバナーで知らせる（小国の出来事は記録のみ）
        // 二つの王国の戦なので、出来事はすべて告知する
        {
            let mut text = tx(e.text).to_string();
            if !notes.is_empty() {
                text.push_str(&format!("\n{}", notes.join(t!("　", "; ", "；"))));
            }
            let title = format!("{}　{}", name, tx(e.title));
            self.queue_banner(Banner { title, text, t: 7.0 });
        }
    }

    /// 在世の人物の効果を集計する
    pub(super) fn refresh_mods(&mut self) {
        let years = self.years;
        for n in self.nations.iter_mut() {
            let (mut e, mut m) = (1.0f32, 1.0f32);
            for f in n.figures.iter().filter(|f| f.until > years) {
                e += f.eco;
                m += f.mil;
            }
            n.eco_mul = e.clamp(0.4, 3.0);
            n.mil_mul = m.clamp(0.5, 2.5);
        }
    }

    /// 時を進めて、その年までの史実の出来事を起こす（ユニットは動かさない。テスト・撮影用）
    pub fn hist_skip_to(&mut self, years: f32) {
        while self.years < years {
            self.years = (self.years + 1.0).min(years);
            if self.era == Era::Rise && START_AD + self.years >= EMPIRE_AD {
                self.begin_empire();
            }
            self.run_history();
            self.refresh_traits();
            if self.territory_dirty {
                self.recompute_territory();
            }
            self.update_stats();
        }
    }

    /// いまの告知を閉じて、次の告知へ
    pub fn dismiss_banner(&mut self) {
        self.banner = self.banner_queue.pop_front();
    }

    /// 待っている告知の数（いま出ているものを除く）
    pub fn banners_waiting(&self) -> usize {
        self.banner_queue.len()
    }

    /// 告知をすべて閉じる（記録欄と史記には残る）
    pub fn dismiss_all_banners(&mut self) {
        self.banner = None;
        self.banner_queue.clear();
    }

    pub fn queue_banner(&mut self, b: Banner) {
        if self.banner.is_none() {
            self.banner = Some(b);
        } else {
            self.banner_queue.push_back(b);
        }
    }
}
