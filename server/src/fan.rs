use std::{
    io,
    sync::{Arc, Mutex, MutexGuard},
};

use crc::{Crc, CRC_16_MODBUS};
use serialport::SerialPort;

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
