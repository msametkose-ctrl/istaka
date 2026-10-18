//! Tablo tabanlı el değerlendirici: bitişe uzaklık ve ukeire.
//!
//! **Uzaklık**, eldeki kaç taşın değişmesi gerektiğidir (0 = el bitmiş). 15 taşlık
//! elde en iyi atıştan sonraki uzaklık verilir. Normal ve 7 çift dalı ayrı hesaplanır,
//! küçüğü alınır.
//!
//! Normal dal: 14 yuvalık bir bitmiş el düşünülür; jokerler daima tutulur, eksik
//! yuvalar çekilen taşlarla dolar. Uzaklık `(14 - J) - B` olur; `B`, toplam yuva
//! sayısı 14'ü geçmeyen "eksik perlerle" (bazı yuvaları boş perler) örtülebilen azami
//! doğal taş sayısıdır. 14'ten az yuva her zaman 14'e tamamlanabilir (seri uzatılır,
//! 3'lü grup 4'lü olur, 4'lü grup iki jokerli 3'lüye bölünür, boşluk 3+ ise yeni per).
//! Eksik yuvalarda kopya sınırı gözetilmez: uzaklık 0'da eksikleri jokerler doldurur
//! (jokerin kopya sınırı yok), bu yüzden bitiş kontrolü kesindir. Uzaklık > 0'da bu,
//! ikinci kopyası görünmüş taşları da çekilebilir sayan bir yaklaşımdır.
//!
//! Her per en az 3 yuvalık olduğundan en çok 4 per vardır. Seriler 3-5 genişlikli
//! pencerelere bölünür (uzunlar parçalanabilir). Her renk için 3^13 girişli bir tablo
//! (her sayıdan 0/1/2 kopya), yalnızca serilerle `S` yuvada örtülebilen azami taş
//! sayısını tutar. Gruplar, sayı sayı denenip renk tablolarına (max, +) katlamasıyla
//! birleştirilir. Perde en çok 2 okey kuralı J <= 2 olduğundan kendiliğinden sağlanır.
//! Perde en çok 1 okey ayarında iki okeyli elde bitiş `is_normal` ile kesin denetlenir;
//! iki okeyli elde uzaklık > 0 yaklaşıktır (okeylerin aynı pere düşmesi izlenmez).
//! Tablolar (12-13-1 açık/kapalı) ilk kullanımda bir kez kurulur (~0,2 sn, 48 MB).
//!
//! **Ukeire**: bir taş örtüyü en çok 1 artırır. Seriye girerek artıranlar, en iyi
//! yapraklarda tek geçişte toplanır (`run_improvers`). Kendi sayısında gruba girerek
//! artıranlar için o grup seçenekleriyle sınırlı arama yapılır; aynı 15 taştan gelen
//! adaylarda `Superset` bu aramaları 15'lik + taş üzerinde bir kez yapıp süzer.

use crate::rules::Rules;
use crate::solver::Packed;
use crate::tile::FACES;
use std::sync::OnceLock;

/// Yuva bütçesi: bitmiş el 14 taş.
const SLOTS: usize = 14;
type Curve = [u8; SLOTS + 1];

const POW3: [u32; 14] = {
    let mut p = [1u32; 14];
    let mut i = 1;
    while i < 14 {
        p[i] = p[i - 1] * 3;
        i += 1;
    }
    p
};
const ENTRIES: usize = POW3[13] as usize;

/// `out[s] = max(out[s], src[s - size] + k)`
#[inline]
fn shift_max(out: &mut Curve, src: &Curve, size: usize, k: u8) {
    for s in size..=SLOTS {
        let v = src[s - size] + k;
        if v > out[s] {
            out[s] = v;
        }
    }
}

