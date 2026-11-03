#[cfg(test)]
mod tests {
    use env::{Bit, Mark, PieToken, TaskId};
    use pipe_api::{
        code, direction, Direction, Fail, Reply, Request, BACK, BIND, CLOSE, CREATE, DATA,
        DEFAULT_CAPACITY, DEMAND_BIT, DIR, ENTRY, LEASE, MAX_CAPACITY, READ, READ_BIT, RELEASE,
        WRITE, WRITE_BIT,
    };
    use wire::Message;

    #[test]
    fn marks_opcodes_and_bits_are_frozen() {
        assert_eq!(ENTRY, Mark::of("pipe-entry"));
        assert_eq!(BACK, Mark::of("pipe-back"));
        assert_eq!(LEASE, Mark::of("pipe-lease"));
        assert_eq!(DATA, Mark::of("pipe-data"));
        assert_eq!(DIR, "pipe/create");
        assert_ne!(ENTRY, BACK);

        assert_eq!([CREATE, BIND, RELEASE, CLOSE], [1, 2, 3, 5]);
        assert_eq!([READ, WRITE], [1, 2]);

        assert_eq!(READ_BIT, Bit::FIRST);
        assert_eq!(WRITE_BIT, Bit::wrap(1));
        assert_eq!(DEMAND_BIT, Bit::wrap(2));
        assert_ne!(READ_BIT, WRITE_BIT);

        assert_eq!(direction(READ), Some(Direction::Read));
        assert_eq!(direction(WRITE), Some(Direction::Write));
        assert_eq!(direction(0), None);
        assert_eq!(direction(CLOSE), None);
        assert_eq!(code(Direction::Read), READ);
        assert_eq!(code(Direction::Write), WRITE);

        assert!(DEFAULT_CAPACITY > 0);
        assert!(MAX_CAPACITY >= DEFAULT_CAPACITY);
    }

    #[test]
    fn request_encoding_is_byte_exact_and_length_strict() {
        let request = Request {
            op: CREATE,
            id: 2,
            capacity: 3,
            target: TaskId::new(4),
            direction: READ,
            lease: PieToken::mint(5),
            back: PieToken::mint(6),
        };

        assert_eq!(Request::LEN, 42);
        let mut out = Request::EMPTY;
        assert_eq!(request.store(&mut out), Some(42));

        let mut golden = [0u8; Request::LEN];
        golden[0] = CREATE;
        golden[1..9].copy_from_slice(&2u64.to_le_bytes());
        golden[9..17].copy_from_slice(&3u64.to_le_bytes());
        golden[17..25].copy_from_slice(&4u64.to_le_bytes());
        golden[25] = READ;
        golden[26..34].copy_from_slice(&5u64.to_le_bytes());
        golden[34..42].copy_from_slice(&6u64.to_le_bytes());
        assert_eq!(out, golden);

        let decoded = Request::fetch(&golden).expect("request frame");
        assert_eq!(decoded.op, CREATE);
        assert_eq!(decoded.id, 2);
        assert_eq!(decoded.capacity, 3);
        assert_eq!(decoded.target.get(), 4);
        assert_eq!(decoded.direction, READ);
        assert_eq!(decoded.lease, PieToken::mint(5));
        assert_eq!(decoded.back, PieToken::mint(6));

        assert!(Request::fetch(&golden[..41]).is_none());
        let mut longer = [0u8; 43];
        longer[..42].copy_from_slice(&golden);
        assert!(Request::fetch(&longer).is_none());

        let mut short = [0u8; 41];
        assert_eq!(request.store(&mut short), None);
    }

    #[test]
    fn reply_encoding_is_byte_exact_and_length_strict() {
        let reply = Reply {
            status: 0,
            id: 1,
            capacity: 2,
            seed: PieToken::mint(3),
        };

        assert_eq!(Reply::LEN, 25);
        let mut out = Reply::EMPTY;
        assert_eq!(reply.store(&mut out), Some(25));

        let mut golden = [0u8; Reply::LEN];
        golden[0] = 0;
        golden[1..9].copy_from_slice(&1u64.to_le_bytes());
        golden[9..17].copy_from_slice(&2u64.to_le_bytes());
        golden[17..25].copy_from_slice(&3u64.to_le_bytes());
        assert_eq!(out, golden);

        let decoded = Reply::fetch(&golden).expect("reply frame");
        assert_eq!(decoded.status, 0);
        assert_eq!(decoded.id, 1);
        assert_eq!(decoded.capacity, 2);
        assert_eq!(decoded.seed, PieToken::mint(3));

        assert!(Reply::fetch(&golden[..24]).is_none());
        let mut longer = [0u8; 26];
        longer[..25].copy_from_slice(&golden);
        assert!(Reply::fetch(&longer).is_none());

        let mut short = [0u8; 24];
        assert_eq!(reply.store(&mut short), None);
    }

    #[test]
    fn failure_codes_and_stream_mapping_are_frozen() {
        assert_eq!(
            [
                Fail::Denied.code(),
                Fail::Dead.code(),
                Fail::Full.code(),
                Fail::Invalid.code(),
                Fail::Protocol.code(),
                Fail::Closed.code(),
                Fail::BrokenPipe.code(),
            ],
            [1, 2, 3, 4, 5, 6, 7]
        );

        assert_eq!(Fail::of(1), Some(Fail::Denied));
        assert_eq!(Fail::of(2), Some(Fail::Dead));
        assert_eq!(Fail::of(3), Some(Fail::Full));
        assert_eq!(Fail::of(4), Some(Fail::Invalid));
        assert_eq!(Fail::of(5), Some(Fail::Protocol));
        assert_eq!(Fail::of(6), Some(Fail::Closed));
        assert_eq!(Fail::of(7), Some(Fail::BrokenPipe));
        assert_eq!(Fail::of(0), None);
        assert_eq!(Fail::of(8), None);

        assert_eq!(Fail::from(stream::Fail::Invalid), Fail::Invalid);
        assert_eq!(Fail::from(stream::Fail::Corrupt), Fail::Protocol);
        assert_eq!(Fail::from(stream::Fail::Closed), Fail::Closed);
        assert_eq!(Fail::from(stream::Fail::BrokenPipe), Fail::BrokenPipe);
    }
}
