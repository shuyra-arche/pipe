use env::{PieToken, Wait};
use pipe_api::{Call, Endpoint, Fail, Reply, Request};

use crate::{Creating, Direction, Pipe, Port};

pub struct Client {
    sender: ipc::rpc::request::Sender<Call>,
}

impl Client {
    pub fn of(entry: PieToken) -> Result<Self, Fail> {
        Ok(Self {
            sender: ipc::rpc::request::Sender::from_raw(entry, Call::BACK)
                .map_err(|_| Fail::Denied)?,
        })
    }

    pub fn find(tree: &system_client::operator::Face, within: Wait) -> Result<Self, Fail> {
        let base = tree
            .resolve(system_api::namespace::SERVICE, within)
            .map_err(|_| Fail::Denied)?;
        let entry = tree
            .tile_from(base, system_api::operator::Path::new(pipe_api::DIR), within)
            .and_then(|t| t.token(within))
            .map_err(|_| Fail::Denied)?;
        Self::of(entry)
    }

    fn call(&self, mut request: Request, within: Wait) -> Result<Reply, Fail> {
        let reply = self
            .sender
            .call(ipc::time::Deadline::new(within), |back| {
                request.back = back;
                request
            })
            .map_err(|_| Fail::Dead)?;
        if reply.status != 0 {
            return Err(Fail::of(reply.status).unwrap_or(Fail::Protocol));
        }
        Ok(reply)
    }

    fn request(op: u8, id: u64) -> Request {
        Request {
            op,
            id,
            capacity: 0,
            target: env::unit::self_id(),
            direction: 0,
            lease: PieToken::NONE,
            back: PieToken::NONE,
        }
    }

    fn lease(&self) -> Result<(PieToken, PieToken), Fail> {
        let local =
            env::hole::unseal(pipe_api::LEASE, env::HoleLimits::default()).map_err(|_| Fail::Full)?;
        match env::pie::accord(
            local,
            self.sender.peer(),
            env::Permission::FETCH,
            pipe_api::LEASE,
        ) {
            Ok(remote) => Ok((local, remote)),
            Err(_) => {
                let _ = env::pie::release(local, env::ReleaseMode::Revoke);
                Err(Fail::Denied)
            }
        }
    }

    pub fn create(&self, capacity: usize, within: Wait) -> Result<Pipe, Fail> {
        let (local, remote) = self.lease()?;
        let mut request = Self::request(pipe_api::CREATE, 0);
        request.capacity = capacity as u64;
        request.lease = remote;
        match self.call(request, within) {
            Ok(reply) if reply.id != 0 => Ok(Pipe {
                id: reply.id,
                capacity: reply.capacity as usize,
                lease: local,
            }),
            result => {
                let _ = env::pie::release(local, env::ReleaseMode::Revoke);
                Err(result.err().unwrap_or(Fail::Protocol))
            }
        }
    }

    pub fn begin_create(&self, capacity: usize, within: Wait) -> Result<Creating, Fail> {
        let (local, remote) = self.lease()?;
        let mut request = Self::request(pipe_api::CREATE, 0);
        request.capacity = capacity as u64;
        request.lease = remote;
        match self.sender.begin(ipc::time::Deadline::new(within), |back| {
            request.back = back;
            request
        }) {
            Ok(pending) => Ok(Creating {
                pending,
                lease: Some(local),
            }),
            Err(_) => {
                let _ = env::pie::release(local, env::ReleaseMode::Revoke);
                Err(Fail::Dead)
            }
        }
    }

    pub fn begin_release(
        &self,
        pipe: &Pipe,
        within: Wait,
    ) -> Result<ipc::rpc::request::Pending<Call>, Fail> {
        let mut request = Self::request(pipe_api::RELEASE, pipe.id);
        self.sender
            .begin(ipc::time::Deadline::new(within), |back| {
                request.back = back;
                request
            })
            .map_err(|_| Fail::Dead)
    }

    pub fn begin_bind(
        &self,
        pipe: &Pipe,
        direction: Direction,
        task: env::TaskId,
        within: Wait,
    ) -> Result<ipc::rpc::request::Pending<Call>, Fail> {
        let mut request = Self::request(pipe_api::BIND, pipe.id);
        request.direction = pipe_api::code(direction);
        request.target = task;
        self.sender
            .begin(ipc::time::Deadline::new(within), |back| {
                request.back = back;
                request
            })
            .map_err(|_| Fail::Dead)
    }

    pub fn bind(
        &self,
        pipe: &Pipe,
        direction: Direction,
        task: env::TaskId,
        within: Wait,
    ) -> Result<Endpoint, Fail> {
        let mut request = Self::request(pipe_api::BIND, pipe.id);
        request.direction = pipe_api::code(direction);
        request.target = task;
        let reply = self.call(request, within)?;
        Ok(Endpoint {
            id: pipe.id,
            capacity: pipe.capacity,
            seed: reply.seed,
            direction,
        })
    }

    pub fn close(&self, pipe: &Pipe, direction: Direction, within: Wait) -> Result<(), Fail> {
        let mut request = Self::request(pipe_api::CLOSE, pipe.id);
        request.direction = pipe_api::code(direction);
        self.call(request, within).map(|_| ())
    }

    pub fn release(&self, pipe: Pipe, within: Wait) -> Result<(), Fail> {
        self.call(Self::request(pipe_api::RELEASE, pipe.id), within)
            .map(|_| ())
    }

    pub fn import(&self, endpoint: Endpoint, within: Wait) -> Result<Port, Fail> {
        let port = Port::import(endpoint)?;
        let _ = within;
        Ok(port)
    }
}
