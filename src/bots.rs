//! Taban botlar: rastgele ve açgözlü.

use crate::game::{Action, Agent, View};
use crate::rng::Rng;
use crate::solver::{best_win15, max_melded, pack, Packed};
use crate::tile::Kind;

fn forbidden(v: &View) -> Option<Kind> {
    if v.rules.rediscard_taken {
        None
    } else {
        v.taken
    }
}

/// Desteden çeker, bitebiliyorsa biter, yoksa rastgele atar.
pub struct RandomBot {
    rng: Rng,
}

impl RandomBot {
    pub fn new(seed: u64) -> Self {
        RandomBot { rng: Rng::new(seed) }
    }
}

impl Agent for RandomBot {
    fn take_offer(&mut self, _v: &View) -> bool {
        false
    }

    fn act(&mut self, v: &View) -> Action {
        if let Some(w) = best_win15(v.hand, v.okey, v.rules, forbidden(v)) {
            return Action::Win(w.discard);
        }
        loop {
            let t = v.hand[self.rng.below(v.hand.len())];
            if Some(t) != forbidden(v) {
                return Action::Discard(t);
            }
        }
    }
}

/// Elin perlere girebilen taş sayısını (ya da çift yolunu) en büyük tutan atışı seçer.
/// Yerden taşı, ancak bu sayıyı artırıyorsa alır. Rakipleri hiç dikkate almaz.
pub struct GreedyBot;

fn score(p: &Packed, rules: &crate::rules::Rules) -> u8 {
    let pairs = p.counts.iter().filter(|&&c| c == 2).count() as u8;
    let singles = p.counts.iter().filter(|&&c| c == 1).count() as u8;
    let pair_route = 2 * (pairs + p.jokers.min(singles));
    max_melded(p, rules).max(pair_route)
}

/// 15 taştan en iyi atış ve ardından kalan elin puanı.
fn best_discard(hand: &[Kind], okey: Kind, rules: &crate::rules::Rules, forbid: Option<Kind>) -> (Kind, u8) {
    let full = pack(hand, okey);
    let mut best = (hand[0], 0u8);
    let mut first = true;
    for (idx, &t) in hand.iter().enumerate() {
        if Some(t) == forbid || hand[..idx].contains(&t) {
            continue;
        }
        let mut p = full;
        let single = [t];
        let one = pack(&single, okey);
        if one.jokers == 1 {
            p.jokers -= 1;
        } else {
            let f = one.counts.iter().position(|&c| c > 0).unwrap();
            p.counts[f] -= 1;
        }
        let s = score(&p, rules);
        // Eşitlikte okeyi atmamak için okey en sona kalır.
        if first || s > best.1 || (s == best.1 && best.0 == okey) {
            best = (t, s);
            first = false;
        }
    }
    best
}

impl Agent for GreedyBot {
    fn take_offer(&mut self, v: &View) -> bool {
        let t = match v.offer {
            Some(t) => t,
            None => return false,
        };
        let mut h = v.hand.to_vec();
        h.push(t);
        let forbid = if v.rules.rediscard_taken { None } else { Some(t) };
        if best_win15(&h, v.okey, v.rules, forbid).is_some() {
            return true;
        }
        let now = score(&pack(v.hand, v.okey), v.rules);
        let (d, after) = best_discard(&h, v.okey, v.rules, forbid);
        d != t && after > now
    }

    fn act(&mut self, v: &View) -> Action {
        if let Some(w) = best_win15(v.hand, v.okey, v.rules, forbidden(v)) {
            return Action::Win(w.discard);
        }
        Action::Discard(best_discard(v.hand, v.okey, v.rules, forbidden(v)).0)
    }
}
