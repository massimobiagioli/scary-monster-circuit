//! Head actuation via a hobby servo driven by PWM.

use embedded_hal::pwm::SetDutyCycle;

/// Servo angle (degrees) for looking fully to one side.
pub const TURN_LEFT_ANGLE: u8 = 45;

/// Servo angle (degrees) for looking straight ahead.
pub const TURN_CENTER_ANGLE: u8 = 90;

/// Servo angle (degrees) for looking fully to the other side.
pub const TURN_RIGHT_ANGLE: u8 = 135;

const MIN_ANGLE: u8 = 0;
const MAX_ANGLE: u8 = 180;
const PERIOD_US: u32 = 20_000;
const MIN_PULSE_US: u32 = 500;
const MAX_PULSE_US: u32 = 2_500;

/// A servo-driven head that can be pointed at a given angle.
pub struct Head<Pwm> {
    pwm: Pwm,
    max_duty: u16,
    /// The angle the servo was last commanded to, or `None` before the first
    /// command.
    angle: Option<u8>,
    /// Set when a new angle was written, so the log can flag a move.
    changed: bool,
}

impl<Pwm: SetDutyCycle> Head<Pwm> {
    /// Wraps a PWM channel that drives the head servo.
    pub fn new(pwm: Pwm) -> Self {
        let max_duty = pwm.max_duty_cycle();
        Self {
            pwm,
            max_duty,
            angle: None,
            changed: false,
        }
    }

    /// The last angle the head was moved to, in degrees.
    pub fn angle(&self) -> u8 {
        self.angle.unwrap_or(TURN_CENTER_ANGLE)
    }

    /// Whether the head was moved since this was last asked.
    pub fn take_changed(&mut self) -> bool {
        core::mem::take(&mut self.changed)
    }

    /// Points the head at `degrees` (0-180), clamped to the safe range.
    ///
    /// Sending the same angle twice does nothing: the servo keeps its pulse
    /// width and does not twitch, so a steady state does not make it "drill".
    pub fn set_angle(&mut self, degrees: u8) -> Result<(), Pwm::Error> {
        let degrees = degrees.clamp(MIN_ANGLE, MAX_ANGLE);
        if self.angle == Some(degrees) {
            return Ok(());
        }
        let pulse_us = MIN_PULSE_US
            + (u32::from(degrees) * (MAX_PULSE_US - MIN_PULSE_US)) / u32::from(MAX_ANGLE);
        let duty = (u32::from(self.max_duty) * pulse_us / PERIOD_US) as u16;
        self.pwm.set_duty_cycle(duty)?;
        self.angle = Some(degrees);
        self.changed = true;
        Ok(())
    }

    /// Points the head straight ahead.
    pub fn center(&mut self) -> Result<(), Pwm::Error> {
        self.set_angle(TURN_CENTER_ANGLE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{servo_head, servo_head_with_max_duty, MAX_DUTY};

    /// The duty cycle the firmware computes for each servo angle, expressed as
    /// the pulse width a real servo sees.
    fn assert_duty_for(angle: u8, expected_duty: u16) {
        let (mut head, pwm) = servo_head();

        head.set_angle(angle).unwrap();

        assert_eq!(pwm.duty(), Some(expected_duty), "{angle} degrees");
    }

    #[test]
    fn points_the_head_straight_ahead() {
        assert_duty_for(TURN_CENTER_ANGLE, 2929);
        assert_duty_for(90, 2929);
    }

    #[test]
    fn maps_the_full_servo_range() {
        assert_duty_for(0, 976);
        assert_duty_for(TURN_LEFT_ANGLE, 1953);
        assert_duty_for(TURN_RIGHT_ANGLE, 3906);
        assert_duty_for(180, 4882);
    }

    #[test]
    fn clamps_angles_above_the_safe_range() {
        let (mut head, pwm) = servo_head();

        head.set_angle(200).unwrap();

        assert_eq!(pwm.duty(), Some(4882), "clamped to 180 degrees");
    }

    #[test]
    fn uses_the_maximum_duty_cycle_of_the_pwm_channel() {
        let (mut head, pwm) = servo_head_with_max_duty(4_095);

        head.set_angle(90).unwrap();

        assert_eq!(pwm.duty(), Some(307));
        assert_ne!(
            pwm.duty(),
            Some(2929),
            "the channel, not the firmware, sets the scale"
        );
    }

    #[test]
    fn centering_and_setting_an_angle_agree() {
        let (mut head, pwm) = servo_head();

        head.center().unwrap();
        let centered = pwm.duty();

        head.set_angle(TURN_CENTER_ANGLE).unwrap();

        assert_eq!(pwm.duty(), centered);
    }

    #[test]
    fn does_not_rewrite_the_same_angle() {
        let (mut head, pwm) = servo_head();

        head.set_angle(TURN_CENTER_ANGLE).unwrap();
        let writes = pwm.writes();

        head.set_angle(TURN_CENTER_ANGLE).unwrap();
        head.center().unwrap();

        assert_eq!(pwm.writes(), writes, "a steady angle is written only once");
    }

    #[test]
    fn reports_when_it_moved() {
        let (mut head, _pwm) = servo_head();

        assert!(!head.take_changed());

        head.set_angle(TURN_LEFT_ANGLE).unwrap();
        assert!(head.take_changed());
        assert!(!head.take_changed(), "the flag clears once read");

        head.set_angle(TURN_LEFT_ANGLE).unwrap();
        assert!(!head.take_changed(), "the same angle is not a move");
    }

    #[test]
    fn reports_a_failing_pwm_channel() {
        let (mut head, pwm) = servo_head();
        pwm.fail(true);

        assert_eq!(head.set_angle(45), Err(crate::test_support::PwmError));
        assert_eq!(head.center(), Err(crate::test_support::PwmError));
    }

    #[test]
    fn never_asks_for_more_than_the_maximum_duty_cycle() {
        let (mut head, pwm) = servo_head_with_max_duty(MAX_DUTY);

        for angle in 0..=255 {
            head.set_angle(angle).unwrap();
            assert!(pwm.duty().unwrap() <= MAX_DUTY, "{angle} degrees");
        }
    }

    #[test]
    fn starts_facing_straight_ahead() {
        let (head, _pwm) = servo_head();

        assert_eq!(head.angle(), TURN_CENTER_ANGLE);
    }

    #[test]
    fn reports_the_last_commanded_angle() {
        let (mut head, _pwm) = servo_head();

        head.set_angle(TURN_LEFT_ANGLE).unwrap();
        assert_eq!(head.angle(), TURN_LEFT_ANGLE);

        head.set_angle(200).unwrap();
        assert_eq!(head.angle(), MAX_ANGLE, "clamped to 180 degrees");
    }

    #[test]
    fn keeps_the_angle_when_the_pwm_channel_fails() {
        let (mut head, pwm) = servo_head();
        head.set_angle(TURN_LEFT_ANGLE).unwrap();
        pwm.fail(true);

        assert_eq!(
            head.set_angle(TURN_RIGHT_ANGLE),
            Err(crate::test_support::PwmError)
        );
        assert_eq!(head.angle(), TURN_LEFT_ANGLE);
    }
}
