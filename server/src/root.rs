use env::PieToken;
use pipe_api::Direction;
use resource::pile::Pile;

use crate::grant::Grant;

pub(crate) struct Root {
    pub(crate) id: u64,
    pub(crate) token: PieToken,
    pub(crate) address: usize,
    pub(crate) capacity: usize,
    pub(crate) owner_lease: PieToken,
    pub(crate) read: Option<Grant>,
    pub(crate) write: Option<Grant>,
}

impl Root {
    pub(crate) fn create(id: u64, capacity: usize, lease: PieToken) -> Result<Self, pipe_api::Fail> {
        let size = (pipe_api::HEADER_SIZE + capacity).div_ceil(env::PAGE_SIZE) * env::PAGE_SIZE;
        let token = ::resource::raw::pole(size, true).map_err(|_| pipe_api::Fail::Full)?;
        let (address, mapped) = match resource::raw::open(token) {
            Ok(view) => view,
            Err(_) => {
                let _ = env::pie::release(token, env::ReleaseMode::Revoke);
                return Err(pipe_api::Fail::Full);
            }
        };
        let root = Self {
            id,
            token,
            address,
            capacity,
            owner_lease: lease,
            read: None,
            write: None,
        };
        if mapped < pipe_api::HEADER_SIZE + capacity
            || address % core::mem::align_of::<pipe_api::Header>() != 0
        {
            return Err(pipe_api::Fail::Protocol);
        }
        root.header().initialize(capacity)?;
        Ok(root)
    }

    pub(crate) fn header(&self) -> &pipe_api::Header {
        // SAFETY: this service retains the root mapping until Root is dropped.
        unsafe { &*(self.address as *const pipe_api::Header) }
    }

    pub(crate) fn close(&self, direction: Direction) {
        self.header().close(direction);
        let bit = match direction {
            Direction::Read => pipe_api::WRITE_BIT,
            Direction::Write => pipe_api::READ_BIT,
        };
        let _ = env::pole::ring(self.token, bit);
    }

    pub(crate) fn detach(&self, pile: &Pile) {
        let _ = pile.detach(env::ToleEventSource::Inspect {
            task: env::unit::self_id(),
            token: self.owner_lease,
        });
        for grant in [&self.read, &self.write].into_iter().flatten() {
            let _ = pile.detach(env::ToleEventSource::Inspect {
                task: grant.target,
                token: grant.seed,
            });
        }
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = env::pole::shut(self.token);
        let _ = env::pie::release(self.token, env::ReleaseMode::Revoke);
        let _ = env::pie::release(self.owner_lease, env::ReleaseMode::Revoke);
    }
}
