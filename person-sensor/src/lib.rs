#![no_std]

mod person_sensor;
mod person_sensor_builder;

pub use person_sensor::PersonSensor;
pub use person_sensor_builder::PersonSensorBuilder;

/// The number of detections returned by the sensor.
const NUM_DETECTIONS: usize = 4;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
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
    pub id: i8,
    /// Indicates if somebody is looking directly at the device
    /// Note ID works most reliably when the face is straight on to the sensor
    pub is_facing: u8,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct ResultsHeader {
    reserved: [u8; 2],
    data_size: u16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct PersonSensorResults {
    header: ResultsHeader,
    pub num_faces: i8,
    pub faces: [Face; NUM_DETECTIONS],
    checksum: u16,
}

#[derive(Debug)]
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
