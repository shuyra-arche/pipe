use env::PieToken;

pub struct Pipe {
    pub id: u64,
    pub capacity: usize,
    pub(crate) lease: PieToken,
}

impl Drop for Pipe {
    fn drop(&mut self) {
        let _ = env::pie::release(self.lease, env::ReleaseMode::Revoke);
    }
}
