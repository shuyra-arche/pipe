use crate::{Direction, READ, WRITE};

pub fn direction(raw: u8) -> Option<Direction> {
    match raw {
        READ => Some(Direction::Read),
        WRITE => Some(Direction::Write),
        _ => None,
    }
}

pub fn code(direction: Direction) -> u8 {
    match direction {
        Direction::Read => READ,
        Direction::Write => WRITE,
    }
}
