#![no_std]

use embedded_hal_async::{digital::Wait, i2c::I2c};

const PERSON_SENSOR_I2C_ADDRESS: u8 = 0x62;
const MAX_FACES: usize = 4;

#[repr(u8)]
pub enum PersonSensorMode {
    /// Lowest power mode, sensor is in standby and not capturing.
    Standby = 0x00,
    /// Capture continuously, setting the GPIO trigger pin to high if a face is detected.
    Continuous = 0x01,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Face {
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
pub struct ResultsHeader {
    reserved: [u8; 2],
    data_size: u16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct PersonSensorResults {
    header: ResultsHeader,
    pub num_faces: i8,
    pub faces: [Face; MAX_FACES],
    checksum: u16,
}

#[derive(Debug)]
pub enum PersonIDError {
    /// IDs can only range from 0 to 7.
    InvalidId,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
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

pub struct ContinuousCaptureMode;
pub struct StandbyMode;

pub struct PersonSensor<I2C, INT, MODE> {
    i2c: I2C,
    interrupt: INT,
    _mode: MODE,
}

impl<I2C, INT, MODE> PersonSensor<I2C, INT, MODE>
where
    I2C: I2c,
    INT: Wait,
{
    pub async fn read_results(&mut self) -> Result<PersonSensorResults, <I2C>::Error> {
        let mut buffer = [0u8; core::mem::size_of::<PersonSensorResults>()];
        self.i2c
            .read(PERSON_SENSOR_I2C_ADDRESS, &mut buffer)
            .await?;

        // TODO: hmm this is a bit sketchy
        let results = unsafe {
            core::mem::transmute::<
                [u8; core::mem::size_of::<PersonSensorResults>()],
                PersonSensorResults,
            >(buffer)
        };

        Ok(results)
    }

    async fn set_mode(&mut self, mode: PersonSensorMode) -> Result<(), I2C::Error> {
        self.i2c
            .write(PERSON_SENSOR_I2C_ADDRESS, &[0x01, mode as u8])
            .await
    }

    /// Enable / Disable the ID model. With this flag set to false, only bounding boxes are
    /// captured and the framerate is increased.
    pub async fn enable_id_model(&mut self, enable: bool) -> Result<(), I2C::Error> {
        self.i2c
            .write(PERSON_SENSOR_I2C_ADDRESS, &[0x02, enable as u8])
            .await
    }

    /// Calibrate the next identified frame as person N, from 0 to 7.
    /// If two frames pass with no person, this label is discarded.
    ///
    /// > Note: this will not return the result of the calibration, the only failure
    /// > is if the I2C write fails.
    pub async fn label_next_id(&mut self, id: PersonID) -> Result<(), I2C::Error> {
        self.i2c
            .write(PERSON_SENSOR_I2C_ADDRESS, &[0x04, id.into()])
            .await
    }

    /// Store any recognized IDs even when unpowered.
    pub async fn set_persist_ids(&mut self, persist: bool) -> Result<(), I2C::Error> {
        self.i2c
            .write(PERSON_SENSOR_I2C_ADDRESS, &[0x05, persist as u8])
            .await
    }

    /// Wipe any recognized IDs from storage.
    pub async fn erase_ids(&mut self) -> Result<(), I2C::Error> {
        self.i2c
            .write(PERSON_SENSOR_I2C_ADDRESS, &[0x06, 0x00])
            .await
    }

    /// Whether to enable the LED indicator on the sensor.
    pub async fn set_indicator(&mut self, enabled: bool) -> Result<(), I2C::Error> {
        self.i2c
            .write(PERSON_SENSOR_I2C_ADDRESS, &[0x07, enabled as u8])
            .await
    }

    /// Wait for the person sensor to trigger an interrupt indicating a person has been detected.
    /// Returns immediately if a person currently detected.
    pub async fn is_person_detected(&mut self) -> Result<(), INT::Error> {
        self.interrupt.wait_for_high().await
    }
}

impl<I2C, INT> PersonSensor<I2C, INT, StandbyMode>
where
    I2C: I2c,
    INT: Wait,
{
    pub async fn trigger_single_shot(&mut self) -> Result<(), I2C::Error> {
        self.i2c
            .write(PERSON_SENSOR_I2C_ADDRESS, &[0x03, 0x00])
            .await
    }

    pub async fn into_continuous_mode(
        self,
    ) -> Result<PersonSensor<I2C, INT, ContinuousCaptureMode>, I2C::Error> {
        let mut sensor = self;
        sensor.set_mode(PersonSensorMode::Continuous).await?;
        Ok(PersonSensor {
            i2c: sensor.i2c,
            interrupt: sensor.interrupt,
            _mode: ContinuousCaptureMode,
        })
    }
}

impl<I2C, INT> PersonSensor<I2C, INT, ContinuousCaptureMode>
where
    I2C: I2c,
    INT: Wait,
{
    pub fn new(i2c: I2C, interrupt: INT) -> PersonSensor<I2C, INT, ContinuousCaptureMode> {
        PersonSensor {
            i2c,
            interrupt,
            _mode: ContinuousCaptureMode,
        }
    }

    pub async fn into_standby_mode(
        self,
    ) -> Result<PersonSensor<I2C, INT, StandbyMode>, I2C::Error> {
        let mut sensor = self;
        sensor.set_mode(PersonSensorMode::Standby).await?;
        Ok(PersonSensor {
            i2c: sensor.i2c,
            interrupt: sensor.interrupt,
            _mode: StandbyMode,
        })
    }
}
