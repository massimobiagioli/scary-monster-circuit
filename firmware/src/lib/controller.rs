//! The main control loop: turns sensor readings into actuator commands.
//!
//! Kept free of hardware so the behavior can be driven by tests with mock
//! actuators.

use embedded_hal::pwm::SetDutyCycle;

use crate::audio::{AudioPlayer, FIRST_TRACK};
use crate::servo::{Head, TURN_CENTER_ANGLE, TURN_LEFT_ANGLE, TURN_RIGHT_ANGLE};
use crate::state_machine::{Direction, Reading, State, StateMachine};

/// Drives the prop: one state machine and one set of actuator commands.
#[derive(Debug, Default)]
pub struct Controller {
    machine: StateMachine,
}

impl Controller {
    /// Creates a controller in the [`State::Idle`] state.
    pub fn new() -> Self {
        Self::default()
    }

    /// The behavior state the controller is in right now.
    pub fn state(&self) -> State {
        self.machine.state()
    }

    /// Advances the prop by one sensor reading.
    ///
    /// Returns the track that playback was just started for, if somebody came
    /// close enough to startle them this iteration.
    pub fn update<Pwm, Player>(
        &mut self,
        reading: Reading,
        head: &mut Head<Pwm>,
        audio: &mut Player,
    ) -> Option<u16>
    where
        Pwm: SetDutyCycle,
        Player: AudioPlayer,
    {
        let previous = self.machine.state();

        match self.machine.update(reading) {
            State::Idle => {
                let _ = head.center();
                None
            }
            State::Follow => {
                let angle = match reading.direction() {
                    Direction::Left => TURN_LEFT_ANGLE,
                    Direction::Center => TURN_CENTER_ANGLE,
                    Direction::Right => TURN_RIGHT_ANGLE,
                };
                let _ = head.set_angle(angle);
                None
            }
            State::Action => {
                let _ = head.set_angle(TURN_CENTER_ANGLE);
                if previous != State::Action {
                    let _ = audio.play_track(FIRST_TRACK);
                    return Some(FIRST_TRACK);
                }
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{servo_head, TestPlayer};

    /// Duty cycle for 90 degrees (1.5 ms pulse), the head looking straight.
    const CENTER_DUTY: u16 = 2929;
    /// Duty cycle for 45 degrees (1.0 ms pulse), the head looking left.
    const LEFT_DUTY: u16 = 1953;
    /// Duty cycle for 135 degrees (2.0 ms pulse), the head looking right.
    const RIGHT_DUTY: u16 = 3906;

    fn reading(left_cm: Option<u16>, right_cm: Option<u16>) -> Reading {
        Reading { left_cm, right_cm }
    }

    #[test]
    fn centers_the_head_when_nobody_is_near() {
        let (mut head, pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        let played = controller.update(reading(Some(250), Some(250)), &mut head, &mut player);

        assert_eq!(played, None);
        assert_eq!(controller.state(), State::Idle);
        assert_eq!(pwm.duty(), Some(CENTER_DUTY));
        assert!(player.tracks.is_empty());
    }

    #[test]
    fn centers_the_head_when_nothing_is_detected_at_all() {
        let (mut head, pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        controller.update(reading(None, None), &mut head, &mut player);

        assert_eq!(controller.state(), State::Idle);
        assert_eq!(pwm.duty(), Some(CENTER_DUTY));
    }

    #[test]
    fn turns_left_towards_a_person_on_the_left() {
        let (mut head, pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        controller.update(reading(Some(40), Some(120)), &mut head, &mut player);

        assert_eq!(controller.state(), State::Follow);
        assert_eq!(pwm.duty(), Some(LEFT_DUTY));
    }

    #[test]
    fn turns_right_towards_a_person_on_the_right() {
        let (mut head, pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        controller.update(reading(Some(120), Some(40)), &mut head, &mut player);

        assert_eq!(controller.state(), State::Follow);
        assert_eq!(pwm.duty(), Some(RIGHT_DUTY));
    }

    #[test]
    fn centers_the_head_when_the_person_is_directly_in_front() {
        let (mut head, pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        controller.update(reading(Some(80), Some(85)), &mut head, &mut player);

        assert_eq!(controller.state(), State::Follow);
        assert_eq!(pwm.duty(), Some(CENTER_DUTY));
    }

    #[test]
    fn follows_when_only_one_sensor_sees_anybody() {
        let (mut head, pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        controller.update(reading(Some(80), None), &mut head, &mut player);

        assert_eq!(controller.state(), State::Follow);
        assert_eq!(pwm.duty(), Some(LEFT_DUTY));
    }

    #[test]
    fn starts_the_scary_track_when_somebody_gets_close() {
        let (mut head, pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        let played = controller.update(reading(Some(20), Some(200)), &mut head, &mut player);

        assert_eq!(played, Some(FIRST_TRACK));
        assert_eq!(controller.state(), State::Action);
        assert_eq!(pwm.duty(), Some(CENTER_DUTY));
        assert_eq!(player.tracks, vec![FIRST_TRACK]);
    }

    #[test]
    fn plays_the_track_once_while_somebody_stays_close() {
        let (mut head, pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        controller.update(reading(Some(20), None), &mut head, &mut player);
        let played = controller.update(reading(Some(22), None), &mut head, &mut player);

        assert_eq!(played, None);
        assert_eq!(player.tracks, vec![FIRST_TRACK]);
        assert_eq!(pwm.duty(), Some(CENTER_DUTY));
    }

    #[test]
    fn plays_again_when_somebody_leaves_and_comes_back() {
        let (mut head, pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        controller.update(reading(Some(20), None), &mut head, &mut player);
        controller.update(reading(Some(250), None), &mut head, &mut player);
        let played = controller.update(reading(Some(20), None), &mut head, &mut player);

        assert_eq!(played, Some(FIRST_TRACK));
        assert_eq!(player.tracks, vec![FIRST_TRACK, FIRST_TRACK]);
        assert_eq!(pwm.duty(), Some(CENTER_DUTY));
    }

    #[test]
    fn keeps_running_when_the_actuators_fail() {
        let (mut head, pwm) = servo_head();
        pwm.fail(true);
        let mut player = TestPlayer {
            fail: true,
            ..TestPlayer::default()
        };
        let mut controller = Controller::new();

        assert_eq!(
            controller.update(reading(Some(250), None), &mut head, &mut player),
            None
        );
        assert_eq!(
            controller.update(reading(Some(40), None), &mut head, &mut player),
            None
        );
        assert_eq!(
            controller.update(reading(Some(20), None), &mut head, &mut player),
            Some(FIRST_TRACK)
        );
        assert_eq!(pwm.duty(), None);
        assert!(player.tracks.is_empty());
    }
}
