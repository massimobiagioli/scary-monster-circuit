//! Live diagnostic snapshot of every component on the prop.
//!
//! [`Snapshot`] gathers the latest known state of the two distance sensors, the
//! head servo, the audio player and the behavior state machine into one line of
//! plain text. It is rendered without allocating, so the firmware can stream a
//! live row every second over the USB serial port (`make logs`), under a header
//! that is printed once.

use core::fmt::{self, Write};

use crate::sensor::SensorError;
use crate::state_machine::State;

/// Consecutive silent probes before a sensor is flagged as suspect.
///
/// One probe is emitted per second, so a sensor that never answers for this
/// many seconds is reported as a probable wiring problem.
pub const SILENT_PROBES_BEFORE_WARNING: u16 = 10;

/// What a distance sensor answered on the last probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorState {
    /// The sensor measured a target at this many centimetres.
    Distance(u16),
    /// The sensor answered with no echo (nothing in range, or a dead line).
    NoEcho,
    /// The sensor failed at the pin level: almost certainly wiring.
    Wiring,
}

impl SensorState {
    /// The distance to feed the state machine, if any.
    pub fn distance(self) -> Option<u16> {
        match self {
            SensorState::Distance(cm) => Some(cm),
            SensorState::NoEcho | SensorState::Wiring => None,
        }
    }
}

impl From<Result<u16, SensorError>> for SensorState {
    fn from(result: Result<u16, SensorError>) -> Self {
        match result {
            Ok(cm) => SensorState::Distance(cm),
            Err(SensorError::Timeout) => SensorState::NoEcho,
            Err(SensorError::Pin) => SensorState::Wiring,
        }
    }
}

impl fmt::Display for SensorState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SensorState::Distance(cm) => {
                write!(f, "{cm} cm")?;
                pad(f, digits(u32::from(*cm)) + 3)
            }
            SensorState::NoEcho => {
                f.write_str("no echo")?;
                pad(f, 7)
            }
            SensorState::Wiring => {
                f.write_str("wiring?")?;
                pad(f, 7)
            }
        }
    }
}

/// One complete picture of the prop, ready to be rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
    /// The behavior state the state machine is in right now.
    pub state: State,
    /// Last reading of the left sensor.
    pub left: SensorState,
    /// Last reading of the right sensor.
    pub right: SensorState,
    /// The left sensor has been silent for too long: probable fault.
    pub left_warning: bool,
    /// The right sensor has been silent for too long: probable fault.
    pub right_warning: bool,
    /// Last angle the head servo was moved to, in degrees.
    pub angle: u8,
    /// Whether the servo received a new angle since the last snapshot.
    pub servo_moved: bool,
    /// Last track the audio player was asked to play (`None`: none so far).
    pub track: Option<u16>,
    /// The last servo command failed.
    pub servo_failed: bool,
    /// The last audio command failed.
    pub audio_failed: bool,
}

/// Writes the header printed once when a host opens the log.
///
/// The column labels line up with [`Snapshot::render_row`].
pub fn render_header(out: &mut dyn Write) -> fmt::Result {
    writeln!(out, "SCARY MONSTER PROBE")?;
    writeln!(
        out,
        "{:<11}{:<13}{:<13}{:<12}{:<13}",
        "state", "left", "right", "servo", "mp3 player"
    )
}

impl Snapshot {
    /// Writes one line holding the current value of every component.
    pub fn render_row(&self, out: &mut dyn Write) -> fmt::Result {
        write!(
            out,
            "{:<11}{:<13}{:<13}{:<12}{:<13}",
            state_name(self.state),
            self.left,
            self.right,
            Angle {
                degrees: self.angle,
                moved: self.servo_moved,
            },
            Track(self.track)
        )?;
        self.render_problems(out)?;
        writeln!(out)
    }

    /// Appends a short note naming every component that looks broken.
    fn render_problems(&self, out: &mut dyn Write) -> fmt::Result {
        if !self.has_problems() {
            return Ok(());
        }
        out.write_str("  !")?;
        if self.left_is_problem() {
            out.write_str(" left sensor")?;
        }
        if self.right_is_problem() {
            out.write_str(" right sensor")?;
        }
        if self.servo_failed {
            out.write_str(" servo")?;
        }
        if self.audio_failed {
            out.write_str(" mp3")?;
        }
        Ok(())
    }

