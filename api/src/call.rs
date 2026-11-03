use crate::{Reply, Request, BACK};

fn reply_to(request: &Request) -> env::PieToken {
    request.back
}

#[mold::contract(request = Request, response = Reply, mark = BACK, back = reply_to)]
pub struct Call;
