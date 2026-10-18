//! Duplicate karşılaştırma: aynı tohumlu dağıtım iki oturma düzeninde oynanır,
//! [A, B, A, B] ve [B, A, B, A]. Böylece taş şansı iki tarafa eşit dağılır ve fark
//! yalnızca oyundan gelir.

use crate::bots::{GreedyBot, RandomBot};
use crate::strong::StrongBot;
use crate::game::{deal, play_hand, Agent, Outcome};
use crate::rng::Rng;
use crate::rules::Rules;

pub const BOTS: [&str; 3] = ["random", "greedy", "strong"];

/// Adıyla bot; rastgele bot tohumla belirlenir.
pub fn make_bot(name: &str, seed: u64) -> Option<Box<dyn Agent>> {
    Some(match name {
        "random" => Box::new(RandomBot::new(seed)),
        "greedy" => Box::new(GreedyBot),
        "strong" => Box::new(StrongBot::default()),
        _ => return None,
    })
}

/// Bir tohumun iki elinin sonucu.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SeedResult {
    /// İki elin ortalaması: A'nın oyuncu başına el puanı eksi B'ninki.
    pub diff: f64,
    pub a_wins: u8,
    pub b_wins: u8,
    pub draws: u8,
}

/// `seed` tohumlu dağıtımı iki düzende oynatır.
pub fn play_seed(a: &str, b: &str, seed: u64, rules: &Rules) -> SeedResult {
    let mut r = SeedResult::default();
    for swap in [false, true] {
        let names = if swap { [b, a, b, a] } else { [a, b, a, b] };
        let mut bots: Vec<Box<dyn Agent>> =
            (0..4).map(|s| make_bot(names[s], seed.wrapping_mul(8) + s as u64 + 4 * swap as u64).expect("bilinmeyen bot")).collect();
        let [p, q, x, y] = &mut bots[..] else { unreachable!() };
        let mut agents: [&mut dyn Agent; 4] = [p.as_mut(), q.as_mut(), x.as_mut(), y.as_mut()];
        let d = deal(&mut Rng::new(0xD0_0000 + seed), (seed % 4) as usize);
        let res = play_hand(d, &mut agents, rules).expect("botlar kural dışı hamle yaptı");
        // A'nın koltukları: düzende 0 ve 2, takasta 1 ve 3.
        let (sa, sb) = if swap { (1, 0) } else { (0, 1) };
        let pa = (res.scores[sa] + res.scores[sa + 2]) as f64 / 2.0;
        let pb = (res.scores[sb] + res.scores[sb + 2]) as f64 / 2.0;
        r.diff += (pa - pb) / 2.0;
        match res.outcome {
            Outcome::Draw => r.draws += 1,
            Outcome::Win { seat, .. } if seat % 2 == sa % 2 => r.a_wins += 1,
            Outcome::Win { .. } => r.b_wins += 1,
        }
    }
    r
}

/// `first..first + n` tohumlarını `threads` iş parçacığına bölerek oynatır; sonuç tohum
/// sırasıyla döner, iş parçacığı sayısından bağımsızdır.
pub fn run(a: &str, b: &str, first: u64, n: u64, threads: usize, rules: &Rules) -> Vec<SeedResult> {
    let threads = threads.max(1);
    let handles: Vec<_> = (0..threads)
        .map(|t| {
            let (a, b, rules) = (a.to_string(), b.to_string(), *rules);
            std::thread::spawn(move || {
                (t as u64..n).step_by(threads).map(|s| (s, play_seed(&a, &b, first + s, &rules))).collect::<Vec<_>>()
            })
        })
        .collect();
    let mut out = vec![SeedResult::default(); n as usize];
    for h in handles {
        for (s, r) in h.join().unwrap() {
            out[s as usize] = r;
        }
    }
    out
}

#[derive(Clone, Copy, Debug)]
pub struct Summary {
    pub seeds: u64,
    pub mean: f64,
    /// Eşleştirilmiş %95 güven aralığı (tohum başına fark üzerinden).
    pub ci: (f64, f64),
    pub hands: u64,
    pub a_wins: u64,
    pub b_wins: u64,
    pub draws: u64,
}

pub fn summarize(rs: &[SeedResult]) -> Summary {
    let n = rs.len() as f64;
    let mean = rs.iter().map(|r| r.diff).sum::<f64>() / n;
    let var = rs.iter().map(|r| (r.diff - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    let half = 1.96 * (var / n).sqrt();
    let sum = |f: fn(&SeedResult) -> u8| rs.iter().map(|r| f(r) as u64).sum::<u64>();
    Summary {
        seeds: rs.len() as u64,
        mean,
        ci: (mean - half, mean + half),
        hands: 2 * rs.len() as u64,
        a_wins: sum(|r| r.a_wins),
        b_wins: sum(|r| r.b_wins),
        draws: sum(|r| r.draws),
    }
}

/// Oranın %95 Wilson aralığı.
pub fn wilson(k: u64, n: u64) -> (f64, f64) {
    if n == 0 {
        return (0.0, 1.0);
    }
    let (z, n, p) = (1.96f64, n as f64, k as f64 / n as f64);
    let d = 1.0 + z * z / n;
    let c = p + z * z / (2.0 * n);
    let h = z * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt();
    ((c - h) / d, (c + h) / d)
}
