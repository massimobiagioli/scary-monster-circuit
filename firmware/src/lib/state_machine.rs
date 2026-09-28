//! Behavior finite state machine.
//!
//! The prop has exactly three states. The next state is a pure function of the
//! nearest distance reported by the two side sensors, which keeps the logic
//! small and easy to test.

/// Distance (cm) below which the head starts following a person.
pub const FOLLOW_DISTANCE_CM: u16 = 100;

/// Distance (cm) below which the scary song is triggered.
pub const ACTION_DISTANCE_CM: u16 = 30;

/// Minimum difference (cm) between the two sensors before the head turns.
pub const TURN_DEADBAND_CM: u16 = 10;

/// What the prop is doing right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum State {
    /// Waiting; nothing is within range.
    #[default]
    Idle,
    /// A person is within `FOLLOW_DISTANCE_CM`; follow their movement.
    Follow,
    /// A person is within `ACTION_DISTANCE_CM`; play the scary song.
    Action,
}

/// Which way the head should turn to face the closest person.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Center,
    Right,
}

/// A single pair of distance readings, one per side sensor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reading {
    pub left_cm: Option<u16>,
    pub right_cm: Option<u16>,
}

impl Reading {
    /// The smallest valid distance across both sensors.
    pub fn nearest_cm(&self) -> Option<u16> {
        match (self.left_cm, self.right_cm) {
            (Some(left), Some(right)) => Some(left.min(right)),
            (Some(left), None) => Some(left),
            (None, Some(right)) => Some(right),
            (None, None) => None,
        }
    }

    /// Where the head should point, based on the closest side.
    pub fn direction(&self) -> Direction {
        match (self.left_cm, self.right_cm) {
            (Some(left), Some(right)) => {
                if left.saturating_add(TURN_DEADBAND_CM) < right {
                    Direction::Left
                } else if right.saturating_add(TURN_DEADBAND_CM) < left {
                    Direction::Right
                } else {
                    Direction::Center
                }
            }
            (Some(_), None) => Direction::Left,
            (None, Some(_)) => Direction::Right,
            (None, None) => Direction::Center,
        }
    }
}

/// The finite state machine that drives the prop.
#[derive(Debug, Default)]
pub struct StateMachine {
    state: State,
}

impl StateMachine {
    /// Creates a state machine in the `Idle` state.
    pub fn new() -> Self {
        Self::default()
    }

    /// The current state.
    pub fn state(&self) -> State {
        self.state
    }

    /// Advances the machine with a new sensor reading and returns the state.
    pub fn update(&mut self, reading: Reading) -> State {
        self.state = match reading.nearest_cm() {
            Some(distance) if distance <= ACTION_DISTANCE_CM => State::Action,
            Some(distance) if distance <= FOLLOW_DISTANCE_CM => State::Follow,
            _ => State::Idle,
        };
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(left_cm: Option<u16>, right_cm: Option<u16>) -> Reading {
        Reading { left_cm, right_cm }
    }

    #[test]
    fn idle_when_nothing_is_detected() {
        let mut machine = StateMachine::new();
        assert_eq!(machine.update(reading(None, None)), State::Idle);
    }

    #[test]
    fn idle_when_person_is_far() {
        let mut machine = StateMachine::new();
        assert_eq!(machine.update(reading(Some(250), Some(300))), State::Idle);
    }

    #[test]
    fn follows_a_person_within_follow_distance() {
        let mut machine = StateMachine::new();
        assert_eq!(machine.update(reading(Some(80), None)), State::Follow);
    }

    #[test]
    fn acts_when_person_is_very_close() {
        let mut machine = StateMachine::new();
        assert_eq!(machine.update(reading(Some(20), Some(200))), State::Action);
    }

    #[test]
    fn returns_to_follow_when_person_steps_back() {
        let mut machine = StateMachine::new();
        machine.update(reading(Some(20), None));
        assert_eq!(machine.update(reading(Some(70), None)), State::Follow);
    }

    #[test]
    fn returns_to_idle_when_person_leaves() {
        let mut machine = StateMachine::new();
        machine.update(reading(Some(20), None));
        assert_eq!(machine.update(reading(Some(400), Some(400))), State::Idle);
    }

    #[test]
    fn direction_turns_towards_the_closest_sensor() {
        assert_eq!(reading(Some(40), Some(120)).direction(), Direction::Left);
        assert_eq!(reading(Some(120), Some(40)).direction(), Direction::Right);
        assert_eq!(reading(Some(80), Some(85)).direction(), Direction::Center);
    }

    #[test]
    fn direction_falls_back_to_the_reading_side() {
        assert_eq!(reading(Some(80), None).direction(), Direction::Left);
        assert_eq!(reading(None, Some(80)).direction(), Direction::Right);
        assert_eq!(reading(None, None).direction(), Direction::Center);
    }

    #[test]
    fn nearest_cm_picks_the_closest_valid_reading() {
        assert_eq!(reading(Some(80), Some(120)).nearest_cm(), Some(80));
        assert_eq!(reading(Some(120), Some(80)).nearest_cm(), Some(80));
        assert_eq!(reading(Some(120), None).nearest_cm(), Some(120));
        assert_eq!(reading(None, Some(80)).nearest_cm(), Some(80));
        assert_eq!(reading(None, None).nearest_cm(), None);
    }

    #[test]
    fn follows_exactly_at_the_follow_distance() {
        let mut machine = StateMachine::new();

        assert_eq!(
            machine.update(reading(Some(FOLLOW_DISTANCE_CM), None)),
            State::Follow
        );
        assert_eq!(
            machine.update(reading(Some(FOLLOW_DISTANCE_CM + 1), None)),
            State::Idle
        );
    }

    #[test]
    fn acts_exactly_at_the_action_distance() {
        let mut machine = StateMachine::new();

        assert_eq!(
            machine.update(reading(Some(ACTION_DISTANCE_CM), None)),
            State::Action
        );
        assert_eq!(
            machine.update(reading(Some(ACTION_DISTANCE_CM + 1), None)),
            State::Follow
        );
    }

    #[test]
    fn direction_ignores_differences_within_the_deadband() {
        let near = 40;
        let far = near + TURN_DEADBAND_CM + 1;

        assert_eq!(reading(Some(39), Some(48)).direction(), Direction::Center);
        assert_eq!(reading(Some(48), Some(39)).direction(), Direction::Center);
        assert_eq!(reading(Some(near), Some(far)).direction(), Direction::Left);
        assert_eq!(reading(Some(far), Some(near)).direction(), Direction::Right);
    }

    #[test]
    fn direction_survives_readings_at_the_end_of_the_range() {
        let mut machine = StateMachine::new();

        assert_eq!(
            machine.update(reading(Some(u16::MAX), Some(u16::MAX))),
            State::Idle
        );
        assert_eq!(
            reading(Some(u16::MAX - 5), Some(u16::MAX)).direction(),
            Direction::Center
        );
    }

    #[test]
    fn one_sensor_alone_decides_the_direction() {
        let mut machine = StateMachine::new();

        assert_eq!(machine.update(reading(Some(80), None)), State::Follow);
        assert_eq!(machine.update(reading(None, Some(80))), State::Follow);
        assert_eq!(machine.update(reading(None, None)), State::Idle);
    }
}
