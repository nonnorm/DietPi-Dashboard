use std::{
    collections::HashMap,
    ops::DerefMut,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

pub struct LoginMap(HashMap<[u8; 12], Instant>);

impl LoginMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn new_token(&mut self) -> String {
        let now = Instant::now();
        let bytes: [u8; 12] = rand::random();

        self.0.insert(bytes, now);

        data_encoding::HEXLOWER.encode(&bytes)
    }

    pub fn contains_token(&mut self, token: &str) -> bool {
        let now = Instant::now();
        self.0
            .retain(|_, earlier| now.duration_since(*earlier) < Duration::from_secs(3600));

        let Ok(bytes) = data_encoding::HEXLOWER.decode(token.as_bytes()) else {
            return false;
        };
        let Ok(bytes) = <[u8; 12]>::try_from(bytes) else {
            return false;
        };

        self.0.contains_key(&bytes)
    }
}

#[derive(Clone)]
pub struct SharedLoginMap(Arc<Mutex<LoginMap>>);

impl SharedLoginMap {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(LoginMap::new())))
    }

    pub fn get(&self) -> impl DerefMut<Target = LoginMap> {
        self.0.lock().unwrap()
    }
}
