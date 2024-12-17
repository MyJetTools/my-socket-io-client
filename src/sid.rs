use rand::Rng;

pub struct SocketIoSid(String);

impl SocketIoSid {
    pub fn new() -> Self {
        let mut sid = String::new();
        let mut rng = rand::thread_rng();
        for _ in 0..20 {
            let random_char: char = std::char::from_u32(rng.gen_range(65..90)).unwrap();
            sid.push(random_char);
        }
        Self(sid)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }

    pub fn into_string(self) -> String {
        self.0
    }
}
