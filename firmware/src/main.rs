#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m::delay::Delay;
use cortex_m_rt::entry;
use rp_pico as bsp;

use bsp::hal::{
    clocks::init_clocks_and_plls,
    fugit::RateExtU32,
    pac,
    pwm::Slices as PwmSlices,
    sio::Sio,
    uart::{DataBits, StopBits, UartConfig, UartPeripheral},
    usb::UsbBus,
    watchdog::Watchdog,
    Clock, Timer,
};

use usb_device::class_prelude::UsbBusAllocator;
use usb_device::prelude::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid};
use usbd_serial::SerialPort;

use scary_monster_lib::audio::DfPlayer;
use scary_monster_lib::controller::Controller;
use scary_monster_lib::probe::{render_header, FixedText, Monitor, SensorState, Snapshot};
use scary_monster_lib::sensor::{HcSr04, MicrosClock};
use scary_monster_lib::servo::Head;
use scary_monster_lib::state_machine::{Reading, State};

/// Loop iterations between two live snapshots (50 ms each, so 1 s).
const PROBE_PERIOD_TICKS: u32 = 20;

/// Size of the stack buffer that holds one rendered snapshot row.
const PROBE_BUFFER_BYTES: usize = 640;

/// Size of the stack buffer that holds the one-time header.
const HEADER_BUFFER_BYTES: usize = 128;

/// Microsecond clock backed by the RP2040 timer.
struct RpTimer(Timer);

impl MicrosClock for RpTimer {
    fn now_micros(&mut self) -> u32 {
        self.0.get_counter().ticks() as u32
    }
}

/// Writes text over the USB serial port (CDC).
///
/// The USB stack runs in the main loop, so the prop needs nothing but a USB
/// cable to show its logs (`make logs`). Nothing is written until a host opens
/// the port, so no data is buffered while nobody is listening.
struct Logger<'a> {
    serial: SerialPort<'a, UsbBus>,
    usb_dev: UsbDevice<'a, UsbBus>,
}

impl<'a> Logger<'a> {
    fn new(bus: &'a UsbBusAllocator<UsbBus>) -> Self {
        let serial = SerialPort::new(bus);
        let usb_dev = UsbDeviceBuilder::new(bus, UsbVidPid(0x2e8a, 0x000a))
            .strings(&[StringDescriptors::default()
                .manufacturer("Scary Monster Circuit")
                .product("Scary Monster Circuit")
                .serial_number("SCARY-MONSTER")])
            .unwrap()
            .device_class(2)
            .build();
        Self { serial, usb_dev }
    }

    /// Polls the USB stack every millisecond for `ms` milliseconds.
    ///
    /// USB enumeration needs frequent polling, so the main loop must not sleep
    /// for long stretches without servicing the device.
    fn idle(&mut self, delay: &mut Delay, ms: u32) {
        for _ in 0..ms {
            self.poll();
            delay.delay_ms(1);
        }
    }

    /// Keeps the USB stack alive.
    fn poll(&mut self) {
        self.usb_dev.poll(&mut [&mut self.serial]);
    }

    /// Whether a host currently has the serial port open.
    fn connected(&self) -> bool {
        self.serial.dtr()
    }

    /// Writes `bytes` best-effort, only while a host has the port open.
    fn write(&mut self, bytes: &[u8]) {
        if !self.connected() {
            return;
        }
        let mut rest = bytes;
        while !rest.is_empty() {
            self.usb_dev.poll(&mut [&mut self.serial]);
            match self.serial.write(rest) {
                Ok(n) if n > 0 => rest = &rest[n..],
                _ => break,
            }
        }
    }
}

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    let clocks = init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut clock = RpTimer(Timer::new(pac.TIMER, &mut pac.RESETS, &clocks));

    let mut left_sensor = HcSr04::new(
        pins.gpio2.into_push_pull_output(),
        pins.gpio3.into_floating_input(),
    );
    let mut right_sensor = HcSr04::new(
        pins.gpio4.into_push_pull_output(),
        pins.gpio5.into_floating_input(),
    );

    let uart_pins = (pins.gpio0.into_function(), pins.gpio1.into_function());
    let uart_config = UartConfig::new(9600.Hz(), DataBits::Eight, None, StopBits::One);
    let uart = UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(uart_config, clocks.peripheral_clock.freq())
        .unwrap();
    let mut audio = DfPlayer::new(uart);

    let pwm_slices = PwmSlices::new(pac.PWM, &mut pac.RESETS);
    let mut pwm = pwm_slices.pwm0;
    pwm.set_div_int(64);
    pwm.set_div_frac(0);
    pwm.set_top(39_062);
    pwm.enable();
    pwm.channel_a.output_to(pins.gpio16);
    let mut head = Head::new(pwm.channel_a);

    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));
    let mut logger = Logger::new(&usb_bus);

    let mut controller = Controller::new();
    let mut monitor = Monitor::new();
    let mut last_track: Option<u16> = None;
    let mut since_report: u32 = 0;

    // Let the host enumerate the USB device before the slower main loop starts.
    logger.idle(&mut delay, 1000);

    let mut header_shown = false;
    let mut servo_moved = false;
    loop {
        logger.poll();

        if logger.connected() {
            if !header_shown {
                header_shown = true;
                let mut header = FixedText::<HEADER_BUFFER_BYTES>::new();
                if render_header(&mut header).is_ok() {
                    logger.write(header.as_bytes());
                }
            }
        } else {
            header_shown = false;
        }

        let left = SensorState::from(left_sensor.distance_cm(&mut clock));
        let right = SensorState::from(right_sensor.distance_cm(&mut clock));
        let reading = Reading {
            left_cm: left.distance(),
            right_cm: right.distance(),
        };

        if let Some(track) = controller.update(reading, &mut head, &mut audio) {
            last_track = Some(track);
        }
        servo_moved |= head.take_changed();

        monitor.observe(left, right);
        since_report += 1;
        if since_report >= PROBE_PERIOD_TICKS {
            since_report = 0;
            let state = controller.state();
            let snapshot = Snapshot {
                state,
                left,
                right,
                left_warning: monitor.left_warning(),
                right_warning: monitor.right_warning(),
                angle: head.angle(),
                servo_moved,
                track: if state == State::Action { last_track } else { None },
                servo_failed: controller.status().servo_failed,
                audio_failed: controller.status().audio_failed,
            };
            let mut text = FixedText::<PROBE_BUFFER_BYTES>::new();
            if snapshot.render_row(&mut text).is_ok() {
                logger.write(text.as_bytes());
            }
            servo_moved = false;
        }

        logger.idle(&mut delay, 50);
    }
}
