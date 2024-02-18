use std::{
    convert::Infallible,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    extract::State,
    response::{sse::Event, Sse},
    routing::get,
    Router,
};
use crc::{Crc, CRC_16_MODBUS};
use serialport::{
    DataBits, Parity, SerialPort, SerialPortInfo, SerialPortType, StopBits, UsbPortInfo,
};
use tokio::stream;
use tokio_stream::StreamExt as _;
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

const PORT_NAME: &str = "/dev/cu.usbserial-2150";
const BAUD_RATE: u32 = 19_200;
const FAN_ADDRESS: u8 = 0x02;
const MAX_SET_POINT: u16 = 64_000;

mod function_codes {
    pub const READ_INPUT_REGISTERS: u8 = 0x04;
    pub const WRITE_SINGLE_REGISTER: u8 = 0x06;
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
enum UpdateSetPointError {
    SerialPortError(serialport::Error),
    /// The value is larger than 6400
    ValueTooLarge,
}

impl From<serialport::Error> for UpdateSetPointError {
    fn from(error: serialport::Error) -> Self {
        UpdateSetPointError::SerialPortError(error)
    }
}

impl From<std::io::Error> for UpdateSetPointError {
    fn from(error: std::io::Error) -> Self {
        UpdateSetPointError::SerialPortError(error.into())
    }
}

fn set_current_set_point(
    port: &mut Box<dyn SerialPort>,
    set_point: u16,
) -> Result<(), UpdateSetPointError> {
    if set_point > MAX_SET_POINT {
        return Err(UpdateSetPointError::ValueTooLarge);
    }

    // Build the message
    let mut message = [0x00u8; 8];

    // Device address
    message[0] = FAN_ADDRESS;

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
    port.write_all(&message)?;

    // Read the response
    // Address 1 + Function code 1 + Address 2 + Value 2 + 2 bytes of CRC
    const RESPONSE_LENGTH: usize = 7;
    let response_buffer = &mut [0u8; RESPONSE_LENGTH];
    port.read_exact(response_buffer)?;
    //TODO validate the response

    Ok(())
}

#[derive(Clone)]
struct AppState {
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    /// The speed of the motor in RPM
    /// None if no value has been read yet
    set_point: Arc<Mutex<Option<u16>>>,
    set_point_sender: Arc<tokio::sync::watch::Sender<Option<u16>>>,
}

fn open_serial_port() -> serialport::Result<Box<dyn SerialPort>> {
    serialport::new(PORT_NAME, BAUD_RATE)
        .timeout(Duration::from_secs(10))
        .data_bits(DataBits::Eight)
        .stop_bits(StopBits::One)
        .parity(Parity::Even)
        .open()
}

mod api {
    use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

    use crate::{AppState, MAX_SET_POINT};

    pub async fn get_current_set_point(
        State(state): State<AppState>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let mut value = state.set_point.lock().unwrap();

        if let Some(value) = value.as_ref() {
            return Ok(value.to_string());
        }

        // Else load and set value as we are the master for modbus and the only ones that can change it on the device
        let mut port = state.port.lock().map_err(|error| {
            log::error!("Failed to access serial port: {}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        let new_value = crate::get_current_set_point(&mut *port).map_err(|error| {
            log::error!("Failed to read current set point: {}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        *value = Some(new_value);
        Ok(new_value.to_string())
    }

    pub async fn update_set_point(
        State(state): State<AppState>,
        Json(value): Json<u16>,
    ) -> impl IntoResponse {
        if value > MAX_SET_POINT {
            return (
                StatusCode::BAD_REQUEST,
                format!("Value needs to be {MAX_SET_POINT} or less"),
            )
                .into_response();
        }

        // Don't update the value if it's the same
        let is_same = {
            let current_value = *state.set_point_sender.borrow();
            current_value.is_some_and(|current_value| current_value == value)
        };

        if is_same {
            return (StatusCode::OK, value.to_string()).into_response();
        }

        let port = state.port.lock();

        let mut port = match port {
            Ok(port) => port,
            Err(error) => {
                log::error!("Failed to access serial port: {}", error);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to access serial port",
                )
                    .into_response();
            }
        };

        if let Err(error) = crate::set_current_set_point(&mut *port, value) {
            return match error {
                crate::UpdateSetPointError::SerialPortError(error) => {
                    log::error!("Failed to update set point: {}", error);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to update set point",
                    )
                        .into_response()
                }
                crate::UpdateSetPointError::ValueTooLarge => (
                    StatusCode::BAD_REQUEST,
                    format!("Value needs to be {MAX_SET_POINT} or less"),
                )
                    .into_response(),
            };
        };

        state.set_point.lock().unwrap().replace(value);
        // Update other connected clients with new value
        state.set_point_sender.send(Some(value)).unwrap();

        let value = value.to_string();
        return (StatusCode::OK, value).into_response();
    }
}

async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    // let stream = stream::repeat_with(|| Event::default().data("hello"));
    // tokio_stream::wrappers::WatchStream::new(rx)

    let receiver = state.set_point_sender.subscribe();
    let stream = tokio_stream::wrappers::WatchStream::new(receiver)
        .map(|value| {
            let value = value
                .map(|value| value.to_string())
                .unwrap_or("None".to_string());
            Event::default().data(value)
        })
        .map(Ok);

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

#[tokio::main]
async fn main() {
    // Serial port setup
    let port = open_serial_port();

    let port = match port {
        Ok(port) => port,
        Err(error) => {
            // List available ports
            let ports = serialport::available_ports().unwrap();
            // Combine list of ports in one string
            let ports = ports
                .iter()
                .map(|port| format!("{}: {:?}", port.port_name, port.port_type))
                .collect::<Vec<String>>()
                .join("\n");

            panic!("Failed to open serial port: {}\n Does the port exist? Is it already in use? Can the app access it?\n Available ports:\n{}",error, ports);
        }
    };
    let port = Arc::new(Mutex::new(port));

    // App state setup
    let (sender, _receiver) = tokio::sync::watch::channel(None::<u16>);

    let app_state = AppState {
        port,
        set_point: Arc::new(Mutex::new(None)),
        set_point_sender: Arc::new(sender),
    };

    // Set up logging
    simple_logger::init_with_level(log::Level::Info).expect("couldn't initialize logging");

    // SPA setup
    let serve_dir = ServeDir::new("../client/dist")
        .not_found_service(ServeFile::new("../client/dist/index.html"));

    // Set up API routes
    let api_routes = Router::new()
        .route(
            "/fan/2/setpoint",
            get(api::get_current_set_point).patch(api::update_set_point),
        )
        .route("/sse", get(sse_handler));

    // Combine everything into a single app
    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(serve_dir)
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    log::info!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
