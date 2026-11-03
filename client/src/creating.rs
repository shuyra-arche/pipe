use env::{PieToken, Wait};
use pipe_api::{Call, Fail};

use crate::Pipe;

pub struct Creating {
    pub(crate) pending: ipc::rpc::request::Pending<Call>,
    pub(crate) lease: Option<PieToken>,
}

impl Creating {
    pub fn source(&self) -> env::ToleEventSource {
        self.pending.source()
    }

    pub fn remaining(&self) -> Wait {
        self.pending.remaining()
    }

    pub fn poll(&mut self) -> Result<Option<Pipe>, Fail> {
        let Some(reply) = self.pending.poll().map_err(|_| Fail::Dead)? else {
            return Ok(None);
        };
        if reply.status != 0 {
            return Err(Fail::of(reply.status).unwrap_or(Fail::Protocol));
        }
        if reply.id == 0 || reply.capacity == 0 || reply.capacity > pipe_api::MAX_CAPACITY as u64 {
            return Err(Fail::Protocol);
        }
        Ok(Some(Pipe {
            id: reply.id,
            capacity: reply.capacity as usize,
            lease: self.lease.take().ok_or(Fail::Protocol)?,
        }))
    }
}

impl Drop for Creating {
    fn drop(&mut self) {
        if let Some(lease) = self.lease.take() {
            let _ = env::pie::release(lease, env::ReleaseMode::Revoke);
        }
    }
}
