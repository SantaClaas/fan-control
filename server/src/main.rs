use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{routing::get, Router};
use crc::{Crc, CRC_16_MODBUS};
use serialport::{
    DataBits, Parity, SerialPort, SerialPortInfo, SerialPortType, StopBits, UsbPortInfo,
};
use tower_http::services::{ServeDir, ServeFile};

const CRC: Crc<u16> = Crc::<u16>::new(&CRC_16_MODBUS);
mod registers;

fn is_usb_serial_adapter(port: &SerialPortInfo) -> bool {
    match port {
        // We only support one very specific device
        SerialPortInfo {
            port_name,
            port_type:
                SerialPortType::UsbPort(UsbPortInfo {
                    vid: 6790,
                    pid: 29987,
                    serial_number: None,
                    manufacturer: None,
                    product: Some(product_name),
                }),
        } if product_name == "USB Serial" && port_name == PORT_NAME => true,
        _ => false,
    }
}

const PORT_NAME: &str = "/dev/cu.usbserial-150";
const BAUD_RATE: u32 = 9_600;
const FAN_ADDRESS: u8 = 0x02;

mod function_codes {
    pub const READ_INPUT_REGISTERS: u8 = 0x04;
}

fn get_current_set_point(port: &mut Box<dyn SerialPort>) -> Result<u16, serialport::Error> {
    // Build the message
    let mut message = [0x00u8; 8];

    // Device address
    message[0] = FAN_ADDRESS;

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

#[derive(Clone)]
struct AppState {
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    /// The speed of the motor in RPM
    /// None if no value has been read yet
    set_point: Arc<Mutex<Option<u16>>>,
}

fn open_serial_port() -> serialport::Result<Box<dyn SerialPort>> {
    serialport::new(PORT_NAME, BAUD_RATE)
        .timeout(Duration::from_secs(10))
        .data_bits(DataBits::Eight)
        .stop_bits(StopBits::One)
        .parity(Parity::None)
        .open()
}

mod api {
    use axum::{
        extract::{rejection::LengthLimitError, State},
        http::{header::ValueDrain, response, StatusCode},
        response::IntoResponse,
    };

    use crate::AppState;

    pub async fn get_current_set_point(
        State(state): State<AppState>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let mut value = state.set_point.lock().unwrap();

        if let Some(value) = value.as_ref() {
            return Ok(value.to_string());
        }

        // Else load and set value as we are the master for modbus and the only ones that can change it on the device
        let mut port = state.port.lock().map_err(|error| {
            eprintln!("Failed to open serial port: {}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        let new_value = crate::get_current_set_point(&mut *port).map_err(|error| {
            eprintln!("Failed to read current set point: {}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        *value = Some(new_value);
        Ok(new_value.to_string())
    }
}

#[tokio::main]
async fn main() {
    let port = open_serial_port().expect("Failed to open serial port. Does the port exist? Is it already in use? Can the app access it?");
    let port = Arc::new(Mutex::new(port));
    let app_state = AppState {
        port,
        set_point: Arc::new(Mutex::new(None)),
    };

    let serve_dir = ServeDir::new("../client/dist")
        .not_found_service(ServeFile::new("../client/dist/index.html"));

    let app = Router::new()
        .route(
            "/api/fan/2/setpoint/current",
            get(api::get_current_set_point),
        )
        .fallback_service(serve_dir)
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
