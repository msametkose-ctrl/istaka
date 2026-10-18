# Istaka

Düz okey için açık kaynak yapay zekâ. Şu an **Faz 0** tamamlandı: kural motoru, el çözücü, iki taban bot ve hız ölçümü. Rust ile yazıldı, hiçbir bağımlılığı yok.

Kural seti: [KURALLAR_V1.md](KURALLAR_V1.md). Araştırma: [KARSILASTIRMA.md](KARSILASTIRMA.md).

## Kullanım

```sh
cargo test --release                        # bütün testler
cargo run --release -- bench random 10      # rastgele botlarla 10 sn, tüm çekirdekler
cargo run --release -- bench greedy 10 1    # açgözlü botlarla, tek çekirdek
cargo run --release -- ornek 3              # bir eli dağıtıp oynatır
cargo run --release -- duel strong greedy 20000   # duplicate karşılaştırma, 20.000 tohum
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
| `src/strong.rs` | `StrongBot`: taş sayma, sağdaki oyuncuya karşı savunma, çift yolu eşiği, okey atarak bitmek için bekleme |
| `src/duel.rs` | Duplicate karşılaştırma: aynı tohum [A,B,A,B] ve [B,A,B,A] düzenlerinde, eşleştirilmiş güven aralığı |
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
- Perde başına 1 okey ayarı da desteklenir: iki okeyli elde bitiş `is_normal` ile kesin denetlenir (crosscheck bu ayarda da `is_normal` ile karşılaştırır); iki okeyli elde uzaklık > 0 yaklaşıktır.

## Faz 1: duplicate karşılaştırma ve güçlü bot

`istaka duel A B [tohum] [iş_parçacığı] [ilk_tohum]` her tohumun dağıtımını iki düzende oynatır: [A,B,A,B] ve [B,A,B,A]. Tohum başına fark, iki elin ortalamasıyla A'nın oyuncu başına el puanı eksi B'ninkidir (sıfır toplamlı puan: kazanan +3M, kaybeden −M). Rapor: ortalama fark ve eşleştirilmiş %95 güven aralığı, el kazanma oranları ve Wilson aralıkları. Sonuç tohum sırasıyla toplanır; aynı tohumlar iş parçacığı sayısından bağımsız olarak aynı sonucu verir (`tests/engine.rs`).

`StrongBot` (`src/strong.rs`), GreedyBot'un üzerine:
- **Taş sayma:** ukeire yalnızca görünmeyen kopyaları sayar (kendi eli, ortadaki atıklar, gösterge); iki kopyası görünen yüz ölüdür. (GreedyBot da aynı sayımı kullanıyor.)
- **Savunma:** sağdaki oyuncunun (atışımı alabilecek kişi) benden aldığı taşlara yakın taşlar tehlikeli, onun attığı ve almadan geçtiği taşlara yakın taşlar güvenli sayılır; okey en tehlikelisidir. Eşit uzaklıklı atışlar arasında `ukeire − 2·(1 − deste/48)²·tehlike` en büyük olan seçilir.
- **Çift yolu:** 15 taşta en az 5 çift varsa atış çift uzaklığına göre seçilir.
- **Okey atarak bitme:** el bitmişken okey eldeyse, okeyin yerine geçecek doğal taşların canlı kopyası / görünmeyen taş oranı 4·0,03'ü aşıyorsa ve destede en az 8 taş varsa bitmeyip beklenir (bir turluk kazanç p·6, üç rakipten birinin bitme riski 3·q·8).

**Kapı** (ayar sırasında hiç kullanılmamış tohumlar, `duel strong greedy 20000 32 1000000`):

| | Değer |
|---|---|
| El başına puan farkı (strong − greedy) | **+0,084**, %95 GA [+0,053, +0,115] |
| El kazanma | strong %50,0 [%49,5, %50,4], greedy %49,1 [%48,6, %49,6], puansız %0,9 |

Ayar ölçümleri (tohum 0..6.000, strong − greedy, el başına):

| Değişken | Fark |
|---|---|
| Yalnızca taş sayma (= GreedyBot) | 0 |
| Taş saymasız (yalnızca kendi eli düşülür; 4.000 tohum) | −0,058 |
| Savunma ağırlığı 1 / 2 / 5 / 12 | +0,101 / +0,108 / +0,095 / +0,071 |
| Çift eşiği 3 / 4 / kapalı (savunma 3) | +0,044 / +0,099 / +0,110 (kapalı: 4.000 tohum; eşik 5 kapalıyla aynı sonucu verdi) |
| Bitmiş eli her durumda bekletmek (deste ≥ 16) | −0,506 |
| Bekleme riski q = 0,02 / 0,03 / 0,04 (deste ≥ 8, savunma 2); hiç beklememek | +0,087 / +0,121 / +0,116; +0,108 |

Gözlemler:
- Kazancın çoğu savunmadan geliyor. Çift eşiği 5'te nadiren devreye giriyor; 3'te zarar veriyor.
- Bitmiş eli bekletmek çok pahalı: rakipler ~29 atışta bitiriyor, okeyin yerine geçecek taşı çekme olasılığı ise tur başına birkaç yüzde. Bekleme ancak canlı kopya çoksa kârlı.
- Ayar tohumlarındaki fark (+0,121), kapı tohumlarındakinden (+0,084) biraz yüksek; seçim etkisi beklenen ölçüde.
