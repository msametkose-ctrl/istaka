//! tests/vectors.txt içindeki kural vektörlerini çalıştırır.

use istaka::rules::Rules;
use istaka::solver::{best_win15, finish14};
use istaka::tile::{okey_of, parse};

fn expected_14(normal: bool, pairs: bool) -> &'static str {
    match (normal, pairs) {
        (true, true) => "IKISI",
        (true, false) => "NORMAL",
        (false, true) => "CIFT",
        (false, false) => "YOK",
    }
}

#[test]
fn rule_vectors() {
    let text = include_str!("vectors.txt");
    let mut failures = Vec::new();
    let mut count = 0;
    for (lineno, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (head, tiles) = line.split_once('|').unwrap_or_else(|| panic!("satır {}: '|' yok", lineno + 1));
        let mut words = head.split_whitespace();
        let id = words.next().unwrap();
        let want = words.next().unwrap();
        let mut rules = Rules::default();
        let mut indicator = None;
        for w in words {
            let (k, v) = w.split_once('=').unwrap_or_else(|| panic!("{id}: anlaşılmayan '{w}'"));
            match k {
                "gösterge" => indicator = parse(v),
                "wrap" => rules.wrap_13_1 = v == "1",
                "maxokey" => rules.max_okeys_per_meld = v.parse().unwrap(),
                _ => panic!("{id}: bilinmeyen ayar '{k}'"),
            }
        }
        let okey = okey_of(indicator.unwrap_or_else(|| panic!("{id}: gösterge yok")));
        let hand: Vec<u8> = tiles
            .split_whitespace()
            .map(|t| parse(t).unwrap_or_else(|| panic!("{id}: taş '{t}'")))
            .collect();
        let got = match hand.len() {
            14 => {
                let f = finish14(&hand, okey, &rules);
                expected_14(f.normal, f.pairs).to_string()
            }
            15 => match best_win15(&hand, okey, &rules, None) {
                None => "YOK".to_string(),
                Some(w) => format!("{}{}", if w.pairs { "CIFT" } else { "NORMAL" }, if w.okey_discard { "+OKEY" } else { "" }),
            },
            n => panic!("{id}: {n} taş"),
        };
        count += 1;
        if got != want {
            failures.push(format!("{id}: beklenen {want}, çıkan {got}"));
        }
    }
    assert!(count >= 30, "yalnızca {count} vektör okundu");
    assert!(failures.is_empty(), "{} vektör tutmadı:\n{}", failures.len(), failures.join("\n"));
}
