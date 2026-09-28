#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
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
    watchdog::Watchdog,
    Clock, Timer,
};

use scary_monster_lib::audio::DfPlayer;
use scary_monster_lib::controller::Controller;
use scary_monster_lib::sensor::{HcSr04, MicrosClock};
use scary_monster_lib::servo::Head;
use scary_monster_lib::state_machine::Reading;

/// Microsecond clock backed by the RP2040 timer.
struct RpTimer(Timer);

impl MicrosClock for RpTimer {
    fn now_micros(&mut self) -> u32 {
        self.0.get_counter().ticks() as u32
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

    let mut controller = Controller::new();
    info!("scary monster ready");

    loop {
        let reading = Reading {
            left_cm: left_sensor.distance_cm(&mut clock).ok(),
            right_cm: right_sensor.distance_cm(&mut clock).ok(),
        };

        if let Some(track) = controller.update(reading, &mut head, &mut audio) {
            info!("person close: playing track {}", track);
        }

        delay.delay_ms(50);
    }
}
