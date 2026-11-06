"""百年戦争の史実年表（src/world/history.rs の HISTORY）と、データの訳の辞書（src/i18n.rs）を生成する。

usage: python3 tools/gendata.py   （リポジトリの根から）

イベントは (西暦, 国, 種類, 見出し(ja,en,zh), 本文(ja,en,zh), 効果[Rust]) で書く。
人物・君主家の名前の訳は FIG / HOUSE に書く。
"""
import re
import sys

E = []


def ev(ad, n, kind, title, text, fx):
    E.append((ad, n, kind, title, text, fx))


# ---------------------------------------------------------------- 人物（Figure・Dismiss に出る名前）
FIG = {
    "シャルル5世": ("Charles V", "查理五世"),
    "リチャード2世": ("Richard II", "理查二世"),
    "シャルル6世の狂気": ("the madness of Charles VI", "查理六世的疯病"),
    "アルマニャック派とブルゴーニュ派の争い": ("the Armagnac–Burgundian feud", "阿马尼亚克派与勃艮第派之争"),
    "ヘンリー5世": ("Henry V", "亨利五世"),
    "シャルル7世": ("Charles VII", "查理七世"),
    "ジャック・クール": ("Jacques Cœur", "雅克·科尔"),
    "ヘンリー6世の狂気": ("the madness of Henry VI", "亨利六世的疯病"),
}

# ---------------------------------------------------------------- 君主家
HOUSE = {
    "プランタジネット家": ("the Plantagenets", "金雀花王朝"),
    "ヴァロワ家": ("the Valois", "瓦卢瓦王朝"),
    "ランカスター家": ("the Lancastrians", "兰开斯特王朝"),
}

# ---------------------------------------------------------------- 史実年表
T = (lambda ja, en, zh: (ja, en, zh))

ev(1337, "FRANCE", "Minister", T("ギュイエンヌの没収", "The confiscation of Guyenne", "没收吉耶讷"),
   T("フィリップ6世は臣下エドワード3世のギュイエンヌ公領を没収した。百年にわたる戦の始まりである。",
     "Philip VI confiscates the duchy of Guyenne from his vassal Edward III. So begins a war that will last a hundred years.",
     "腓力六世没收了其封臣爱德华三世的吉耶讷公国。长达百年的战争由此开始。"),
   'Levy(3)')
ev(1337, "ENGLAND", "Hero", T("フランス王位の要求", "The claim to the French crown", "对法兰西王位的主张"),
   T("母イザベルを通じてカペー家の血を引くエドワード3世は、自らこそフランス王であると宣言した。",
     "Through his mother Isabella, Edward III carries the blood of the Capetians, and he declares himself the rightful King of France.",
     "爱德华三世经由母亲伊莎贝拉继承卡佩家族的血统，宣称自己才是法兰西的合法国王。"),
   'Gold(200.0), Levy(3)')
ev(1338, "ENGLAND", "Minister", T("羊毛の禁輸", "The wool embargo", "羊毛禁运"),
   T("エドワード3世は羊毛の輸出を止め、イングランドの羊毛なしには織機の止まるフランドルの都市を揺さぶった。",
     "Edward III halts the export of wool, shaking the Flemish towns whose looms cannot run without English wool.",
     "爱德华三世停止羊毛出口，动摇了没有英格兰羊毛就无法开工的佛兰德斯城市。"),
   'Gold(150.0)')
ev(1340, "ENGLAND", "Hero", T("フランドルの同盟", "The Flemish alliance", "佛兰德斯同盟"),
   T("ヘントの指導者ヤーコプ・ファン・アルテフェルデはエドワード3世をフランス王と認め、フランドルの都市はイングランドに付いた。",
     "Jacob van Artevelde of Ghent recognises Edward III as King of France, and the Flemish towns go over to England.",
     "根特的领袖雅各布·范·阿特维尔德承认爱德华三世为法兰西国王，佛兰德斯诸城倒向英格兰。"),
   'Defect(L_FLANDERS, ENGLAND)')
ev(1340, "FRANCE", "Disaster", T("スロイスの海戦", "The Battle of Sluys", "斯勒伊斯海战"),
   T("ズウィン湾に集めたフランスの艦隊は、イングランドの弓兵に射すくめられて全滅した。",
     "The French fleet gathered in the Zwin is shot to pieces by English archers and destroyed.",
     "集结于兹温湾的法兰西舰队被英格兰弓兵射得溃不成军，全军覆没。"),
   'Plunder(0.15)')
ev(1341, "ENGLAND", "Hero", T("ブルターニュ継承戦争", "The War of the Breton Succession", "布列塔尼继承战争"),
   T("ブルターニュ公の後継をめぐり、モンフォール家はイングランドと、ブロワ家はフランスと結んだ。ブレストとヴァンヌがイングランドに付く。",
     "In the struggle for the Breton duchy, the house of Montfort allies with England and the house of Blois with France. Brest and Vannes go over to England.",
     "围绕布列塔尼公位继承，蒙福尔家与英格兰结盟，布卢瓦家与法兰西结盟。布雷斯特与瓦讷倒向英格兰。"),
   'Defect(L_MONTFORT, ENGLAND)')
ev(1346, "ENGLAND", "Hero", T("クレシーの戦い", "The Battle of Crécy", "克雷西会战"),
   T("ノルマンディーを駆けたエドワード3世の軍は、クレシーでフランス騎士の突撃を長弓で打ち砕いた。",
     "Edward III's army, after its ride through Normandy, breaks the charges of the French knights with the longbow at Crécy.",
     "纵横诺曼底的爱德华三世之军，在克雷西以长弓击溃了法兰西骑士的冲锋。"),
   'Train(0.5)')
ev(1346, "ENGLAND", "Revolt", T("デイヴィッド2世の侵攻", "David II invades", "大卫二世入侵"),
   T("フランス王の求めに応じ、スコットランド王デイヴィッド2世が国境を越えて攻め込んだ。",
     "At the request of the King of France, David II of Scotland crosses the border.",
     "应法兰西国王之请，苏格兰国王大卫二世越过边境来犯。"),
   'ScotsRaid')
ev(1348, "ENGLAND", "Disaster", T("黒死病", "The Black Death", "黑死病"),
   T("東から来た疫病がフランスとイングランドを覆い、人々の三人に一人が死んだ。王たちも戦を休まざるを得なかった。",
     "A pestilence from the East sweeps over France and England, killing one in three. Even the kings must pause their war.",
     "来自东方的瘟疫席卷法兰西与英格兰，三人之中便有一人死去。连国王们也不得不暂停战争。"),
   'Plague(0.25), Truce(ENGLAND, FRANCE, 3.0)')
ev(1349, "FRANCE", "Minister", T("フランドル伯ルイ・ド・マール", "Louis de Male, Count of Flanders", "佛兰德斯伯爵路易·德·马勒"),
   T("アルテフェルデの死の後、フランドル伯ルイ・ド・マールは都市を従え、フランス王のもとに戻った。",
     "After Artevelde's death, Count Louis de Male brings the towns to heel and returns to the King of France.",
     "阿特维尔德死后，佛兰德斯伯爵路易·德·马勒收服诸城，重归法兰西国王麾下。"),
   'Defect(L_FLANDERS, FRANCE)')
ev(1354, "ENGLAND", "Hero", T("ナバラ王シャルル（悪王）", "Charles the Bad of Navarre", "纳瓦拉的恶人查理"),
   T("ノルマンディーに領地を持つナバラ王シャルルはフランス王の大元帥を暗殺し、イングランドと結んだ。",
     "Charles of Navarre, who holds lands in Normandy, has the Constable of France murdered and allies with England.",
     "在诺曼底拥有领地的纳瓦拉国王查理刺杀了法兰西大元帅，与英格兰结盟。"),
   'Defect(L_NAVARRE, ENGLAND)')
ev(1355, "ENGLAND", "Hero", T("黒太子の大騎行", "The Black Prince's great chevauchée", "黑太子的大骑行"),
   T("黒太子はボルドーから地中海の近くまで駆け、五百の村を焼いて莫大な戦利品を持ち帰った。",
     "The Black Prince rides from Bordeaux almost to the Mediterranean, burning five hundred villages and carrying home vast plunder.",
     "黑太子从波尔多一路驰骋至地中海附近，焚毁五百座村庄，满载战利品而归。"),
   'Gold(250.0), Train(0.3)')
ev(1356, "ENGLAND", "Hero", T("ポワティエの戦い", "The Battle of Poitiers", "普瓦捷会战"),
   T("黒太子はポワティエで数に勝るフランス軍を破った。王を守る騎士たちは馬を降りて戦い、倒れていった。",
     "The Black Prince defeats a larger French army at Poitiers; the king's knights fight on foot around him and fall.",
     "黑太子在普瓦捷击败了兵力占优的法兰西军。守护国王的骑士们下马作战，相继倒下。"),
   'Train(0.3)')
ev(1357, "FRANCE", "Revolt", T("エティエンヌ・マルセルの反乱", "Étienne Marcel's revolt", "艾蒂安·马塞尔起义"),
   T("王を失った混乱の中、パリの商人頭エティエンヌ・マルセルが三部会を率いて王太子に迫った。",
     "In the chaos of a captive king, Étienne Marcel, provost of the merchants of Paris, leads the Estates against the Dauphin.",
     "在国王被俘的混乱之中，巴黎商会会长艾蒂安·马塞尔率领三级会议逼迫王太子。"),
   'Plunder(0.15), Purge(0.1)')
ev(1358, "FRANCE", "Revolt", T("ジャックリーの乱", "The Jacquerie", "扎克雷起义"),
   T("戦と略奪に苦しむボーヴェジの農民が領主に背き、城館を焼いた。",
     "The peasants of the Beauvaisis, crushed by war and plunder, rise against their lords and burn their manors.",
     "饱受战争与劫掠之苦的博韦地区农民反抗领主，焚烧庄园。"),
   'Plunder(0.15), Purge(0.1)')
ev(1360, "FRANCE", "Minister", T("ブレティニー条約", "The Treaty of Brétigny", "布雷蒂尼条约"),
   T("ランスの包囲に失敗したエドワード3世は和を結んだ。フランスは王の身代金三百万エキュを約した。",
     "After failing before Reims, Edward III makes peace. France promises three million écus for the king's ransom.",
     "围攻兰斯失败后，爱德华三世议和。法兰西承诺支付三百万埃居作为国王的赎金。"),
   'Truce(ENGLAND, FRANCE, 9.0), Plunder(0.1)')
ev(1361, "ENGLAND", "Disaster", T("疫病の再来", "The return of the plague", "瘟疫卷土重来"),
   T("黒死病が再び両国を襲った。ランカスター公ヘンリーもこの年に世を去った。",
     "The Black Death strikes both kingdoms again; Henry, Duke of Lancaster, dies this year.",
     "黑死病再度侵袭两国。兰开斯特公爵亨利也于这一年去世。"),
   'Plague(0.12)')
