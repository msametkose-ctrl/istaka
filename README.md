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
| `src/bots.rs` | `RandomBot` ve `GreedyBot` |
| `tests/vectors.txt` | 35 kural vektörü (elle yazılmış, düz metin) |
| `tests/crosscheck.rs` | Çözücüyü bağımsız bir kaba kuvvet denetleyiciyle 30.000 rastgele elde karşılaştırır |
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
