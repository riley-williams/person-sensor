use core::marker::PhantomData;

use crc16::MCRF4XX;
use embedded_hal_async::{digital::Wait, i2c::I2c};

use crate::{Face, PersonID, MAX_DETECTIONS};

const PERSON_SENSOR_I2C_ADDRESS: u8 = 0x62;

#[repr(u8)]
pub(crate) enum PersonSensorMode {
    /// Lowest power mode, sensor is in standby and not capturing.
    Standby = 0x00,
    /// Capture continuously, setting the GPIO trigger pin to high if a face is detected.
    Continuous = 0x01,
}

/// The operating mode of the face labeling model.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum IDMode {
    /// Faces will be returned with both the location and ID, if IDs have been set up.
    ///
    /// The device will generate results at a lower rate in this mode due to the additional
    /// processing.
    LabelAndLocation = 0x01,
    /// Faces will be returned with only the location, even if IDs are persisted.
    ///
    /// The device will generate results at a higher rate in this mode.
    LocationOnly = 0x00,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ReadError<E> {
    ChecksumMismatch,
    I2CError(E),
}

impl<E> From<E> for ReadError<E> {
    fn from(error: E) -> Self {
        Self::I2CError(error)
    }
}

pub struct ContinuousCaptureMode;
pub struct StandbyMode;

/// Person sensor driver.
///
/// The sensor can be used in two modes: continuous capture and standby. In continuous capture
/// mode, the sensor continuously captures frames and sends the results over I2C. In standby mode,
/// the sensor is in a low-power state and only captures a single frame when requested.
///
/// To create the sensor, use a `PersonSensorBuilder`. The builder allows you to set the mode
/// and interrupt pin.
///
/// Example:
///
/// ```ignore
/// let sda = p.PIN_2;
/// let scl = p.PIN_3;
/// let i2c = I2c::new_async(p.I2C1, scl, sda, Irqs, Config::default());
/// let interrupt = p.PIN_4;
///
/// let mut person_sensor = PersonSensorBuilder::new_continuous(i2c, true)
///     .with_interrupt(interrupt)
///     .build()
///     .await
///     .unwrap();
///
/// loop {
///    if let Ok(faces) = person_sensor.get_detections().await {
///        // Do something with the results
///    }
/// }
/// ```
#[derive(Debug)]
pub struct PersonSensor<I2C, INT, MODE> {
    pub(crate) i2c: I2C,
    pub(crate) interrupt: INT,
    pub(crate) mode: PhantomData<MODE>,
    pub(crate) validate_checksum: bool,
}

impl<I2C, INT, MODE> PersonSensor<I2C, INT, MODE>
where
    I2C: I2c,
{
    /// Returns the latest results from the sensor.
    async fn latest_results(
        &mut self,
    ) -> Result<heapless::Vec<Face, MAX_DETECTIONS>, ReadError<I2C::Error>> {
        let mut buffer = [0u8; 39];
        self.i2c
            .read(PERSON_SENSOR_I2C_ADDRESS, &mut buffer)
            .await?;

        if self.validate_checksum {
            let checksum = crc16::State::<MCRF4XX>::calculate(&buffer[..37]);
            if u16::from_le_bytes([buffer[37], buffer[38]]) != checksum {
                return Err(ReadError::<I2C::Error>::ChecksumMismatch);
            }
        }

        let mut faces = heapless::Vec::<Face, MAX_DETECTIONS>::new();

        let num_faces = buffer[4];

        #[expect(clippy::cast_possible_wrap)]
        for face_num in 0..num_faces {
            let face_start_offset = 5 + face_num as usize * 8;

            let id_confidence = buffer[face_start_offset + 5] as i8;
            let person_id = match id_confidence {
                0 => None,
                _ => Some(PersonID::new_unchecked(buffer[face_start_offset + 6])),
            };

            let face = Face {
                box_confidence: buffer[face_start_offset],
                box_left: buffer[face_start_offset + 1],
                box_top: buffer[face_start_offset + 2],
                box_right: buffer[face_start_offset + 3],
                box_bottom: buffer[face_start_offset + 4],
                id_confidence,
                id: person_id,
                is_facing: buffer[face_start_offset + 7] > 0,
            };

            match faces.push(face) {
                Ok(()) => {}
                Err(_) => break,
            };
        }

        Ok(faces)
    }

    /// Sets the mode of the sensor.
    pub(crate) async fn set_mode(&mut self, mode: PersonSensorMode) -> Result<(), I2C::Error> {
        self.i2c
            .write(PERSON_SENSOR_I2C_ADDRESS, &[0x01, mode as u8])
            .await
    }

    /// Enable / Disable the ID model. Greater performance can be achieved by disabling ID labeling.
    pub async fn set_id_mode(&mut self, mode: IDMode) -> Result<(), I2C::Error> {
        self.i2c
            .write(PERSON_SENSOR_I2C_ADDRESS, &[0x02, mode as u8])
            .await
    }

    /// Enable / Disable the ID model.
    ///
    /// With this flag set to false, only bounding boxes are captured and the framerate is increased.
    #[deprecated(
        since = "0.3.1",
        note = "Please use `set_id_mode` instead. This method will be removed in a future release."
    )]
    pub async fn enable_id_model(&mut self, enable: bool) -> Result<(), I2C::Error> {
        self.i2c
            .write(PERSON_SENSOR_I2C_ADDRESS, &[0x02, u8::from(enable)])
            .await
    }

    /// Validate the checksum of the data received from the sensor. On by default.
    ///
    /// Checksum validation may be useful for diagnosing I2C communication issues,
    /// but is not necessary for normal operation.
    pub fn set_checksum_enabled(&mut self, enable: bool) {
        self.validate_checksum = enable;
    }

    /// Calibrate the next identified frame as person N, from 0 to 7.
    /// If two frames pass with no person, this label is discarded.
    ///
    /// This will not return the result of the calibration. The only failure
    /// is if the I2C write fails. You may wish to follow calibration with a
    /// check for detections containing the ID you just attempted to label.
    pub async fn label_next_id(&mut self, id: PersonID) -> Result<(), I2C::Error> {
        self.i2c
            .write(PERSON_SENSOR_I2C_ADDRESS, &[0x04, id.into()])
            .await
    }

    /// Store any recognized IDs even when unpowered.
    ///
    /// Both current and future IDs will be retained when this is set to true.
    pub async fn set_persist_ids(&mut self, persist: bool) -> Result<(), I2C::Error> {
        self.i2c
            .write(PERSON_SENSOR_I2C_ADDRESS, &[0x05, u8::from(persist)])
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
            .write(PERSON_SENSOR_I2C_ADDRESS, &[0x07, u8::from(enabled)])
            .await
    }
}

