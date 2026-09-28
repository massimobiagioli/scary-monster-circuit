//! Deterministic test doubles for the firmware's hardware traits.
//!
//! The sensor doubles model the *virtual wires* of the circuit: the clock and
//! both sensor pins share one cell of simulated microseconds, so a test states
//! how far away the target is and lets the real driver measure it.

#![allow(dead_code)]

use core::cell::{Cell, RefCell};
use std::rc::Rc;

use embedded_hal::digital::{
    Error as DigitalError, ErrorKind as DigitalErrorKind, ErrorType as DigitalErrorType, InputPin,
    OutputPin,
};
use embedded_hal::pwm::{
    Error as PwmErrorTrait, ErrorKind as PwmErrorKind, ErrorType as PwmErrorType, SetDutyCycle,
};
use embedded_io::{
    Error as IoErrorTrait, ErrorKind as IoErrorKind, ErrorType as IoErrorType, Write,
};

use crate::audio::AudioPlayer;
use crate::sensor::{HcSr04, MicrosClock, SensorError};
use crate::servo::Head;

/// Maximum duty cycle of the firmware's PWM setup (`set_top(39_062)`).
pub const MAX_DUTY: u16 = 39_063;

/// Microseconds of echo width per centimetre, as measured by the driver.
pub const US_PER_CM: u32 = 58;

/// Delay between the trigger pulse and the echo starting, in microseconds.
const ECHO_DELAY_US: u32 = 2;

/// Error returned by a mocked digital pin.
#[derive(Debug, PartialEq, Eq)]
pub struct PinError;

impl DigitalError for PinError {
    fn kind(&self) -> DigitalErrorKind {
        DigitalErrorKind::Other
    }
}

/// Error returned by a mocked PWM channel.
#[derive(Debug, PartialEq, Eq)]
pub struct PwmError;

impl PwmErrorTrait for PwmError {
    fn kind(&self) -> PwmErrorKind {
        PwmErrorKind::Other
    }
}

/// Error returned by a mocked byte sink.
#[derive(Debug, PartialEq, Eq)]
pub struct IoError;

impl IoErrorTrait for IoError {
    fn kind(&self) -> IoErrorKind {
        IoErrorKind::Other
    }
}

/// State shared by the mocked sensor, its pins and its clock.
#[derive(Default)]
struct SensorWire {
    now_us: Cell<u32>,
    echo: Cell<Option<(u32, u32)>>,
    target_cm: Cell<Option<u16>>,
    trigger_low_fails: Cell<bool>,
    trigger_low_fails_after_pulse: Cell<bool>,
    trigger_high_fails: Cell<bool>,
    echo_high_read_fails: Cell<bool>,
    echo_low_read_fails: Cell<bool>,
    trigger_pulses: Cell<u32>,
}

impl SensorWire {
    fn tick(&self) -> u32 {
        let now = self.now_us.get().wrapping_add(1);
        self.now_us.set(now);
        now
    }

    fn now(&self) -> u32 {
        self.now_us.get()
    }

    /// Answers the trigger pulse with the echo a real sensor would send back.
    fn arm_echo(&self) {
        let width = self.target_cm.get().map(|cm| u32::from(cm) * US_PER_CM);
        self.echo.set(width.map(|width| {
            let rise = self.now().wrapping_add(ECHO_DELAY_US);
            (rise, rise.wrapping_add(width))
        }));
    }

    fn echo_is_high(&self) -> bool {
        match self.echo.get() {
            Some((rise, fall)) => self.now() >= rise && self.now() < fall,
            None => false,
        }
    }

    fn echo_width_us(&self) -> Option<u32> {
        self.echo.get().map(|(rise, fall)| fall - rise)
    }
}

/// A microsecond clock that advances one microsecond per read.
pub struct TestClock(Rc<SensorWire>);

impl MicrosClock for TestClock {
    fn now_micros(&mut self) -> u32 {
        self.0.tick()
    }
}

/// The trigger pin of a mocked HC-SR04.
pub struct TestTrig(Rc<SensorWire>);

impl DigitalErrorType for TestTrig {
    type Error = PinError;
}

impl OutputPin for TestTrig {
    fn set_low(&mut self) -> Result<(), PinError> {
        let fails_after_pulse =
            self.0.trigger_low_fails_after_pulse.get() && self.0.trigger_pulses.get() > 0;
        if self.0.trigger_low_fails.get() || fails_after_pulse {
            return Err(PinError);
        }
        self.0.arm_echo();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), PinError> {
        if self.0.trigger_high_fails.get() {
            return Err(PinError);
        }
        self.0.trigger_pulses.set(self.0.trigger_pulses.get() + 1);
        Ok(())
    }
}

/// The echo pin of a mocked HC-SR04.
pub struct TestEcho(Rc<SensorWire>);

impl DigitalErrorType for TestEcho {
    type Error = PinError;
}

impl InputPin for TestEcho {
    fn is_high(&mut self) -> Result<bool, PinError> {
        if self.0.echo_high_read_fails.get() {
            return Err(PinError);
        }
        Ok(self.0.echo_is_high())
    }

    fn is_low(&mut self) -> Result<bool, PinError> {
        if self.0.echo_low_read_fails.get() {
            return Err(PinError);
        }
        Ok(!self.0.echo_is_high())
    }
}