/// En küçük sayısı `i` olan renk vektörü `d` için `i`'nin bir kopyasını içeren seri
/// pencereleri: `f(seçilen diğer sayılar (bit), yuva, doğal taş)`. Sağdaki 4 sayıdan
/// taşı olanların her birinden 0 ya da 1 kopya seçilir; 12-13-1 açıksa 1, 14. konumda
/// soldaki 10..13 ile de pencere kurar. Pencere genişliği seçilen taşların açıklığıdır
/// (en az 3); 5'ten uzun seriler parçalara bölünebildiğinden yeterlidir.
fn windows(d: &[u8; 13], i: usize, wrap: bool, mut f: impl FnMut(u16, usize, u8)) {
    let mut cand = [0usize; 4];
    let mut nc = 0;
    for j in i + 1..=(i + 4).min(12) {
        if d[j] > 0 {
            cand[nc] = j;
            nc += 1;
        }
    }
    for mask in 0u32..(1 << nc) {
        let (mut picked, mut hi) = (0u16, i);
        for b in 0..nc {
            if mask & (1 << b) != 0 {
                picked |= 1 << cand[b];
                hi = cand[b];
            }
        }
        f(picked, (hi - i + 1).max(3), 1 + mask.count_ones() as u8);
    }
    if i == 0 && wrap {
        let mut nc = 0;
        for j in 9..=12 {
            if d[j] > 0 {
                cand[nc] = j;
                nc += 1;
            }
        }
        for mask in 1u32..(1 << nc) {
            let (mut picked, mut lo) = (0u16, 13);
            for b in 0..nc {
                if mask & (1 << b) != 0 {
                    picked |= 1 << cand[b];
                    lo = lo.min(cand[b]);
                }
            }
            f(picked, (13 - lo + 1).max(3), 1 + mask.count_ones() as u8);
        }
    }
}

/// Seçilen sayıların birer kopyasının tablo indeksindeki karşılığı.
fn picked_idx(picked: u16) -> usize {
    (0..13).filter(|&j| picked & (1 << j) != 0).map(|j| POW3[j] as usize).sum()
}

fn digits(mut idx: usize) -> [u8; 13] {
    let mut d = [0u8; 13];
    for x in d.iter_mut() {
        *x = (idx % 3) as u8;
        idx /= 3;
    }
    d
}

/// Tek renkte yalnızca serilerle: `t[v][s]`, `s` yuvaya sığan azami doğal taş.
fn build_runs(wrap: bool) -> Vec<Curve> {
    let mut t = vec![[0u8; SLOTS + 1]; ENTRIES];
    let mut d = [0u8; 13];
    for idx in 1..ENTRIES {
        let mut k = 0;
        loop {
            d[k] += 1;
            if d[k] == 3 {
                d[k] = 0;
                k += 1;
            } else {
                break;
            }
        }
        // En küçük sayı i ya atılır ya da i'yi içeren bir pencereye girer.
        let i = d.iter().position(|&x| x > 0).unwrap();
        let base = idx - POW3[i] as usize;
        let mut best = t[base];
        windows(&d, i, wrap, |picked, size, k| {
            let src = t[base - picked_idx(picked)];
            shift_max(&mut best, &src, size, k);
        });
        t[idx] = best;
    }
    t
}

/// `v` renk vektörünü `s` yuvada en az `k` taşla örten serilerin kullandığı taşları
/// `used`'a ekler (tablodan geriye doğru).
fn reconstruct(t: &RunTable, wrap: bool, mut v: usize, mut s: usize, mut k: u8, used: &mut [u8; 13]) {
    while k > 0 {
        let d = digits(v);
        let i = d.iter().position(|&x| x > 0).expect("örtü tabloyla uyuşmuyor");
        let base = v - POW3[i] as usize;
        if t.dense[base][s] >= k {
            v = base;
            continue;
        }
        let mut next = None;
        windows(&d, i, wrap, |picked, size, kk| {
            let rest = base - picked_idx(picked);
            if next.is_none() && size <= s && t.dense[rest][s - size] + kk >= k {
                next = Some((rest, size, kk, picked));
            }
        });
        let (rest, size, kk, picked) = next.expect("örtü tabloyla uyuşmuyor");
        used[i] += 1;
        (0..13).filter(|&j| picked & (1 << j) != 0).for_each(|j| used[j] += 1);
        v = rest;
        s -= size;
        k = k.saturating_sub(kk);
    }
}

/// Eğrinin seyrek biçimi: `[n, p1..pn]`, her nokta `s << 4 | k` (eğrinin `s`'de `k`'ye
/// çıktığı yer). `k <= 14` olduğundan en çok 14 nokta vardır.
type Steps = [u8; SLOTS + 1];

fn steps(c: &Curve) -> Steps {
    let mut st = [0u8; SLOTS + 1];
    for s in 1..=SLOTS {
        if c[s] > c[s - 1] {
            st[0] += 1;
            st[st[0] as usize] = (s as u8) << 4 | c[s];
        }
    }
    st
}

