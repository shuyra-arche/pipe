#[derive(Clone)]
pub struct Record {
    pub id: u64,
    pub owner: usize,
    pub read: Option<usize>,
    pub write: Option<usize>,
}