ev(1364, "FRANCE", "Hero", T("賢明王シャルル5世", "Charles V the Wise", "贤明王查理五世"),
   T("シャルル5世が即位した。会戦を避けて税を整え、城を一つずつ取り戻す王である。",
     "Charles V takes the throne — a king who avoids pitched battle, sets the taxes in order and wins back the castles one by one.",
     "查理五世即位。这位国王避免会战，整顿税制，逐一收复城堡。"),
   'Figure("シャルル5世", 0.2, 0.15, 16.0)')
ev(1364, "FRANCE", "Hero", T("コシュレルの戦い", "The Battle of Cocherel", "科歇雷尔之战"),
   T("デュ・ゲクランがコシュレルでナバラ王の軍を破り、ナバラ王はフランス王と和を結んだ。",
     "Du Guesclin defeats the Navarrese army at Cocherel, and the King of Navarre makes his peace with the King of France.",
     "迪盖克兰在科歇雷尔击败纳瓦拉军，纳瓦拉国王与法兰西国王议和。"),
   'Defect(L_NAVARRE, FRANCE)')
ev(1369, "FRANCE", "Hero", T("ガスコーニュ諸侯の上訴", "The appeal of the Gascon lords", "加斯科涅诸侯的上诉"),
   T("黒太子の重い炉税に怒ったアルブレ家とアルマニャック家はパリの王に訴え、シャルル5世は再び戦を始めた。",
     "Angered by the Black Prince's hearth tax, the lords of Albret and Armagnac appeal to the king in Paris, and Charles V reopens the war.",
     "不满黑太子沉重炉税的阿尔布雷家与阿马尼亚克家向巴黎的国王申诉，查理五世重开战端。"),
   'Defect(L_GASCON, FRANCE), War(FRANCE, ENGLAND)')
ev(1372, "FRANCE", "Hero", T("ラ・ロシェルの海戦", "The Battle of La Rochelle", "拉罗谢尔海战"),
   T("フランスと結んだカスティーリャのガレー船がイングランドの艦隊を焼き、ラ・ロシェルの市民はフランス王に城門を開いた。",
     "Castilian galleys allied to France burn the English fleet, and the citizens of La Rochelle open their gates to the King of France.",
     "与法兰西结盟的卡斯蒂利亚桨帆船焚毁了英格兰舰队，拉罗谢尔的市民向法兰西国王打开了城门。"),
   'Cede("ラ・ロシェル", FRANCE)')
ev(1372, "ENGLAND", "Hero", T("ブルターニュ公ジャン4世の同盟", "The alliance of Duke John IV", "布列塔尼公爵约翰四世的同盟"),
   T("オーレーで勝ってブルターニュ公となったモンフォール家のジャン4世は、イングランドと同盟を結んだ。",
     "John IV of Montfort, duke since his victory at Auray, signs an alliance with England.",
     "在欧赖获胜而成为布列塔尼公爵的蒙福尔家约翰四世与英格兰结盟。"),
   'Defect(L_BRITTANY, ENGLAND), Defect(L_MONTFORT, ENGLAND)')
ev(1373, "FRANCE", "Minister", T("ブルターニュの諸侯", "The Breton barons", "布列塔尼的诸侯"),
   T("イングランドと結んだ公に背いたブルターニュの諸侯は、デュ・ゲクランを迎え入れ、公はイングランドへ逃れた。",
     "The Breton barons turn against a duke allied with England and welcome Du Guesclin; the duke flees to England.",
     "布列塔尼诸侯背弃与英格兰结盟的公爵，迎入迪盖克兰，公爵逃往英格兰。"),
   'Defect(L_BRITTANY, FRANCE)')
ev(1375, "FRANCE", "Minister", T("ブルッヘの休戦", "The Truce of Bruges", "布鲁日休战"),
   T("教皇の使節の仲立ちで、両国はブルッヘで休戦を結んだ。",
     "Through papal mediation the two kingdoms agree a truce at Bruges.",
     "经教皇使节斡旋，两国在布鲁日缔结休战。"),
   'Truce(ENGLAND, FRANCE, 2.0)')
ev(1377, "ENGLAND", "Minister", T("リチャード2世の即位", "The accession of Richard II", "理查二世即位"),
   T("黒太子に続いてエドワード3世も世を去り、十歳のリチャード2世が王となった。",
     "After the Black Prince, Edward III too is dead, and the ten-year-old Richard II becomes king.",
     "继黑太子之后，爱德华三世也去世，十岁的理查二世成为国王。"),
   'Figure("リチャード2世", -0.05, -0.1, 22.0)')
ev(1378, "ENGLAND", "Minister", T("シェルブールとブレストの質入れ", "Cherbourg and Brest are leased", "瑟堡与布雷斯特的出租"),
   T("ナバラ王はシェルブールを、ブルターニュ公はブレストをイングランドに貸し与えた。シャルル5世はナバラ王のエヴルーを没収する。",
     "The King of Navarre leases Cherbourg, and the Duke of Brittany Brest, to England. Charles V seizes Navarrese Évreux.",
     "纳瓦拉国王将瑟堡、布列塔尼公爵将布雷斯特租借给英格兰。查理五世没收了纳瓦拉国王的埃夫勒。"),
   'Cede("シェルブール", ENGLAND), Cede("ブレスト", ENGLAND), Cede("エヴルー", FRANCE)')
ev(1380, "FRANCE", "Disaster", T("賢明王とデュ・ゲクランの死", "The deaths of Charles V and Du Guesclin", "查理五世与迪盖克兰之死"),
   T("シャルル5世が世を去り、十一歳のシャルル6世が王となった。叔父たちが国を争う。",
     "Charles V dies and the eleven-year-old Charles VI becomes king; his uncles quarrel over the realm.",
     "查理五世去世，十一岁的查理六世成为国王。叔父们争夺国政。"),
   'Dismiss("シャルル5世"), Plunder(0.1)')
ev(1381, "ENGLAND", "Revolt", T("ワット・タイラーの乱", "Wat Tyler's rebellion", "瓦特·泰勒起义"),
   T("人頭税に怒った農民がロンドンに攻め入り、大法官と大蔵卿を殺した。",
     "Peasants enraged by the poll tax storm London and kill the Chancellor and the Treasurer.",
     "因人头税而愤怒的农民攻入伦敦，杀死了大法官与财政大臣。"),
   'Purge(0.1), Plunder(0.2)')
ev(1382, "FRANCE", "Hero", T("ローゼベーケの戦い", "The Battle of Roosebeke", "罗泽贝克之战"),
   T("フランスの騎士たちはローゼベーケでフランドルの都市の軍を打ち破り、伯の支配を取り戻した。",
     "The French knights crush the army of the Flemish towns at Roosebeke and restore the count's rule.",
     "法兰西骑士在罗泽贝克击溃佛兰德斯城市军，恢复了伯爵的统治。"),
   'Defect(L_FLANDERS, FRANCE)')
ev(1384, "FRANCE", "Minister", T("ブルゴーニュ公、フランドルを継ぐ", "Burgundy inherits Flanders", "勃艮第公继承佛兰德斯"),
   T("ブルゴーニュ公フィリップ（豪胆公）が妻の遺産としてフランドルとアルトワを継ぎ、大諸侯となった。",
     "Philip the Bold, Duke of Burgundy, inherits Flanders and Artois through his wife and becomes the greatest of the princes.",
     "勃艮第公爵勇敢的腓力通过妻子继承了佛兰德斯与阿图瓦，成为最强大的诸侯。"),
   'Grant(200.0, 100.0, 50.0)')
ev(1385, "ENGLAND", "Revolt", T("ジャン・ド・ヴィエンヌのスコットランド遠征", "Jean de Vienne in Scotland", "让·德·维埃纳远征苏格兰"),
   T("フランスの提督ジャン・ド・ヴィエンヌが兵を率いてスコットランドに渡り、スコットランド軍とともにイングランドの北を襲った。",
     "The French admiral Jean de Vienne lands in Scotland with an army and raids the north of England alongside the Scots.",
     "法兰西海军上将让·德·维埃纳率军渡海至苏格兰，与苏格兰军一同袭击英格兰北部。"),
   'ScotsRaid')
ev(1388, "ENGLAND", "Disaster", T("オッターバーンの戦い", "The Battle of Otterburn", "奥特本之战"),
   T("ダグラス伯のスコットランド軍がノーサンバランドに攻め込み、オッターバーンでホットスパーを捕らえた。",
     "The Earl of Douglas leads the Scots into Northumberland and captures Hotspur at Otterburn.",
     "道格拉斯伯爵率苏格兰军攻入诺森伯兰，在奥特本俘获了霍茨波。"),
   'ScotsRaid')
ev(1389, "FRANCE", "Minister", T("ルランゲムの休戦", "The Truce of Leulinghem", "勒兰冈休战"),
   T("疲れ果てた両国はカレーの近くのルランゲムで休戦を結んだ。",
     "Exhausted, the two kingdoms agree a truce at Leulinghem near Calais.",
     "精疲力竭的两国在加来附近的勒兰冈缔结休战。"),
   'Truce(ENGLAND, FRANCE, 6.0)')
ev(1392, "FRANCE", "Disaster", T("シャルル6世の狂気", "The madness of Charles VI", "查理六世的疯病"),
   T("ル・マンの森でシャルル6世は狂気に取りつかれ、従者に斬りかかった。以後、王の心は晴れたり曇ったりを繰り返す。",
     "In the forest of Le Mans, Charles VI is seized by madness and attacks his own men. From now on the king's mind comes and goes.",
     "在勒芒的森林中，查理六世突发疯病，挥剑砍向随从。此后国王的神志时清时乱。"),
   'Figure("シャルル6世の狂気", -0.15, -0.1, 30.0)')
ev(1394, "FRANCE", "Minister", T("シェルブールの返還", "Cherbourg returned", "瑟堡归还"),
   T("イングランドは借りていたシェルブールをナバラ王に返し、町はフランス王に従った。",
     "England hands Cherbourg back to Navarre, and the town comes under the King of France.",
     "英格兰将租借的瑟堡归还纳瓦拉国王，该城归附法兰西国王。"),
   'Cede("シェルブール", FRANCE)')
ev(1396, "ENGLAND", "Minister", T("長い休戦", "The long truce", "长期休战"),
   T("リチャード2世はシャルル6世の娘イザベルを妃に迎え、二十八年の休戦が結ばれた。",
     "Richard II marries Isabella, daughter of Charles VI, and a truce of twenty-eight years is sealed.",
     "理查二世迎娶查理六世之女伊莎贝拉为王后，双方缔结二十八年的休战。"),
   'Truce(ENGLAND, FRANCE, 12.0)')
