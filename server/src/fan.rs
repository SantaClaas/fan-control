use std::{
    io,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use crc::{Crc, CRC_16_MODBUS};
use serialport::{DataBits, Parity, SerialPort, StopBits};

use crate::registers;

mod function_codes {
    pub const READ_INPUT_REGISTERS: u8 = 0x04;
    pub const WRITE_SINGLE_REGISTER: u8 = 0x06;
}

const CRC: Crc<u16> = Crc::<u16>::new(&CRC_16_MODBUS);
pub(crate) const MAX_SET_POINT: u16 = 64_000;
/// Describes a fan device on the modbus
#[derive(Clone)]
pub(crate) struct Fan {
    address: u8,
    port: Arc<Mutex<Box<dyn SerialPort>>>,
}

#[derive(Debug)]
pub(crate) enum UpdateSetPointError<Guard> {
    SerialPortError(serialport::Error),
    /// The value is larger than 6400
    ValueTooLarge,
    PoisonError(std::sync::PoisonError<Guard>),
}

impl<Guard> From<serialport::Error> for UpdateSetPointError<Guard> {
    fn from(error: serialport::Error) -> Self {
        UpdateSetPointError::SerialPortError(error)
    }
}

impl<Guard> From<std::sync::PoisonError<Guard>> for UpdateSetPointError<Guard> {
    fn from(error: std::sync::PoisonError<Guard>) -> Self {
        UpdateSetPointError::PoisonError(error)
    }
}

impl<Guard> From<std::io::Error> for UpdateSetPointError<Guard> {
    fn from(error: std::io::Error) -> Self {
        UpdateSetPointError::SerialPortError(error.into())
    }
}

#[derive(Debug)]
pub(crate) enum GetSetPointError<Guard> {
    SerialPortError(io::Error),
    PoisonError(std::sync::PoisonError<Guard>),
}

impl<Guard> From<io::Error> for GetSetPointError<Guard> {
    fn from(error: io::Error) -> Self {
        GetSetPointError::SerialPortError(error)
    }
}

impl<Guard> From<std::sync::PoisonError<Guard>> for GetSetPointError<Guard> {
    fn from(error: std::sync::PoisonError<Guard>) -> Self {
        GetSetPointError::PoisonError(error)
    }
}

impl Fan {
    pub(crate) fn new(address: u8, port: Arc<Mutex<Box<dyn SerialPort>>>) -> Self {
        Self { address, port }
    }

    pub(crate) fn set_current_set_point(
        &self,
        set_point: u16,
    ) -> Result<(), UpdateSetPointError<MutexGuard<'_, Box<dyn SerialPort>>>> {
        if set_point > MAX_SET_POINT {
            return Err(UpdateSetPointError::ValueTooLarge);
        }

        // Build the message
        let mut message = [0x00u8; 8];

        // Device address
        message[0] = self.address;

        // Function code
        message[1] = function_codes::WRITE_SINGLE_REGISTER;

        // Address
        let address = registers::holding_registers::REFERENCE_SET_POINT.to_be_bytes();
        message[2] = address[0];
        message[3] = address[1];

        // Value
        let value = set_point.to_be_bytes();
        message[4] = value[0];
        message[5] = value[1];

        // Checksum
        let checksum: [u8; 2] = CRC.checksum(&message[..6]).to_be_bytes();
        // They come out reversed...
        message[6] = checksum[1];
        message[7] = checksum[0];

        // Send the message

        let mut port = self.port.lock()?;
        port.write_all(&message)?;

        // Read the response
        // Address 1 + Function code 1 + Address 2 + Value 2 + 2 bytes of CRC
        const RESPONSE_LENGTH: usize = 7;
        let response_buffer = &mut [0u8; RESPONSE_LENGTH];
        port.read_exact(response_buffer)?;
        //TODO validate the response

        Ok(())
    }

    pub(crate) fn get_current_set_point(
        &self,
    ) -> Result<u16, GetSetPointError<MutexGuard<'_, Box<dyn SerialPort>>>> {
        // Build the message
        let mut message = [0x00u8; 8];

        // Device address
        message[0] = self.address;

        // Function code
        message[1] = function_codes::READ_INPUT_REGISTERS;

        // Address
        let address = registers::input_registers::CURRENT_SET_POINT
            .address
            .to_be_bytes();
        message[2] = address[0];
        message[3] = address[1];

        // Number of registers
        // message[4] = 0x00;
        message[5] = 0x01;

        // Checksum
        let checksum: [u8; 2] = CRC.checksum(&message[..6]).to_be_bytes();
        // They come out reversed...
        message[6] = checksum[1];
        message[7] = checksum[0];

        // Send the message
        let mut port = self.port.lock()?;
        port.write_all(&message)?;

        // Read the response
        // Address 1 + Function code 1 + Byte count 1 + data n + 2 bytes of CRC
        const RESPONSE_LENGTH: usize =
            5 + registers::input_registers::CURRENT_SET_POINT.length as usize;
        let response_buffer = &mut [0u8; RESPONSE_LENGTH];
        port.read_exact(response_buffer)?;
        //TODO validate the response

        // Extract the value
        let value = u16::from_be_bytes([response_buffer[3], response_buffer[4]]);
        Ok(value)
    }
}

