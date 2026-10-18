//! Ayarlanabilir kurallar. Varsayılanlar KURALLAR_V1.md ile aynıdır.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// 12-13-1 serisi geçerli mi (13-1-2 her durumda geçersiz).
    pub wrap_13_1: bool,
    /// Bir perde en fazla kaç gerçek okey olabilir.
    pub max_okeys_per_meld: u8,
    /// Göstergenin eşini ilk çekişten önce gösterince rakiplerden 1'er puan.
    pub indicator_show: bool,
    /// Yerden alınan taş aynı turda geri atılabilir mi.
    pub rediscard_taken: bool,
}

impl Default for Rules {
    fn default() -> Self {
        Rules {
            wrap_13_1: true,
            max_okeys_per_meld: 2,
            indicator_show: false,
            rediscard_taken: true,
        }
    }
}