ev(1397, "FRANCE", "Minister", T("ブレストの返還", "Brest returned", "布雷斯特归还"),
   T("休戦にともない、イングランドはブレストをブルターニュ公に返した。",
     "Under the truce, England returns Brest to the Duke of Brittany.",
     "随着休战，英格兰将布雷斯特归还布列塔尼公爵。"),
   'Cede("ブレスト", FRANCE)')
ev(1399, "ENGLAND", "Usurp", T("ランカスター家の簒奪", "The Lancastrian usurpation", "兰开斯特家的篡位"),
   T("追放されていたヘンリー・ボリンブルックが兵を挙げてリチャード2世を廃し、ヘンリー4世として即位した。",
     "The exiled Henry Bolingbroke raises an army, deposes Richard II and takes the throne as Henry IV.",
     "遭到放逐的亨利·博林布罗克起兵废黜理查二世，以亨利四世之名即位。"),
   'House("ランカスター家"), Dismiss("リチャード2世"), Purge(0.05)')
ev(1400, "ENGLAND", "Revolt", T("オワイン・グリンドゥールの反乱", "Owain Glyndŵr's rising", "欧文·格林杜尔起义"),
   T("ウェールズのオワイン・グリンドゥールがプリンス・オブ・ウェールズを名乗り、イングランドに背いた。",
     "Owain Glyndŵr proclaims himself Prince of Wales and rises against England.",
     "威尔士的欧文·格林杜尔自称威尔士亲王，起兵反抗英格兰。"),
   'Purge(0.05)')
ev(1402, "ENGLAND", "Revolt", T("ホミルドン・ヒルの戦い", "The Battle of Homildon Hill", "霍米尔顿山之战"),
   T("スコットランド軍が再びイングランドの北に攻め込んだ。",
     "The Scots once more invade the north of England.",
     "苏格兰军再度攻入英格兰北部。"),
   'ScotsRaid')
ev(1404, "FRANCE", "Hero", T("グリンドゥールとの同盟", "The alliance with Glyndŵr", "与格林杜尔结盟"),
   T("グリンドゥールはフランスと同盟を結び、フランスの兵がウェールズに上陸した。ウェールズの城はフランス王の側に付く。",
     "Glyndŵr allies with France, and French troops land in Wales. The Welsh castles take the side of the King of France.",
     "格林杜尔与法兰西结盟，法兰西军队登陆威尔士。威尔士诸城站到了法兰西国王一边。"),
   'Defect(L_WALES, FRANCE)')
ev(1407, "FRANCE", "Revolt", T("オルレアン公の暗殺", "The murder of the Duke of Orléans", "奥尔良公爵遇刺"),
   T("ブルゴーニュ公ジャン（無畏公）がパリの路上でオルレアン公ルイを殺させた。アルマニャック派とブルゴーニュ派の内乱が始まる。",
     "John the Fearless, Duke of Burgundy, has Louis of Orléans murdered in a Paris street. Civil war between Armagnacs and Burgundians begins.",
     "勃艮第公爵无畏的约翰派人在巴黎街头刺杀了奥尔良公爵路易。阿马尼亚克派与勃艮第派的内战爆发。"),
   'Figure("アルマニャック派とブルゴーニュ派の争い", -0.1, -0.1, 28.0)')
ev(1409, "ENGLAND", "Hero", T("ハーレックの陥落", "The fall of Harlech", "哈莱克陷落"),
   T("王太子ヘンリーがハーレック城を落とし、グリンドゥールの反乱は山に追いやられた。",
     "Prince Henry takes Harlech Castle, and Glyndŵr's rising is driven into the hills.",
     "王太子亨利攻陷哈莱克城堡，格林杜尔的起义被逼入山中。"),
   'Defect(L_WALES, ENGLAND)')
ev(1413, "ENGLAND", "Hero", T("ヘンリー5世の即位", "The accession of Henry V", "亨利五世即位"),
   T("若き王ヘンリー5世は、曾祖父エドワード3世のフランス王位の要求をふたたび掲げた。",
     "The young Henry V takes up again his great-grandfather Edward III's claim to the throne of France.",
     "年轻的亨利五世重新举起其曾祖父爱德华三世对法兰西王位的主张。"),
   'Figure("ヘンリー5世", 0.1, 0.15, 9.0)')
ev(1415, "ENGLAND", "Hero", T("アルフルールとアジャンクール", "Harfleur and Agincourt", "阿夫勒尔与阿金库尔"),
   T("ヘンリー5世はノルマンディーに上陸してアルフルールを囲み、アジャンクールでフランスの貴族の花を散らした。",
     "Henry V lands in Normandy, besieges Harfleur, and at Agincourt cuts down the flower of the French nobility.",
     "亨利五世登陆诺曼底，围攻阿夫勒尔，并在阿金库尔使法兰西贵族精华凋零。"),
   'War(ENGLAND, FRANCE), Levy(6), Train(0.3)')
ev(1417, "ENGLAND", "Revolt", T("「愚かな侵攻」", "The \"Foul Raid\"", "“愚蠢的入侵”"),
   T("王がフランスにいる隙を突き、スコットランド軍がロクスバラとベリックに攻め寄せた。",
     "With the king away in France, the Scots march on Roxburgh and Berwick.",
     "趁国王远在法兰西，苏格兰军进逼罗克斯堡与贝里克。"),
   'ScotsRaid')
ev(1418, "FRANCE", "Revolt", T("ブルゴーニュ派のパリ掌握", "The Burgundians seize Paris", "勃艮第派控制巴黎"),
   T("ブルゴーニュ派がパリに入り、アルマニャック派を殺した。王太子シャルルはパリを逃れた。",
     "The Burgundians enter Paris and massacre the Armagnacs; the Dauphin Charles flees the city.",
     "勃艮第派进入巴黎，屠杀阿马尼亚克派。王太子查理逃离巴黎。"),
   'Purge(0.1), Plunder(0.1)')
ev(1419, "FRANCE", "Disaster", T("モントローの暗殺", "The murder at Montereau", "蒙特罗刺杀事件"),
   T("和解の会見の場で、王太子の家臣がブルゴーニュ公ジャンを斬り殺した。息子フィリップ（善良公）はイングランドと結ぶ。",
     "At a meeting meant for reconciliation, the Dauphin's men cut down Duke John of Burgundy. His son Philip the Good allies with England.",
     "在本为和解的会面中，王太子的家臣砍杀了勃艮第公爵约翰。其子好人腓力转而与英格兰结盟。"),
   'Defect(L_BURGUNDY, ENGLAND), Defect(L_FLANDERS, ENGLAND)')
ev(1419, "FRANCE", "Hero", T("スコットランド軍の来援", "The Army of Scotland", "苏格兰军来援"),
   T("「古い同盟」に応え、バカン伯の率いるスコットランドの兵がラ・ロシェルに上陸した。",
     "Answering the Auld Alliance, Scottish troops under the Earl of Buchan land at La Rochelle.",
     "响应“古老同盟”，巴肯伯爵率领的苏格兰士兵在拉罗谢尔登陆。"),
   'ScotsArmy(6)')
ev(1420, "FRANCE", "Disaster", T("トロワ条約", "The Treaty of Troyes", "特鲁瓦条约"),
   T("狂気の王シャルル6世はヘンリー5世を娘婿とし、王位の後継者と定めた。パリとシャンパーニュの都市はイングランド王に従う。",
     "The mad Charles VI makes Henry V his son-in-law and heir to the throne. Paris and the towns of Champagne submit to the King of England.",
     "疯王查理六世将亨利五世招为女婿，并立为王位继承人。巴黎与香槟诸城归顺英格兰国王。"),
   'Defect(L_PARIS, ENGLAND), Defect(L_CHAMPAGNE, ENGLAND)')
ev(1421, "FRANCE", "Hero", T("ボージェの戦い", "The Battle of Baugé", "博热之战"),
   T("スコットランドとフランスの軍がボージェでクラレンス公を討ち取った。",
     "The Scots and French kill the Duke of Clarence at Baugé.",
     "苏格兰与法兰西联军在博热斩杀了克拉伦斯公爵。"),
   'Train(0.3)')
ev(1422, "ENGLAND", "Disaster", T("ヘンリー5世の死", "The death of Henry V", "亨利五世之死"),
   T("ヘンリー5世はヴァンセンヌで赤痢に倒れた。後を継いだヘンリー6世は生後九か月だった。",
     "Henry V dies of dysentery at Vincennes. His heir, Henry VI, is nine months old.",
     "亨利五世在万塞讷死于痢疾。继位的亨利六世年仅九个月。"),
   'Dismiss("ヘンリー5世")')
ev(1422, "FRANCE", "Minister", T("シャルル7世", "Charles VII", "查理七世"),
   T("シャルル6世も世を去った。ロワールの南に拠る王太子はシャルル7世を名乗ったが、人々は「ブールジュの王」と呼んだ。",
     "Charles VI dies too. The Dauphin, holding out south of the Loire, calls himself Charles VII — but men call him \"the King of Bourges\".",
     "查理六世也去世了。据守卢瓦尔河以南的王太子自称查理七世，但人们称他为“布尔日之王”。"),
   'Dismiss("シャルル6世の狂気"), Figure("シャルル7世", 0.05, 0.0, 39.0)')
ev(1423, "ENGLAND", "Hero", T("アミアン条約", "The Treaty of Amiens", "亚眠条约"),
   T("ブルターニュ公ジャン5世は、ベッドフォード公とブルゴーニュ公と同盟を結んだ。",
     "John V, Duke of Brittany, joins Bedford and Burgundy in alliance.",
     "布列塔尼公爵约翰五世与贝德福德公爵、勃艮第公爵结成同盟。"),
   'Defect(L_BRITTANY, ENGLAND), Defect(L_MONTFORT, ENGLAND)')
ev(1424, "FRANCE", "Hero", T("スコットランドの第二陣", "A second Scottish army", "苏格兰第二批援军"),
   T("ダグラス伯がさらに六千の兵を率いてフランスに渡った。",
     "The Earl of Douglas crosses to France with six thousand more men.",
     "道格拉斯伯爵又率六千兵马渡海来到法兰西。"),
   'ScotsArmy(6)')
ev(1425, "FRANCE", "Hero", T("リッシュモンの大元帥就任", "Richemont made Constable", "里什蒙出任大元帅"),
   T("ブルターニュ公の弟アルテュール・ド・リッシュモンがフランスの大元帥となり、ブルターニュはフランス王の側に戻った。",
     "Arthur de Richemont, the Duke of Brittany's brother, becomes Constable of France, and Brittany returns to the King of France.",
     "布列塔尼公爵之弟阿蒂尔·德·里什蒙出任法兰西大元帅，布列塔尼重归法兰西国王一方。"),
   'Defect(L_BRITTANY, FRANCE), Defect(L_MONTFORT, FRANCE)')
