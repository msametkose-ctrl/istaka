# Dört raporun karşılaştırması

Tarih: 18 Ekim 2026.

| Rapor | Yazan | Nerede |
|---|---|---|
| Bizim notlar | Claude | [ARASTIRMA_NOTLARI.md](ARASTIRMA_NOTLARI.md) |
| MiMo | MiMo v2.6 Flash | `gelen/mimo/` (Next.js sitesi, `src/sections/*.tsx`) |
| Straw | Straw | `gelen/straw/` (Next.js sitesi, rapor ve çalışan TS prototipi; motor `src/lib/okey.ts`) |
| Argon | Gemini 4 Argon | [gelen/argon/ARGON_RAPORU.md](gelen/argon/ARGON_RAPORU.md) |

Zip'lerdeki siteleri çalıştırmadım, içeriklerini okudum.

---

## 1. Hepsinin hemfikir olduğu konular

- **İlk sürüm düz okey olmalı**, 101 ikinci sürüme kalmalı.
- **Alan boş.** Okey üzerine tek ciddi akademik iş 2019 Boğaziçi tezi: iki kişilik bir okey ve rakibi rastgele oynayan bir bot. Ticari botlar ne yöntemlerini ne güçlerini yayınlıyor.
- **En yakın örnek mahjong.** Suphx'ten şu fikirler alınmalı:
  - oracle guiding: eğitime gizli taşları gören bir ajanla başlayıp bu bilgiyi yavaş yavaş geri çekmek
  - tüm maçın sonucunu tahmin eden ödül modeli (global reward prediction)
- **Nash dengesine dayalı yöntemlerin (CFR) peşine düşülmemeli.** Okey 4 kişilik ve kazanılan puanların toplamı sabit değil, bu yüzden denge hedefi burada işe yaramıyor. Bilgi kümeleri de çok büyük.
- **Pekiştirmeli öğrenmeye sıfırdan başlanmamalı**, insan verisi de beklenmemeli (yok).
- **Mimari üçünde de aynı sırada:**
  1. Rust ile yazılmış, kuralları doğrulanmış hızlı bir motor
  2. eli kesin olarak perlere ayıran bir çözücü
  3. kural tabanlı bir taban bot
  4. gizli taşları örnekleyerek arama yapan bir bot (rakip ellerine dair tahminle ağırlıklandırılmış)
  5. politika/değer ağı: aramadan ya da taban bottan öğrenir, sonra eski sürümlerden oluşan bir ligle self-play yapar
  6. Rust motorunu tarayıcıda çalıştıran bir web arayüzü (WASM)
- **Ölçme düzeni yapay zekâdan önce kurulmalı.** Aynı dağıtım her koltukta tekrar oynatılmalı (duplicate) ve sonuçlar güven aralığıyla verilmeli. Birincilikte %2'lik bir farkı ayırt etmek için yaklaşık 1.800 el gerekiyor.
- **Deste bitince el puansız biter.**

---

## 2. Kural kararları

