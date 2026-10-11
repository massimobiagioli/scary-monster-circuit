//! The main control loop: turns sensor readings into actuator commands.
//!
//! Kept free of hardware so the behavior can be driven by tests with mock
//! actuators.

use embedded_hal::pwm::SetDutyCycle;

use crate::audio::{AudioPlayer, FIRST_TRACK};
use crate::servo::{Head, TURN_CENTER_ANGLE, TURN_LEFT_ANGLE, TURN_RIGHT_ANGLE};
use crate::state_machine::{Direction, Reading, State, StateMachine};

/// Which actuator commands failed during the last [`Controller::update`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ActuatorStatus {
    /// The servo refused the last command.
    pub servo_failed: bool,
    /// The audio player refused the last command.
    pub audio_failed: bool,
}

/// Drives the prop: one state machine and one set of actuator commands.
#[derive(Debug, Default)]
pub struct Controller {
    machine: StateMachine,
    status: ActuatorStatus,
    /// The state the head was last positioned for (`None` before the first).
    commanded: Option<State>,
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

    /// Which actuator commands failed during the last [`Controller::update`].
    pub fn status(&self) -> ActuatorStatus {
        self.status
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
        let mut status = ActuatorStatus::default();
        let mut played = None;

        let state = self.machine.update(reading);

        // The head gets a pulse only when the behavior state changes: it is
        // positioned once for the new state and then left perfectly still.
        if self.commanded != Some(state) {
            self.commanded = Some(state);
            let desired = match state {
                State::Follow => match reading.direction() {
                    Direction::Left => TURN_LEFT_ANGLE,
                    Direction::Center => TURN_CENTER_ANGLE,
                    Direction::Right => TURN_RIGHT_ANGLE,
                },
                State::Idle | State::Action | State::PostAction => TURN_CENTER_ANGLE,
            };
            status.servo_failed = head.set_angle(desired).is_err();
        }

        if state == State::Action && previous != State::Action {
            match audio.play_track(FIRST_TRACK) {
                Ok(()) => played = Some(FIRST_TRACK),
                Err(_) => status.audio_failed = true,
            }
        }

        self.status = status;
        played
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_machine::ACTION_DURATION_TICKS;
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
    fn does_not_twitch_the_head_while_idle() {
        let (mut head, pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        controller.update(reading(Some(250), Some(250)), &mut head, &mut player);
        let writes = pwm.writes();

        controller.update(reading(Some(250), Some(250)), &mut head, &mut player);
        controller.update(reading(Some(250), Some(250)), &mut head, &mut player);

        assert_eq!(pwm.writes(), writes, "idle must not keep re-centring");
    }

    #[test]
    fn only_uses_the_three_servo_positions() {
        let (mut head, _pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        let positions = [TURN_LEFT_ANGLE, TURN_CENTER_ANGLE, TURN_RIGHT_ANGLE];
        let readings = [
            reading(Some(5), Some(8)),
            reading(Some(250), Some(250)),
            reading(Some(40), Some(120)),
            reading(Some(120), Some(40)),
            reading(Some(80), Some(80)),
        ];

        for reading in readings {
            controller.update(reading, &mut head, &mut player);
            assert!(
                positions.contains(&head.angle()),
                "unexpected angle {}",
                head.angle()
            );
        }
    }

    #[test]
    fn positions_the_head_once_per_state() {
        let (mut head, pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        controller.update(reading(Some(15), Some(28)), &mut head, &mut player);
        let writes = pwm.writes();

        // Still in Follow, but the person is now on the other side: no move.
        controller.update(reading(Some(28), Some(15)), &mut head, &mut player);

        assert_eq!(pwm.writes(), writes, "the head is set once per state");
        assert_eq!(controller.state(), State::Follow);
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

        controller.update(reading(Some(15), Some(28)), &mut head, &mut player);

        assert_eq!(controller.state(), State::Follow);
        assert_eq!(pwm.duty(), Some(LEFT_DUTY));
    }

    #[test]
    fn turns_right_towards_a_person_on_the_right() {
        let (mut head, pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        controller.update(reading(Some(28), Some(15)), &mut head, &mut player);

        assert_eq!(controller.state(), State::Follow);
        assert_eq!(pwm.duty(), Some(RIGHT_DUTY));
    }

    #[test]
    fn centers_the_head_when_the_person_is_directly_in_front() {
        let (mut head, pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        controller.update(reading(Some(20), Some(25)), &mut head, &mut player);

        assert_eq!(controller.state(), State::Follow);
        assert_eq!(pwm.duty(), Some(CENTER_DUTY));
    }

    #[test]
    fn follows_when_only_one_sensor_sees_anybody() {
        let (mut head, pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        controller.update(reading(Some(20), None), &mut head, &mut player);

        assert_eq!(controller.state(), State::Follow);
        assert_eq!(pwm.duty(), Some(LEFT_DUTY));
    }

    #[test]
    fn starts_the_scary_track_when_somebody_gets_close() {
        let (mut head, pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        let played = controller.update(reading(Some(5), Some(8)), &mut head, &mut player);

        assert_eq!(played, Some(FIRST_TRACK));
        assert_eq!(controller.state(), State::Action);
        assert_eq!(pwm.duty(), Some(CENTER_DUTY));
        assert_eq!(player.tracks, vec![FIRST_TRACK]);
        assert_eq!(controller.status(), ActuatorStatus::default());
    }

    #[test]
    fn plays_the_track_once_while_somebody_stays_close() {
        let (mut head, pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        controller.update(reading(Some(5), Some(8)), &mut head, &mut player);
        let played = controller.update(reading(Some(6), Some(9)), &mut head, &mut player);

        assert_eq!(played, None);
        assert_eq!(player.tracks, vec![FIRST_TRACK]);
        assert_eq!(pwm.duty(), Some(CENTER_DUTY));
    }

    #[test]
    fn keeps_playing_while_the_song_plays() {
        let (mut head, pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        controller.update(reading(Some(5), Some(8)), &mut head, &mut player);
        // Stepping back does not cut the song short.
        let played = controller.update(reading(Some(70), Some(80)), &mut head, &mut player);

        assert_eq!(played, None);
        assert_eq!(controller.state(), State::Action);
        assert_eq!(player.tracks, vec![FIRST_TRACK]);
        assert_eq!(pwm.duty(), Some(CENTER_DUTY));
    }

    #[test]
    fn stays_in_post_action_without_replaying() {
        let (mut head, pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        controller.update(reading(Some(5), Some(8)), &mut head, &mut player);
        for _ in 0..ACTION_DURATION_TICKS {
            controller.update(reading(Some(5), Some(8)), &mut head, &mut player);
        }

        let played = controller.update(reading(Some(6), Some(9)), &mut head, &mut player);

        assert_eq!(played, None);
        assert_eq!(controller.state(), State::PostAction);
        assert_eq!(pwm.duty(), Some(CENTER_DUTY));
        assert_eq!(player.tracks, vec![FIRST_TRACK]);
    }

    #[test]
    fn plays_again_when_somebody_leaves_and_comes_back_after_the_song() {
        let (mut head, _pwm) = servo_head();
        let mut player = TestPlayer::default();
        let mut controller = Controller::new();

        controller.update(reading(Some(5), Some(8)), &mut head, &mut player);
        for _ in 0..ACTION_DURATION_TICKS {
            controller.update(reading(Some(5), Some(8)), &mut head, &mut player);
        }
        // Nobody near: post-action releases to idle.
        controller.update(reading(Some(250), Some(250)), &mut head, &mut player);

        let played = controller.update(reading(Some(5), Some(8)), &mut head, &mut player);

        assert_eq!(played, Some(FIRST_TRACK));
        assert_eq!(controller.state(), State::Action);
        assert_eq!(player.tracks, vec![FIRST_TRACK, FIRST_TRACK]);
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
            controller.update(reading(Some(250), Some(250)), &mut head, &mut player),
            None
        );
        assert_eq!(
            controller.update(reading(Some(40), Some(50)), &mut head, &mut player),
            None
        );
        assert_eq!(
            controller.update(reading(Some(5), Some(8)), &mut head, &mut player),
            None,
            "a failed playback does not report a started track"
        );
        assert_eq!(pwm.duty(), None);
        assert!(player.tracks.is_empty());
        assert_eq!(
            controller.status(),
            ActuatorStatus {
                servo_failed: true,
                audio_failed: true,
            }
        );
    }
}