ev(1428, "ENGLAND", "Hero", T("オルレアンの包囲", "The siege of Orléans", "奥尔良之围"),
   T("イングランド軍はロワールの要オルレアンを囲んだ。この町が落ちれば、王太子の南の国は開け放たれる。",
     "The English lay siege to Orléans, the key to the Loire. If it falls, the Dauphin's south lies open.",
     "英格兰军包围了卢瓦尔河的要冲奥尔良。此城若陷，王太子的南方将门户洞开。"),
   'Levy(4)')
ev(1429, "FRANCE", "Hero", T("ランスの戴冠", "The coronation at Reims", "兰斯加冕"),
   T("ジャンヌ・ダルクはオルレアンを解き、パテーで勝ち、王太子をランスへ導いて戴冠させた。道の街々は次々と城門を開いた。",
     "Joan of Arc relieves Orléans, wins at Patay and leads the Dauphin to his coronation at Reims; town after town on the road opens its gates.",
     "贞德解奥尔良之围，在帕提获胜，引导王太子前往兰斯加冕。沿途诸城纷纷打开城门。"),
   'DefectIfHero(L_CHAMPAGNE, FRANCE, 13), Train(0.3)')
ev(1431, "ENGLAND", "Minister", T("ヘンリー6世のパリ戴冠", "Henry VI crowned in Paris", "亨利六世在巴黎加冕"),
   T("十歳のヘンリー6世がパリのノートルダムでフランス王として戴冠した。だがフランスの人々の心は離れていた。",
     "The ten-year-old Henry VI is crowned King of France in Notre-Dame de Paris — but French hearts are no longer with him.",
     "十岁的亨利六世在巴黎圣母院加冕为法兰西国王，但法兰西的人心早已离去。"),
   'Gold(150.0)')
ev(1435, "FRANCE", "Hero", T("アラスの和約", "The Congress of Arras", "阿拉斯和约"),
   T("ブルゴーニュ公フィリップはシャルル7世と和を結び、イングランドとの同盟を捨てた。",
     "Philip, Duke of Burgundy, makes peace with Charles VII and abandons the English alliance.",
     "勃艮第公爵腓力与查理七世议和，抛弃了与英格兰的同盟。"),
   'Defect(L_BURGUNDY, FRANCE), Defect(L_FLANDERS, FRANCE)')
ev(1435, "ENGLAND", "Disaster", T("ベッドフォード公の死", "The death of Bedford", "贝德福德公爵之死"),
   T("フランスを治めた摂政ベッドフォード公がルーアンで世を去った。",
     "The Duke of Bedford, regent of France, dies at Rouen.",
     "治理法兰西的摄政贝德福德公爵在鲁昂去世。"),
   'Bias(0.92)')
ev(1436, "FRANCE", "Hero", T("パリの回復", "Paris recovered", "收复巴黎"),
   T("パリの市民は城門を開いてリッシュモンの軍を迎え入れた。",
     "The citizens of Paris open the gates to Richemont's army.",
     "巴黎市民打开城门，迎接里什蒙的军队。"),
   'Defect(L_PARIS, FRANCE)')
ev(1439, "FRANCE", "Minister", T("恒常のタイユ", "The permanent taille", "永久的人头税"),
   T("オルレアンの三部会は、王が常備の兵を養うための税を毎年取ることを認めた。",
     "The Estates at Orléans allow the king to levy a yearly tax to keep a standing army.",
     "奥尔良三级会议同意国王每年征税以供养常备军。"),
   'Gold(300.0), Bias(1.05)')
ev(1440, "FRANCE", "Revolt", T("プラグリーの乱", "The Praguerie", "布拉格里叛乱"),
   T("王の改革を嫌う大貴族たちが王太子ルイを担いで背いた。",
     "Great nobles who resent the king's reforms rebel with the Dauphin Louis at their head.",
     "厌恶国王改革的大贵族拥立王太子路易起兵叛乱。"),
   'Purge(0.05)')
ev(1440, "FRANCE", "Minister", T("ジャック・クール", "Jacques Cœur", "雅克·科尔"),
   T("ブールジュの商人ジャック・クールが王の財務を預かり、地中海の交易で国庫を満たした。",
     "Jacques Cœur, merchant of Bourges, takes charge of the king's finances and fills the treasury with Mediterranean trade.",
     "布尔日商人雅克·科尔执掌国王财政，以地中海贸易充实国库。"),
   'Figure("ジャック・クール", 0.2, 0.0, 11.0)')
ev(1444, "FRANCE", "Minister", T("トゥールの休戦", "The Truce of Tours", "图尔休战"),
   T("ヘンリー6世はアンジューのマルグリットを妃に迎え、両国は休戦した。",
     "Henry VI marries Margaret of Anjou, and the two kingdoms agree a truce.",
     "亨利六世迎娶安茹的玛格丽特为王后，两国休战。"),
   'Truce(ENGLAND, FRANCE, 5.0)')
ev(1445, "FRANCE", "Hero", T("勅令隊の創設", "The Compagnies d'ordonnance", "敕令骑士团的创建"),
   T("シャルル7世は傭兵団を解き、王が給金を払う常備の騎兵隊を置いた。",
     "Charles VII disbands the free companies and creates standing companies of cavalry paid by the crown.",
     "查理七世解散雇佣兵团，设立由王室支薪的常备骑兵队。"),
   'Train(0.5)')
ev(1449, "ENGLAND", "Disaster", T("フージェールの略奪", "The sack of Fougères", "洗劫富热尔"),
   T("イングランドの傭兵隊長が休戦を破ってブルターニュのフージェールを襲い、ブルターニュはフランスと結び、戦が再び始まった。",
     "An English captain breaks the truce and sacks Fougères in Brittany; Brittany joins France and the war begins again.",
     "英格兰雇佣兵队长撕毁休战，袭击布列塔尼的富热尔；布列塔尼与法兰西联手，战事再起。"),
   'War(FRANCE, ENGLAND), Defect(L_BRITTANY, FRANCE), Defect(L_MONTFORT, FRANCE)')
ev(1450, "ENGLAND", "Revolt", T("ジャック・ケイドの乱", "Jack Cade's rebellion", "杰克·凯德起义"),
   T("ノルマンディーの喪失に怒ったケントの民がロンドンに攻め入った。",
     "Enraged by the loss of Normandy, the men of Kent march on London.",
     "因诺曼底的丧失而愤怒的肯特民众攻入伦敦。"),
   'Purge(0.08), Plunder(0.15)')
ev(1452, "ENGLAND", "Hero", T("ボルドーの蜂起", "The rising of Bordeaux", "波尔多起义"),
   T("三百年イングランド王に仕えてきたボルドーの市民は、フランスの重い税を嫌ってタルボットを呼び戻した。",
     "The citizens of Bordeaux, subjects of the English king for three hundred years, resent the heavy French taxes and call Talbot back.",
     "效忠英格兰国王三百年的波尔多市民不满法兰西的重税，召回了塔尔博特。"),
   'Defect(L_GUYENNE, ENGLAND)')
ev(1453, "FRANCE", "Hero", T("カスティヨンの戦い", "The Battle of Castillon", "卡斯蒂永之战"),
   T("ビュロー兄弟の大砲が、突撃するタルボットの軍をなぎ倒した。この年のうちに決着をつけねばならない。",
     "The Bureau brothers' guns mow down Talbot's charging army. The war must be decided before this year is out.",
     "比罗兄弟的大炮扫倒了冲锋的塔尔博特军。必须在今年之内分出胜负。"),
   'Train(0.3)')
ev(1453, "ENGLAND", "Disaster", T("ヘンリー6世の狂気", "The madness of Henry VI", "亨利六世的疯病"),
   T("ヘンリー6世は祖父シャルル6世と同じく正気を失い、王国は薔薇戦争へと傾いてゆく。",
     "Like his grandfather Charles VI, Henry VI loses his reason, and the kingdom slides towards the Wars of the Roses.",
     "亨利六世与其外祖父查理六世一样失去了神智，王国逐渐滑向玫瑰战争。"),
   'Figure("ヘンリー6世の狂気", -0.1, -0.1, 5.0)')

