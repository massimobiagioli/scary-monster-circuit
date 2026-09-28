//! Distance sensing for the two side-mounted sensors.

use embedded_hal::digital::{InputPin, OutputPin};

/// A source of monotonically increasing microseconds.
///
/// Implemented on the target by the RP2040 timer. Kept as a trait so the
/// sensor driver can be tested without hardware.
pub trait MicrosClock {
    fn now_micros(&mut self) -> u32;
}

/// Anything that can report a distance in centimeters.
pub trait DistanceSensor {
    type Error;

    fn distance_cm(&mut self) -> Result<u16, Self::Error>;
}

/// Failure modes of an ultrasonic sensor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorError {
    /// No echo came back within the timeout.
    Timeout,
    /// A GPIO operation failed.
    Pin,
}

/// Maximum measurable distance (cm), used to clamp noisy readings.
pub const MAX_RANGE_CM: u16 = 400;

const TRIGGER_PULSE_US: u32 = 10;
const ECHO_TIMEOUT_US: u32 = 30_000;
const US_PER_CM: u32 = 58;

/// HC-SR04 ultrasonic distance sensor.
pub struct HcSr04<Trig, Echo> {
    trig: Trig,
    echo: Echo,
}

impl<Trig, Echo> HcSr04<Trig, Echo>
where
    Trig: OutputPin,
    Echo: InputPin,
{
    /// Creates a sensor from a trigger output and an echo input pin.
    pub fn new(trig: Trig, echo: Echo) -> Self {
        Self { trig, echo }
    }

    /// Triggers a measurement and returns the distance in centimeters.
    pub fn distance_cm<Clock: MicrosClock>(
        &mut self,
        clock: &mut Clock,
    ) -> Result<u16, SensorError> {
        self.trig.set_low().map_err(|_| SensorError::Pin)?;
        self.delay_us(clock, 2);
        self.trig.set_high().map_err(|_| SensorError::Pin)?;
        self.delay_us(clock, TRIGGER_PULSE_US);
        self.trig.set_low().map_err(|_| SensorError::Pin)?;

        let start = self.wait_for_echo(clock, true)?;
        let end = self.wait_for_echo(clock, false)?;
        let elapsed_us = end.wrapping_sub(start);

        Ok((elapsed_us / US_PER_CM).min(u32::from(MAX_RANGE_CM)) as u16)
    }

    fn delay_us<Clock: MicrosClock>(&self, clock: &mut Clock, micros: u32) {
        let start = clock.now_micros();
        while clock.now_micros().wrapping_sub(start) < micros {}
    }

    fn wait_for_echo<Clock: MicrosClock>(
        &mut self,
        clock: &mut Clock,
        high: bool,
    ) -> Result<u32, SensorError> {
        let start = clock.now_micros();
        loop {
            let level = if high {
                self.echo.is_high()
            } else {
                self.echo.is_low()
            };
            if level.map_err(|_| SensorError::Pin)? {
                return Ok(clock.now_micros());
            }
            if clock.now_micros().wrapping_sub(start) > ECHO_TIMEOUT_US {
                return Err(SensorError::Timeout);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{SensorRig, US_PER_CM};

    #[test]
    fn measures_the_distance_of_a_target() {
        for cm in [2, 40, 85, 250, 400] {
            let mut rig = SensorRig::new(Some(cm));
            assert_eq!(rig.measure(), Ok(cm), "target at {cm} cm");
        }
    }

    #[test]
    fn pulses_the_trigger_once_per_measurement() {
        let mut rig = SensorRig::new(Some(85));

        assert_eq!(rig.pulse_count(), 0);
        rig.measure().unwrap();
        assert_eq!(rig.pulse_count(), 1);
        rig.measure().unwrap();
        assert_eq!(rig.pulse_count(), 2);
    }

    #[test]
    fn answers_with_58_microseconds_of_echo_per_centimetre() {
        let mut rig = SensorRig::new(Some(85));

        rig.measure().unwrap();

        assert_eq!(rig.echo_width_us(), Some(85 * US_PER_CM));
    }

    #[test]
    fn follows_the_target_when_it_moves() {
        let mut rig = SensorRig::new(Some(200));
        assert_eq!(rig.measure(), Ok(200));

        rig.set_distance(Some(45));
        assert_eq!(rig.measure(), Ok(45));

        rig.set_distance(None);
        assert_eq!(rig.measure(), Err(SensorError::Timeout));
    }

    #[test]
    fn clamps_readings_to_the_maximum_range() {
        let mut rig = SensorRig::new(Some(MAX_RANGE_CM + 100));

        assert_eq!(rig.measure(), Ok(MAX_RANGE_CM));
    }

    #[test]
    fn times_out_when_nothing_is_in_range() {
        let mut rig = SensorRig::new(None);
        assert_eq!(rig.measure(), Err(SensorError::Timeout));
    }

    #[test]
    fn reports_a_failing_trigger_pin_driven_low() {
        let mut rig = SensorRig::new(Some(85));
        rig.fail_trigger_low(true);

        assert_eq!(rig.measure(), Err(SensorError::Pin));
    }

    #[test]
    fn reports_a_failing_trigger_pin_driven_high() {
        let mut rig = SensorRig::new(Some(85));
        rig.fail_trigger_high(true);

        assert_eq!(rig.measure(), Err(SensorError::Pin));
    }

    #[test]
    fn reports_a_failing_echo_pin_when_waiting_for_the_pulse() {
        let mut rig = SensorRig::new(Some(85));
        rig.fail_echo_read(true);

        assert_eq!(rig.measure(), Err(SensorError::Pin));
    }

    #[test]
    fn reports_a_trigger_pin_that_fails_after_the_pulse() {
        let mut rig = SensorRig::new(Some(85));
        rig.fail_trigger_low_after_pulse(true);

        assert_eq!(rig.measure(), Err(SensorError::Pin));
    }

    #[test]
    fn reports_an_echo_pin_that_fails_when_the_pulse_ends() {
        let mut rig = SensorRig::new(Some(85));
        rig.fail_echo_low_read(true);

        assert_eq!(rig.measure(), Err(SensorError::Pin));
    }
}