/// Renk tablosu: aynı eğri hem yoğun (ilk renk için) hem seyrek biçimde.
struct RunTable {
    dense: Vec<Curve>,
    sparse: Vec<Steps>,
}

fn runs_table(wrap: bool) -> &'static RunTable {
    static TABLES: [OnceLock<RunTable>; 2] = [OnceLock::new(), OnceLock::new()];
    TABLES[wrap as usize].get_or_init(|| {
        let dense = build_runs(wrap);
        let sparse = dense.iter().map(steps).collect();
        RunTable { dense, sparse }
    })
}

/// Bir sayıdaki grup seçeneği: her renkten kullanılan taş, doğal taş, yuva.
#[derive(Clone, Copy)]
struct GroupOpt {
    used: [u8; 4],
    k: u8,
    size: u8,
}

/// 4 rengin bir sayıdaki kopya deseni (taban 3) için grup seçenekleri. İlk seçenek
/// "grup yok". Tek doğal taşlı grup, tek taşlık seri penceresine denk olduğundan yok.
fn group_opts() -> &'static [Vec<GroupOpt>] {
    static OPTS: OnceLock<Vec<Vec<GroupOpt>>> = OnceLock::new();
    OPTS.get_or_init(|| {
        (0..81u32)
            .map(|code| {
                let have: [u8; 4] = std::array::from_fn(|c| (code / POW3[c]) as u8 % 3);
                let mut v = vec![GroupOpt { used: [0; 4], k: 0, size: 0 }];
                for ucode in 1..81u32 {
                    let u: [u8; 4] = std::array::from_fn(|c| (ucode / POW3[c]) as u8 % 3);
                    if (0..4).any(|c| u[c] > have[c]) {
                        continue;
                    }
                    if let Some(size) = group_size(u) {
                        let k = u.iter().sum();
                        v.push(GroupOpt { used: u, k, size });
                    }
                }
                // Aynı ya da daha az yuvayla daha fazla taş kullanan seçenek varsa bu
                // seçenek gereksizdir: fazladan taş seriden çekilirse oradaki yuva boş
                // kalır, toplam yuva artmaz.
                let all = v.clone();
                v.retain(|o| {
                    !all.iter().any(|w| {
                        w.k > o.k && w.size <= o.size && (0..4).all(|c| w.used[c] >= o.used[c])
                    })
                });
                v
            })
            .collect()
    })
}

/// `u` kullanımıyla 1 ya da 2 grup (her biri 2-4 doğal taş, renkleri farklı) kurmanın
/// en az yuvası. İki kopyalı renk iki gruba da girer.
fn group_size(u: [u8; 4]) -> Option<u8> {
    let total: u8 = u.iter().sum();
    if u.iter().all(|&x| x <= 1) {
        return (total >= 2).then(|| total.max(3));
    }
    let twos = u.iter().filter(|&&x| x == 2).count() as u8;
    let ones = total - 2 * twos;
    // İkinci gruba giden tekler: o; gruplar twos+ (ones-o) ve twos+o doğal taşlı.
    (0..=ones)
        .filter_map(|o| {
            let (a, b) = (twos + ones - o, twos + o);
            (a >= 2 && b >= 2).then(|| a.max(3) + b.max(3))
        })
        .min()
}

/// Bütçe içinde 4 eğrinin (max, +) katlaması: toplam yuva <= budget.
#[inline]
fn combine(t: &RunTable, idx: [u32; 4], budget: usize) -> u8 {
    let mut ixs = [0usize; 4];
    let mut n = 0;
    for &ix in &idx {
        if ix != 0 {
            ixs[n] = ix as usize;
            n += 1;
        }
    }
    if n == 0 {
        return 0;
    }
    let point = |p: u8| ((p >> 4) as usize, p & 15);
    let pts = |ix: usize| {
        let st = &t.sparse[ix];
        &st[1..1 + st[0] as usize]
    };
    let mut acc = t.dense[ixs[0]];
    for &ix in ixs.iter().take(n.saturating_sub(1)).skip(1) {
        let mut out = acc;
        for &p in pts(ix) {
            let (sp, k) = point(p);
            for x in sp..=budget {
                out[x] = out[x].max(acc[x - sp] + k);
            }
        }
        acc = out;
    }
    let mut best = acc[budget];
    if n > 1 {
        for &p in pts(ixs[n - 1]) {
            let (sp, k) = point(p);
            if sp <= budget {
                best = best.max(acc[budget - sp] + k);
            }
        }
    }
    best
}