/// An HC-SR04 on mocked hardware, with a target the test can move around.
pub struct SensorRig {
    wire: Rc<SensorWire>,
    clock: TestClock,
    sensor: HcSr04<TestTrig, TestEcho>,
}

impl SensorRig {
    /// A sensor looking at a target `distance_cm` away (`None`: out of range).
    pub fn new(distance_cm: Option<u16>) -> Self {
        let wire = Rc::new(SensorWire::default());
        wire.target_cm.set(distance_cm);
        let clock = TestClock(Rc::clone(&wire));
        let sensor = HcSr04::new(TestTrig(Rc::clone(&wire)), TestEcho(Rc::clone(&wire)));
        Self {
            wire,
            clock,
            sensor,
        }
    }

    /// Takes one measurement, exactly as the main loop does.
    pub fn measure(&mut self) -> Result<u16, SensorError> {
        self.sensor.distance_cm(&mut self.clock)
    }

    /// Moves the target in front of the sensor.
    pub fn set_distance(&self, distance_cm: Option<u16>) {
        self.wire.target_cm.set(distance_cm);
    }

    /// How many trigger pulses the sensor has sent.
    pub fn pulse_count(&self) -> u32 {
        self.wire.trigger_pulses.get()
    }

    /// Width of the echo pulse the sensor answered the last trigger with.
    pub fn echo_width_us(&self) -> Option<u32> {
        self.wire.echo_width_us()
    }

    /// Makes the trigger pin fail while driving low.
    pub fn fail_trigger_low(&self, fail: bool) {
        self.wire.trigger_low_fails.set(fail);
    }

    /// Makes the trigger pin fail while driving low, but only after the pulse.
    pub fn fail_trigger_low_after_pulse(&self, fail: bool) {
        self.wire.trigger_low_fails_after_pulse.set(fail);
    }

    /// Makes the trigger pin fail while driving high.
    pub fn fail_trigger_high(&self, fail: bool) {
        self.wire.trigger_high_fails.set(fail);
    }

    /// Makes the echo pin fail every time it is read.
    pub fn fail_echo_read(&self, fail: bool) {
        self.fail_echo_high_read(fail);
        self.fail_echo_low_read(fail);
    }

    /// Makes the echo pin fail while waiting for the pulse to start.
    pub fn fail_echo_high_read(&self, fail: bool) {
        self.wire.echo_high_read_fails.set(fail);
    }

    /// Makes the echo pin fail while waiting for the pulse to end.
    pub fn fail_echo_low_read(&self, fail: bool) {
        self.wire.echo_low_read_fails.set(fail);
    }
}

/// What a mocked PWM channel was commanded to do.
#[derive(Default)]
struct PwmState {
    duty: Option<u16>,
    fail: bool,
}

/// A readable handle on the state of a mocked PWM channel.
#[derive(Clone)]
pub struct PwmHandle(Rc<RefCell<PwmState>>);

impl PwmHandle {
    /// The duty cycle the channel was last set to.
    pub fn duty(&self) -> Option<u16> {
        self.0.borrow().duty
    }

    /// Makes every duty cycle update fail.
    pub fn fail(&self, fail: bool) {
        self.0.borrow_mut().fail = fail;
    }
}

/// The PWM channel driving the head servo.
pub struct TestPwm {
    max_duty: u16,
    state: Rc<RefCell<PwmState>>,
}

impl PwmErrorType for TestPwm {
    type Error = PwmError;
}

impl SetDutyCycle for TestPwm {
    fn max_duty_cycle(&self) -> u16 {
        self.max_duty
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), PwmError> {
        let mut state = self.state.borrow_mut();
        if state.fail {
            return Err(PwmError);
        }
        state.duty = Some(duty);
        Ok(())
    }
}

/// A head servo on a mocked PWM channel, plus a handle to inspect it.
pub fn servo_head() -> (Head<TestPwm>, PwmHandle) {
    servo_head_with_max_duty(MAX_DUTY)
}

/// A head servo on a mocked PWM channel with a custom maximum duty cycle.
pub fn servo_head_with_max_duty(max_duty: u16) -> (Head<TestPwm>, PwmHandle) {
    let state = Rc::new(RefCell::new(PwmState::default()));
    let pwm = TestPwm {
        max_duty,
        state: Rc::clone(&state),
    };
    (Head::new(pwm), PwmHandle(state))
}

/// A UART that records every byte written to it.
#[derive(Default)]
pub struct TestWriter {
    pub bytes: Vec<u8>,
    pub fail: bool,
}

impl IoErrorType for TestWriter {
    type Error = IoError;
}

impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, IoError> {
        if self.fail {
            return Err(IoError);
        }
        self.bytes.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), IoError> {
        Ok(())
    }
}

/// An audio player that records the tracks it was asked to play.
#[derive(Default)]
pub struct TestPlayer {
    pub tracks: Vec<u16>,
    pub fail: bool,
}

impl AudioPlayer for TestPlayer {
    type Error = IoError;

    fn play_track(&mut self, track: u16) -> Result<(), IoError> {
        if self.fail {
            return Err(IoError);
        }
        self.tracks.push(track);
        Ok(())
    }
}
