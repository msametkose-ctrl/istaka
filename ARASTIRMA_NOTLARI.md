# Istaka: ilk araştırma notları

Tarih: 18 Ekim 2026. Bu notlar ilk tur araştırmanın özetidir. Diğer yapay zekâlardan gelen raporlarla karşılaştırıp kesinleştireceğiz. "(Tahmin)" diye işaretlenen yerler kaynak değil, benim hesabım ya da değerlendirmem.

---

## 1. Kurallar: standart okey (106 taş, 4 kişi)

Wikipedia'nın Okey maddesi ve Türkçe kaynaklar şu noktalarda birbirini tutuyor:

**Taşlar**
- 4 renk (kırmızı, siyah, mavi, sarı ya da yeşil) × 1–13 × 2 kopya = 104 taş, artı 2 sahte okey. Toplam 106.

**Dağıtım ve okey**
- Taşlar 21 adet 5'li yığına dizilir, 1 taş artar. Zarla bir yığın seçilir ve artan taş oraya eklenip 6'lı yapılır. İkinci zarla bu yığından bir taş seçilir ve yüzü açık şekilde üste konur: **gösterge**.
- **Okey taşı**, göstergeyle aynı renkte ve bir büyük değerdeki taştır. Gösterge 13 ise okey 1'dir. Okey joker olarak her taşın yerine geçer.
- **Sahte okey** joker değildir, sadece okey taşının kendisi yerine geçer. Örneğin okey kırmızı 5 ise sahte okey kırmızı 5 sayılır.
- Dağıtanın sağındaki oyuncu 15 taş alır, diğerleri 14'er alır. Oyun saat yönünün tersine döner.

**Sıra akışı**
- Her sırada oyuncu ya ortadaki desteden bir taş çeker ya da bir önceki oyuncunun (solundakinin) son attığı taşı alır. Sonra bir taş atar.
- Atılan taşlar, atanın sağında üst üste yığılır. Oyuncu kendi sağındaki ve solundaki yığınları inceleyebilir, karşıdaki ikisinin sadece en üst taşını görür. (Bu kural kaynağa ve uygulamaya göre değişebilir, kontrol edilmeli.)
- El açma yok. Herkes taşlarını bitirene kadar gizli tutar.

**Bitiş**
- 14 taşın tamamı perlere dizilince ve 15. taş atılınca el biter. Perler şunlar olabilir:
  - **seri:** aynı renkte ardışık en az 3 taş
  - **grup:** aynı sayıda, farklı renkte 3 ya da 4 taş
  - **çift:** aynı renk ve sayıda 7 çift
- Okey atarak bitmek (son atılan taş okey) elin değerini iki katına çıkarır.
- **Gösterme:** göstergenin aynısı elinde olan oyuncu bunu ilk sırasında gösterirse puan kazanır.

**Deste bitince:** Wikipedia'ya göre gösterge yığınında açık taş hiç çekilemez. Çekilebilecek taş kalmadığında ve sıradaki oyuncu yandan taş almak istemediğinde el **puansız biter**.

**Puanlama (Wikipedia, klasik sistem)**
- Herkes 20 puanla başlar. Biri bitirince diğerlerinden puan düşer: normal bitiş 2, okey atarak bitiş 4, çiftten bitiş 4, gösterme 1.
- Biri sıfıra düşünce oyun biter. En yüksek iki puan kazanır.
- Kahvehanelerde ve uygulamalarda farklı sistemler yaygın. Bu sistemi belirlemek gerekiyor.

### Tartışmalı ya da yere göre değişen noktalar

| Konu | Durum |
|---|---|
| **12-13-1 serisi** | Wikipedia standart okeyde geçerli diyor, 13-1-2'yi geçersiz sayıyor. 101 okeyde ikisi de geçersiz ([turnuva kuralları](https://medya.barobirlik.org.tr/barowebsite/uploads/52/kural1.pdf), [okey101cetele](https://okey101cetele.com/blog/101-okey-kurallari-kitabi)). Zynga'nın destek sayfasında bu soruya ayrılmış bir madde var ama sayfa açılmadı. |
| Çiftten bitişte okeyin kullanımı | Kesin kural bulunamadı. |
| Puanlama sistemi | Wikipedia'daki 20 puan sistemi mi, uygulamalardaki sistem mi? Belirlenmeli. |
| Karşıdaki atık yığınlarının görünürlüğü | Kaynaklar farklı anlatıyor. |
| Okeyi elde tutup okey atarak bitmeyi beklemek | Wikipedia'da var, risk/ödül kararı olarak yapay zekâ için önemli. |

