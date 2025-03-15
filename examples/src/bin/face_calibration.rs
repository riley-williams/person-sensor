//! This example flashes the Pico onboard LED 3 times, then captures whatever face is present / largest
//! as ID 0. The Pico onboard LED will turn on when this face is recognized again.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    gpio::{Level, Output},
    i2c::{self, Config, I2c},
    peripherals::I2C1,
};
use embassy_time::Timer;
use person_sensor::PersonSensorBuilder;
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
    let i2c = I2c::new_async(p.I2C1, scl, sda, Irqs, Config::default());

    // Create a sensor instance without an interrupt, initialized in standby mode, with the ID
    // model enabled. This is not necessary for this example, and could be initially created in
    // continuous mode.
    let mut person_sensor = PersonSensorBuilder::new_standby(i2c, true)
        .build()
        .await
        .unwrap();

    // Turn on the sensor indicator LED
    person_sensor.set_indicator(true).await.unwrap();

    let mut led = Output::new(p.PIN_25, Level::High);

    // Blink the LED 3 times before attempting to capture a face
    for _ in 0..3 {
        led.set_high();
        Timer::after_millis(800).await;

        led.set_low();
        Timer::after_millis(800).await;
    }

    led.set_high();

    Timer::after_millis(1600).await;

    person_sensor
        .label_next_id(0.try_into().unwrap())
        .await
        .unwrap();

    led.set_low();

    // Convert the sensor to continuous mode.
    let mut person_sensor = person_sensor.into_continuous_mode().await.unwrap();

    // Repeatedly loop in continuous capture mode
    // The Pico LED will turn on in sync with the sensor LED when the calibrated face is detected
    loop {
        if let Ok(faces) = person_sensor.get_detections().await {
            if !faces.is_empty() && faces.iter().any(|face| face.id_confidence > 90) {
                led.set_high();
            } else {
                led.set_low();
            }
        };
    }
}
