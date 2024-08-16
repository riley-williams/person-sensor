// This example flashes the pico onboard LED 3 times, then captures whatever face is present / largest
// as ID 0. The pico onboard LED will turn on when this face is recognized again.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    gpio::{Input, Level, Output, Pull},
    i2c::{self, Config},
    peripherals::I2C1,
};
use embassy_time::Timer;
use person_sensor::PersonSensor;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
});

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

    Timer::after_millis(800).await;
    led.set_high();

    Timer::after_millis(800).await;
    led.set_low();

    Timer::after_millis(800).await;
    led.set_high();

    Timer::after_millis(800).await;
    led.set_low();

    Timer::after_millis(800).await;
    led.set_high();

    Timer::after_millis(1600).await;

    person_sensor
        .label_next_id(0.try_into().unwrap())
        .await
        .unwrap();

    led.set_low();

    // repeatedly loop in continuous capture mode
    // The pico LED will turn on in sync with the sensor LED when the calibrated face is detected
    loop {
        if let Ok(result) = person_sensor.read_results().await {
            if result.num_faces > 0 && result.faces.iter().any(|face| face.id_confidence > 90) {
                led.set_low();
            } else {
                led.set_high();
            }
        };
    }
}
