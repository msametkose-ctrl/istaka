# Istaka: Okey yapay zekâsı için araştırma görevi

## Biz kimiz, ne yapıyoruz

Klasik masa oyunları için güçlü, açık kaynak yapay zekâ motorları yazan küçük bir ekibiz. Daha önce Quoridor ve tavla için birer motor yazıp GitHub'da yayınladık. Yeni projemizin adı **Istaka**: **Okey** oynayan, olabildiğince güçlü bir yapay zekâ.

Hedefimiz şunlar:

- Okeyi kurallarına tam uyarak oynayan bir oyun motoru.
- İyi insan oyuncuları düzenli olarak yenen, mümkünse insan seviyesinin üstüne çıkan bir yapay zekâ oyuncusu.
- Bu oyuncunun gücünü sayılarla, güvenilir şekilde ölçebileceğimiz bir test düzeni.
- Tarayıcıda Istaka'ya karşı (ve Istaka'larla aynı masada) oynanabilen bir arayüz. Arayüz isteğe bağlı olarak "burada hangi taşı atmalıydım, elimin bitme ihtimali ne" gibi analizler de gösterebilmeli.
- Her şey açık kaynak olacak ve tek bir ev bilgisayarında (çok çekirdekli işlemci, tek bir tüketici ekran kartı) eğitilip çalıştırılabilecek. Bulut kümesi ya da büyük bütçe yok.

Senden bu projenin başında ihtiyacımız olan araştırmayı yapmanı istiyoruz. **Yöntemi sana bırakıyoruz.** Hangi yaklaşımın, tekniğin ya da araçların kullanılması gerektiğine dair bir varsayımımız yok. Kendi analizine göre en iyi yolu bulmanı ve gerekçelendirmeni istiyoruz. Alışılmış bir yaklaşımı da önerebilirsin, yepyeni bir yaklaşımı da. Önemli olan, neden en iyisi olduğunu ikna edici biçimde anlatman.

## Araştırmanı istediğimiz konular

### 1. Oyunun kuralları (en önemli kısım)

Motoru yazmadan önce kuralları eksiksiz ve kesin bilmemiz gerekiyor. Lütfen şunları yap:

- Türkiye'de en yaygın oynanan **standart okeyin** (106 taş, 4 oyuncu) kurallarını, bir programcının hiç soru sormadan kodlayabileceği kesinlikte yaz. En az şunlar yer almalı:
  - taş seti ve sahte okeyler
  - dağıtım, kimin 15 taşla başladığı, göstergenin nasıl belirlendiği ve okey taşının nasıl bulunduğu
  - geçerli perler: seriler ve gruplar, en az kaç taş gerektiği, 13'ten 1'e geçişin olup olmadığı (12-13-1 ve 13-1-2 gibi), okey taşının joker olarak kullanımı, sahte okeyin değeri
  - sıra akışı: desteden ya da soldaki oyuncunun attığı taştan çekme, taş atma, kimin hangi taşı alabildiği
  - bitiş türleri: normal bitiş, çiftten bitiş, okey atarak bitiş, gösterge gösterme
  - deste bittiğinde ne olduğu
  - puanlama ve ceza sistemi, oyunun kaç el sürdüğü, ve bu kuralların hangi noktalarda masadan masaya değiştiği
- **101 okeyi**, eşli okeyi ve diğer yaygın varyantları ayrı ayrı özetle. Standarttan farklarını açıkça belirt.
- Kuralların tartışmalı olduğu ya da yöreye, uygulamaya göre değiştiği noktaları tek tek listele. Her biri için en yaygın uygulamayı ve bunun kaynağını ver.
- Popüler okey uygulamalarının (mobil ve web) hangi kural setini kullandığını araştır. Bunlar fiilen bir standart oluşturmuş olabilir.
- Sonunda bizim için **önerdiğin tek bir kural setini** (ilk sürümün hedefi) gerekçesiyle tanımla. Ayrıca motorun ileride hangi seçenekleri ayarlanabilir şekilde desteklemesi gerektiğini söyle.

### 2. Daha önce yapılmış işler

