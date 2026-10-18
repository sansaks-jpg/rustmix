//! Rational progressive frame deadlines. Slow consumers skip obsolete work;
//! there is no catch-up queue and no accumulated rounded-millisecond drift.
use crate::Rate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameTiming {
    pub index: u64,
    pub pts_ns: u64,
    pub duration_ns: u64,
}
pub struct FrameClock {
    rate: Rate,
    origin_ns: u64,
    next: Option<u64>,
    skipped: u64,
}
impl Rate {
    pub fn frame_at_ns(self, ns: u64) -> u64 {
        (u128::from(ns) * u128::from(self.numerator)
            / (1_000_000_000 * u128::from(self.denominator.max(1))))
        .min(u128::from(u64::MAX)) as u64
    }
    pub fn frame_start_ns(self, frame: u64) -> u64 {
        (u128::from(frame) * 1_000_000_000 * u128::from(self.denominator))
            .div_ceil(u128::from(self.numerator.max(1)))
            .min(u128::from(u64::MAX)) as u64
    }
}
impl FrameClock {
    pub fn new(rate: Rate) -> Self {
        Self::at(rate, 0)
    }
    pub fn at(rate: Rate, origin_ns: u64) -> Self {
        Self {
            rate,
            origin_ns,
            next: None,
            skipped: 0,
        }
    }
    pub fn skipped(&self) -> u64 {
        self.skipped
    }
    pub fn due(&mut self, now_ns: u64) -> Option<FrameTiming> {
        if now_ns < self.origin_ns {
            return None;
        }
        let index = self.rate.frame_at_ns(now_ns - self.origin_ns);
        let next = self.next.unwrap_or(index);
        if index < next {
            return None;
        }
        self.skipped = self.skipped.saturating_add(index.saturating_sub(next));
        self.next = Some(index.saturating_add(1));
        let pts_ns = self
            .rate
            .frame_start_ns(index)
            .saturating_add(self.origin_ns);
        Some(FrameTiming {
            index,
            pts_ns,
            duration_ns: self
                .rate
                .frame_start_ns(index.saturating_add(1))
                .saturating_add(self.origin_ns)
                .saturating_sub(pts_ns),
        })
    }
    pub fn wait_ns(&self, now_ns: u64) -> u64 {
        self.rate
            .frame_start_ns(self.next.unwrap_or(0))
            .saturating_add(self.origin_ns)
            .saturating_sub(now_ns)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    #[test]
    fn progressive_rates_have_exact_long_term_deadlines() {
        for (n, d) in [
            (24, 1),
            (25, 1),
            (30, 1),
            (50, 1),
            (60, 1),
            (24000, 1001),
            (30000, 1001),
            (60000, 1001),
        ] {
            let rate = Rate::new(n, d).unwrap();
            assert_eq!(
                rate.frame_start_ns(u64::from(n)),
                u64::from(d) * 1_000_000_000
            );
            let frame = rate.frame_at_ns(3_600_000_000_000);
            let start = rate.frame_start_ns(frame);
            assert_eq!(rate.frame_at_ns(start), frame);
            assert!(start <= 3_600_000_000_000);
            assert!(rate.frame_start_ns(frame + 1) > 3_600_000_000_000);
        }
    }
    #[test]
    fn a_stall_skips_old_frames_without_bursting_or_rounding_to_16ms() {
        let mut clock = FrameClock::new(Rate::new(60, 1).unwrap());
        assert_eq!(clock.due(0).unwrap().index, 0);
        assert!(clock.due(16_666_666).is_none());
        assert_eq!(clock.wait_ns(16_666_666), 1);
        assert_eq!(clock.due(16_666_667).unwrap().index, 1);
        assert_eq!(clock.due(1_000_000_000).unwrap().index, 60);
        assert_eq!(clock.skipped(), 58);
        assert!(clock.due(1_000_000_000).is_none());
    }
    #[test]
    fn format_changes_start_a_new_monotonic_epoch() {
        let mut old = FrameClock::new(Rate::new(60, 1).unwrap());
        let previous = old.due(100_100_000).unwrap();
        let mut next = FrameClock::at(Rate::new(24, 1).unwrap(), 101_000_000);
        assert!(next.due(100_999_999).is_none());
        let first = next.due(101_000_000).unwrap();
        assert_eq!(first.index, 0);
        assert!(first.pts_ns > previous.pts_ns);
        assert_eq!(next.wait_ns(101_000_000), 41_666_667);
        assert!(next.due(142_666_666).is_none());
        assert_eq!(next.due(142_666_667).unwrap().pts_ns, 142_666_667);
    }
}
