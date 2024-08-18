# Useful Things Person Sensor

A small driver for the Useful Things Person Sensor.

Original [https://usfl.ink/ps_dev](developer guide)

This driver has been tested with v1.1 of the sensor, but should also work with v1 and v2.
If you're able to validate the other board revisions, please open a pr :)

## Usage

The sensor offers two modes: continuous and single shot.
It can be converted between the two modes, and the compiler will prevent you from misusing
functionality that is only available in a specific mode, or when an interrupt pin is provided.

```rust
use person_sensor::PersonSensor;

let i2c = /* ... */;
let interrupt_pin = /* ... */;

// The driver can be initialized with or without the interrupt pin using the builder
let mut person_sensor = PersonSensorBuilder::new_standby(i2c)
    .with_interrupt(interrupt_pin) // optional
    .build()
    .await
    .unwrap();

let detections = person_sensor.capture_once().await.unwrap();

// ERROR: an interrupt pin was provided, but person_sensor is in standby mode
// person_sensor.wait_for_person().await.unwrap();

// To use the functionality in continuous mode, convert the sensor like below,
// or use the builder with new_continuous(...)
let mut person_sensor = sensor.into_continuous_mode();

person_sensor.get_detections().await.unwrap();

// Now we meet all the requirements to wait for the next detection using the interrupt
_ = person_sensor.wait_for_person().await.unwrap();
// Read the latest detections.
// Note wait_for_person() does not automatically read the detections
let detections = person_sensor.get_detections().await.unwrap();
```

## Examples

To run the examples on a pi pico, it should be sufficient to enter bootloader mode and run:

```bash
cd examples
cargo run --bin <example_name> --release
```

## License

Licensed under either of

Apache License, Version 2.0 (LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
MIT license (LICENSE-MIT or <http://opensource.org/licenses/MIT>)
at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