impl<I2C, INT> PersonSensor<I2C, INT, StandbyMode>
where
    I2C: I2c,
{
    /// Capture a single frame and reads the results
    pub async fn capture_once(
        &mut self,
    ) -> Result<heapless::Vec<Face, MAX_DETECTIONS>, ReadError<I2C::Error>> {
        self.i2c
            .write(PERSON_SENSOR_I2C_ADDRESS, &[0x03, 0x00])
            .await?;

        self.latest_results().await
    }

    /// Switches the sensor to continuous capture mode
    pub async fn into_continuous_mode(
        self,
    ) -> Result<PersonSensor<I2C, INT, ContinuousCaptureMode>, I2C::Error> {
        let mut sensor = self;
        sensor.set_mode(PersonSensorMode::Continuous).await?;
        Ok(PersonSensor {
            i2c: sensor.i2c,
            interrupt: sensor.interrupt,
            mode: PhantomData,
            validate_checksum: sensor.validate_checksum,
        })
    }
}

impl<I2C, INT> PersonSensor<I2C, INT, ContinuousCaptureMode>
where
    I2C: I2c,
{
    /// Switches the sensor into a lower power standby mode. Only single-shot capture is possible
    /// in this mode.
    pub async fn into_standby_mode(
        self,
    ) -> Result<PersonSensor<I2C, INT, StandbyMode>, I2C::Error> {
        let mut sensor = self;
        sensor.set_mode(PersonSensorMode::Standby).await?;
        Ok(PersonSensor {
            i2c: sensor.i2c,
            interrupt: sensor.interrupt,
            mode: PhantomData,
            validate_checksum: sensor.validate_checksum,
        })
    }

    /// Returns the latest results from the sensor.
    ///
    /// Depending on the device version and configuration, detections are updated at different
    /// rates. This method does not wait for _new_ detections to be available. The last detections
    /// will repeatedly read until the device produces new ones.
    ///
    /// It is your responsibility to either sensibly rate-limit fetching results to the maximum rate
    /// of the sensor or initialize the driver with an interrupt pin to be notified when new
    /// results are available.
    pub async fn get_detections(
        &mut self,
    ) -> Result<heapless::Vec<Face, MAX_DETECTIONS>, ReadError<I2C::Error>> {
        self.latest_results().await
    }
}

impl<I2C, INT> PersonSensor<I2C, INT, ContinuousCaptureMode>
where
    INT: Wait,
{
    /// Wait for the person sensor to trigger an interrupt indicating a person has been detected.
    /// Returns immediately if a person is currently detected.
    pub async fn wait_for_person(&mut self) -> Result<(), INT::Error> {
        self.interrupt.wait_for_high().await
    }
}
