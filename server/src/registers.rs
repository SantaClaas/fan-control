pub struct Register {
    pub address: u16,
    /// Register length in bytes
    pub length: u8,
}

pub mod holding_registers {
    pub const REFERENCE_SET_POINT: u16 = 0xd001;
}

pub mod input_registers {
    use super::Register;

    /// Encoding matches [REFERENCE_SET_POINT](crate::registers::holding_registers::REFERENCE_SET_POINT)
    // pub const CURRENT_SET_POINT: u16 = 0xD01A;
    pub const CURRENT_SET_POINT: Register = Register {
        address: 0xD01A,
        length: 2,
    };
}
