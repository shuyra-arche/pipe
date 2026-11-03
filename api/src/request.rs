use env::{PieToken, TaskId};

#[derive(Clone, Copy, env::Frame)]
pub struct Request {
    pub op: u8,
    pub id: u64,
    pub capacity: u64,
    pub target: TaskId,
    pub direction: u8,
    pub lease: PieToken,
    pub back: PieToken,
}

impl wire::Message for Request {
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
