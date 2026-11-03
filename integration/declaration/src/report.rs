/// Peer -> parent report. `seed` is installed in the receiver's table by the
/// server; the frame carries only the descriptor value.
#[derive(Clone, Copy, env::Frame)]
pub struct Report {
    pub kind: u8,
    pub code: u8,
    pub id: u64,
    pub capacity: u32,
    pub seed: [u8; 8],
}

pub const PIPE_A: u8 = 1;
pub const B_DEAD: u8 = 2;
pub const FAIL: u8 = 3;

impl wire::Message for Report {
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