# ---------------------------------------------------------------- その他のデータの訳（日本語, 英語, 中国語）
OTHER = [
    ("イングランド", "England", "英格兰"), ("フランス", "France", "法兰西"), ("部族", "Free companies", "部族"), ("天の軍勢", "The Hosts of Heaven", "天之军势"),
    ("ロンドンを都とする王国。海峡の向こうにボルドーとポンティユーを持ち、フランス王位を求める。船を造れる",
     "A kingdom ruled from London, holding Bordeaux and Ponthieu across the Channel and claiming the French crown. Can build ships",
     "以伦敦为都的王国，在海峡对岸领有波尔多与蓬蒂厄，并要求法兰西王位。能造船"),
    ("パリを都とする西欧一の大国。諸侯の忠誠は揺れ動く。船は造れず、ガレー船を雇う",
     "The greatest kingdom in the West, ruled from Paris; its lords' loyalty wavers. Cannot build ships, but hires galleys",
     "以巴黎为都的西欧第一大国，诸侯的忠诚摇摆不定。无法造船，需雇佣桨帆船"),
    # 諸侯
    ("王領", "Royal domain", "王室领地"), ("王が直接治める土地", "Land ruled directly by the king", "国王直辖的土地"),
    ("フランドル", "Flanders", "佛兰德斯"), ("毛織物の都市ヘント・ブルッヘ・リール。イングランドの羊毛なしには立ち行かない", "The cloth towns of Ghent, Bruges and Lille, which cannot live without English wool", "根特、布鲁日、里尔等毛纺城市，离不开英格兰的羊毛"),
    ("ブルターニュ", "Brittany", "布列塔尼"), ("ブルターニュ公の土地。継承戦争の後も、英仏の間で揺れ動いた", "The lands of the Duke of Brittany, which swung between England and France even after the war of succession", "布列塔尼公爵的领地，在继承战争后仍摇摆于英法之间"),
    ("モンフォール派", "The Montfort party", "蒙福尔派"), ("ブルターニュ継承戦争でイングランドと結んだモンフォール家の拠点（ブレスト・ヴァンヌ）", "Strongholds of the house of Montfort, England's ally in the War of the Breton Succession (Brest, Vannes)", "布列塔尼继承战争中与英格兰结盟的蒙福尔家据点（布雷斯特、瓦讷）"),
    ("ナバラ王", "The King of Navarre", "纳瓦拉国王"), ("エヴルー家のナバラ王シャルル（悪王）がノルマンディーに持つ土地", "The Norman lands of Charles the Bad, King of Navarre, of the house of Évreux", "埃夫勒家的纳瓦拉国王恶人查理在诺曼底拥有的土地"),
    ("ブルゴーニュ公", "The Duke of Burgundy", "勃艮第公爵"), ("ヴァロワ家の分家。1419年のモントローの暗殺の後、イングランドと結ぶ", "A cadet branch of the Valois, which allied with England after the murder at Montereau in 1419", "瓦卢瓦家的旁支，1419年蒙特罗刺杀事件后与英格兰结盟"),
    ("ガスコーニュ諸侯", "The Gascon lords", "加斯科涅诸侯"), ("アルブレ家・アルマニャック家などのガスコーニュの領主たち", "The lords of Gascony, such as the houses of Albret and Armagnac", "阿尔布雷家、阿马尼亚克家等加斯科涅领主"),
    ("パリ市民", "The citizens of Paris", "巴黎市民"), ("1418年にブルゴーニュ派が掌握したパリ", "Paris, seized by the Burgundians in 1418", "1418年被勃艮第派控制的巴黎"),
    ("シャンパーニュの諸都市", "The towns of Champagne", "香槟诸城"), ("ランス・トロワ・ラン・コンピエーニュなど、トロワ条約でイングランド王に従った北の都市", "Reims, Troyes, Laon, Compiègne and other northern towns that submitted to the King of England under the Treaty of Troyes", "兰斯、特鲁瓦、拉昂、贡比涅等依特鲁瓦条约归顺英格兰国王的北方城市"),
    ("ウェールズ", "Wales", "威尔士"), ("1400年にオワイン・グリンドゥールが反乱を起こし、フランスと結んだ", "Where Owain Glyndŵr rose in 1400 and allied with France", "1400年欧文·格林杜尔起兵反叛，并与法兰西结盟"),
    ("ギュイエンヌの都市", "The towns of Guyenne", "吉耶讷诸城"), ("ボルドーとバイヨンヌ。三百年来イングランド王に仕え、ワインの交易で結ばれている", "Bordeaux and Bayonne — subjects of the English king for three hundred years, bound to him by the wine trade", "波尔多与巴约讷，三百年来效忠英格兰国王，以葡萄酒贸易相连"),
    # 街
    ("ロンドン", "London", "伦敦"), ("ドーヴァー", "Dover", "多佛尔"), ("サウサンプトン", "Southampton", "南安普敦"), ("エクセター", "Exeter", "埃克塞特"),
    ("プリマス", "Plymouth", "普利茅斯"), ("ブリストル", "Bristol", "布里斯托尔"), ("コヴェントリー", "Coventry", "考文垂"), ("ノリッジ", "Norwich", "诺里奇"),
    ("リンカン", "Lincoln", "林肯"), ("ヨーク", "York", "约克"), ("チェスター", "Chester", "切斯特"), ("ニューカッスル", "Newcastle", "纽卡斯尔"),
    ("カーライル", "Carlisle", "卡莱尔"), ("ベリック", "Berwick", "贝里克"), ("ハーレック", "Harlech", "哈莱克"), ("カーマーゼン", "Carmarthen", "卡马森"),
    ("ボルドー", "Bordeaux", "波尔多"), ("バイヨンヌ", "Bayonne", "巴约讷"), ("モン＝ド＝マルサン", "Mont-de-Marsan", "蒙德马桑"), ("アブヴィル", "Abbeville", "阿布维尔"),
    ("パリ", "Paris", "巴黎"), ("ルーアン", "Rouen", "鲁昂"), ("アルフルール", "Harfleur", "阿夫勒尔"), ("ディエップ", "Dieppe", "迪耶普"),
    ("カーン", "Caen", "卡昂"), ("シェルブール", "Cherbourg", "瑟堡"), ("エヴルー", "Évreux", "埃夫勒"), ("モン＝サン＝ミシェル", "Mont-Saint-Michel", "圣米歇尔山"),
    ("カレー", "Calais", "加来"), ("アミアン", "Amiens", "亚眠"), ("ボーヴェ", "Beauvais", "博韦"), ("コンピエーニュ", "Compiègne", "贡比涅"),
    ("ラン", "Laon", "拉昂"), ("ランス", "Reims", "兰斯"), ("トロワ", "Troyes", "特鲁瓦"), ("アラス", "Arras", "阿拉斯"),
    ("リール", "Lille", "里尔"), ("ヘント", "Ghent", "根特"), ("ブルッヘ", "Bruges", "布鲁日"), ("ディジョン", "Dijon", "第戎"),
    ("オセール", "Auxerre", "欧塞尔"), ("マコン", "Mâcon", "马孔"), ("ヴォークルール", "Vaucouleurs", "沃库勒尔"), ("シャルトル", "Chartres", "沙特尔"),
    ("ル・マン", "Le Mans", "勒芒"), ("アランソン", "Alençon", "阿朗松"), ("フージェール", "Fougères", "富热尔"), ("レンヌ", "Rennes", "雷恩"),
    ("サン＝マロ", "Saint-Malo", "圣马洛"), ("ナント", "Nantes", "南特"), ("ヴァンヌ", "Vannes", "瓦讷"), ("ブレスト", "Brest", "布雷斯特"),
    ("アンジェ", "Angers", "昂热"), ("トゥール", "Tours", "图尔"), ("ブロワ", "Blois", "布卢瓦"), ("オルレアン", "Orléans", "奥尔良"),
    ("ブールジュ", "Bourges", "布尔日"), ("ヌヴェール", "Nevers", "讷韦尔"), ("ポワティエ", "Poitiers", "普瓦捷"), ("ラ・ロシェル", "La Rochelle", "拉罗谢尔"),
    ("サント", "Saintes", "桑特"), ("アングレーム", "Angoulême", "昂古莱姆"), ("リモージュ", "Limoges", "利摩日"), ("ペリグー", "Périgueux", "佩里格"),
    ("ベルジュラック", "Bergerac", "贝尔热拉克"), ("アジャン", "Agen", "阿让"), ("カオール", "Cahors", "卡奥尔"), ("トゥールーズ", "Toulouse", "图卢兹"),
    ("オーシュ", "Auch", "欧什"), ("タルブ", "Tarbes", "塔布"), ("カルカソンヌ", "Carcassonne", "卡尔卡松"), ("ロデーズ", "Rodez", "罗德兹"),
    ("クレルモン", "Clermont", "克莱蒙"), ("ル・ピュイ", "Le Puy", "勒皮"), ("リヨン", "Lyon", "里昂"), ("ニーム", "Nîmes", "尼姆"),
]

# 兵種
UNITS = [
    ("民", "Worker", "民"), ("資源を集め、村や建物を建てる", "Gathers resources and builds villages and buildings", "采集资源，建造村庄与建筑"),
    ("槍兵", "Spearmen", "枪兵"), ("長槍で騎兵を止める歩兵。城攻めに優れる", "Infantry whose long spears stop cavalry; good at sieges", "以长枪阻挡骑兵的步兵，擅长攻城"),
    ("弓兵", "Archers", "弓兵"), ("遠くから射る。歩兵に強いが騎兵に弱い", "Shoots from range. Strong against infantry, weak against cavalry", "远程射击。克制步兵，惧骑兵"),
    ("騎兵", "Cavalry", "骑兵"), ("速い突撃兵。弓兵と民に強い", "Fast shock troops, strong against archers and workers", "高速突击兵，克制弓兵与民"),
    ("ロングボウ兵", "Longbowmen", "长弓兵"), ("イチイの長弓を引くヨーマン。射程が長く速射し、突撃する騎士を射すくめる", "Yeomen who draw the yew longbow — long range, fast shooting, and deadly to charging knights", "拉开紫杉长弓的自耕农。射程远、射速快，能射退冲锋的骑士"),
    ("下馬重装兵", "Dismounted men-at-arms", "下马重装兵"), ("馬を降りて弓兵と並ぶ重装の騎士・従者。騎兵の突撃を受け止める", "Armoured knights and squires who fight on foot beside the archers and take the shock of cavalry", "下马与弓兵并肩作战的重装骑士与侍从，能顶住骑兵冲锋"),
    ("騎乗弓兵", "Mounted archers", "骑乘弓兵"), ("馬で駆け、降りて射る弓兵。騎行（シュヴォシェ）で村々を荒らす", "Archers who ride and dismount to shoot; the scourge of villages on the chevauchée", "骑马行军、下马射击的弓兵，在骑行中蹂躏村庄"),
    ("騎士", "Knights", "骑士"), ("イングランド王に仕える騎士", "Knights in the service of the King of England", "效忠英格兰国王的骑士"),
    ("コグ船", "Cog", "柯克船"), ("兵6人を乗せて海峡を渡る丸い帆船。船首と船尾の楼から射る", "A round-hulled sailing ship that carries 6 troops across the Channel and shoots from its castles fore and aft", "可载6名士兵渡过海峡的圆身帆船，从船首船尾的楼台射击"),
    ("フランス騎士", "French knights", "法兰西骑士"), ("ヨーロッパに名高いフランスの騎士。突撃は何よりも重い", "The famed chivalry of France — no charge in Europe hits harder", "闻名欧洲的法兰西骑士，冲锋之势无可匹敌"),
    ("弩兵", "Crossbowmen", "弩兵"), ("鎧を射抜くクロスボウ兵。強いが装填は遅い", "Crossbowmen whose bolts pierce armour — powerful but slow to reload", "能射穿铠甲的弩兵，威力大但装填慢"),
    ("ジェノヴァ弩兵", "Genoese crossbowmen", "热那亚弩兵"), ("大盾（パヴィース）を携えたジェノヴァの傭兵弩兵", "Genoese mercenary crossbowmen with their great pavise shields", "携带大盾（帕维斯盾）的热那亚雇佣弩兵"),
    ("ガレー船", "Galley", "桨帆船"), ("ジェノヴァなどイタリアの都市から雇う櫂船。兵6人を乗せて海を渡る", "An oared warship hired from Genoa and the other Italian cities; carries 6 troops across the sea", "向热那亚等意大利城市雇佣的桨船，可载6名士兵渡海"),
    ("勅令隊", "Compagnies d'ordonnance", "敕令骑士团"), ("シャルル7世が置いたヨーロッパ初の常備軍の重騎兵（1445年〜）", "The heavy cavalry of Europe's first standing army, created by Charles VII (from 1445)", "查理七世设立的欧洲首支常备军重骑兵（1445年起）"),
    ("フラン・アルシェ", "Francs-archers", "自由弓兵"), ("教区ごとに一人出す、税を免じられた自由弓兵（1448年〜）", "Free archers, one from each parish, exempt from tax (from 1448)", "每个教区出一人、免除赋税的自由弓兵（1448年起）"),
    ("ボンバード", "Bombard", "射石炮"), ("鉄の輪を束ねた大砲。遅いが城壁を崩す（1375年〜）", "A great gun of hooped iron bars — slow, but it brings down walls (from 1375)", "以铁箍束成的大炮，虽慢却能轰塌城墙（1375年起）"),
    ("スコットランド槍兵", "Scottish spearmen", "苏格兰枪兵"), ("長槍を林のように並べるシルトロンの兵", "Men of the schiltron, their long spears ranked like a forest", "长枪林立的圆阵（希尔特隆）士兵"),
    ("傭兵団", "Free company", "雇佣兵团"), ("休戦で雇い主を失い、村々を荒らす大傭兵団（グランド・カンパニー）", "Soldiers left without a master by the truce, who ravage the villages (the Great Companies)", "因休战失去雇主、蹂躏村庄的大雇佣兵团"),
    ("王の砲兵", "Royal artillery", "王家炮兵"), ("ビュロー兄弟が鍛えたフランス王の砲兵。フォルミニーとカスティヨンで勝負を決めた（1436年〜）", "The King of France's artillery, forged by the Bureau brothers; it decided Formigny and Castillon (from 1436)", "比罗兄弟锤炼的法兰西王家炮兵，决定了福米尼与卡斯蒂永之战的胜负（1436年起）"),
    ("天使", "Angel", "天使"), ("ジャンヌ・ダルクの亡霊に率いられた天の軍勢。人の剣も矢も届かない", "The hosts of Heaven, led by the ghost of Joan of Arc. No mortal sword or arrow can touch them", "由贞德亡灵率领的天之军势，凡人的刀剑弓矢皆无法触及"),
    ("ジャンヌの亡霊", "Ghost of Joan", "贞德的亡灵"), ("ルーアンで灰となった乙女の亡霊。百年の戦を終わらせなかった両国を裁く", "The ghost of the Maid, burned to ashes at Rouen, come to judge the two kingdoms that would not end their war", "在鲁昂化为灰烬的少女亡灵，前来审判未能结束百年战争的两国"),
]

