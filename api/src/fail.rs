#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fail {
    Denied,
    Dead,
    Full,
    Invalid,
    Protocol,
    Closed,
    BrokenPipe,
}

impl Fail {
    pub const fn code(self) -> u8 {
        match self {
            Self::Denied => 1,
            Self::Dead => 2,
            Self::Full => 3,
            Self::Invalid => 4,
            Self::Protocol => 5,
            Self::Closed => 6,
            Self::BrokenPipe => 7,
        }
    }

    pub fn of(code: u8) -> Option<Self> {
        Some(match code {
            1 => Self::Denied,
            2 => Self::Dead,
            3 => Self::Full,
            4 => Self::Invalid,
            5 => Self::Protocol,
            6 => Self::Closed,
            7 => Self::BrokenPipe,
            _ => return None,
        })
    }
}

impl From<stream::Fail> for Fail {
    fn from(fail: stream::Fail) -> Self {
        match fail {
            stream::Fail::Invalid => Self::Invalid,
            stream::Fail::Corrupt => Self::Protocol,
            stream::Fail::Closed => Self::Closed,
            stream::Fail::BrokenPipe => Self::BrokenPipe,
        }
    }
}
