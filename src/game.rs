//! Tek elin durum makinesi: dağıtım, çekme, atma, bitiş ve puan.

use crate::rng::Rng;
use crate::rules::Rules;
use crate::solver::{finish14, points};
use crate::tile::{okey_of, Kind, FALSE_OKEY, KINDS};

#[derive(Clone, Debug)]
pub struct Deal {
    pub hands: [Vec<Kind>; 4],
    pub indicator: Kind,
    /// Çekme destesi; sıradaki taş sondadır.
    pub stock: Vec<Kind>,
    pub starter: usize,
}

/// 106 taşı karıştırır; başlayana 15, diğerlerine 14 taş; kalanlardan sahte olmayan
/// ilk taş gösterge, geri kalan 48 taş deste olur.
pub fn deal(rng: &mut Rng, starter: usize) -> Deal {
    let mut all: Vec<Kind> = (0..KINDS as u8).flat_map(|k| [k, k]).collect();
    rng.shuffle(&mut all);
    let mut it = all.into_iter();
    let mut hands: [Vec<Kind>; 4] = Default::default();
    for off in 0..4 {
        let seat = (starter + off) % 4;
        let n = if off == 0 { 15 } else { 14 };
        hands[seat] = it.by_ref().take(n).collect();
    }
    let mut rest: Vec<Kind> = it.collect();
    let pos = rest.iter().position(|&t| t != FALSE_OKEY).expect("49 taşın hepsi sahte olamaz");
    let indicator = rest.remove(pos);
    Deal { hands, indicator, stock: rest, starter }
}

/// Bir oyuncunun karar anında gördükleri.
pub struct View<'a> {
    pub seat: usize,
    pub hand: &'a [Kind],
    pub indicator: Kind,
    pub okey: Kind,
    /// Soldaki oyuncunun alınabilir son atığı.
    pub offer: Option<Kind>,
    /// Bu tur yerden alınan taş (geri atma kuralı için).
    pub taken: Option<Kind>,
    pub stock_left: usize,
    /// Her koltuğun attığı taşlar, sırasıyla (alınanlar listeden çıkar).
    pub discards: &'a [Vec<Kind>; 4],
    pub rules: &'a Rules,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Discard(Kind),
    /// Verilen taşı ortaya atarak bitir.
    Win(Kind),
}

pub trait Agent {
    /// Soldaki oyuncunun attığı taş alınsın mı (yoksa desteden çekilir)?
    fn take_offer(&mut self, v: &View) -> bool;
    /// 15 taşla: at ya da bit.
    fn act(&mut self, v: &View) -> Action;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Win { seat: usize, pairs: bool, okey_discard: bool, points: u8 },
    /// Deste bitti, kimse bitemedi.
    Draw,
}

#[derive(Clone, Debug)]
pub struct HandResult {
    pub outcome: Outcome,
    /// Atış sayısı (bitiş atışı dahil).
    pub turns: u32,
    /// Sıfır toplamlı el puanı: kazanan +3M, her kaybeden −M; gösterge +3/−1.
    pub scores: [i32; 4],
    pub indicator_shown: [bool; 4],
}

#[derive(Debug)]
pub enum Illegal {
    TileNotInHand(usize, Kind),
    RediscardTaken(usize, Kind),
    FalseWin(usize, Kind),
}

pub fn play_hand(d: Deal, agents: &mut [&mut dyn Agent; 4], rules: &Rules) -> Result<HandResult, Illegal> {
    let Deal { mut hands, indicator, mut stock, starter } = d;
    let okey = okey_of(indicator);
    let mut discards: [Vec<Kind>; 4] = Default::default();
    let mut scores = [0i32; 4];
    let mut shown = [false; 4];
    let mut first_turn = [true; 4];
    let mut seat = starter;
    let mut turns = 0u32;
    let mut offer: Option<Kind> = None;

    loop {
        let left = (seat + 3) % 4;
        if rules.indicator_show && first_turn[seat] && hands[seat].contains(&indicator) {
            shown[seat] = true;
            for s in 0..4 {
                scores[s] += if s == seat { 3 } else { -1 };
            }
        }
        let mut taken = None;
        if !(seat == starter && turns == 0) {
            if stock.is_empty() {
                // Yalnızca son atıkla bitebilen bitebilir; yoksa el puansız biter.
                if let Some(t) = offer {
                    hands[seat].push(t);
                    let w = crate::solver::best_win15(&hands[seat], okey, rules, if rules.rediscard_taken { None } else { Some(t) });
                    if let Some(w) = w {
                        discards[left].pop();
                        turns += 1;
                        return Ok(finish(seat, w.pairs, w.okey_discard, turns, scores, shown));
                    }
                    hands[seat].pop();
                }
                return Ok(HandResult { outcome: Outcome::Draw, turns, scores, indicator_shown: shown });
            }
            let v = View {
                seat,
                hand: &hands[seat],
                indicator,
                okey,
                offer,
                taken: None,
                stock_left: stock.len(),
                discards: &discards,
                rules,
            };
            let tile = match offer {
                Some(t) if agents[seat].take_offer(&v) => {
                    discards[left].pop();
                    taken = Some(t);
                    t
                }
                _ => stock.pop().unwrap(),
            };
            hands[seat].push(tile);
        }
        first_turn[seat] = false;

        let v = View {
            seat,
            hand: &hands[seat],
            indicator,
            okey,
            offer: None,
            taken,
            stock_left: stock.len(),
            discards: &discards,
            rules,
        };
        let action = agents[seat].act(&v);
        turns += 1;
        let t = match action {
            Action::Discard(t) | Action::Win(t) => t,
        };
        let idx = hands[seat].iter().position(|&x| x == t).ok_or(Illegal::TileNotInHand(seat, t))?;
        if !rules.rediscard_taken && taken == Some(t) && hands[seat].iter().filter(|&&x| x == t).count() == 1 {
            return Err(Illegal::RediscardTaken(seat, t));
        }
        hands[seat].swap_remove(idx);
        if let Action::Win(_) = action {
            let f = finish14(&hands[seat], okey, rules);
            if !f.normal && !f.pairs {
                return Err(Illegal::FalseWin(seat, t));
            }
            return Ok(finish(seat, f.pairs, t == okey, turns, scores, shown));
        }
        discards[seat].push(t);
        offer = Some(t);
        seat = (seat + 1) % 4;
    }
}

fn finish(seat: usize, pairs: bool, okey_discard: bool, turns: u32, mut scores: [i32; 4], shown: [bool; 4]) -> HandResult {
    let m = points(pairs, okey_discard);
    for s in 0..4 {
        scores[s] += if s == seat { 3 * m as i32 } else { -(m as i32) };
    }
    HandResult {
        outcome: Outcome::Win { seat, pairs, okey_discard, points: m },
        turns,
        scores,
        indicator_shown: shown,
    }
}