# 国の特徴
TRAITS_T = [
    ("ロングボウ", "The longbow", "长弓"), ("ウェールズとイングランドのヨーマンが引く長弓。弓兵の攻撃+25%", "The longbow drawn by Welsh and English yeomen. Archer attack +25%", "威尔士与英格兰自耕农所用的长弓。弓兵攻击+25%"),
    ("騎行（シュヴォシェ）", "The chevauchée", "骑行"), ("敵地を駆け、村を焼いて王の威信を損なう戦い方。村の略奪で奪う物資+60%、騎兵の訓練費-15%", "Riding through enemy land, burning villages and shaming the enemy king. Village plunder +60%, cavalry training cost −15%", "纵横敌境、焚烧村庄以损敌王威信的战法。劫掠村庄所得+60%，骑兵训练费-15%"),
    ("羊毛の富", "The wealth of wool", "羊毛之富"), ("フランドルの織物を支えるイングランドの羊毛。商人が集まる（×1.4）、市場の値-15%", "English wool feeds the looms of Flanders. Merchants are drawn (×1.4), market prices −15%", "英格兰羊毛支撑着佛兰德斯的纺织业。吸引商人（×1.4），市场价格-15%"),
    ("海峡の向こう", "Across the Channel", "海峡彼岸"), ("援軍も糧も海を越えねばならない。維持費+10%", "Reinforcements and supplies alike must cross the sea. Upkeep +10%", "援军与粮草都必须渡海。维持费+10%"),
    ("ランカスター家の征服", "The Lancastrian conquest", "兰开斯特征服"), ("1415年、ヘンリー5世はノルマンディーの征服に乗り出した。歩兵の攻撃+15%", "In 1415 Henry V set out to conquer Normandy. Infantry attack +15%", "1415年，亨利五世着手征服诺曼底。步兵攻击+15%"),
    ("騎士道", "Chivalry", "骑士精神"), ("名誉を重んじる騎士の突撃。騎兵の攻撃+20%（ただしロングボウに弱い）", "The charge of knights who prize honour above all. Cavalry attack +20% (but beware the longbow)", "重视荣誉的骑士冲锋。骑兵攻击+20%（但惧长弓）"),
    ("西欧一の大国", "The greatest kingdom in the West", "西欧第一大国"), ("人口はイングランドの四倍。生産+10%、民の訓練費-10%", "Four times England's population. Production +10%, worker training cost −10%", "人口为英格兰的四倍。生产+10%，民的训练费-10%"),
    ("城壁の都市", "Walled towns", "城墙之城"), ("城壁に籠もる街は簡単には落ちない。街・建物の受ける損害-15%", "Towns behind their walls do not fall easily. Damage to towns and buildings −15%", "据守城墙的城镇不易攻陷。城镇与建筑所受伤害-15%"),
    ("賢明王の改革", "The reforms of Charles the Wise", "贤明王的改革"), ("シャルル5世は会戦を避け、城を一つずつ取り返した。鍛錬の費用-25%、維持費-10%", "Charles V avoided battle and won back the castles one by one. Training cost −25%, upkeep −10%", "查理五世避免会战，逐一收复城堡。锻炼费用-25%，维持费-10%"),
    ("常備軍と砲兵", "Standing army and artillery", "常备军与炮兵"), ("勅令隊とビュロー兄弟の砲兵。維持費-20%、弓兵（砲兵を含む）の攻撃+15%", "The Compagnies d'ordonnance and the Bureau brothers' guns. Upkeep −20%, archer (and artillery) attack +15%", "敕令骑士团与比罗兄弟的炮兵。维持费-20%，弓兵（含炮兵）攻击+15%"),
]

