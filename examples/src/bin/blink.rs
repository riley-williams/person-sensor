// This example turns on the onboard LED when a certain number of faces are detected

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    gpio::{Input, Level, Output, Pull},
    i2c::{self, Config},
    peripherals::I2C1,
};
use person_sensor::PersonSensor;

use {defmt_rtt as _, panic_probe as _};
bind_interrupts!(struct Irqs {
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
});

/// The number of faces that must be detected to turn on the LED
const NUM_FACES: i8 = 2;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

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
            if result.num_faces >= NUM_FACES {
                led.set_high();
            } else {
                led.set_low();
            }
        };
    }
}
