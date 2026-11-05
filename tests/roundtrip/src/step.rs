#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Step {
    Session = 1,
    Find,
    Create,
    BindRead,
    BindWrite,
    ImportRead,
    ImportWrite,
    Write,
    Read,
    Contents,
    CloseWrite,
    Eof,
    CloseRead,
    ReleaseWrite,
    ReleaseRead,
    Release,
    Revocation,
}

impl Step {
    pub fn fail(self, fail: pipe_api::Fail) -> env::Reason {
        0x50_0000 | ((self as usize) << 8) | usize::from(fail.code())
    }
}