# 英雄（名前・時代・説明）
HERO_T = [
    ("エドワード3世", "Edward III", "爱德华三世"), ("イングランド王・1337〜1377年", "King of England, 1337–1377", "英格兰国王・1337〜1377年"),
    ("フランス王位を求めて百年戦争を始めた王。クレシーで大勝し、カレーを囲んだ", "The king who began the Hundred Years' War to claim the crown of France; victor of Crécy and besieger of Calais", "为争夺法兰西王位而发动百年战争的国王。在克雷西大胜，并围攻加来"),
    ("エドワード黒太子", "Edward the Black Prince", "黑太子爱德华"), ("イングランド・1346〜1376年", "England, 1346–1376", "英格兰・1346〜1376年"),
    ("十六歳でクレシーに戦い、ポワティエでフランス王ジャン2世を捕らえた。黒い鎧の王太子", "He fought at Crécy at sixteen and captured King John II of France at Poitiers — the prince in black armour", "十六岁参战克雷西，在普瓦捷俘获法王约翰二世。身披黑甲的王太子"),
    ("ランカスター公ヘンリー", "Henry, Duke of Lancaster", "兰开斯特公爵亨利"), ("イングランド・1345〜1361年", "England, 1345–1361", "英格兰・1345〜1361年"),
    ("ヘンリー・オブ・グロスモント。ガスコーニュとブルターニュで勝ち続けた名将。黒死病に倒れた", "Henry of Grosmont, the great general who won again and again in Gascony and Brittany. He died of the plague", "格罗斯蒙特的亨利，在加斯科涅与布列塔尼屡战屡胜的名将，死于黑死病"),
    ("ジョン・シャンドス", "Sir John Chandos", "约翰·钱多斯"), ("イングランド・1346〜1370年", "England, 1346–1370", "英格兰・1346〜1370年"),
    ("黒太子の右腕。オーレーでデュ・ゲクランを捕らえ、リュサックの橋で討たれた", "The Black Prince's right hand. He captured Du Guesclin at Auray and fell at the bridge of Lussac", "黑太子的左膀右臂。在欧赖俘获迪盖克兰，战死于吕萨克桥"),
    ("ロバート・ノールズ", "Sir Robert Knolles", "罗伯特·诺利斯"), ("イングランド・1351〜1407年", "England, 1351–1407", "英格兰・1351〜1407年"),
    ("傭兵隊長から身を起こし、騎行でパリの城門まで迫った", "A captain of free companies who rose to lead a chevauchée to the very gates of Paris", "由雇佣兵队长起家，骑行直抵巴黎城门"),
    ("ジョン・オブ・ゴーント", "John of Gaunt", "冈特的约翰"), ("イングランド・1369〜1399年", "England, 1369–1399", "英格兰・1369〜1399年"),
    ("エドワード3世の子、ランカスター公。カレーからボルドーへの大騎行を率いた", "Son of Edward III and Duke of Lancaster; he led the great chevauchée from Calais to Bordeaux", "爱德华三世之子、兰开斯特公爵，率领了从加来到波尔多的大骑行"),
    ("ヘンリー・パーシー（ホットスパー）", "Henry Percy (Hotspur)", "亨利·珀西（霍茨波）"), ("イングランド・1385〜1403年", "England, 1385–1403", "英格兰・1385〜1403年"),
    ("スコットランドとの境を守った北の猛将。オッターバーンで捕らえられ、王に背いて討たれた", "The fierce northern lord who held the Scottish border; captured at Otterburn, he died rebelling against the king", "镇守苏格兰边境的北方猛将，曾在奥特本被俘，后因反叛国王而战死"),
    ("ヘンリー5世", "Henry V", "亨利五世"), ("イングランド王・1413〜1422年", "King of England, 1413–1422", "英格兰国王・1413〜1422年"),
    ("アジャンクールで大勝し、ノルマンディーを征し、トロワ条約でフランス王の後継者となった", "Victor of Agincourt and conqueror of Normandy, made heir to the King of France by the Treaty of Troyes", "在阿金库尔大胜、征服诺曼底，并依特鲁瓦条约成为法兰西国王的继承人"),
    ("ベッドフォード公ジョン", "John, Duke of Bedford", "贝德福德公爵约翰"), ("イングランド・1422〜1435年", "England, 1422–1435", "英格兰・1422〜1435年"),
    ("幼いヘンリー6世に代わってフランスを治めた摂政。ヴェルヌイユで勝った", "Regent of France for the infant Henry VI; victor of Verneuil", "代年幼的亨利六世治理法兰西的摄政，赢得韦尔讷伊之战"),
    ("ソールズベリー伯トマス・モンタギュー", "Thomas Montagu, Earl of Salisbury", "索尔兹伯里伯爵托马斯·蒙塔古"), ("イングランド・1415〜1428年", "England, 1415–1428", "英格兰・1415〜1428年"),
    ("攻城の名手。オルレアンの包囲のさなか、砲弾に倒れた", "A master of sieges, struck down by a cannonball during the siege of Orléans", "攻城高手，在奥尔良围城战中被炮弹击中身亡"),
    ("ジョン・タルボット", "John Talbot", "约翰·塔尔博特"), ("イングランド・1427〜1453年", "England, 1427–1453", "英格兰・1427〜1453年"),
    ("「イングランドのアキレス」。パテーで捕らえられ、八十に近くしてカスティヨンで討たれた", "\"The English Achilles\". Captured at Patay, he was killed at Castillon at nearly eighty", "“英格兰的阿喀琉斯”。曾在帕提被俘，年近八十时战死于卡斯蒂永"),
    ("ジョン・ファストルフ", "Sir John Fastolf", "约翰·法斯托夫"), ("イングランド・1415〜1459年", "England, 1415–1459", "英格兰・1415〜1459年"),
    ("ニシンの戦いで補給の荷車を守り抜いた老練の騎士", "The veteran knight who held the supply wagons at the Battle of the Herrings", "在鲱鱼之战中守住补给车队的老练骑士"),
    ("フィリップ6世", "Philip VI", "腓力六世"), ("フランス王・1337〜1350年", "King of France, 1337–1350", "法兰西国王・1337〜1350年"),
    ("ヴァロワ朝の初代の王。ギュイエンヌを没収して戦を始め、クレシーで敗れた", "First king of the Valois line; he began the war by confiscating Guyenne and lost at Crécy", "瓦卢瓦王朝首位国王，没收吉耶讷挑起战争，兵败克雷西"),
    ("ジャンヌ・ダルク", "Joan of Arc", "贞德"), ("フランス・1429〜1431年", "France, 1429–1431", "法兰西・1429〜1431年"),
    ("神の声を聞いたロレーヌの乙女。オルレアンを解放し、王をランスで戴冠させた。身代金では救えない", "The Maid of Lorraine who heard the voice of God; she relieved Orléans and had the king crowned at Reims. She cannot be ransomed", "听见神之声音的洛林少女。解救奥尔良，令国王在兰斯加冕。无法以赎金赎回"),
    ("ジャン2世", "John II", "约翰二世"), ("フランス王・1350〜1364年", "King of France, 1350–1364", "法兰西国王・1350〜1364年"),
    ("善良王。ポワティエで黒太子に捕らえられ、三百万エキュの身代金を求められた", "John the Good, captured by the Black Prince at Poitiers and held for a ransom of three million écus", "好人约翰。在普瓦捷被黑太子俘获，被索要三百万埃居赎金"),
    ("ベルトラン・デュ・ゲクラン", "Bertrand du Guesclin", "贝特朗·迪盖克兰"), ("フランス・1357〜1380年", "France, 1357–1380", "法兰西・1357〜1380年"),
    ("ブルターニュの小貴族から大元帥へ。会戦を避けて城を一つずつ取り返した", "From minor Breton noble to Constable of France; he avoided battle and won back the castles one by one", "由布列塔尼小贵族升任大元帅，避开会战，逐一收复城堡"),
    ("オリヴィエ・ド・クリッソン", "Olivier de Clisson", "奥利维耶·德·克利松"), ("フランス・1370〜1407年", "France, 1370–1407", "法兰西・1370〜1407年"),
    ("「屠殺者」の異名を持つブルターニュの大元帥。デュ・ゲクランの後を継いだ", "The Breton constable called \"the Butcher\", successor to Du Guesclin", "绰号“屠夫”的布列塔尼大元帅，继承了迪盖克兰之位"),
    ("ジャン・ド・ヴィエンヌ", "Jean de Vienne", "让·德·维埃纳"), ("フランス・1373〜1396年", "France, 1373–1396", "法兰西・1373〜1396年"),
    ("フランスの提督。ガレー船でイングランドの海岸を襲い、ニコポリスで斃れた", "Admiral of France who raided the English coast with galleys; he fell at Nicopolis", "法兰西海军上将，率桨帆船袭击英格兰海岸，战死于尼科波利斯"),
    ("ブーシコー", "Boucicaut", "布西科"), ("フランス・1391〜1421年", "France, 1391–1421", "法兰西・1391〜1421年"),
    ("騎士道の鑑と讃えられた元帥。アジャンクールで捕らえられ、イングランドで世を去った", "The marshal hailed as the mirror of chivalry; captured at Agincourt, he died in England", "被誉为骑士典范的元帅，在阿金库尔被俘，客死英格兰"),
    ("バカン伯ジョン・ステュアート", "John Stewart, Earl of Buchan", "巴肯伯爵约翰·斯图尔特"), ("フランス（スコットランド人）・1419〜1424年", "France (a Scot), 1419–1424", "法兰西（苏格兰人）・1419〜1424年"),
    ("スコットランド軍を率いてフランスに渡り、ボージェで勝って大元帥となった。ヴェルヌイユで斃れた", "He brought a Scottish army to France, won at Baugé and became Constable; he fell at Verneuil", "率苏格兰军渡海来法，在博热获胜而出任大元帅，战死于韦尔讷伊"),
    ("ラ・イール", "La Hire", "拉海尔"), ("フランス・1418〜1443年", "France, 1418–1443", "法兰西・1418〜1443年"),
    ("「怒り」の名を持つ荒武者。ジャンヌとともにオルレアンとパテーで戦った", "The rough warrior nicknamed \"Wrath\"; he fought beside Joan at Orléans and Patay", "绰号“愤怒”的猛将，与贞德并肩奋战于奥尔良与帕提"),
    ("ジャン・ド・デュノワ", "Jean de Dunois", "让·德·迪努瓦"), ("フランス・1427〜1468年", "France, 1427–1468", "法兰西・1427〜1468年"),
    ("オルレアンの私生児。オルレアンを守り抜き、ノルマンディーとギュイエンヌを取り戻した", "The Bastard of Orléans, who held Orléans and won back Normandy and Guyenne", "奥尔良的私生子，坚守奥尔良，收复诺曼底与吉耶讷"),
    ("アルテュール・ド・リッシュモン", "Arthur de Richemont", "阿蒂尔·德·里什蒙"), ("フランス・1425〜1458年", "France, 1425–1458", "法兰西・1425〜1458年"),
    ("ブルターニュ公の弟、大元帥。パリを取り返し、フォルミニーで勝ちを決めた", "Brother of the Duke of Brittany and Constable of France; he retook Paris and clinched victory at Formigny", "布列塔尼公爵之弟、大元帅，收复巴黎，决胜福米尼"),
    ("ジャン・ポトン・ド・ザントライユ", "Jean Poton de Xaintrailles", "让·波通·德·桑特拉耶"), ("フランス・1418〜1461年", "France, 1418–1461", "法兰西・1418〜1461年"),
    ("ラ・イールの戦友。パテーでタルボットを捕らえた", "Comrade-in-arms of La Hire; he captured Talbot at Patay", "拉海尔的战友，在帕提俘获塔尔博特"),
    ("ジャン・ビュロー", "Jean Bureau", "让·比罗"), ("フランス・1436〜1463年", "France, 1436–1463", "法兰西・1436〜1463年"),
    ("王の砲兵長官。弟ガスパールとともに大砲でノルマンディーの城を次々と落とし、カスティヨンで勝った", "Master of the king's artillery; with his brother Gaspard he battered down the castles of Normandy one after another and won at Castillon", "国王的炮兵总管，与弟弟加斯帕尔以大炮接连攻陷诺曼底诸城，赢得卡斯蒂永之战"),
]

# 国ではない兵・道
TRADE_T = [
    ("スコットランド", "The Scots", "苏格兰"), ("「古い同盟」でフランスと結ぶ北の王国。国境を越えてイングランドの北を荒らす", "The northern kingdom bound to France by the Auld Alliance; it crosses the border to ravage the north of England", "以“古老同盟”与法兰西结盟的北方王国，越过边境蹂躏英格兰北部"),
    ("ジェノヴァ弩兵", "Genoese crossbowmen", "热那亚弩兵"), ("リグリアの海の共和国が送り出す、名高い傭兵弩兵", "The famous mercenary crossbowmen sent out by the sea republic of Liguria", "利古里亚海上共和国派出的著名雇佣弩兵"),
    ("ジェノヴァのガレー船", "Genoese galleys", "热那亚桨帆船"), ("フランス王が雇うイタリアの櫂船。ジブラルタルを回って海峡に現れ、兵を運び、敵の船を襲う", "Italian oared warships in the pay of the King of France; they come round Gibraltar into the Channel to carry troops and attack enemy ships", "法兰西国王雇佣的意大利桨船，绕过直布罗陀进入海峡，运送士兵并袭击敌船"),
    ("大傭兵団", "The Great Companies", "大雇佣兵团"), ("休戦で職を失った兵の群れ（グランド・カンパニー、後のエコルシュール）。村を荒らすが、金で雇える", "Bands of soldiers left idle by the truces (the Great Companies, later the Écorcheurs). They ravage villages, but can be hired with gold", "因休战而失业的士兵团伙（大雇佣兵团，后来的“剥皮者”）。蹂躏村庄，但可用金币雇佣"),
    ("カスティーリャのガレー船", "Castilian galleys", "卡斯蒂利亚桨帆船"), ("1369年、トラスタマラ家のエンリケ2世はフランスと結んだ。ラ・ロシェルの海戦でイングランドの艦隊を焼いた", "In 1369 Henry II of Trastámara allied with France; his galleys burned the English fleet at La Rochelle", "1369年特拉斯塔马拉家的恩里克二世与法兰西结盟，其桨帆船在拉罗谢尔海战中焚毁英格兰舰队"),
    ("羊毛の道（フランドル）", "Wool Road (Flanders)", "羊毛之路（佛兰德斯）"), ("アントウェルペンからヘント・ブルッヘ・リールへ。イングランドの羊毛が織物になる", "From Antwerp to Ghent, Bruges and Lille, where English wool becomes cloth", "从安特卫普前往根特、布鲁日、里尔，英格兰羊毛在此织成布匹"),
    ("ワインの道（ガスコーニュ）", "Wine Road (Gascony)", "葡萄酒之路（加斯科涅）"), ("ボルドーのクラレットを積む商人。ギュイエンヌの富の源", "Merchants shipping the claret of Bordeaux — the source of Guyenne's wealth", "运载波尔多红葡萄酒的商人，吉耶讷财富之源"),
    ("イタリア商人（ロンバルディア）", "Italian merchants (Lombardy)", "意大利商人（伦巴第）"), ("アルプスを越えてリヨン・シャンパーニュの市へ来るフィレンツェやミラノの商人", "Florentine and Milanese merchants crossing the Alps to the fairs of Lyon and Champagne", "翻越阿尔卑斯山前往里昂与香槟集市的佛罗伦萨、米兰商人"),
    ("ハンザ商人（スティールヤード）", "Hanseatic merchants (the Steelyard)", "汉萨商人（钢院）"), ("ロンドンのスティールヤードに商館を持つドイツの商人", "German merchants with their hall at the Steelyard in London", "在伦敦钢院设有商馆的德意志商人"),
]

