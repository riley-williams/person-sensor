#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    clocks::ClockConfig,
    gpio::{Input, Level, Output, Pull},
    i2c::{self, Config},
    peripherals::I2C1,
};
use person_sensor::PersonSensor;

use {defmt_rtt as _, panic_probe as _};
bind_interrupts!(struct Irqs {
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let clock_config = ClockConfig::crystal(12_000_000);
    // clock_config.sys_clk.div_int = 100;
    // clock_config.sys_clk.div_frac = 12;
    let config = embassy_rp::config::Config::new(clock_config);
    let p = embassy_rp::init(config);

    // Set up I2C1 on pins 2 and 3
    let sda = p.PIN_2;
    let scl = p.PIN_3;
    let i2c = embassy_rp::i2c::I2c::new_async(p.I2C1, scl, sda, Irqs, Config::default());

    let interrupt = p.PIN_4;
    let interrupt = Input::new(interrupt, Pull::Down);
    let mut person_sensor = PersonSensor::new(i2c, interrupt);

    let mut led = Output::new(p.PIN_25, Level::Low);

    // repeatedly loop in continuous capture mode
    // The pico LED should turn on in sync with the sensor LED
    loop {
        if let Ok(result) = person_sensor.read_results().await {
            if result.num_faces > 0 {
                led.set_high();
            } else {
                led.set_low();
            }
        };
    }
}
