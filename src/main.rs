use istaka::bots::{GreedyBot, RandomBot};
use istaka::game::{deal, play_hand, Agent, Outcome};
use istaka::rng::Rng;
use istaka::rules::Rules;
use std::time::{Duration, Instant};

const USAGE: &str = "kullanım:
  istaka bench [random|greedy] [saniye=10] [iş_parçacığı=tümü]
  istaka ornek [tohum=1]          bir eli greedy botlarla oynatıp yazar
  istaka duel A B [tohum=20000] [iş_parçacığı=tümü] [ilk_tohum=0]
                                  duplicate karşılaştırma (botlar: random, greedy, strong)";

#[derive(Default, Clone, Copy)]
struct Stats {
    hands: u64,
    draws: u64,
    normal: u64,
    pairs: u64,
    okey: u64,
    turns: u64,
    seat_wins: [u64; 4],
}

impl Stats {
    fn add(&mut self, o: &Stats) {
        self.hands += o.hands;
        self.draws += o.draws;
        self.normal += o.normal;
        self.pairs += o.pairs;
        self.okey += o.okey;
        self.turns += o.turns;
        for s in 0..4 {
            self.seat_wins[s] += o.seat_wins[s];
        }
    }
}

fn run(bot: &str, seed: u64, deadline: Instant) -> Stats {
    let rules = Rules::default();
    let mut rng = Rng::new(seed);
    let mut st = Stats::default();
    let mut r: Vec<RandomBot> = (0..4).map(|k| RandomBot::new(seed * 4 + k)).collect();
    let mut g = [GreedyBot, GreedyBot, GreedyBot, GreedyBot];
    while Instant::now() < deadline {
        for _ in 0..16 {
            let starter = (st.hands % 4) as usize;
            let d = deal(&mut rng, starter);
            let res = if bot == "greedy" {
                let [a, b, c, e] = &mut g;
                let mut agents: [&mut dyn Agent; 4] = [a, b, c, e];
                play_hand(d, &mut agents, &rules)
            } else {
                let [a, b, c, e] = &mut r[..] else { unreachable!() };
                let mut agents: [&mut dyn Agent; 4] = [a, b, c, e];
                play_hand(d, &mut agents, &rules)
            }
            .expect("botlar kural dışı hamle yaptı");
            st.hands += 1;
            st.turns += res.turns as u64;
            match res.outcome {
                Outcome::Draw => st.draws += 1,
                Outcome::Win { seat, pairs, okey_discard, .. } => {
                    st.seat_wins[(seat + 4 - starter) % 4] += 1;
                    if pairs {
                        st.pairs += 1;
                    } else {
                        st.normal += 1;
                    }
                    if okey_discard {
                        st.okey += 1;
                    }
                }
            }
        }
    }
    st
}

fn bench(args: &[String]) {
    let bot = args.first().map(String::as_str).unwrap_or("random").to_string();
    if bot != "random" && bot != "greedy" {
        eprintln!("{USAGE}");
        std::process::exit(2);
    }
    let secs: f64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(10.0);
    let threads: usize = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get()));
    let start = Instant::now();
    let deadline = start + Duration::from_secs_f64(secs);
    let handles: Vec<_> = (0..threads)
        .map(|t| {
            let bot = bot.clone();
            std::thread::spawn(move || run(&bot, 1000 + t as u64, deadline))
        })
        .collect();
    let mut st = Stats::default();
    for h in handles {
        st.add(&h.join().unwrap());
    }
    let el = start.elapsed().as_secs_f64();
    let n = st.hands as f64;
    let pct = |x: u64| 100.0 * x as f64 / n;
    println!("bot: {bot}, iş parçacığı: {threads}, süre: {el:.1} sn");
    println!("el: {}  ->  {:.0} el/sn ({:.0} el/sn/çekirdek)", st.hands, n / el, n / el / threads as f64);
    println!("ortalama atış/el: {:.1}", st.turns as f64 / n);
    println!(
        "sonuç: normal %{:.1}, çift %{:.1}, puansız (deste bitti) %{:.1}; okey atarak %{:.1}",
        pct(st.normal),
        pct(st.pairs),
        pct(st.draws),
        pct(st.okey)
    );
    let wins = (st.hands - st.draws) as f64;
    println!(
        "kazanan koltuk (başlayana göre 0..3): {}",
        st.seat_wins.iter().map(|&w| format!("%{:.1}", 100.0 * w as f64 / wins)).collect::<Vec<_>>().join(" ")
    );
}

fn ornek(args: &[String]) {
    let seed: u64 = args.first().and_then(|s| s.parse().ok()).unwrap_or(1);
    let rules = Rules::default();
    let d = deal(&mut Rng::new(seed), 0);
    println!("gösterge {}  okey {}", istaka::tile::name(d.indicator), istaka::tile::name(istaka::tile::okey_of(d.indicator)));
    for (s, h) in d.hands.iter().enumerate() {
        let mut h = h.clone();
        h.sort();
        println!("koltuk {s}: {}", istaka::tile::names(&h));
    }
    let mut g = [GreedyBot, GreedyBot, GreedyBot, GreedyBot];
    let [a, b, c, e] = &mut g;
    let mut agents: [&mut dyn Agent; 4] = [a, b, c, e];
    let res = play_hand(d, &mut agents, &rules).expect("kural dışı hamle");
    println!("{:?}, {} atış, puan {:?}", res.outcome, res.turns, res.scores);
}

fn duel(args: &[String]) {
    let (a, b) = match (args.first(), args.get(1)) {
        (Some(a), Some(b)) if [a, b].iter().all(|x| istaka::duel::BOTS.contains(&x.as_str())) => (a.as_str(), b.as_str()),
        _ => {
            eprintln!("{USAGE}");
            std::process::exit(2);
        }
    };
    let n: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(20_000);
    let threads: usize = args
        .get(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get()));
    let first: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
    let start = Instant::now();
    let rs = istaka::duel::run(a, b, first, n, threads, &Rules::default());
    let s = istaka::duel::summarize(&rs);
    let pct = |x: u64| 100.0 * x as f64 / s.hands as f64;
    let wil = |k: u64| {
        let (lo, hi) = istaka::duel::wilson(k, s.hands);
        format!("%{:.1} [%{:.1}, %{:.1}]", pct(k), 100.0 * lo, 100.0 * hi)
    };
    println!("{a} - {b}: {} tohum ({first}..), {} el (her tohum [A,B,A,B] ve [B,A,B,A]), {:.1} sn", s.seeds, s.hands, start.elapsed().as_secs_f64());
    println!(
        "el başına puan farkı ({a} - {b}, oyuncu başına): {:+.4}  %95 GA [{:+.4}, {:+.4}]",
        s.mean, s.ci.0, s.ci.1
    );
    println!("el kazanma: {a} {}, {b} {}, puansız {}", wil(s.a_wins), wil(s.b_wins), wil(s.draws));
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("bench") => bench(&args[1..]),
        Some("ornek") => ornek(&args[1..]),
        Some("duel") => duel(&args[1..]),
        _ => {
            eprintln!("{USAGE}");
            std::process::exit(2);
        }
    }
}
