use env::{PieToken, TaskId};

pub(crate) struct Grant {
    pub(crate) seed: PieToken,
    pub(crate) target: TaskId,
}
