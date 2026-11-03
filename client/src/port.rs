use core::sync::atomic::AtomicU8;
use env::{PieToken, Wait};
use pipe_api::{Direction, Endpoint, Fail, Read, Write};
use resource::dock::Dock;

pub struct Port {
    dock: Option<Dock>,
    endpoint: Endpoint,
    closed: bool,
}

impl Port {
    pub fn import(endpoint: Endpoint) -> Result<Self, Fail> {
        if endpoint.capacity == 0 || endpoint.capacity > pipe_api::MAX_CAPACITY {
            return Err(Fail::Invalid);
        }
        let data = resource::raw::inspect(endpoint.seed).map_err(|_| Fail::Dead)?;
        if !data.alive
            || data.kind != env::PieKind::Pole
            || !data
                .permission
                .contains(env::Permission::FETCH | env::Permission::STORE)
        {
            return Err(Fail::Denied);
        }
        let dock = Dock::open(endpoint.seed).map_err(|_| Fail::Dead)?;
        let port = Self {
            dock: Some(dock),
            endpoint,
            closed: false,
        };
        port.ring()?;
        Ok(port)
    }

    pub fn token(&self) -> PieToken {
        self.endpoint.seed
    }

    pub fn direction(&self) -> Direction {
        self.endpoint.direction
    }

    fn ring(&self) -> Result<stream::Ring<'_>, Fail> {
        if !resource::raw::alive(self.endpoint.seed) {
            return Err(Fail::Dead);
        }
        let view = self.dock.as_ref().ok_or(Fail::Closed)?.view();
        if self.endpoint.capacity > view.size().saturating_sub(pipe_api::HEADER_SIZE)
            || view.base() % core::mem::align_of::<pipe_api::Header>() != 0
        {
            return Err(Fail::Protocol);
        }
        // SAFETY: the held Dock bounds both slices; all shared fields use atomic access.
        let (header, bytes) = unsafe {
            (
                &*(view.base() as *const pipe_api::Header),
                core::slice::from_raw_parts(
                    (view.base() + pipe_api::HEADER_SIZE) as *const AtomicU8,
                    self.endpoint.capacity,
                ),
            )
        };
        stream::Ring::attach(header, bytes, self.endpoint.capacity).map_err(Fail::from)
    }

    fn notify(&self, bit: env::Bit) -> Result<(), Fail> {
        match env::pole::ring(self.token(), bit) {
            Ok(()) => Ok(()),
            Err(e) if e.source.is_busy() => Ok(()),
            Err(_) => Err(Fail::Dead),
        }
    }

    fn hush(&self, bit: env::Bit) -> Result<(), Fail> {
        match env::pole::hush(self.token(), bit) {
            Ok(()) => Ok(()),
            Err(e) if e.source.is_busy() => Ok(()),
            Err(_) => Err(Fail::Dead),
        }
    }

    fn bit(&self) -> env::Bit {
        match self.direction() {
            Direction::Read => pipe_api::READ_BIT,
            Direction::Write => pipe_api::WRITE_BIT,
        }
    }

    pub fn source(&self) -> env::ToleEventSource {
        env::ToleEventSource::Pole {
            token: self.token(),
            bit: self.bit(),
        }
    }

    pub fn prepare_wait(&self) -> Result<bool, Fail> {
        if self.closed {
            return Err(Fail::Closed);
        }
        if self.ring()?.ready(self.direction())? {
            return Ok(true);
        }
        self.hush(self.bit())?;
        self.ring()?.ready(self.direction()).map_err(Fail::from)
    }

    pub fn wait(&self, within: Wait) -> Result<bool, Fail> {
        let deadline = ipc::time::Deadline::new(within);
        loop {
            if self.closed {
                return Err(Fail::Closed);
            }
            if self.ring()?.ready(self.direction())? {
                return Ok(true);
            }
            self.hush(self.bit())?;
            if self.ring()?.ready(self.direction())? {
                return Ok(true);
            }
            let left = deadline.remaining();
            if left == Wait::POLL {
                return Ok(false);
            }
            env::pole::wait(self.token(), env::PoleCondition::Signal(self.bit()), left)
                .map_err(|_| Fail::Dead)?;
        }
    }

    pub fn read(&mut self, out: &mut [u8], within: Wait) -> Result<Read, Fail> {
        if self.direction() != Direction::Read {
            return Err(Fail::Denied);
        }
        let deadline = ipc::time::Deadline::new(within);
        loop {
            if self.closed {
                return Err(Fail::Closed);
            }
            match self.ring()?.read(out)? {
                Read::Pending => {
                    self.notify(pipe_api::DEMAND_BIT)?;
                    if !self.wait(deadline.remaining())? {
                        return Ok(Read::Pending);
                    }
                }
                progress => {
                    if matches!(progress, Read::Bytes(n) if n != 0) {
                        self.notify(pipe_api::WRITE_BIT)?;
                    }
                    return Ok(progress);
                }
            }
        }
    }

    pub fn write(&mut self, bytes: &[u8], within: Wait) -> Result<Write, Fail> {
        if self.direction() != Direction::Write {
            return Err(Fail::Denied);
        }
        let deadline = ipc::time::Deadline::new(within);
        loop {
            if self.closed {
                return Err(Fail::Closed);
            }
            match self.ring()?.write(bytes)? {
                Write::Pending => {
                    if !self.wait(deadline.remaining())? {
                        return Ok(Write::Pending);
                    }
                }
                progress => {
                    if matches!(progress, Write::Bytes(n) if n != 0) {
                        self.notify(pipe_api::READ_BIT)?;
                    }
                    return Ok(progress);
                }
            }
        }
    }

    pub fn close(&mut self) -> Result<(), Fail> {
        if self.closed {
            return Ok(());
        }
        self.ring()?.close(self.direction());
        self.closed = true;
        let _ = self.notify(match self.direction() {
            Direction::Read => pipe_api::WRITE_BIT,
            Direction::Write => pipe_api::READ_BIT,
        });
        Ok(())
    }
}

impl Drop for Port {
    fn drop(&mut self) {
        let _ = self.close();
        if let Some(dock) = self.dock.take() {
            let _ = dock.shut();
        }
        let _ = env::pie::release(self.endpoint.seed, env::ReleaseMode::Revoke);
    }
}
