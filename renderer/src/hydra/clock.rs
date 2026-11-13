//! Session time on the existing monotonic renderer clock, retained through recovery.
use std::time::Instant;
#[derive(Clone, Copy)]
pub struct Clock {
    anchor: Instant,
    seconds: f64,
    pub bpm: f32,
    pub speed: f32,
}
impl Clock {
    pub fn new() -> Self {
        Self {
            anchor: Instant::now(),
            seconds: 0.,
            bpm: 30.,
            speed: 1.,
        }
    }
    fn at(&self, now: Instant) -> f64 {
        self.seconds + now.duration_since(self.anchor).as_secs_f64() * f64::from(self.speed)
    }
    fn configure_at(&mut self, now: Instant, bpm: f32, speed: f32, phase: Option<f32>) {
        self.seconds = phase.map_or_else(|| self.at(now), f64::from);
        self.anchor = now;
        self.bpm = bpm;
        self.speed = speed;
    }
    pub fn configure(&mut self, bpm: f32, speed: f32, phase: Option<f32>) {
        self.configure_at(Instant::now(), bpm, speed, phase);
    }
    pub fn frame(&self, now: Instant, delta: f32, index: u32) -> super::parameters::Frame {
        super::parameters::Frame {
            time: self.at(now) as f32,
            delta: delta * self.speed,
            frame: index,
            bpm: self.bpm,
            speed: self.speed,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn speed_pause_reverse_and_reset() {
        let mut c = Clock::new();
        let t = c.anchor;
        c.configure_at(t + Duration::from_secs(2), 120., 2., None);
        assert_eq!(c.at(t + Duration::from_secs(3)), 4.);
        c.configure_at(t + Duration::from_secs(3), 120., 0., None);
        assert_eq!(c.at(t + Duration::from_secs(8)), 4.);
        c.configure_at(t + Duration::from_secs(8), 90., -1., Some(10.));
        assert_eq!(c.at(t + Duration::from_secs(9)), 9.);
        let f = c.frame(t + Duration::from_secs(9), 0.1, 17);
        assert_eq!(f.delta, -0.1);
        assert_eq!(f.bpm, 90.);
        assert_eq!(f.frame, 17);
    }
}
