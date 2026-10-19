//! Monotonic, independent transport for the procedural sixty-second sources.
//! This clock is shared by pictures and transport displays; it is not a decoder.
pub const DEMO_DURATION_MS: u64 = 60_000;

#[derive(Debug, Clone)]
pub struct PlaybackClock {
    position_ms: u64,
    last_ms: u64,
    paused: bool,
    looping: bool,
    restart_generation: u64,
}

impl PlaybackClock {
    pub(crate) fn new(now: u64) -> Self {
        Self {
            position_ms: 0,
            last_ms: now,
            paused: false,
            looping: true,
            restart_generation: 0,
        }
    }

    pub fn position_ms(&self) -> u64 {
        self.position_ms
    }
    pub fn paused(&self) -> bool {
        self.paused || (!self.looping && self.position_ms == DEMO_DURATION_MS)
    }
    pub fn looping(&self) -> bool {
        self.looping
    }
    /// Explicit operator pause, independent of the procedural 60-second endpoint.
    pub fn manually_paused(&self) -> bool {
        self.paused
    }
    /// Changes only for an explicit restart, never for a natural loop wrap.
    pub fn restart_generation(&self) -> u64 {
        self.restart_generation
    }

    pub(crate) fn advance(&mut self, now: u64, pause_all: bool) {
        let now = now.max(self.last_ms);
        let delta = now - self.last_ms;
        self.last_ms = now;
        if !self.paused && !pause_all {
            self.position_ms = if self.looping {
                (self.position_ms % DEMO_DURATION_MS + delta % DEMO_DURATION_MS) % DEMO_DURATION_MS
            } else {
                self.position_ms.saturating_add(delta).min(DEMO_DURATION_MS)
            };
        }
    }

    pub(crate) fn set_paused(&mut self, paused: bool) {
        if !paused && self.position_ms == DEMO_DURATION_MS {
            self.restart();
        }
        self.paused = paused;
    }
    pub(crate) fn restart(&mut self) {
        self.position_ms = 0;
        self.restart_generation = self.restart_generation.saturating_add(1);
    }
    pub(crate) fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pause_resume_restart_and_nonmonotonic_ticks_are_continuous() {
        let mut clock = PlaybackClock::new(0);
        clock.advance(1234, false);
        clock.set_paused(true);
        clock.advance(9000, false);
        assert_eq!(clock.position_ms(), 1234);
        clock.set_paused(false);
        clock.advance(9100, false);
        assert_eq!(clock.position_ms(), 1334);
        clock.advance(8000, false);
        clock.advance(9200, false);
        assert_eq!(clock.position_ms(), 1434);
        clock.restart();
        clock.advance(9300, false);
        assert_eq!(clock.position_ms(), 100);
    }
    #[test]
    fn global_pause_loop_endpoint_and_overflow_are_bounded() {
        let mut clock = PlaybackClock::new(0);
        clock.advance(1000, false);
        clock.advance(50_000, true);
        clock.advance(50_100, false);
        assert_eq!(clock.position_ms(), 1100);
        clock.set_looping(false);
        clock.advance(200_000, false);
        assert_eq!(clock.position_ms(), DEMO_DURATION_MS);
        assert!(clock.paused());
        clock.set_paused(false);
        assert_eq!(clock.position_ms(), 0);
        clock.set_looping(true);
        clock.advance(u64::MAX, false);
        assert!(clock.position_ms() < DEMO_DURATION_MS);
    }

    #[test]
    fn restart_identity_is_independent_of_procedural_wraps() {
        let mut clock = PlaybackClock::new(0);
        clock.advance(DEMO_DURATION_MS + 10, false);
        assert_eq!(clock.restart_generation(), 0);
        clock.restart();
        assert_eq!(clock.restart_generation(), 1);
        clock.set_looping(false);
        clock.advance(DEMO_DURATION_MS * 3, false);
        assert!(clock.paused());
        assert!(!clock.manually_paused());
        clock.set_paused(false);
        assert_eq!(clock.restart_generation(), 2);
    }
}