// Might expand to use mock in deveopment later to allow for testing without having the physical fan device
#[cfg(test)]
mod mock {
    use serialport::{DataBits, Parity, SerialPort, StopBits};
    use std::{io, time::Duration};

    #[derive(Clone)]
    pub(crate) struct MockedSerialPort {
        baud_rate: u32,
        data_bits: DataBits,
        flow_control: serialport::FlowControl,
        parity: Parity,
        stop_bits: StopBits,
        timeout: Duration,
    }

    impl Default for MockedSerialPort {
        /// Based on the default settings for the fan we are using in production
        fn default() -> Self {
            use crate::fan_defaults::*;
            Self {
                baud_rate: BAUD_RATE,
                data_bits: DATA_BITS,
                flow_control: serialport::FlowControl::None,
                parity: PARITY,
                stop_bits: STOP_BITS,
                timeout: DURATION,
            }
        }
    }

    impl io::Write for MockedSerialPort {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl io::Read for MockedSerialPort {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            Ok(buffer.len())
        }
    }

    impl SerialPort for MockedSerialPort {
        fn name(&self) -> Option<String> {
            let name = std::any::type_name::<MockedSerialPort>().to_string();
            Some(name)
        }

        fn baud_rate(&self) -> serialport::Result<u32> {
            Ok(self.baud_rate)
        }

        fn data_bits(&self) -> serialport::Result<DataBits> {
            Ok(self.data_bits)
        }

        fn flow_control(&self) -> serialport::Result<serialport::FlowControl> {
            Ok(self.flow_control)
        }

        fn parity(&self) -> serialport::Result<Parity> {
            Ok(self.parity)
        }

        fn stop_bits(&self) -> serialport::Result<StopBits> {
            Ok(self.stop_bits)
        }

        fn timeout(&self) -> Duration {
            self.timeout
        }

        fn set_baud_rate(&mut self, baud_rate: u32) -> serialport::Result<()> {
            self.baud_rate = baud_rate;
            Ok(())
        }

        fn set_data_bits(&mut self, data_bits: DataBits) -> serialport::Result<()> {
            self.data_bits = data_bits;
            Ok(())
        }

        fn set_flow_control(
            &mut self,
            flow_control: serialport::FlowControl,
        ) -> serialport::Result<()> {
            self.flow_control = flow_control;
            Ok(())
        }

        fn set_parity(&mut self, parity: Parity) -> serialport::Result<()> {
            self.parity = parity;
            Ok(())
        }

        fn set_stop_bits(&mut self, stop_bits: StopBits) -> serialport::Result<()> {
            self.stop_bits = stop_bits;
            Ok(())
        }

        fn set_timeout(&mut self, timeout: Duration) -> serialport::Result<()> {
            self.timeout = timeout;
            Ok(())
        }

        fn write_request_to_send(&mut self, _level: bool) -> serialport::Result<()> {
            Ok(())
        }

        fn write_data_terminal_ready(&mut self, _level: bool) -> serialport::Result<()> {
            Ok(())
        }

        fn read_clear_to_send(&mut self) -> serialport::Result<bool> {
            Ok(true)
        }

        fn read_data_set_ready(&mut self) -> serialport::Result<bool> {
            Ok(true)
        }

        fn read_ring_indicator(&mut self) -> serialport::Result<bool> {
            Ok(true)
        }

        fn read_carrier_detect(&mut self) -> serialport::Result<bool> {
            Ok(true)
        }

        fn bytes_to_read(&self) -> serialport::Result<u32> {
            Ok(0)
        }

        fn bytes_to_write(&self) -> serialport::Result<u32> {
            Ok(0)
        }

        fn clear(&self, _buffer_to_clear: serialport::ClearBuffer) -> serialport::Result<()> {
            Ok(())
        }

        fn try_clone(&self) -> serialport::Result<Box<dyn SerialPort>> {
            Ok(Box::new(MockedSerialPort {
                baud_rate: self.baud_rate,
                data_bits: self.data_bits,
                flow_control: self.flow_control,
                parity: self.parity,
                stop_bits: self.stop_bits,
                timeout: self.timeout,
            }))
        }

        fn set_break(&self) -> serialport::Result<()> {
            Ok(())
        }

        fn clear_break(&self) -> serialport::Result<()> {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use self::mock::MockedSerialPort;

    use super::*;
    use crate::fan_defaults::FAN_ADDRESS;

    // To be fair these tests aren't very useful yet
    #[test]
    fn can_set_set_point() {
        // Arrange
        let serial_port: MockedSerialPort = MockedSerialPort::default();
        let port: Arc<Mutex<Box<dyn SerialPort>>> = Arc::new(Mutex::new(Box::new(serial_port)));
        let fan = Fan::new(FAN_ADDRESS, port.clone());

        // Act
        let result = fan.set_current_set_point(0);

        // Assert
        result.expect("Failed to set set point");
    }

    #[test]
    fn can_get_set_point() {
        // Arrange
        let serial_port: MockedSerialPort = MockedSerialPort::default();
        let fan = Fan::new(FAN_ADDRESS, Arc::new(Mutex::new(Box::new(serial_port))));

        // Act
        let result = fan.get_current_set_point();

        // Assert
        let result = result.expect("Failed to get set point");
        assert_eq!(result, 0);
    }
}
