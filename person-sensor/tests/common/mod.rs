use embedded_hal_async::i2c::{self, ErrorKind, ErrorType, I2c, Operation, SevenBitAddress};

#[derive(Debug)]
pub struct MockPersonSensorBus<'a> {
    mode: u8,
    payload: &'a [u8; 39],
}

impl<'a> MockPersonSensorBus<'a> {
    pub fn new(mode: u8, payload: &'a [u8; 39]) -> Self {
        // Set the mode to 1 to indicate that the sensor is in continuous
        Self { mode, payload }
    }

    fn mock_write(&mut self, data: &[u8]) {
        match data[0] {
            0x01 => self.mode = data[1],
            0x03 => {
                assert_eq!(self.mode, 0);
            }
            _ => {}
        }
    }

    fn mock_read(&self, buffer: &mut [u8]) {
        buffer.copy_from_slice(&self.payload[..buffer.len()]);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockError {
    IoError,
}

impl i2c::Error for MockError {
    fn kind(&self) -> i2c::ErrorKind {
        ErrorKind::Other
    }
}

impl ErrorType for MockPersonSensorBus<'_> {
    type Error = MockError;
}

impl I2c<SevenBitAddress> for MockPersonSensorBus<'_> {
    async fn transaction(
        &mut self,
        address: SevenBitAddress,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        if address != 0x62 {
            return Err(MockError::IoError);
        }

        for operation in operations {
            match operation {
                Operation::Read(buffer) => self.mock_read(buffer),
                Operation::Write(data) => self.mock_write(data),
            }
        }
        Ok(())
    }
}
