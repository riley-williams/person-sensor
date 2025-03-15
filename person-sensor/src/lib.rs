//! # Useful Things Person Sensor
//!
//! A small driver for the Useful Things Person Sensor.
//!
//! Original [developer guide](https://usfl.ink/ps_dev)
//!
//! This driver has been tested with v1.1 of the sensor, but should also work with v1 and v2.
//! If you're able to validate the other board revisions, please open a PR to update this message :)
//!
//! ## Usage
//!
//! The sensor offers two modes: continuous and single shot.
//! It can be converted between the two modes, and the compiler will prevent you from misusing
//! functionality that is only available in a specific mode, or when an interrupt pin is provided.
//!
//! ```ignore
//! use person_sensor::PersonSensorBuilder;
//!
//! let i2c = /* ... */;
//! let interrupt_pin = /* ... */;
//!
//! // The driver can be initialized with or without the interrupt pin using the builder
//! let mut person_sensor = PersonSensorBuilder::new_standby(i2c, true)
//!     .with_interrupt(interrupt_pin) // optional
//!     .build()
//!     .await
//!     .unwrap();
//!
//! let detections = person_sensor.capture_once().await.unwrap();
//!
//! // ERROR: an interrupt pin was provided, but person_sensor is in standby mode
//! // person_sensor.wait_for_person().await.unwrap();
//!
//! // To use the functionality in continuous mode, convert the sensor like below,
//! // or use the builder with new_continuous(...)
//! let mut person_sensor = sensor.into_continuous_mode();
//!
//! // Now we meet all the requirements to wait for the next detection using the interrupt
//! _ = person_sensor.wait_for_person().await.unwrap();
//! // Read the latest detections.
//! // Note wait_for_person() does not automatically read the detections
//! let detections = person_sensor.get_detections().await.unwrap();
//! ```
//!
//! ## Examples
//!
//! To run the examples on a pi pico, it should be sufficient to enter bootloader mode and run:
//!
//! ```bash
//! cd examples
//! cargo run --bin <example_name> --release
//! ```

#![no_std]

mod person_sensor;
mod person_sensor_builder;

pub use person_sensor::IDMode;
pub use person_sensor::PersonSensor;
pub use person_sensor::ReadError;
pub use person_sensor_builder::PersonSensorBuilder;

/// The number of detections returned by the sensor.
const MAX_DETECTIONS: usize = 4;

#[repr(C, packed)]
#[derive(Debug, Clone, PartialEq)]
pub struct Face {
    /// Confidence of the box prediction, ranges from 1 to 100.
    pub box_confidence: u8,
    pub box_left: u8,
    pub box_top: u8,
    pub box_right: u8,
    pub box_bottom: u8,
    /// The confidence of "calibrated" identities ranges from 1 to 100, and will be 0 or less if
    /// this face is not recognized as any of the calibrated identities or is not the largest face
    /// in the frame.
    pub id_confidence: i8,
    /// The ID number of the face, if it is recognized as any of the
    /// calibrated identities *and* is the largest face in the frame.
    /// By default, the sensor will not run any recognition until calibration has been performed.
    /// After at least one person has been calibrated, the sensor will always run recognition on
    /// the largest face present, and assign an ID number if it’s recognized as one that it has been calibrated on.
    pub id: Option<PersonID>,
    /// Indicates if somebody is looking directly at the device
    /// > Note: ID works most reliably when the face is straight on to the sensor
    pub is_facing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PersonIDError {
    /// IDs can only range from 0 to 7.
    InvalidId,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PersonID(u8);

impl PersonID {
    pub fn new(id: u8) -> Result<Self, PersonIDError> {
        if id < 8 {
            Ok(PersonID(id))
        } else {
            Err(PersonIDError::InvalidId)
        }
    }

    /// Create a new person ID without checking the value
    #[must_use]
    pub fn new_unchecked(id: u8) -> Self {
        PersonID(id)
    }
}

impl TryFrom<u8> for PersonID {
    type Error = PersonIDError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PersonID> for u8 {
    fn from(id: PersonID) -> u8 {
        id.0
    }
}
