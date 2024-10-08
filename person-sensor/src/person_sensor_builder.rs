use core::marker::PhantomData;

use embedded_hal_async::{digital::Wait, i2c::I2c};

use crate::{
    person_sensor::{ContinuousCaptureMode, PersonSensorMode, StandbyMode},
    PersonSensor,
};

/// Builder for the [`PersonSensor`] driver
///
/// Use this to create a new instance of the `PersonSensor` driver
pub struct PersonSensorBuilder<I2C, INT, MODE> {
    i2c: I2C,
    interrupt: INT,
    mode: PhantomData<MODE>,
    id_enabled: bool,
}

impl<I2C> PersonSensorBuilder<I2C, (), ()>
where
    I2C: I2c,
{
    /// Create a new driver instance without an interrupt, initialized in standby mode
    pub fn new_standby(i2c: I2C, id_enabled: bool) -> PersonSensorBuilder<I2C, (), StandbyMode> {
        PersonSensorBuilder {
            i2c,
            interrupt: (),
            mode: PhantomData,
            id_enabled,
        }
    }

    /// Create a new driver instance without an interrupt, initialized in continuous mode
    pub fn new_continuous(
        i2c: I2C,
        id_enabled: bool,
    ) -> PersonSensorBuilder<I2C, (), ContinuousCaptureMode> {
        PersonSensorBuilder {
            i2c,
            interrupt: (),
            mode: PhantomData,
            id_enabled,
        }
    }
}

impl<I2C, MODE> PersonSensorBuilder<I2C, (), MODE>
where
    I2C: I2c,
{
    /// Sets an interrupt pin
    pub fn with_interrupt<INT: Wait>(self, interrupt: INT) -> PersonSensorBuilder<I2C, INT, MODE> {
        PersonSensorBuilder {
            i2c: self.i2c,
            interrupt,
            mode: self.mode,
            id_enabled: self.id_enabled,
        }
    }
}

impl<I2C, INT> PersonSensorBuilder<I2C, INT, ContinuousCaptureMode>
where
    I2C: I2c,
{
    /// Initialize the sensor in continuous mode
    pub async fn build(self) -> Result<PersonSensor<I2C, INT, ContinuousCaptureMode>, I2C::Error> {
        let mut sensor = PersonSensor {
            i2c: self.i2c,
            interrupt: self.interrupt,
            mode: PhantomData,
        };
        sensor.set_mode(PersonSensorMode::Continuous).await?;
        sensor.enable_id_model(self.id_enabled).await?;
        Ok(sensor)
    }
}

impl<I2C, INT> PersonSensorBuilder<I2C, INT, StandbyMode>
where
    I2C: I2c,
{
    /// Initialize the sensor in standby mode
    pub async fn build(self) -> Result<PersonSensor<I2C, INT, StandbyMode>, I2C::Error> {
        let mut sensor = PersonSensor {
            i2c: self.i2c,
            interrupt: self.interrupt,
            mode: PhantomData,
        };
        sensor.set_mode(PersonSensorMode::Standby).await?;
        sensor.enable_id_model(true).await?;
        Ok(sensor)
    }
}