    /// Whether the left sensor is a problem: explicit wiring failure or a
    /// prolonged silence reported by the [`Monitor`].
    fn left_is_problem(&self) -> bool {
        matches!(self.left, SensorState::Wiring) || self.left_warning
    }

    /// Whether the right sensor is a problem.
    fn right_is_problem(&self) -> bool {
        matches!(self.right, SensorState::Wiring) || self.right_warning
    }

    /// Whether any component is reporting a problem.
    fn has_problems(&self) -> bool {
        self.left_is_problem() || self.right_is_problem() || self.servo_failed || self.audio_failed
    }
}

/// Human-readable name of a behavior state.
fn state_name(state: State) -> &'static str {
    match state {
        State::Idle => "Idle",
        State::Follow => "Follow",
        State::Action => "Action",
        State::PostAction => "PostAction",
    }
}

/// A servo angle rendered for the dashboard.
///
/// A move (a new angle was written) is flagged with a trailing `*`.
struct Angle {
    degrees: u8,
    moved: bool,
}

impl fmt::Display for Angle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} deg", self.degrees)?;
        let mut len = digits(u32::from(self.degrees)) + 4;
        if self.moved {
            f.write_str(" *")?;
            len += 2;
        }
        pad(f, len)
    }
}

/// An audio track rendered for the dashboard.
struct Track(Option<u16>);

impl fmt::Display for Track {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(track) => {
                write!(f, "track {track}")?;
                pad(f, digits(u32::from(track)) + 6)
            }
            None => {
                f.write_str("-")?;
                pad(f, 1)
            }
        }
    }
}

/// Pads a value written so far to the width requested by the formatter.
///
/// A custom [`fmt::Display`] does not inherit the padding of the outer format
/// string, so the value is followed by spaces to fill its column.
fn pad(f: &mut fmt::Formatter<'_>, text_len: usize) -> fmt::Result {
    for _ in text_len..f.width().unwrap_or(0) {
        f.write_str(" ")?;
    }
    Ok(())
}

/// Number of decimal digits needed to write `n`.
fn digits(mut n: u32) -> usize {
    let mut count = 1;
    while n >= 10 {
        n /= 10;
        count += 1;
    }
    count
}

/// Tracks how long each sensor has been answering with no echo.
///
/// A single empty reading is normal; a sensor that stays silent for
/// [`SILENT_PROBES_BEFORE_WARNING`] probes in a row is likely unplugged, and
/// the [`Snapshot`] then reports it as a problem.
#[derive(Debug, Default)]
pub struct Monitor {
    left_silent: u16,
    right_silent: u16,
}

impl Monitor {
    /// Creates a monitor that has not seen any reading yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records one probe from both sensors.
    pub fn observe(&mut self, left: SensorState, right: SensorState) {
        self.left_silent = advance(self.left_silent, left);
        self.right_silent = advance(self.right_silent, right);
    }

    /// Whether the left sensor has been silent for too long.
    pub fn left_warning(&self) -> bool {
        self.left_silent >= SILENT_PROBES_BEFORE_WARNING
    }

    /// Whether the right sensor has been silent for too long.
    pub fn right_warning(&self) -> bool {
        self.right_silent >= SILENT_PROBES_BEFORE_WARNING
    }
}

/// Resets the silence counter when a target is measured, counts up otherwise.
fn advance(silent: u16, sensor: SensorState) -> u16 {
    match sensor {
        SensorState::Distance(_) => 0,
        SensorState::NoEcho | SensorState::Wiring => silent.saturating_add(1),
    }
}

/// A fixed-capacity text sink that formats without allocating.
///
/// Rendering the dashboard on a bare-metal target cannot use an allocator, so
/// the text is written into a stack buffer of `N` bytes. Writing past the end
/// fails instead of truncating, which the caller can notice and skip.
pub struct FixedText<const N: usize> {
    bytes: [u8; N],
    len: usize,
}

impl<const N: usize> FixedText<N> {
    /// Creates an empty buffer.
    pub const fn new() -> Self {
        Self {
            bytes: [0; N],
            len: 0,
        }
    }

    /// The bytes written so far, ready to be streamed as ASCII text.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

impl<const N: usize> Default for FixedText<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Write for FixedText<N> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        write_into(&mut self.bytes, &mut self.len, text)
    }
}

