# Istaka v1 kural seti (düz okey)

18 Ekim 2026 itibarıyla kesinleşti. Bu dosya motorun varsayılan ayarlarıdır. Gerekçeleri [KARSILASTIRMA.md](KARSILASTIRMA.md) içinde.

## Taşlar ve dağıtım
- 106 taş: 4 renk × 1–13 × 2 kopya, artı 2 sahte okey.
- Karıştırma tohumlanabilir RNG ile yapılır. İlk 57 taş dağıtılır: başlayan oyuncuya 15, diğer üçüne 14'er.
- Kalan taşlardan sahte okey olmayan ilk taş gösterge olur. Geriye 48 taşlık deste kalır.
- Okey, göstergeyle aynı renkte ve bir büyük değerdeki taştır. Gösterge 13 ise okey 1 olur. Okeyin iki kopyası da jokerdir.
- Sahte okey joker değildir. Okey taşının yüzü yerine geçen sıradan bir taştır.
- Sıra saat yönünün tersine döner. Dağıtıcılık sağa geçer, dağıtıcının sağındaki oyuncu başlar.

## Perler
- **Seri:** aynı renkte, ardışık, en az 3 taş. 12-13-1 geçerli; 13-1-2 ve 12-13-1-2 geçersiz.
- **Grup:** aynı sayıda, renkleri farklı 3 ya da 4 taş.
- Bir perde 2 okeye kadar olabilir. Perde en az bir sayı taşı (normal ya da sahte okey) bulunmalı.
- Okey ve sahte okey aynı perde olabilir.
- **Çift:** aynı renk ve sayıda iki taş. Okey bir tekin eşi olabilir, iki okey de bir çift sayılır.

## Sıra
- Başlayan oyuncu ilk turda çekmeden atar.
- Sırası gelen oyuncu ya desteden çeker ya da bir önceki oyuncunun son attığı taşı alır. Başka atıklar alınamaz.
- Yerden alınan taş aynı turda geri atılabilir.
- Ardından ya bir taş atar ya da bitirir.
- Okeyi yere atmanın cezası yoktur.

## Bitiş ve puan
- **Normal bitiş:** 14 taş perlere ayrılır ve 15. taş ortaya atılır.
- **Çiftten bitiş:** 14 taş 7 çifte ayrılır.
- **Okey atarak bitiş:** atılan son taş gerçek okeydir. Son taş sahte okeyse bu normal bitiş sayılır.
- Puanlar (kaybeden her oyuncudan düşülür):

  | Bitiş | Puan |
  |---|---|
  | Normal | 2 |
  | Okey atarak | 4 |
  | Çiftten | 4 |
  | Çiftten ve okey atarak | 8 |

  Çarpanlar birbiriyle çarpılır.
- **Deste biterse:** son atılan taşla bitebilecek oyuncu bitebilir. Bitemezse el puansız biter.
- **Maç modu:** herkes 20 puanla başlar. Bitiş puanı kaybeden üç oyuncudan düşülür. Bir elin sonunda biri 0'a ya da altına inerse maç biter; en yüksek puanlı oyuncu kazanır.
- **Eğitim ve test modu:** tek el, sıfır toplamlı. Kazanan +3M alır, her kaybeden −M alır (M = 2, 4 ya da 8).

## Ayarlanabilir, v1'de kapalı
- `gosterge_gosterme`: göstergenin eşini ilk çekişten önce gösterince rakiplerden 1'er puan düşer.
- `seri_13_1_gecerli`: varsayılan açık. 101 için kapatılır.
- `perde_azami_okey`: varsayılan 2.
- `okey_atma_cezasi`, `renkli_carpani`, `cifte_ilan_atiklari_acar`, `tur_donme`, `gostergeyi_cifte_baglama`: hepsi kapalı.
- `oyuncu_sayisi`: varsayılan 4. `esli`: kapalı.