struct Search<'a> {
    table: &'a RunTable,
    ranks: [(u8, &'a [GroupOpt]); 13],
    n: usize,
    best: u8,
    cap: u8,
    budget: u8,
    /// En iyi yaprak: grupların bıraktığı renk indeksleri, kalan yuva, grup taşları.
    leaf: Option<([u32; 4], u8, u8)>,
    /// (sayı, renk): o sayıda yalnızca o rengi kullanan grup seçenekleri denenir.
    force: Option<(u8, usize)>,
}

/// Yoğun iki eğrinin `budget`'e kadar (max, +) katlaması.
fn conv(a: &Curve, b: &Curve, budget: usize) -> Curve {
    let mut out = [0u8; SLOTS + 1];
    for x in 0..=budget {
        out[x] = (0..=x).map(|y| a[y] + b[x - y]).max().unwrap();
    }
    out
}

impl Search<'_> {
    /// `run_improvers` için: değeri `b` olan bütün yapraklarda iyileştiren yüzleri topla.
    #[allow(clippy::too_many_arguments)]
    fn collect(&mut self, pos: usize, idx: [u32; 4], k: u8, size: u8, b: u8, code: &[u8; 13], mask: &mut u64) {
        let left = self.budget - size;
        if k + left < b {
            return;
        }
        if pos < self.n {
            let (r, opts) = self.ranks[pos];
            for o in opts {
                if size + o.size > self.budget {
                    continue;
                }
                let mut ix = idx;
                for c in 0..4 {
                    ix[c] -= o.used[c] as u32 * POW3[r as usize];
                }
                self.collect(pos + 1, ix, k + o.k, size + o.size, b, code, mask);
            }
            return;
        }
        let l = left as usize;
        if k + combine(self.table, idx, l) < b {
            return;
        }
        let dn: [&Curve; 4] = std::array::from_fn(|c| &self.table.dense[idx[c] as usize]);
        let p01 = conv(dn[0], dn[1], l);
        let s23 = conv(dn[2], dn[3], l);
        let others = [conv(dn[1], &s23, l), conv(dn[0], &s23, l), conv(&p01, dn[3], l), conv(&p01, dn[2], l)];
        let target = b + 1 - k;
        for c in 0..4 {
            let dg = digits(idx[c] as usize);
            for r in 0..13 {
                let f = c * 13 + r;
                // El sayısı < 2 ise yapraktaki sayı da < 2'dir.
                if *mask & (1 << f) != 0 || (code[r] / POW3[c] as u8) % 3 >= 2 || dg[r] >= 2 {
                    continue;
                }
                let v = &self.table.dense[(idx[c] + POW3[r]) as usize];
                if (0..=l).any(|a| others[c][a] + v[l - a] >= target) {
                    *mask |= 1 << f;
                }
            }
        }
    }

    fn dfs(&mut self, pos: usize, idx: [u32; 4], k: u8, size: u8) {
        // Her boş yuva en çok bir taş alır: kalan bütçe iyileştiremiyorsa dal kesilir.
        let left = self.budget - size;
        if self.best >= self.cap || k + left <= self.best {
            return;
        }
        if pos == self.n {
            let v = k + combine(self.table, idx, left as usize);
            if v > self.best {
                self.best = v;
                self.leaf = Some((idx, left, k));
            }
            return;
        }
        let (r, opts) = self.ranks[pos];
        for o in opts {
            if size + o.size > self.budget || self.force.is_some_and(|(fr, fc)| fr == r && o.used[fc] == 0) {
                continue;
            }
            let mut ix = idx;
            for c in 0..4 {
                ix[c] -= o.used[c] as u32 * POW3[r as usize];
            }
            self.dfs(pos + 1, ix, k + o.k, size + o.size);
        }
    }
}

/// Aramaya hazır el: renk başına tablo indeksi ve sayı başına renk deseni.
#[derive(Clone, Copy)]
struct Prepared {
    idx: [u32; 4],
    code: [u8; 13],
}