/// Appends `text` to `bytes`, failing when it would not fit.
///
/// Kept non-generic so the success and failure paths are compiled once and can
/// be fully covered, instead of once per [`FixedText`] capacity.
fn write_into(bytes: &mut [u8], len: &mut usize, text: &str) -> fmt::Result {
    let end = *len + text.len();
    if end > bytes.len() {
        return Err(fmt::Error);
    }
    bytes[*len..end].copy_from_slice(text.as_bytes());
    *len = end;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(state: State, left: SensorState, right: SensorState) -> Snapshot {
        Snapshot {
            state,
            left,
            right,
            left_warning: false,
            right_warning: false,
            angle: 90,
            servo_moved: false,
            track: None,
            servo_failed: false,
            audio_failed: false,
        }
    }

    fn row(snapshot: &Snapshot) -> String {
        let mut out = String::new();
        snapshot.render_row(&mut out).unwrap();
        out
    }

    fn header() -> String {
        let mut out = String::new();
        render_header(&mut out).unwrap();
        out
    }

    #[test]
    fn header_names_the_board_and_every_column() {
        let header = header();

        assert!(header.starts_with("SCARY MONSTER PROBE\n"));
        for column in ["state", "left", "right", "servo", "mp3 player"] {
            assert!(header.contains(column), "missing column {column}");
        }
    }

    #[test]
    fn renders_a_row_with_every_component() {
        let snapshot = Snapshot {
            state: State::Follow,
            left: SensorState::Distance(42),
            right: SensorState::NoEcho,
            left_warning: false,
            right_warning: false,
            angle: 45,
            servo_moved: false,
            track: Some(1),
            servo_failed: false,
            audio_failed: false,
        };

        let out = row(&snapshot);

        assert!(out.starts_with("Follow     "));
        assert!(out.contains("42 cm"));
        assert!(out.contains("no echo"));
        assert!(out.contains("45 deg"));
        assert!(out.contains("track 1"));
        assert!(!out.contains('*'), "a still head is not flagged");
        assert!(!out.contains('!'), "a healthy row has no problem note");
    }

    #[test]
    fn flags_a_servo_move_with_a_star() {
        let mut snapshot = snapshot(
            State::Follow,
            SensorState::Distance(40),
            SensorState::Distance(120),
        );
        snapshot.angle = 45;
        snapshot.servo_moved = true;

        assert!(row(&snapshot).contains("45 deg *"));
    }

    #[test]
    fn row_shows_a_dash_while_the_mp3_player_is_silent() {
        let snapshot = snapshot(
            State::Follow,
            SensorState::Distance(40),
            SensorState::Distance(45),
        );

        let out = row(&snapshot);

        assert!(out.contains('-'));
        assert!(!out.contains("track"));
    }

    #[test]
    fn renders_the_post_action_state() {
        let snapshot = snapshot(
            State::PostAction,
            SensorState::Distance(20),
            SensorState::NoEcho,
        );

        assert!(row(&snapshot).contains("PostAction"));
    }

    #[test]
    fn row_lists_a_silent_sensor() {
        let mut snapshot = snapshot(
            State::Follow,
            SensorState::NoEcho,
            SensorState::Distance(80),
        );
        snapshot.left_warning = true;

        let out = row(&snapshot);

        assert!(out.contains("! left sensor"));
        assert!(!out.contains("right sensor"));
    }

    #[test]
    fn row_lists_failing_actuators() {
        let mut snapshot = snapshot(
            State::Action,
            SensorState::Distance(20),
            SensorState::Distance(80),
        );
        snapshot.servo_failed = true;
        snapshot.audio_failed = true;

        let out = row(&snapshot);

        assert!(out.contains("! servo mp3"));
        assert!(!out.contains("sensor"));
    }

    #[test]
    fn row_lists_every_problem() {
        let mut snapshot = snapshot(State::Action, SensorState::Wiring, SensorState::Wiring);
        snapshot.left_warning = true;
        snapshot.right_warning = true;
        snapshot.servo_failed = true;
        snapshot.audio_failed = true;

        assert!(row(&snapshot).contains("! left sensor right sensor servo mp3"));
    }

    #[test]
    fn monitor_starts_without_warnings() {
        let monitor = Monitor::new();

        assert!(!monitor.left_warning());
        assert!(!monitor.right_warning());
    }

    #[test]
    fn monitor_warns_after_a_sensor_stays_silent() {
        let mut monitor = Monitor::new();

        for _ in 0..SILENT_PROBES_BEFORE_WARNING {
            monitor.observe(SensorState::NoEcho, SensorState::Distance(80));
        }

        assert!(monitor.left_warning());
        assert!(!monitor.right_warning());
    }

    #[test]
    fn monitor_clears_the_warning_when_the_sensor_recovers() {
        let mut monitor = Monitor::new();
        for _ in 0..SILENT_PROBES_BEFORE_WARNING {
            monitor.observe(SensorState::NoEcho, SensorState::Distance(80));
        }

        monitor.observe(SensorState::Distance(80), SensorState::Distance(80));

        assert!(!monitor.left_warning());
    }

    #[test]
    fn monitor_counts_a_wiring_failure_like_a_silence() {
        let mut monitor = Monitor::new();

        for _ in 0..SILENT_PROBES_BEFORE_WARNING {
            monitor.observe(SensorState::Wiring, SensorState::Wiring);
        }

        assert!(monitor.left_warning());
        assert!(monitor.right_warning());
    }

    #[test]
    fn monitor_saturates_instead_of_overflowing() {
        let mut monitor = Monitor::new();

        for _ in 0..=u32::from(u16::MAX) {
            monitor.observe(SensorState::NoEcho, SensorState::NoEcho);
        }

        assert!(monitor.left_warning());
        assert!(monitor.right_warning());
    }

    #[test]
    fn converts_sensor_results_to_states() {
        assert_eq!(SensorState::from(Ok(85)), SensorState::Distance(85));
        assert_eq!(
            SensorState::from(Err(SensorError::Timeout)),
            SensorState::NoEcho
        );
        assert_eq!(
            SensorState::from(Err(SensorError::Pin)),
            SensorState::Wiring
        );
    }

    #[test]
    fn exposes_the_distance_of_a_sensor_state() {
        assert_eq!(SensorState::Distance(85).distance(), Some(85));
        assert_eq!(SensorState::NoEcho.distance(), None);
        assert_eq!(SensorState::Wiring.distance(), None);
    }

    #[test]
    fn fixed_text_accumulates_written_strs() {
        let mut text = FixedText::<16>::new();

        text.write_str("mon").unwrap();
        text.write_str("ster").unwrap();

        assert_eq!(text.as_bytes(), b"monster");
    }

    #[test]
    fn fixed_text_starts_empty() {
        assert_eq!(FixedText::<8>::default().as_bytes(), b"");
    }

    #[test]
    fn fixed_text_rejects_text_that_does_not_fit() {
        let mut text = FixedText::<4>::new();

        assert_eq!(text.write_str("12345"), Err(fmt::Error));
        assert_eq!(text.as_bytes(), b"");
    }

    #[test]
    fn fixed_text_holds_a_full_row() {
        let snapshot = snapshot(
            State::Follow,
            SensorState::Distance(120),
            SensorState::Distance(35),
        );
        let mut text = FixedText::<640>::new();

        snapshot.render_row(&mut text).unwrap();

        assert_eq!(text.as_bytes(), row(&snapshot).as_bytes());
    }

    /// A sink that accepts at most `remaining` bytes before failing.
    struct FailAfter {
        remaining: usize,
    }

    impl Write for FailAfter {
        fn write_str(&mut self, text: &str) -> fmt::Result {
            if text.len() > self.remaining {
                return Err(fmt::Error);
            }
            self.remaining -= text.len();
            Ok(())
        }
    }

    #[test]
    fn render_propagates_every_write_failure() {
        let with_problems = Snapshot {
            state: State::Action,
            left: SensorState::Wiring,
            right: SensorState::NoEcho,
            left_warning: true,
            right_warning: true,
            angle: 180,
            servo_moved: true,
            track: Some(65535),
            servo_failed: true,
            audio_failed: true,
        };
        let clean = snapshot(
            State::Follow,
            SensorState::Distance(120),
            SensorState::Distance(35),
        );

        for snapshot in [with_problems, clean] {
            let full = row(&snapshot);

            for remaining in 0..full.len() {
                let mut writer = FailAfter { remaining };
                assert_eq!(
                    snapshot.render_row(&mut writer),
                    Err(fmt::Error),
                    "render_row must fail with at most {remaining} bytes"
                );
            }
        }

        let full = header();
        for remaining in 0..full.len() {
            let mut writer = FailAfter { remaining };
            assert_eq!(
                render_header(&mut writer),
                Err(fmt::Error),
                "render_header must fail with at most {remaining} bytes"
            );
        }
    }
}