# チュートリアル（新しく書いた段だけ。変わらない段は今の辞書から引き継ぐ）
TUT_T = [
    ("ようこそ、フランス王よ", "Welcome, King of France", "欢迎你，法兰西国王"),
    ("▶ やってみよう：右下の「民」ボタン（Q）を押して、民を1人訓練してください。\n\n民・兵の数には上限があります。上限は街と村が増えると上がります。", "▶ Try it: press the Worker button (Q) at the bottom right to train one worker.\n\nThere is a limit on workers and soldiers. It rises as you hold more towns and villages.", "▶ 试试看：按右下角的“民”按钮（Q），训练一名民。\n\n民与兵的数量有上限。城镇与村庄越多，上限越高。"),
    ("兵営を選ぶと兵を訓練できます。作れる兵は国ごとに違います。フランスなら\n・槍兵：騎兵に強い\n・弩兵：遠くから鎧を射抜く\n・フランス騎士：弓兵に強い重い突撃兵\n・年代が進むと大砲・勅令隊・フラン・アルシェ\n\n▶ やってみよう：兵営を選び、どれか1つ訓練してください。", "Select the barracks to train soldiers. Each kingdom fields different troops. For France:\n• Spearmen: strong against cavalry\n• Crossbowmen: pierce armour from range\n• French knights: heavy shock troops, strong against archers\n• Later: bombards, the Compagnies d'ordonnance and the francs-archers\n\n▶ Try it: select the barracks and train any one unit.", "选中兵营即可训练士兵。各国可训练的兵种不同。以法兰西为例：\n・枪兵：克制骑兵\n・弩兵：远距离射穿铠甲\n・法兰西骑士：克制弓兵的重装突击兵\n・随着年代推进：射石炮、敕令骑士团、自由弓兵\n\n▶ 试试看：选中兵营，训练任意一个单位。"),
    ("イングランドとは最初から戦っています。休戦や和睦、捕らえた英雄の身代金の請求はここで行います。\n\n▶ やってみよう：右上の「外交」ボタン（F2）を押して、外交画面を開いてください。", "You are at war with England from the start. Truces, peace and ransom demands for captured heroes are handled here.\n\n▶ Try it: press the Diplomacy button (F2) at the top right to open the Diplomacy screen.", "与英格兰的战争从一开始就已打响。休战、议和以及索要被俘英雄的赎金都在这里进行。\n\n▶ 试试看：按右上角的“外交”按钮（F2），打开外交界面。"),
    ("英雄と身代金", "Heroes and ransom", "英雄与赎金"),
    ("デュ・ゲクラン、ジャンヌ・ダルク、黒太子など、実在の英雄が史実の年に現れます。英雄のそばの兵は奮い立ちますが、英雄は史実の没年を越えて生きられません。\n\n戦場で討たれた英雄は捕らえられ、捕らえた国は身代金を請求できます。ジャンヌ・ダルクだけは身代金では救えません。", "Real heroes — Du Guesclin, Joan of Arc, the Black Prince and others — appear in their historical years. Troops near a hero fight harder, but no hero can live past the year of their historical death.\n\nA hero defeated in battle is captured, and the captor may demand a ransom. Only Joan of Arc cannot be ransomed.", "迪盖克兰、贞德、黑太子等真实英雄会在史实年份登场。英雄身边的士兵士气高昂，但英雄无法活过史实中的卒年。\n\n在战场上被击败的英雄会被俘，俘获方可索要赎金。唯有贞德无法用赎金赎回。"),
    ("村を建てる", "Building villages", "建造村庄"),
    ("街は新しく築けません。主要な街は最初からあり、攻め落とすか、諸侯の寝返りで手に入れます。\n\n民で「村」ボタン（E）を押すと村を建てられます。村は糧を生み、資源の運び先になり、民と兵の上限を上げます。ただし敵に壊されると略奪されます。", "Towns cannot be founded: the great towns exist from the start, and you win them by siege or when their lords change sides.\n\nPress the Village button (E) with a worker to build a village. Villages produce food, serve as drop-off points and raise your worker and soldier limits — but an enemy who destroys one plunders it.", "城镇无法新建：主要城镇一开始就已存在，只能通过攻城或诸侯倒戈来取得。\n\n用民按“村”按钮（E）即可建造村庄。村庄产粮、可作为资源的运送点，并提高民与兵的上限——但被敌人摧毁时会遭到劫掠。"),
    ("船と略奪", "Ships and plunder", "船与劫掠"),
    ("海峡を渡るには船が要ります。イングランドは港でコグ船を造れますが、フランスは船を造れないので、交易画面（F4）でジェノヴァのガレー船を雇います。海の向こうへ移動を命じれば、船が自動で運びます。\n\n敵の村を攻めて壊すと略奪になり、糧・木・鉄・金貨を奪えます。諸侯の寝返りや同盟で、街が丸ごと相手に付くこともあります。", "Crossing the Channel needs ships. England builds cogs at a Harbor, but France cannot build ships and hires Genoese galleys on the Trade screen (F4). Order troops across the sea and the ships ferry them automatically.\n\nAttack and destroy an enemy village to plunder it for food, wood, iron and gold. When lords change sides or make alliances, whole towns may go over to the other king.", "渡过海峡需要船。英格兰可在港口建造柯克船，法兰西无法造船，需在贸易界面（F4）雇佣热那亚桨帆船。命令部队前往海对岸，船只就会自动运送。\n\n攻击并摧毁敌方村庄即为劫掠，可夺取粮、木、铁与金币。诸侯倒戈或结盟时，整座城镇也可能倒向对方国王。"),
    ("目的：史実で百年戦争が終わった1453年のうちに、フランスからイングランド人を追い出すこと（イングランドならフランスを占領し尽くすこと）。決着がつかなければ、ジャンヌ・ダルクの亡霊が天使を引き連れて両国を滅ぼします。\n\n困ったら F1 でヘルプ。文字が小さいときは Ctrl＋＋ で拡大、F11 で全画面にできます。\n\n「このまま続ける」で、この国のまま遊べます（イングランドからも攻められるようになります）。", "Goal: drive the English out of France by the end of 1453, the year the Hundred Years' War historically ended (as England: conquer all of France). If neither side wins, the ghost of Joan of Arc leads a host of angels to destroy both kingdoms.\n\nIf you get stuck, press F1 for Help. If the text is small, press Ctrl++ to enlarge it, or F11 for fullscreen.\n\nKeep playing to continue with this kingdom (England may now attack you).", "目标：在史实中百年战争结束的1453年之内，把英格兰人逐出法兰西（若为英格兰，则占领法兰西全境）。若无法分出胜负，贞德的亡灵将率领天使毁灭两国。\n\n遇到困难请按 F1 查看帮助。文字太小时可按 Ctrl＋＋ 放大，按 F11 全屏。\n\n选择“继续游玩”即可以此国继续游戏（之后英格兰也会来进攻）。"),
]


def rs(s):
    return '"' + s.replace('\\', '\\\\').replace('"', '\\"').replace('\n', '\\n') + '"'


def main():
    root = '.'
    # ---- 年表
    E.sort(key=lambda e: e[0])  # 安定ソート（同じ年は書いた順）
    lines = []
    for ad, n, kind, (tja, _, _), (xja, _, _), fx in E:
        lines.append(f'    ev!({ad}, {n}, {kind}, {rs(tja)}, {rs(xja)}, [{fx}]),')
    hp = f'{root}/src/world/history.rs'
    s = open(hp).read()
    a = s.index('pub const HISTORY: &[HEvent] = &[')
    b = s.index('];', a)
    s = s[:a] + 'pub const HISTORY: &[HEvent] = &[\n' + '\n'.join(lines) + '\n' + s[b:]
    open(hp, 'w').write(s)

    # ---- 辞書
    old = open(f'{root}/src/i18n.rs').read()
    old_dict = {}
    for m in re.finditer(r'\(\s*("(?:[^"\\]|\\.)*")\s*,\s*("(?:[^"\\]|\\.)*")\s*,\s*("(?:[^"\\]|\\.)*")\s*\)', old):
        old_dict[m.group(1)] = (m.group(2), m.group(3))

    entries = []  # (ja, en, zh) を Rust リテラルで
    seen = set()

    def add(ja, en, zh):
        k = rs(ja)
        if k in seen:
            return
        seen.add(k)
        entries.append((k, rs(en), rs(zh)))

    for ja, en, zh in OTHER:
        add(ja, en, zh)
    for ja, (en, zh) in HOUSE.items():
        add(ja, en, zh)
    for ja, (en, zh) in FIG.items():
        add(ja, en, zh)
    for _, _, _, t, x, _ in E:
        add(*t)
        add(*x)
    for ja, en, zh in UNITS + TRAITS_T + HERO_T + TRADE_T + TUT_T:
        add(ja, en, zh)
    # チュートリアルの変わらない段は今の辞書から
    tut = open(f'{root}/src/game/tutorial.rs').read()
    for m in re.finditer(r'(?:title|body): ("(?:[^"\\]|\\.)*")', tut):
        k = m.group(1)
        if k not in seen:
            if k in old_dict:
                seen.add(k)
                entries.append((k, old_dict[k][0], old_dict[k][1]))
            else:
                print('チュートリアルの訳が無い:', k[:60], file=sys.stderr)

    out = ['//! データの訳（日本語 → 英語・中国語）。tools/gendata.py が生成する',
           '//!',
           '//! 国名・国の説明・街・諸侯・君主家・人物・史実の出来事・国の特徴・兵種・英雄・国ではない兵・遠方の道・チュートリアル。',
           '//! キーはデータに書かれた日本語そのもの。`lang::tx` が引く。',
           '',
           'pub const DICT: &[(&str, &str, &str)] = &[']
    for k, en, zh in entries:
        out.append(f'    ({k}, {en}, {zh}),')
    out.append('];')
    out.append(TESTS)
    open(f'{root}/src/i18n.rs', 'w').write('\n'.join(out) + '\n')
    print('events', len(E), 'dict', len(entries))


TESTS = '''
#[cfg(test)]
mod tests {
    use crate::geo::{LORDS, NATIONS, TOWNS};
    use crate::lang::has;
    use crate::world::*;

    /// データに出てくる日本語はすべて辞書に訳がある
    #[test]
    fn dict_covers_data() {
        let mut all: Vec<&'static str> = Vec::new();
        let mut need = |s: &'static str| all.push(s);
        for d in NATIONS.iter() {
            need(d.name);
            need(d.desc);
        }
        need("部族");
        need("天の軍勢");
        for h in INITIAL_HOUSE {
            need(h);
        }
        for t in TOWNS {
            need(t.name);
        }
        for l in LORDS {
            need(l.name);
            need(l.desc);
        }
        for e in HISTORY {
            need(e.title);
            need(e.text);
            for fx in e.fx {
                match fx {
                    Fx::Figure(n, ..) | Fx::Dismiss(n) | Fx::House(n) => need(n),
                    _ => {}
                }
            }
        }
        for t in TRAITS {
            need(t.name);
            need(t.desc);
        }
        for u in UNIT_DEFS {
            need(u.name);
            need(u.desc);
        }
        for h in HEROES {
            need(h.name);
            need(h.era);
            need(h.desc);
        }
        for t in TRIBES {
            need(t.name);
            need(t.desc);
        }
        for r in ROUTES {
            need(r.name);
            need(r.desc);
        }
        let mut missing: Vec<&str> = all.into_iter().filter(|s| !has(s)).collect();
        missing.dedup();
        assert!(missing.is_empty(), "辞書に無い：{:?}", missing);
    }
}'''

if __name__ == '__main__':
    main()
