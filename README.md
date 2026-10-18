# Istaka

Düz okey için açık kaynak yapay zekâ. Şu an **Faz 0** tamamlandı: kural motoru, el çözücü, iki taban bot ve hız ölçümü. Rust ile yazıldı, hiçbir bağımlılığı yok.

Kural seti: [KURALLAR_V1.md](KURALLAR_V1.md). Araştırma: [KARSILASTIRMA.md](KARSILASTIRMA.md).

## Kullanım

```sh
cargo test --release                        # bütün testler
cargo run --release -- bench random 10      # rastgele botlarla 10 sn, tüm çekirdekler
cargo run --release -- bench greedy 10 1    # açgözlü botlarla, tek çekirdek
cargo run --release -- ornek 3              # bir eli dağıtıp oynatır
```

## Yapı

| Dosya | İçerik |
|---|---|
| `src/tile.rs` | Taş türleri (0..51 yüzler, 52 sahte okey), okey hesabı, `K5`/`F` biçiminde okuma |
| `src/rules.rs` | Ayarlanabilir kurallar: 12-13-1, perde azami okey, gösterge gösterme, yerden alınan taşı geri atma |
| `src/solver.rs` | Bitiş kontrolü (seri/grup ve 7 çift), 15 taştan en iyi bitiş, perlere girebilen azami taş sayısı |
| `src/game.rs` | Dağıtım ve tek elin durum makinesi, sıfır toplamlı el puanı |
| `src/eval.rs` | Tablo tabanlı değerlendirici: bitişe uzaklık (normal ve 7 çift) ve ukeire |
| `src/bots.rs` | `RandomBot` ve `GreedyBot` (en düşük uzaklık, eşitlikte en yüksek ukeire) |
| `tests/vectors.txt` | 35 kural vektörü (elle yazılmış, düz metin) |
| `tests/crosscheck.rs` | Çözücüyü bağımsız bir kaba kuvvet denetleyiciyle 30.000 rastgele elde karşılaştırır; değerlendiriciyi kaba kuvvet, `is_normal` ve ukeire tanımıyla karşılaştırır |
| `tests/engine.rs` | Dağıtım sayıları, tohum tekrarlanabilirliği, deste bitişi, geri atma kuralı, botlarla binlerce el |

## Faz 0 ölçümleri (18 Ekim 2026, 32 iş parçacıklı makine)

| Bot | Tek çekirdek | 32 iş parçacığı | Atış/el | Sonuç |
|---|---|---|---|---|
| Rastgele | ~2.500 el/sn | ~34.000 el/sn | 48,9 | %99,5 puansız (rastgele oyuncu neredeyse hiç bitemiyor) |
| Açgözlü | ~59 el/sn | ~620 el/sn | 38 | normal %87, çift %3, puansız %10 |

Gözlemler:
- Motor hızlı ama **açgözlü botun sezgiseli yavaş**. Darboğaz `max_melded`'in geri izlemeli araması. Botlarla büyük ölçekte veri üretmeden önce el değerlendirmesinin tablo tabanlı bir dinamik programlamaya çevrilmesi gerekiyor (Faz 1'in ilk işi).
- Açgözlü bot okeyi atarak bitmeyi hiç denemiyor (%0). İlk fırsatta bitiyor. Bunu öğrenecek olan yapay zekâ.
- Koltuk avantajı belirgin değil: kazanan dağılımı başlayana göre aşağı yukarı %25 civarında.

## Faz 1: tablo tabanlı değerlendirici

`src/eval.rs`, `max_melded`'in geri izlemeli aramasının yerini aldı. Bitişe uzaklık, eldeki kaç taşın değişmesi gerektiğidir; ukeire, çekilince bu uzaklığı düşüren görünmeyen taş sayısıdır. Her renk için 3^13 girişli bir tablo (her sayıdan 0/1/2 kopya) yalnızca serilerle kaç yuvada kaç taş örtülebildiğini tutar; gruplar sayı sayı denenip renk tablolarıyla (max, +) katlanarak birleştirilir. 7 çift dalı ayrı hesaplanır. Tablolar ilk kullanımda ~0,2 saniyede kurulur.

Doğruluk (`tests/crosscheck.rs`): 24.000 elde uzaklık 0 ⇔ bitmiş el (kaba kuvvet ve `is_pairs`), 15 taşta uzaklık 0 ⇔ bir atışla bitiş, normal uzaklık ≤ 1 ⇔ tek taş değişimiyle bitiş (`is_normal`); 6.000 elin bütün atışlarında ukeire, her taşı tek tek ekleyen tanımla aynı. Uzaklık > 0'da eksik taşların kopya sınırı gözetilmez (ikinci kopyası görünmüş bir taş da çekilebilir sayılır).

Ölçüm (18 Ekim 2026, aynı makinede eski ve yeni sürüm art arda, makine o sırada yüklüydü; `bench <bot> 10 1`):

| Bot | Önce | Sonra | Atış/el | Sonuç (sonra) |
|---|---|---|---|---|
| Açgözlü | 46–48 el/sn | 635–678 el/sn (~14 kat) | 38 → 29,1 | normal %98, çift %1, puansız %1 |
| Rastgele | 1.941–2.012 el/sn | 1.969–1.985 el/sn | 48,9 | değişmedi (değerlendiriciyi kullanmıyor) |

Gözlemler:
- Yeni açgözlü bot daha güçlü olduğu için eller kısaldı (38 → 29 atış). Atış başına hızlanma ~11 kat.
- Uzaklık hesabı tek başına el başına ~1 µs. Botun süresinin çoğu, eşit uzaklıklı atışlar arasında ukeire hesabında geçiyor (atış başına ortalama ~6 aday).
- Değerlendirici perde başına en az 2 okey kuralını varsayar (`max_okeys_per_meld >= 2`, aksi halde durur).
