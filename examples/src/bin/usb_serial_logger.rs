// Logs the sensor data over USB serial.
// It is critical to run this example with `--release` to avoid panics
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    gpio::{Input, Level, Output, Pull},
    i2c::{self, Config},
    peripherals::{I2C1, USB},
    usb::{Driver, InterruptHandler},
};
use embassy_time::Timer;
use embedded_hal::digital::OutputPin;
use person_sensor::PersonSensor;
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
    let clock_config = embassy_rp::clocks::ClockConfig::crystal(12_000_000);
    // clock_config.sys_clk.div_int = 100;
    // clock_config.sys_clk.div_frac = 12;
    let config = embassy_rp::config::Config::new(clock_config);
    let p = embassy_rp::init(config);
    // let p = embassy_rp::init(Default::default());

    // set up USB serial logging
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    // Set up I2C1 on pins 2 and 3
    let sda = p.PIN_2;
    let scl = p.PIN_3;
    let i2c = embassy_rp::i2c::I2c::new_async(p.I2C1, scl, sda, Irqs, Config::default());

    let interrupt = p.PIN_4;
    let interrupt = Input::new(interrupt, Pull::Down);

    let mut person_sensor = PersonSensor::new(i2c, interrupt);

    let mut led = Output::new(p.PIN_25, Level::Low);

    log::info!("Looking for calibration face - smile!");

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
    // The pico LED should turn on in sync with the sensor LED
    loop {
        if let Ok(result) = person_sensor.read_results().await {
            if result.num_faces > 0 {
                result
                    .faces
                    .iter()
                    // .filter(|f| f.box_confidence > 0)
                    .enumerate()
                    .for_each(|(i, face)| {
                        let center_x = (face.box_left + face.box_right) / 2;
                        let center_y = (face.box_top + face.box_bottom) / 2;
                        let size_x = face.box_right - face.box_left;
                        let size_y = face.box_bottom - face.box_top;
                        if face.id_confidence > 0 {
                            log::info!(
                                "Person {} - x:{}, y:{} - {}x{} - ID: {}",
                                i,
                                center_x,
                                center_y,
                                size_x,
                                size_y,
                                face.id
                            );
                        } else {
                            log::info!(
                                "Person {} - x:{}, y:{} - {}x{} - Unknown",
                                i,
                                center_x,
                                center_y,
                                size_x,
                                size_y
                            );
                        }
                    });
            }
        };
    }
}
