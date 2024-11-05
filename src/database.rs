use crate::source::{Currency, Rate, RateType, Source};
use std::{collections::HashMap, sync::Arc, time::SystemTime};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Database {
    data: Mutex<Data>,
    cache: Mutex<Cache>,
}

#[derive(Debug)]
pub struct Data {
    rates: HashMap<Source, Vec<Rate>>,
    updated_at: SystemTime,
}

impl Data {
    fn get_rates(&self) -> HashMap<Source, Vec<Rate>> {
        self.rates.clone()
    }

    fn set_rates(&mut self, src: Source, rates: Vec<Rate>) {
        self.rates.insert(src, rates);
        self.updated_at = SystemTime::now();
    }

    fn get_updated_at(&self) -> SystemTime {
        self.updated_at
    }
}

#[derive(Debug)]
pub struct Cache {
    conv: HashMap<String, String>,
    src: HashMap<String, String>,
}

impl Cache {
    const KEY_SEP: &'static str = "_";

    fn clear(&mut self) {
        self.conv.clear();
        self.src.clear();
    }

    fn add_conv(
        &mut self,
        from: Currency,
        to: Currency,
        rate_type: RateType,
        inv: bool,
        value: String,
    ) {
        self.conv
            .insert(self.format_conv_key(from, to, rate_type, inv), value);
    }

    fn add_src(&mut self, src: Source, rate_type: RateType, value: String) {
        self.src.insert(self.format_src_key(src, rate_type), value);
    }

    fn format_src_key(&self, src: Source, rate_type: RateType) -> String {
        [
            src.to_string().to_lowercase(),
            (rate_type as u8).to_string(),
        ]
        .join(Self::KEY_SEP)
    }

    fn get_conv(
        &self,
        from: Currency,
        to: Currency,
        rate_type: RateType,
        inv: bool,
    ) -> Option<String> {
        self.conv
            .get(&self.format_conv_key(from, to, rate_type, inv))
            .cloned()
    }

    fn get_src(&self, src: Source, rate_type: RateType) -> Option<String> {
        self.src.get(&self.format_src_key(src, rate_type)).cloned()
    }

    fn format_conv_key(
        &self,
        from: Currency,
        to: Currency,
        rate_type: RateType,
        inv: bool,
    ) -> String {
        [
            from.to_string().to_lowercase(),
            to.to_string().to_uppercase(),
            (rate_type as u8).to_string(),
            (inv as i32).to_string(),
        ]
        .join(Self::KEY_SEP)
    }
}

impl Database {
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            data: Mutex::new(Data {
                rates: HashMap::new(),
                updated_at: SystemTime::now(),
            }),
            cache: Mutex::new(Cache {
                conv: HashMap::new(),
                src: HashMap::new(),
            }),
        })
    }

    pub async fn get_rates(&self) -> HashMap<Source, Vec<Rate>> {
        let data = self.data.lock().await;
        data.get_rates()
    }

    pub async fn set_rates(&self, src: Source, rates: Vec<Rate>) {
        let mut data = self.data.lock().await;
        data.set_rates(src, rates);
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }

    pub async fn get_cache_src(&self, src: Source, rate_type: RateType) -> Option<String> {
        let cache = self.cache.lock().await;
        cache.get_src(src, rate_type)
    }

    pub async fn get_cache_conv(
        &self,
        from: Currency,
        to: Currency,
        rate_type: RateType,
        inv: bool,
    ) -> Option<String> {
        let cache = self.cache.lock().await;
        cache.get_conv(from, to, rate_type, inv)
    }

    pub async fn set_cache_src(&self, src: Source, rate_type: RateType, value: String) {
        let mut cache = self.cache.lock().await;
        cache.add_src(src, rate_type, value);
    }

    pub async fn set_cache_conv(
        &self,
        from: Currency,
        to: Currency,
        rate_type: RateType,
        inv: bool,
        value: String,
    ) {
        let mut cache = self.cache.lock().await;
        cache.add_conv(from, to, rate_type, inv, value);
    }

    pub async fn get_updated_at(&self) -> SystemTime {
        let data = self.data.lock().await;
        data.get_updated_at()
    }
}
