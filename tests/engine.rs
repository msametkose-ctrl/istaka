//! Dağıtım ve oyun akışı testleri.

use istaka::bots::{GreedyBot, RandomBot};
use istaka::game::{deal, play_hand, Action, Agent, Deal, Outcome, View};
use istaka::rng::Rng;
use istaka::rules::Rules;
use istaka::tile::{okey_of, parse, FALSE_OKEY, KINDS};

#[test]
fn deal_counts() {
    let mut rng = Rng::new(7);
    for starter in 0..4 {
        for _ in 0..500 {
            let d = deal(&mut rng, starter);
            for s in 0..4 {
                assert_eq!(d.hands[s].len(), if s == starter { 15 } else { 14 });
            }
            assert_eq!(d.stock.len(), 48);
            assert_ne!(d.indicator, FALSE_OKEY);
            let mut seen = [0u8; KINDS];
            for t in d.hands.iter().flatten().chain(d.stock.iter()).chain([d.indicator].iter()) {
                seen[*t as usize] += 1;
            }
            assert!(seen.iter().all(|&c| c == 2), "her türden tam 2 taş olmalı");
        }
    }
}

#[test]
fn same_seed_same_deal() {
    let a = deal(&mut Rng::new(42), 1);
    let b = deal(&mut Rng::new(42), 1);
    assert_eq!(a.hands, b.hands);
    assert_eq!(a.stock, b.stock);
}

fn play_many<F: FnMut(u64) -> [Box<dyn Agent>; 4]>(n: u64, mut make: F) {
    let rules = Rules::default();
    let mut rng = Rng::new(99);
    for k in 0..n {
        let mut bots = make(k);
        let [a, b, c, d] = &mut bots;
        let mut agents: [&mut dyn Agent; 4] = [a.as_mut(), b.as_mut(), c.as_mut(), d.as_mut()];
        let res = play_hand(deal(&mut rng, (k % 4) as usize), &mut agents, &rules).expect("kural dışı hamle");
        assert_eq!(res.scores.iter().sum::<i32>(), 0, "el puanı sıfır toplamlı olmalı");
        if let Outcome::Win { points, .. } = res.outcome {
            assert!([2, 4, 8].contains(&points));
        }
    }
}

#[test]
fn random_bots_finish_hands() {
    play_many(2000, |k| {
        [
            Box::new(RandomBot::new(k * 4)),
            Box::new(RandomBot::new(k * 4 + 1)),
            Box::new(RandomBot::new(k * 4 + 2)),
            Box::new(RandomBot::new(k * 4 + 3)),
        ]
    });
}

#[test]
fn greedy_bots_finish_hands() {
    play_many(200, |_| [Box::new(GreedyBot), Box::new(GreedyBot), Box::new(GreedyBot), Box::new(GreedyBot)]);
}

#[test]
fn strong_bots_finish_hands() {
    use istaka::strong::StrongBot;
    play_many(200, |_| {
        [Box::new(StrongBot::default()), Box::new(GreedyBot), Box::new(StrongBot::default()), Box::new(RandomBot::new(5))]
    });
}

/// Duplicate karşılaştırma tekrarlanabilir: aynı tohumlar, iş parçacığı sayısından
/// bağımsız olarak aynı sonucu verir; aynı bot kendisine karşı tam 0 farkla biter.
#[test]
fn duel_is_reproducible() {
    use istaka::duel::{run, summarize};
    let rules = Rules::default();
    let a = run("strong", "greedy", 500, 60, 1, &rules);
    let b = run("strong", "greedy", 500, 60, 7, &rules);
    assert_eq!(a, b);
    let same = summarize(&run("greedy", "greedy", 0, 40, 3, &rules));
    assert_eq!(same.mean, 0.0);
    assert_eq!(same.a_wins, same.b_wins);
}

/// Hep desteden çeker ve elindeki ilk taşı atar; bitmeyi asla denemez.
struct Passive;
impl Agent for Passive {
    fn take_offer(&mut self, _v: &View) -> bool {
        false
    }
    fn act(&mut self, v: &View) -> Action {
        Action::Discard(v.hand[0])
    }
}

fn kinds(s: &str) -> Vec<u8> {
    s.split_whitespace().map(|t| parse(t).unwrap()).collect()
}

#[test]
fn exhausted_stock_is_a_draw() {
    let rules = Rules::default();
    let d = deal(&mut Rng::new(5), 0);
    let mut p = [Passive, Passive, Passive, Passive];
    let [a, b, c, e] = &mut p;
    let mut agents: [&mut dyn Agent; 4] = [a, b, c, e];
    let res = play_hand(d, &mut agents, &rules).unwrap();
    // Pasif oyuncular hiç bitmez: 1 başlangıç atışı + 48 çekiş.
    assert_eq!(res.outcome, Outcome::Draw);
    assert_eq!(res.turns, 49);
    assert_eq!(res.scores, [0; 4]);
}

#[test]
fn exhausted_stock_last_discard_can_win() {
    // Koltuk 1, son atıkla bitecek bir el bekliyor; deste boş.
    let rules = Rules::default();
    let indicator = parse("S13").unwrap();
    let winner = kinds("K1 K2 K3 K4 M4 M5 M6 S7 S8 S9 Y10 Y11 Y12 K9");
    let d = Deal {
        hands: [
            kinds("Y13 K5 K6 K7 K8 K10 K11 K12 K13 M1 M2 M3 M7 M8 M9"),
            winner,
            kinds("Y1 Y2 Y3 Y4 Y5 Y6 Y7 Y8 Y9 S10 S11 S12 M10 M11"),
            kinds("Y1 Y2 Y3 Y4 Y5 Y6 Y7 Y8 Y9 S10 S11 S12 M10 M11"),
        ],
        indicator,
        stock: vec![],
        starter: 0,
    };
    // Koltuk 0 ilk atışını Y13 yapar.
    struct First;
    impl Agent for First {
        fn take_offer(&mut self, _v: &View) -> bool {
            false
        }
        fn act(&mut self, _v: &View) -> Action {
            Action::Discard(parse("Y13").unwrap())
        }
    }
    let (mut a, mut b, mut c, mut e) = (First, Passive, Passive, Passive);
    let mut agents: [&mut dyn Agent; 4] = [&mut a, &mut b, &mut c, &mut e];
    let res = play_hand(d, &mut agents, &rules).unwrap();
    match res.outcome {
        Outcome::Win { seat, points, .. } => {
            assert_eq!(seat, 1);
            assert_eq!(points, 2);
        }
        o => panic!("koltuk 1 bitmeliydi: {o:?}"),
    }
    assert_eq!(okey_of(indicator), parse("S1").unwrap());
}

#[test]
fn rediscard_rule_is_enforced_when_off() {
    let rules = Rules { rediscard_taken: false, ..Rules::default() };
    // Her zaman yerden alıp aldığını geri atan bot: kural kapalıyken hata vermeli.
    struct Echo;
    impl Agent for Echo {
        fn take_offer(&mut self, _v: &View) -> bool {
            true
        }
        fn act(&mut self, v: &View) -> Action {
            Action::Discard(v.taken.unwrap_or(v.hand[0]))
        }
    }
    let mut rng = Rng::new(3);
    let mut bad = 0;
    for _ in 0..50 {
        let (mut a, mut b, mut c, mut e) = (Echo, Echo, Echo, Echo);
        let mut agents: [&mut dyn Agent; 4] = [&mut a, &mut b, &mut c, &mut e];
        if play_hand(deal(&mut rng, 0), &mut agents, &rules).is_err() {
            bad += 1;
        }
    }
    assert!(bad > 0);
}