**101 okey** ayrı ve daha karmaşık bir oyun: 21/22 taş, el açmak için en az 101 sayı ya da 5 çift, ceza puanları, birden çok el. Önerim ilk sürümü **standart okey** üzerine kurmak, 101'i sonra eklemek.

---

## 2. Daha önce yapılmış işler

### Doğrudan okey üzerine
- **Uygun, İ. (2019), "Playing the Turkish tile game Okey with deep reinforcement learning"**, Boğaziçi Üniversitesi yüksek lisans tezi ([YÖK Tez](https://tez.yok.gov.tr/UlusalTezMerkezi/tezDetay.jsp?id=DJjT4A637BkHZrB6BvkFHg&no=ZXrFbM87LnUDVZMBlukY5Q), [Boğaziçi arşivi](https://digitalarchive.library.bogazici.edu.tr/handle/123456789/18098)).
  - İki oyunculu, sadeleştirilmiş bir okey kullanıyor.
  - Actor-critic ve öncelikli tekrar (prioritized replay) ile eğitilmiş. Rakip rastgele oynayan bir bilgisayar, yani self-play yok.
  - Kendini okeyde pekiştirmeli öğrenme kullanan ilk çalışma olarak tanımlıyor.
  - Sonuç sayılarına ulaşamadım. PDF okunmalı.
- **"EigenBots for the Okey Social Board Game"** ([ResearchGate](https://www.researchgate.net/publication/290978148_EigenBots_for_the_Okey_Social_Board_Game)).
  - Farklı seviyelerdeki insanları taklit eden botlar yapmak üzerine. Amaç güçlü oynamak değil.
  - Ayrıntılarına ulaşamadım.
- **Açık kaynak:** GitHub'da ciddi bir okey yapay zekâsı bulamadım. Ticari uygulamaların (Zynga Okey Plus, Ahoy Okey, "Okey – İnternetsiz" gibi) botları kapalı.
- **Sonuç:** Güçlü, açık kaynak, ölçülmüş bir okey yapay zekâsı yok gibi görünüyor. Alan boş.

### Benzer oyunlar
- **Mahjong (Suphx, Microsoft, 2020)** ([arXiv 2003.13590](https://arxiv.org/abs/2003.13590)). Okeye en yakın büyük başarı: 4 kişi, gizli taşlar, çek-at döngüsü. Tenhou'da resmi sıralamadaki insanların %99,99'unun üstüne çıktı. İki önemli fikri var:
  - **oracle guiding:** eğitim, gizli taşları gören bir ajanla başlıyor ve bu bilgi yavaş yavaş elinden alınıyor
  - **global reward prediction:** tek elin değil, tüm oyunun sonucunu tahmin eden bir ödül modeli
- **Mortal**: açık kaynak, güçlü bir Riichi mahjong yapay zekâsı. (Kendi bilgim, bu turda kaynağına bakmadım.)
- **Indian Rummy (IRumAI, 2026)** ([arXiv 2606.21975](https://arxiv.org/abs/2606.21975)).
  - İki oyunculu rummy. Önce davranış klonlama ile ısıtılıyor, sonra zayıf ama hızlı sezgisel rakiplerden oluşan bir havuza karşı PPO ile eğitiliyor.
  - Girdi olarak açık per tamamlama bilgilerini (eksik taşlar, yarım perler) veriyorlar.
  - Hamle başına 0,33 ms ile çok hızlı.
- **DouZero (Dou Dizhu, 2021)**: Basit Monte Carlo hedefleriyle derin ağ eğitiliyor, çok sayıda paralel self-play kullanılıyor. Çok oyunculu kart oyunlarında güçlü sonuç verdi. (Kendi bilgim.)
- **Rummikub**: Bir elden en iyi perleri çıkarma problemi polinom zamanda çözülebiliyor ([van Rijn, Takes, Vis, BNAIC 2015](https://liacs.leidenuniv.nl/~visjk/bnaic2015.pdf)). Bunun için tamsayılı programlama çözücüleri de var ([rummikub-solver](https://rummikub-solver.readthedocs.io/)). Okeyde el dizme problemi bunun daha küçük bir hali.
- **ISMCTS** (Cowling, Powley, Whitehouse, 2012, IEEE TCIAIG) ([White Rose](https://eprints.whiterose.ac.uk/75048)).
  - Gizli bilgili oyunlarda MCTS'in standart uyarlaması. Durumlar yerine bilgi kümeleri üzerinde arama yapıyor.
  - Gizli bilgiyi rastgele tamamlayıp her biri için ayrı arama yapan yöntemin (PIMC) iki sorununu, strategy fusion ve non-locality'yi hafifletiyor.
- **ReBeL** (Brown ve ark., NeurIPS 2020) ([arXiv 2007.13544](https://arxiv.org/abs/2007.13544)). Güçlü bir yöntem ama teorik garantisi iki kişilik sıfır toplamlı oyunlarla sınırlı. Ortak bilgi az olduğunda girdisi şişiyor. Dört kişilik okey için doğrudan uygun görünmüyor.

---

## 3. Problemin kaba analizi (tahmin)

- **Deste büyüklüğü:** 106 − 1 gösterge − 57 dağıtılan = **48 taş**. Kimse bitiremezse bir el en fazla yaklaşık 48 çekiş sürer, oyuncu başına yaklaşık 12 tur. Eller kısa, bu da hızlı simülasyon demek.
- **Seçenek sayısı:** Her sırada 2 çekme seçeneği (deste ya da yandan) ve en fazla 15 farklı atış var. Tur başına en fazla yaklaşık 30 hamle. Satrançtan çok daha dar.
- **Gizli bilgi:** Kendi 14 taşını, göstergeyi ve atıkları biliyorsun. Geri kalan yaklaşık 70–90 taş, rakiplerin elleri ile deste arasında dağılmış. Ama rakipler bilgi sızdırıyor: yandan aldıkları taşlar, attıkları ve atmadıkları taşlar.
- **Asıl zor kısım:** Hamle sayısı az ama gizli bilgi çok, şans payı yüksek. Ayrıca 4 oyuncu var ve herkes kendisi için oynuyor. Tehlikeli taşı atmamak (sağdakinin işine yarayacak taşı vermemek) ile kendi elini ilerletmek arasında sürekli bir denge kurmak gerekiyor.
- **El değerlendirme:** "Bu 14 taş bitmeye ne kadar yakın, hangi taşlar bekleniyor" sorusu çok sık sorulacak. Tek renkli seriler ve gruplarla 2 joker hesaba katılarak hızlı bir dinamik programlama ya da önceden hesaplanmış tablolar ile çözülebilir. Gedik ve Pulat'taki gibi bunun da Rust'ta yazılması mantıklı.
- **Şans:** Okeyde tek elin sonucu büyük ölçüde şansa bağlı. Gücü güvenilir ölçmek için çok sayıda el ve şansı dengeleyen bir düzen gerekecek: aynı karılmış desteyi oyuncuların yerleri değiştirilerek tekrar oynatmak (duplicate).

---

## 4. Gücü ölçme fikirleri

- **Duplicate düzen:** Aynı karılmış deste, oyuncular 4 koltuğa sırayla yerleştirilerek oynanır. Kart oyunlarında şans etkisini azaltmanın standart yolu.
- **Rakip havuzu:** rastgele oyuncu, açgözlü oyuncu (en çok per tamamlayan atışı yapan), güvenli oyuncu (tehlikeli taş atmayan), eski Istaka sürümleri.
- **Ölçüler:** el kazanma oranı, ortalama puan farkı, güven aralıkları.
- **İnsanlara karşı:** Tarayıcı arayüzü açılınca gerçek oyuncu verisi toplanabilir.

---

## 5. Belirlenmesi gerekenler

1. Kural seti: 12-13-1, puanlama sistemi ve atık yığınlarının görünürlüğü.
2. İlk sürüm standart okey mi olsun? (Önerim evet.)
3. Diğer yapay zekâların raporları gelince yöntemi birlikte seçelim. Bu notlar bilerek belirli bir yöntemi dayatmıyor.

## Kaynaklar

- [Okey – Wikipedia](https://en.wikipedia.org/wiki/Okey)
- [Eşli 101 turnuva kuralları (Barolar Birliği)](https://medya.barobirlik.org.tr/barowebsite/uploads/52/kural1.pdf)
- [101 Okey Kuralları – okey101cetele](https://okey101cetele.com/blog/101-okey-kurallari-kitabi)
- [Uygun 2019 tezi – YÖK](https://tez.yok.gov.tr/UlusalTezMerkezi/tezDetay.jsp?id=DJjT4A637BkHZrB6BvkFHg&no=ZXrFbM87LnUDVZMBlukY5Q)
- [EigenBots for Okey – ResearchGate](https://www.researchgate.net/publication/290978148_EigenBots_for_the_Okey_Social_Board_Game)
- [Suphx – arXiv 2003.13590](https://arxiv.org/abs/2003.13590)
- [IRumAI – arXiv 2606.21975](https://arxiv.org/abs/2606.21975)
- [Rummikub karmaşıklığı – BNAIC 2015](https://liacs.leidenuniv.nl/~visjk/bnaic2015.pdf)
- [ISMCTS – Cowling, Powley, Whitehouse 2012](https://eprints.whiterose.ac.uk/75048)
- [ReBeL – arXiv 2007.13544](https://arxiv.org/abs/2007.13544)
