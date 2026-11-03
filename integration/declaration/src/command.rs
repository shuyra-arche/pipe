/// Parent -> peer command descriptor.
#[derive(Clone, Copy, env::Frame)]
pub struct Command {
    pub kind: u8,
    pub id: u64,
    pub capacity: u32,
    pub seed: [u8; 8],
}

pub const BIND_B: u8 = 1;
pub const RELEASED_B: u8 = 2;

impl wire::Message for Command {
    type In = Self;
    type Buf = [u8; Self::LEN];
    const EMPTY: Self::Buf = [0; Self::LEN];

    fn store(&self, out: &mut [u8]) -> Option<usize> {
        env::wire::Span::store_at(self, out, 0)
    }

    fn fetch(bytes: &[u8]) -> Option<Self> {
        let (value, end) = env::wire::Span::fetch_at(bytes, 0)?;
        (end == bytes.len()).then_some(value)
    }
}
