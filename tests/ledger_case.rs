#[cfg(test)]
mod tests {
    use crate::ledger::{Ledger, LIMIT};
    use pipe_api::{Direction, Fail, MAX_CAPACITY};

    #[test]
    fn pipe_registry_authorizes_owner_and_keeps_roles_exclusive() {
        let mut ledger = Ledger::default();
        let id = ledger.create(1, 4).unwrap();
        assert!(ledger.bind(id, 2, Direction::Read, 3).is_err());
        ledger.bind(id, 1, Direction::Read, 3).unwrap();
        assert!(ledger.bind(id, 1, Direction::Read, 3).is_err());
        assert!(ledger.bind(id, 1, Direction::Read, 4).is_err());
        ledger.bind(id, 1, Direction::Write, 4).unwrap();
        ledger.release(id);
        assert!(ledger.record(id).is_err());
    }

    #[test]
    fn create_rejects_invalid_owner_and_capacity() {
        let mut ledger = Ledger::default();
        assert_eq!(ledger.create(0, 4).unwrap_err(), Fail::Invalid);
        assert_eq!(ledger.create(1, 0).unwrap_err(), Fail::Invalid);
        assert_eq!(
            ledger.create(1, MAX_CAPACITY + 1).unwrap_err(),
            Fail::Invalid
        );
        assert!(ledger.create(1, MAX_CAPACITY).is_ok());
    }

    #[test]
    fn create_rejects_global_and_per_owner_limits() {
        let mut global = Ledger::default();
        for owner in 1..=LIMIT {
            global.create(owner, 1).unwrap();
        }
        assert_eq!(global.create(LIMIT + 1, 1).unwrap_err(), Fail::Full);

        let mut per_owner = Ledger::default();
        for _ in 0..64 {
            per_owner.create(9, 1).unwrap();
        }
        assert_eq!(per_owner.create(9, 1).unwrap_err(), Fail::Full);
        assert!(per_owner.create(10, 1).is_ok());
    }

    #[test]
    fn unknown_ids_are_dead_and_wrong_owner_is_denied() {
        let mut ledger = Ledger::default();
        let id = ledger.create(1, 8).unwrap();
        assert!(matches!(ledger.record(u64::MAX), Err(Fail::Dead)));
        assert_eq!(ledger.authorize(id, 2).unwrap_err(), Fail::Denied);
        assert!(ledger.authorize(id, 1).is_ok());
        assert!(matches!(
            ledger.bind(id + 1, 1, Direction::Read, 3),
            Err(Fail::Dead)
        ));
        assert_eq!(
            ledger.bind(id, 1, Direction::Read, 0).unwrap_err(),
            Fail::Invalid
        );
    }

    #[test]
    fn roles_stay_reserved_for_the_whole_pipe_lifetime() {
        let mut ledger = Ledger::default();
        let id = ledger.create(1, 4).unwrap();
        ledger.bind(id, 1, Direction::Read, 3).unwrap();
        assert_eq!(ledger.target(id, Direction::Read).unwrap(), Some(3));
        assert_eq!(
            ledger.bind(id, 1, Direction::Read, 3).unwrap_err(),
            Fail::Denied
        );
        assert_eq!(
            ledger.bind(id, 1, Direction::Read, 4).unwrap_err(),
            Fail::Denied
        );
        assert_eq!(ledger.target(id, Direction::Write).unwrap(), None);
        ledger.bind(id, 1, Direction::Write, 4).unwrap();
        assert_eq!(ledger.target(id, Direction::Write).unwrap(), Some(4));

        ledger.release(id);
        assert!(matches!(ledger.record(id), Err(Fail::Dead)));
        let next = ledger.create(1, 4).unwrap();
        assert_eq!(next, id + 1);
    }

    #[test]
    fn release_removes_one_record_and_ids_stay_monotonic() {
        let mut ledger = Ledger::default();
        let first = ledger.create(1, 1).unwrap();
        let second = ledger.create(1, 1).unwrap();
        assert_eq!(second, first + 1);

        ledger.release(first);
        assert!(matches!(ledger.record(first), Err(Fail::Dead)));
        assert!(ledger.record(second).is_ok());

        let third = ledger.create(1, 1).unwrap();
        assert_eq!(third, second + 1);
    }
}
