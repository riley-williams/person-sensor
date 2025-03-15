//! This example turns on the onboard LED when a certain number of faces are detected
//! The Person Sensor must have the interrupt pin connected to GPIO 4

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    gpio::{Input, Level, Output, Pull},
    i2c::{self, Config, I2c},
    peripherals::I2C1,
};
use person_sensor::PersonSensorBuilder;

use {defmt_rtt as _, panic_probe as _};
bind_interrupts!(struct Irqs {
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
});

/// The number of faces that must be detected to turn on the LED
const NUM_FACES: usize = 2;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Set up I2C1 on pins 2 and 3
    let sda = p.PIN_2;
    let scl = p.PIN_3;
    let i2c = I2c::new_async(p.I2C1, scl, sda, Irqs, Config::default());

    let interrupt = p.PIN_4;
    let interrupt = Input::new(interrupt, Pull::Down);

    // Create a sensor instance with an interrupt, initialized in continuous mode, with the ID
    // model disabled
    let mut person_sensor = PersonSensorBuilder::new_continuous(i2c, false)
        .with_interrupt(interrupt)
        .build()
        .await
        .unwrap();

    let mut led = Output::new(p.PIN_25, Level::Low);

    // Wait for the interrupt pin to trigger a result, then read the number of faces
    // The Pico LED should turn on in sync with the sensor LED when enough faces are detected
    loop {
        person_sensor.wait_for_person().await.unwrap();
        if let Ok(faces) = person_sensor.get_detections().await {
            if faces.len() >= NUM_FACES {
                led.set_high();
            } else {
                led.set_low();
            }
        };
    }
}
