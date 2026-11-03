use env::PieToken;

use crate::Direction;

pub struct Endpoint {
    pub id: u64,
    pub capacity: usize,
    pub seed: PieToken,
    pub direction: Direction,
}