impl Prepared {
    fn new(counts: &[u8; FACES]) -> Self {
        let mut h = Prepared { idx: [0; 4], code: [0; 13] };
        for c in 0..4 {
            for r in 0..13 {
                h.add(c, r, counts[c * 13 + r]);
            }
        }
        h
    }

    #[inline]
    fn add(&mut self, c: usize, r: usize, n: u8) {
        self.idx[c] += n as u32 * POW3[r];
        self.code[r] += n * POW3[c] as u8;
    }

    /// `budget` yuvaya sığan eksik perlerle örtülebilen azami doğal taş (`B`). `cap`'e
    /// ulaşınca durur (en çok `cap` döner); `floor`'u aşamıyorsa `floor` döner.
    fn covered(&self, wrap: bool, budget: u8, floor: u8, cap: u8) -> u8 {
        self.search(wrap, budget, floor, cap).best.min(cap)
    }

    fn search(&self, wrap: bool, budget: u8, floor: u8, cap: u8) -> Search<'static> {
        let mut s = self.search_setup(wrap, budget, floor, cap);
        s.dfs(0, self.idx, 0, 0);
        s
    }

    fn search_setup(&self, wrap: bool, budget: u8, floor: u8, cap: u8) -> Search<'static> {
        let opts = group_opts();
        let mut s = Search {
            table: runs_table(wrap),
            ranks: [(0, &[][..]); 13],
            n: 0,
            best: floor,
            cap,
            budget,
            leaf: None,
            force: None,
        };
        for r in 0..13 {
            let o = &opts[self.code[r] as usize];
            if o.len() > 1 {
                s.ranks[s.n] = (r as u8, o);
                s.n += 1;
            }
        }
        s
    }

    /// En iyi örtüsü `b` taş olan elde, seriye eklenince örtüyü `b + 1`'e çıkaran yüzler
    /// (bit maskesi). Bir taş örtüyü en çok 1 artırdığından yalnızca değeri `b` olan
    /// yapraklara bakılır; orada taş gruplar aynı kalırken kendi renginin serisine girer.
    /// Taşın gruba girdiği durumlar burada yoktur.
    fn run_improvers(&self, wrap: bool, b: u8) -> u64 {
        let mut s = self.search_setup(wrap, SLOTS as u8, b, b);
        let mut mask = 0u64;
        s.collect(0, self.idx, 0, 0, b, &self.code, &mut mask);
        mask
    }

    /// 14 yuvada en az `need` taşlık bir örtü varsa kullandığı taş sayıları (yüz başına).
    /// Eklenen `f` yüzünün kendi sayısında bir gruba girdiği örtülerle `need`'e ulaşan
    /// örnek örtünün kullandığı taşlar. (Seriye girdiği durum `run_improvers`'tadır.)
    fn group_witness(&self, wrap: bool, need: u8, f: usize) -> Option<[u8; FACES]> {
        let s = self.group_search(wrap, need, f);
        self.witness_of(&s, wrap, need)
    }

    fn group_search(&self, wrap: bool, need: u8, f: usize) -> Search<'static> {
        let (c, r) = (f / 13, f % 13);
        let mut s = self.search_setup(wrap, SLOTS as u8, need - 1, need);
        // Taşın gruba girmediği örtüler `run_improvers`'ta denendi; burada bulunan her
        // örtü yine de geçerlidir, yani sonuç kesin kalır.
        s.force = Some((r as u8, c));
        s.dfs(0, self.idx, 0, 0);
        s
    }

    /// Aramanın en iyi yaprağından, en az `need` taşlık örtünün kullandığı taşlar.
    fn witness_of(&self, s: &Search, wrap: bool, need: u8) -> Option<[u8; FACES]> {
        let (idx, left, kg) = s.leaf.filter(|_| s.best >= need)?;
        let t = s.table;
        let mut used = [0u8; FACES];
        for c in 0..4 {
            for (r, &g) in digits((self.idx[c] - idx[c]) as usize).iter().enumerate() {
                used[c * 13 + r] += g;
            }
        }
        // Yuvaları renklere geriye doğru dağıt: pre[i], ilk i+1 rengin katlaması.
        let cs: Vec<usize> = (0..4).filter(|&c| idx[c] != 0).collect();
        if cs.is_empty() {
            return Some(used);
        }
        let mut pre = vec![t.dense[idx[cs[0]] as usize]];
        for &c in &cs[1..] {
            let (a, b) = (pre.last().unwrap(), &t.dense[idx[c] as usize]);
            let conv: Curve = std::array::from_fn(|x| (0..=x).map(|y| a[y] + b[x - y]).max().unwrap());
            pre.push(conv);
        }
        let (mut x, mut tgt) = (left as usize, need.saturating_sub(kg));
        for i in (0..cs.len()).rev() {
            let cur = &t.dense[idx[cs[i]] as usize];
            let b = if i == 0 {
                x
            } else {
                (0..=x).find(|&b| pre[i - 1][x - b] + cur[b] >= tgt).expect("örtü bölünemedi")
            };
            let k = cur[b].min(tgt);
            let mut u = [0u8; 13];
            reconstruct(t, wrap, idx[cs[i]] as usize, b, k, &mut u);
            for r in 0..13 {
                used[cs[i] * 13 + r] += u[r];
            }
            x -= b;
            tgt -= k;
        }
        Some(used)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Distance {
    pub normal: u8,
    pub pairs: u8,
}

impl Distance {
    pub fn min(self) -> u8 {
        self.normal.min(self.pairs)
    }
}

/// Çift yolunda uzaklık: her tekin eşi bir okey olabilir, artan okeyler çifttir.
fn pairs_distance(pair: u8, single: u8, j: u8) -> u8 {
    let mate = j.min(single);
    7 - (pair + mate + (j - mate) / 2).min(7)
}

fn check(p: &Packed, rules: &Rules) {
    assert!(rules.max_okeys_per_meld >= 1, "değerlendirici perde başına en az 1 okey varsayar");
    debug_assert!(p.counts.iter().all(|&c| c <= 2) && p.jokers <= 2);
}

/// Perde en çok 1 okey kuralında iki okey ayrı perlere girmelidir; örtü modeli bunu
/// izlemez. Bu durumda (iki okey) sonuç yaklaşık sayılır ve gerekirse kesin denetime
/// geçilir.
fn split_jokers(p: &Packed, rules: &Rules) -> bool {
    rules.max_okeys_per_meld < 2 && p.jokers >= 2
}

/// 14 ya da 15 taşlık el (15'te bir atıştan sonra) normal bitmiş mi? Kesin, yavaş.
fn finished_exact(p: &Packed, rules: &Rules) -> bool {
    let total: u8 = p.counts.iter().sum::<u8>() + p.jokers;
    if total == SLOTS as u8 {
        return crate::solver::is_normal(&mut p.clone(), rules);
    }
    let natural = (0..FACES).filter(|&f| p.counts[f] > 0).any(|f| {
        let mut q = *p;
        q.counts[f] -= 1;
        crate::solver::is_normal(&mut q, rules)
    });
    natural || p.jokers > 0 && {
        let mut q = *p;
        q.jokers -= 1;
        crate::solver::is_normal(&mut q, rules)
    }
}

/// 14 ya da 15 taşlık elin bitişe uzaklığı (15'te en iyi atıştan sonra).
pub fn distance(p: &Packed, rules: &Rules) -> Distance {
    check(p, rules);
    let j = p.jokers;
    let naturals: u8 = p.counts.iter().sum();
    let cap = (SLOTS as u8 - j).min(naturals);
    let b = Prepared::new(&p.counts).covered(rules.wrap_13_1, SLOTS as u8, 0, cap);
    let pair = p.counts.iter().filter(|&&c| c == 2).count() as u8;
    let single = p.counts.iter().filter(|&&c| c == 1).count() as u8;
    let mut normal = SLOTS as u8 - j - b;
    // İki okey aynı pere düşüyorsa biri atılmalıdır: bitmiş görünen el 1 uzakta.
    // Uzaklık > 0'da bu ayrım yapılmaz (yaklaşık).
    if normal == 0 && split_jokers(p, rules) && !finished_exact(p, rules) {
        normal = 1;
    }
    Distance { normal, pairs: pairs_distance(pair, single, j) }
}

/// 15 taşlık elden her tür atılınca kalan 14'lüğün uzaklığı (elde olmayan tür `None`).
/// 15'liğin en iyi örtüsü bir kez bulunur; örnek örtünün bütün kopyalarını kullanmadığı
/// taş atılınca örtü değişmez, diğerleri için örtünün korunup korunmadığı aranır.
pub fn discard_distances(full: &Packed, rules: &Rules) -> [Option<Distance>; PACKED_KINDS] {
    check(full, rules);
    if split_jokers(full, rules) {
        return std::array::from_fn(|x| {
            let mut q = *full;
            let present = if x == FACES { q.jokers > 0 } else { q.counts[x] > 0 };
            if x == FACES && present {
                q.jokers -= 1;
            } else if present {
                q.counts[x] -= 1;
            }
            present.then(|| distance(&q, rules))
        });
    }
    let wrap = rules.wrap_13_1;
    let base = Prepared::new(&full.counts);
    let naturals: u8 = full.counts.iter().sum();
    let pair = full.counts.iter().filter(|&&c| c == 2).count() as u8;
    let single = full.counts.iter().filter(|&&c| c == 1).count() as u8;
    let mut out = [None; PACKED_KINDS];
    if naturals == 0 {
        return out;
    }
    let j = full.jokers;
    // Doğal taş atılınca: J aynı, doğal taş bir eksik.
    let cap = (SLOTS as u8 - j).min(naturals - 1);
    let s = base.search(wrap, SLOTS as u8, 0, cap + 1);
    let b15 = s.best.min(cap + 1);
    let used = base.witness_of(&s, wrap, b15).unwrap_or([0; FACES]);
    for f in 0..FACES {
        if full.counts[f] == 0 {
            continue;
        }
        let b = if b15 == 0 {
            0
        } else if full.counts[f] > used[f] {
            b15.min(cap)
        } else {
            let mut h = base;
            h.idx[f / 13] -= POW3[f % 13];
            h.code[f % 13] -= POW3[f / 13] as u8;
            h.covered(wrap, SLOTS as u8, b15 - 1, b15.min(cap))
        };
        let (pr, sg) = if full.counts[f] == 2 { (pair - 1, single + 1) } else { (pair, single - 1) };
        out[f] = Some(Distance { normal: SLOTS as u8 - j - b, pairs: pairs_distance(pr, sg, j) });
    }
    if j > 0 {
        let mut p = *full;
        p.jokers -= 1;
        out[FACES] = Some(distance(&p, rules));
    }
    out
}

/// Packed uzayında tür sayısı: 52 yüz + joker (indeks 52).
pub const PACKED_KINDS: usize = FACES + 1;

/// 14 taşlık eli ilerleten görünmeyen taş sayısı: çekilince en iyi atıştan sonraki
/// uzaklığı düşüren taşlar. `unseen[f]` yüz başına (okey yüzü sahte okeylerdir),
/// `unseen[52]` gerçek okeylerdir. `d`, elin `distance(p)` değeridir.
pub fn ukeire(p: &Packed, rules: &Rules, unseen: &[u8; PACKED_KINDS], d: Distance) -> u32 {
    ukeire_within(p, rules, unseen, d, None)
}

/// Aynı 15 taşlık elden farklı atışlarla kalan 14'lüklerin ukeire'si için ortak süzgeç.
/// 14'lük + f, 15'lik + f'nin alt kümesi olduğundan `B(15'lik + f) < need` ise f hiçbir
/// atışta eli ilerletmez. Bu, ilerletmeyen (aramanın sonuna kadar gitmesi gereken)
/// taşları her aday atış için ayrı ayrı denemekten kurtarır.
pub struct Superset {
    base: Prepared,
    counts: [u8; FACES],
    wrap: bool,
    need: u8,
    reach: [Option<bool>; FACES],
    /// Ulaşan f için örnek örtünün kullandığı taşlar.
    used: Vec<[u8; FACES]>,
}

impl Superset {
    pub fn new(p15: &Packed, rules: &Rules) -> Self {
        Superset {
            base: Prepared::new(&p15.counts),
            counts: p15.counts,
            wrap: rules.wrap_13_1,
            need: 0,
            reach: [None; FACES],
            used: vec![[0; FACES]; FACES],
        }
    }

    /// `may_reach(f)` true döndükten sonra: örnek örtü atılan `x`'in bütün kopyalarını
    /// kullanmıyorsa `x` atılınca da örtü kalır.
    fn spare(&self, f: usize, x: usize) -> bool {
        x < FACES && self.counts[f] < 2 && self.counts[x] + (x == f) as u8 > self.used[f][x]
    }

    /// f eklenip kendi sayısında gruba girince örtü `need`'e ulaşabilir mi? Bilinemiyorsa
    /// true.
    fn may_reach(&mut self, f: usize, need: u8) -> bool {
        if self.counts[f] >= 2 {
            return true;
        }
        if need != self.need {
            self.need = need;
            self.reach = [None; FACES];
        }
        if let Some(r) = self.reach[f] {
            return r;
        }
        let mut h = self.base;
        h.add(f / 13, f % 13, 1);
        let w = h.group_witness(self.wrap, need, f);
        if let Some(u) = w {
            self.used[f] = u;
        }
        self.reach[f] = Some(w.is_some());
        w.is_some()
    }
}

/// `ukeire`; `sup` verilirse `p`, `sup`'un 15 taşından `discard` (Packed uzayında)
/// atılmış eldir.
pub fn ukeire_within(
    p: &Packed,
    rules: &Rules,
    unseen: &[u8; PACKED_KINDS],
    d: Distance,
    mut sup: Option<(&mut Superset, usize)>,
) -> u32 {
    check(p, rules);
    let now = d.min();
    if now == 0 {
        return 0;
    }
    // İki okeyli elde (perde 1 okey ayarı) kısayollar geçersiz: tanımla say.
    if split_jokers(p, rules) {
        return ukeire_direct(p, rules, unseen, now);
    }
    let j = p.jokers;
    let naturals: u8 = p.counts.iter().sum();
    let pair = p.counts.iter().filter(|&&c| c == 2).count() as u8;
    let single = p.counts.iter().filter(|&&c| c == 1).count() as u8;
    let base = Prepared::new(&p.counts);
    let mut total = 0;
    // Gerçek okey: doğal taşlar aynı kalır, joker bir artar; normal uzaklık 1 düşer.
    if unseen[FACES] > 0 && j < 2 {
        let better = if rules.max_okeys_per_meld < 2 && j == 1 {
            let mut q = *p;
            q.jokers += 1;
            distance(&q, rules).min() < now
        } else {
            d.normal == now || pairs_distance(pair, single, j + 1) < now
        };
        if better {
            total += unseen[FACES] as u32;
        }
    }
    // Normal yolda uzaklığın düşmesi için gereken örtü B' >= need; B' en çok B + 1.
    let wrap = rules.wrap_13_1;
    let b = SLOTS as u8 - j - d.normal;
    let need = SLOTS as u8 + 1 - j - now;
    let cap = (SLOTS as u8 - j).min(naturals + 1);
    let normal_possible = need <= cap && need <= b + 1;
    let runs = if normal_possible { base.run_improvers(wrap, b) } else { 0 };
    for f in 0..FACES {
        if unseen[f] == 0 || p.counts[f] >= 2 {
            continue;
        }
        let (pr, sg) = if p.counts[f] == 1 { (pair + 1, single - 1) } else { (pair, single + 1) };
        let (c, r) = (f / 13, f % 13);
        // Seriyle iyileştirmiyorsa ancak aynı sayıdaki başka renkli taşlarla gruba girerek
        // iyileştirebilir; yalnızca o grup seçenekleriyle aranır.
        let better = pairs_distance(pr, sg, j) < now
            || runs & (1 << f) != 0
            || (normal_possible && (0..4).any(|o| o != c && p.counts[o * 13 + r] > 0) && {
                let reach = sup.as_mut().map_or(true, |(s, _)| s.may_reach(f, need));
                reach && sup.as_ref().is_some_and(|(s, x)| s.spare(f, *x)) || reach && {
                    let mut h = base;
                    h.add(c, r, 1);
                    h.group_search(wrap, need, f).best >= need
                }
            });
        if better {
            total += unseen[f] as u32;
        }
    }
    total
}

/// Ukeire'nin doğrudan tanımı: her taşı ekleyip uzaklığa bakar.
fn ukeire_direct(p: &Packed, rules: &Rules, unseen: &[u8; PACKED_KINDS], now: u8) -> u32 {
    (0..PACKED_KINDS)
        .filter(|&f| unseen[f] > 0)
        .filter(|&f| {
            let mut q = *p;
            if f == FACES {
                if q.jokers >= 2 {
                    return false;
                }
                q.jokers += 1;
            } else {
                if q.counts[f] >= 2 {
                    return false;
                }
                q.counts[f] += 1;
            }
            distance(&q, rules).min() < now
        })
        .map(|f| unseen[f] as u32)
        .sum()
}