| Konu | MiMo | Straw | Argon | Öneri |
|---|---|---|---|---|
| **12-13-1 serisi** | Geçersiz | Geçerli | Geçerli | **Geçerli** (3'e 1). MiMo burada 101 kaynaklarını düz okeye uygulamış. 13-1-2 her durumda geçersiz. |
| Bir perde kaç okey | 1 | 2 | 2 | **2**, ama perde en az bir sayı taşı olmalı. |
| Okey ve sahte okey aynı perde | Yasak | Serbest | Serbest | **Serbest.** |
| Gösterge gösterme | Açık, 1 puan | Kapalı | Açık, rakiplerden 1'er puan düşer | **Senin kararın.** Ben eğitim için kapalı başlatıp ayarla açılmasından yanayım. |
| Yerden alınan taş aynı elde geri atılabilir mi | Hayır | Evet | Evet | **Evet** (2'ye 1). Senin masanda farklıysa söyle. |
| İki okey bir çift sayılır mı | — | Evet | Evet | **Evet.** |
| Puanlama | Artı puan, 5 el ya da 151'e ulaşan kazanır | Ceza birikimi, 11 el | 20'den rakiplerden düşmeli (−1/−2/−4/−8) | **Maç modu:** 20'den düşmeli (Argon'un dediği gibi en yaygın sistem bu). **Eğitim:** tek elde sıfır toplamlı puan (kazanan +3M, kaybedenler −M). |
| Bitiş puanları | 2/4/4/8 | 2/4/4/8 | 2/4/4/8 | Aynı. Çarpanlar birbiriyle çarpılır. |
| Gösterge sahte okey çıkarsa | Gösterge 104 sayılı taştan seçilir | Yeniden çekilir | Yeniden çekilir | Aynı sonuç, fark yok. |
| Okeyi yere atmanın cezası | — | Yok | Varsayılan yok, ayarla açılabilir | **Yok.** |

Sadece Argon'un getirdiği niş kurallar: göstergeyi çifte bağlama, iki okeyle "tur dönme", çifte gidince atıkların açılması. Argon üçünü de varsayılan olarak kapalı öneriyor, ben de aynı fikirdeyim.

---

## 3. Yönteme bakışları

| | MiMo | Straw | Argon |
|---|---|---|---|
| Öğrenmenin ilk adımı | Aramadan damıtma (search distillation) | Expert iteration: arama öğretir, ağ taklit eder | Taban bottan davranış klonlama |
| Ardından | PPO ve lig | PPO ya da derin Monte Carlo, artı lig | Oracle guiding'li PPO ve lig |
| Oyun anında | Ağ ve arama | Ağ ve kısıtlı arama | Ağ, ilk 4–5 hamle adayı için arama |
| Ağ | Küçük MLP ya da CNN (1–10 milyon parametre) | Küçük ResNet ile Transformer karışımı | 1D ResNet, 12–16 blok (~2,5 milyon parametre) |
| Süre tahmini | 4–5 ay, tek kişi | Haftalar, yarı zamanlı | **Eğitim 2–3 gün, "1 haftada insan üstü"** |

Fark esas olarak öğrenmenin nereden başladığında. İkisini birleştirebiliriz: ağ önce taban bottan ucuzca ısıtılır (Argon), ardından aramayı öğretmen olarak kullanarak güçlenir (Straw ve MiMo).

---

## 4. Hatalar ve doğrulanamayan iddialar

**MiMo**
- `ibatuhandinc/OkeyArena` deposu **yok** (404). Raporun "en değerli repo, 138 test vektörü" dediği proje bu.
- "Bir elde en fazla 192 tur (48×4)" yanlış. Yandan alınan taşlar desteyi azaltmadığı için tur sayısının kesin bir üst sınırı yok.
- Diğer depolar gerçekten var ama hepsi küçük ve hiçbirinin gücü ölçülmemiş: `erkamesen/metin2-okey-ai`, `hawarmarka/okey`, `ilkinulas/okeybot`, `ISO53/AciyoMu`, `derebaba/okey`, `sevo582/Okey-game`.

**Straw**
- Üçü arasında en dikkatli olanı. Belirgin bir hata bulmadım.
- Gin Rummy kaynaklarını kontrol etmedim.

**Argon**
- **Süre tahminleri aşırı iyimser:**
  - "1 milyon el 1,5 dakikada"
  - "2–3 günde insan üstü"
  - "%31 el kazanma" gibi kesin hedefler

  Bu sayılar ölçülmemiş. Motor hızı Faz 0'da ölçülmeden hiçbirine güvenmeyelim.
- **Doğrulanamadı:**
  - "Nuray Kutlu (Amazon), Okey and Random Combinatorial Games, Ekim 2025" semineri
  - "ilkinulas/okeybot, Okey Mini uygulamasında kullanılıyor"
  - Mahjax (arXiv 2605.20577)
  - Mortal hakkındaki sayılar (40 blok, 298 bin adım, saatte 40 bin maç)

  `yunnsbz/101okey` ve Mortal deposu gerçekten var. Mortal'ın lisansı da gerçekten AGPL-3.0.
- **Çift uzaklığı formülü** `max(0, 7 − P − J)` küçük bir hata içeriyor: okeyle çift tamamlamak için eşi olmayan bir taş gerekiyor. Pratikte neredeyse hep doğru sonuç verir.
- Rummikub makalesinin yazarları "Van Rijn, Vis, Kosters" diye yazılmış. Doğrusu van Rijn, Takes, Vis.
- **Doğru ve işe yarar hesaplar:**
  - 157 temel per (48 + 44 + 52 + 13)
  - farklı 14 taşlı el sayısı ≈ 4,8 × 10¹³
  - tek renk için 3¹³ girişli bir tablo (her renkte 13 sayı, her sayıdan elde 0, 1 ya da 2 kopya). El çözücü için iyi bir fikir.

---

## 5. Raporlardan alınacak iyi fikirler

- **MiMo**
  - Kural maddelerine kimlik verip her biri için test vektörü yazmak.
  - Şans etkisini düzelten bir ölçüm terimi kullanmak (pokerdeki AIVAT'ın okey uyarlaması).
  - Aşama kapılarını sayılarla tanımlamak.
- **Straw**
  - Rakiplerin elini tahmin eden ayrı bir modül.
  - Ana ölçütün ceza farkı olması.
  - Gizli taşları gören hileli bir botla becerinin üst sınırını ölçmek.
  - Ön kayıt: testin kuralları baştan yazılır, sonuç iyi görününce yarıda durdurulmaz.
  - Çalışan TS prototipi, Rust motoru için referans olur.
- **Argon**
  - Tek renk tablosu ile shanten/ukeire hesabı (ele bitmeye kaç taş kaldığı ve eli ilerleten kaç taş olduğu).
  - Göstergenin tek eşi olduğu için o taşın daha nadir geldiğini hesaba katmak.
  - "Okeye dönme" kararının beklenen değeri: hemen bitmek yerine okeyi atarak bitmeyi beklemek.
  - Skora göre risk alma. Örnek: normal bitiş maçı bitirip başkasını kazandırıyorsa bitmemek.
  - İnsana karşı "gölge duplicate": insanın oynadığı eli arka planda botlara da oynatmak.
  - Hamle başına beklenen değer kaybı (ΔEV) ile hata analizi.
  - Arayüzde eski atıkların görünürlüğünü ayrı bir karar olarak ele almak.

---

## 6. Sonuç

Dört rapor yöntem konusunda aynı yolu gösteriyor. Kural tablosunda da 3'e 1 çoğunluk netleşti. Açık kalan iki karar 18 Ekim'de verildi:
- gösterge gösterme v1'de kapalı, ayarla açılabiliyor
- yerden alınan taş aynı turda geri atılabiliyor

Kesinleşen kural seti [KURALLAR_V1.md](KURALLAR_V1.md) dosyasında. Sıradaki adım Faz 0: Rust motoru, kural test vektörleri ve motorun saniyede kaç el oynattığının ölçümü.
