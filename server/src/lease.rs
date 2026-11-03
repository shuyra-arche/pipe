use env::{PieToken, TaskId};
use pipe_api::Fail;

pub(crate) fn lease(token: PieToken, actor: TaskId) -> Result<(), Fail> {
    let info = resource::raw::inspect(token).map_err(|_| Fail::Denied)?;
    if !info.alive
        || info.owner != actor
        || info.vestor != actor
        || info.kind != env::PieKind::Hole
        || info.mark != pipe_api::LEASE
    {
        return Err(Fail::Denied);
    }
    Ok(())
}