- Okey ve 101 okey üzerine yapılmış her türlü yapay zekâ çalışmasını bul: makaleler, yüksek lisans ve doktora tezleri, açık kaynak projeler, blog yazıları, ticari uygulamaların botları hakkında bilinenler. Her biri için ne yaptığını, ne kadar güçlü olduğunu (ölçüldüyse), eksiklerini ve bize ne kattığını yaz.
- Okeye benzeyen oyunlarda yapılmış önemli çalışmaları da araştır: Rummikub, rummy ailesi, mahjong, ve gizli bilgi içeren, çok oyunculu, şansa dayalı diğer kart ve taş oyunları. Bu oyunlarda insan seviyesine ya da üstüne nasıl çıkıldığını, neyin işe yarayıp neyin yaramadığını özetle. Bu deneyimlerden hangilerinin okeye aktarılabileceğini, hangilerinin aktarılamayacağını ve nedenini tartış.
- Bu alanda bildiğin, bizim gözden kaçırabileceğimiz önemli kaynak, veri seti, yarışma ya da araç varsa ekle.

### 3. Problemin analizi

Okeyi bir yapay zekâ problemi olarak incele:

- Bir oyuncunun neyi bilip neyi bilmediğini, hangi bilgilerin herkese açık olduğunu, rakiplerin ellerine dair neyin çıkarılabileceğini anlat.
- Oyunun büyüklüğünü ve zorluğunu yaklaşık olarak hesapla: olası durum sayısı, her hamledeki seçenek sayısı, bir elin ortalama uzunluğu, gizli bilginin miktarı.
- Okeyde şansın ve becerinin payı ne kadar? Güçlü bir oyuncu zayıf bir oyuncuya karşı ne kadar avantaj sağlayabilir?
- Oyunun dört kişilik ve herkesin kendi için oynadığı bir oyun olmasının (eşli versiyonda takım oyunu olmasının) yapay zekâ açısından getirdiği zorlukları açıkla.
- Okeyi zorlaştıran ve kolaylaştıran özel noktalar neler? Örneğin el dizme (bir elin en iyi şekilde perlere ayrılması) problemi ne kadar zor, nasıl hızlı çözülür?
- İyi bir insan okey oyuncusunun kullandığı strateji ve taktikleri (taş sayma, rakibin ne topladığını okuma, tehlikeli taşı atmama, ne zaman çifte gidilir gibi) araştır. Bunlar bize yapay zekânın öğrenmesi gereken davranışlar hakkında fikir verir.

### 4. Gücü nasıl ölçeceğiz

- Şansın payı yüksek bir oyunda bir oyuncunun diğerinden gerçekten daha iyi olduğunu nasıl güvenilir şekilde gösteririz? Kaç oyun gerekir, şansın etkisini azaltmanın yolları neler?
- Hangi rakiplere karşı test etmeliyiz (basit kural tabanlı botlar, mevcut uygulamalar, insanlar)? Bunları nasıl kurarız?
- Tek tek hamlelerin kalitesini ölçmenin bir yolu var mı?
- Projenin her aşamasında "başardık" diyebileceğimiz somut ve ölçülebilir hedefler öner.

### 5. Senin önerin

Yukarıdaki her şeye dayanarak:

- Istaka'yı nasıl yapmamızı önerirsin? Önerdiğin yaklaşımı, neden seçtiğini ve neden diğer seçeneklerden iyi olduğunu ayrıntılı açıkla. Düşünüp elediğin alternatifleri ve eleme nedenlerini de yaz.
- Tek bir ev bilgisayarıyla bu hedefe ulaşmak gerçekçi mi? Kabaca ne kadar hesaplama ve süre gerekir?
- Projeyi aşamalara böl: her aşamada ne yapılır, ne çıkar, nasıl test edilir.
- En büyük riskleri ve takılabileceğimiz noktaları, her biri için ne yapılabileceğiyle birlikte listele.
- Ekibin cevaplaması gereken açık soruları ve vermesi gereken kararları belirt.

## Yanıtın biçimi

- Türkçe yaz. Teknik terimlerin İngilizcesini ilk geçtiği yerde parantez içinde verebilirsin.
- Yukarıdaki beş başlığı koru, sonuna kısa bir yönetici özeti ekle.
- Kural bölümünü, doğrudan koda dökülebilecek kesinlikte ve düzende yaz.
- Her önemli iddia için kaynak göster (bağlantı, makale adı, yazar, yıl). Kaynağı olmayan çıkarımlarını ve tahminlerini açıkça "tahmin" ya da "kendi değerlendirmem" diye işaretle.
- Emin olmadığın şeyleri uydurma. "Bulamadım" ya da "emin değilim" demek, yanlış bilgi vermekten çok daha değerli.
- Uzunluk sınırı yok. Ayrıntı ve doğruluk kısalıktan daha önemli.
