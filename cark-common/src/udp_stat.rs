// Sequence is used to identify the order of messages
pub type Sequence = u16;

#[derive(Debug)]
pub struct SequenceGen {
    sequence: Sequence,
}

impl SequenceGen {
    pub fn next(&mut self) -> Sequence {
        self.sequence = self.sequence.overflowing_add(1).0;
        self.sequence
    }
}

impl Default for SequenceGen {
    fn default() -> Self {
        Self {
            sequence: Sequence::MAX,
        }
    }
}

#[derive(Debug)]
pub struct UdpStat {
    // FIXME: This will overflow
    pub sent: u64,
    pub received: u64,
    pub last_sequence: Sequence,
}

impl UdpStat {
    pub fn new() -> Self {
        Self {
            sent: 0,
            received: 0,
            last_sequence: Sequence::MAX,
        }
    }

    pub fn update(&mut self, sequence: Sequence) {
        self.received += 1;

        let diff = sequence.overflowing_sub(self.last_sequence).0;
        if diff < Sequence::MAX / 2 {
            self.sent += diff as u64;
        }

        // When packets are duplicated, received may exceed sent, so in that case, received is adjusted to sent.
        if self.sent < self.received {
            self.received = self.sent;
        }

        self.last_sequence = sequence;
    }

    pub fn dropped(&self) -> u64 {
        self.sent - self.received
    }

    pub fn loss_rate(&self) -> f64 {
        self.dropped() as f64 / self.sent as f64
    }
}

#[test]
fn test() {
    let mut stat = UdpStat::new();
    stat.update(0);
    assert_eq!(stat.dropped(), 0);
    stat.update(1);
    assert_eq!(stat.dropped(), 0);
    stat.update(4);
    assert_eq!(stat.dropped(), 2);
    stat.update(5);
    assert_eq!(stat.dropped(), 2);
    stat.update(3);
    assert_eq!(stat.dropped(), 1);
    stat.update(2);
    assert_eq!(stat.dropped(), 0);

    let mut stat = UdpStat {
        sent: 0,
        received: 0,
        last_sequence: Sequence::MAX - 3,
    };
    stat.update(2);
    assert_eq!(stat.dropped(), 5);
    stat.update(0);
    assert_eq!(stat.dropped(), 4);
    stat.update(Sequence::MAX - 2);
    assert_eq!(stat.dropped(), 3);
}
