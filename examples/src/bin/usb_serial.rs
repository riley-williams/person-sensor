//! This example logs capture results over USB serial
//!
//! IMPORTANT: It is critical to run this example with `--release` to avoid panics due to USB timing

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    i2c::{self, Config, I2c},
    peripherals::{I2C1, USB},
    usb::{Driver, InterruptHandler},
};
use embassy_time::Timer;
use person_sensor::PersonSensorBuilder;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(4096, log::LevelFilter::Info, driver);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // set up USB serial logging
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    // Set up I2C1 on pins 2 and 3
    let sda = p.PIN_2;
    let scl = p.PIN_3;
    let i2c = I2c::new_async(p.I2C1, scl, sda, Irqs, Config::default());

    // Create a sensor instance without an interrupt, initialized in continuous mode, with the ID
    // model enabled
    let mut person_sensor = PersonSensorBuilder::new_continuous(i2c, true)
        .build()
        .await
        .unwrap();

    // Turn off the indicator LED
    person_sensor.set_indicator(false).await.unwrap();

    loop {
        Timer::after_millis(200).await;
        if let Ok(faces) = person_sensor.get_detections().await {
            if faces.is_empty() {
                log::info!("No faces detected");
                continue;
            }

            faces.iter().for_each(|face| {
                let center_x = face.box_left / 2 + face.box_right / 2;
                let center_y = face.box_top / 2 + face.box_bottom / 2;
                let size_x = face.box_right - face.box_left;
                let size_y = face.box_bottom - face.box_top;

                match face.id {
                    Some(id) => log::info!(
                        "Person {} - x:{}, y:{} - {}x{}",
                        u8::from(id),
                        center_x,
                        center_y,
                        size_x,
                        size_y,
                    ),
                    None => log::info!(
                        "Person _ - x:{}, y:{} - {}x{}",
                        center_x,
                        center_y,
                        size_x,
                        size_y,
                    ),
                };
            });
        } else {
            log::info!("Error reading faces");
        }
    }
}
