use std::{
    convert::Infallible,
    io,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    extract::State,
    response::{sse::Event, Sse},
    routing::get,
    Router,
};
use fan::Fan;
use serialport::{
    DataBits, Parity, SerialPort, SerialPortInfo, SerialPortType, StopBits, UsbPortInfo,
};
use tokio_stream::StreamExt as _;
use tower_http::services::{ServeDir, ServeFile};

mod fan;
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
        } if product_name == "USB Serial" && port_name == fan_defaults::PORT_NAME => true,
        _ => false,
    }
}

mod fan_defaults {
    use std::time::Duration;

    use serialport::{DataBits, Parity, StopBits};

    pub(crate) const PORT_NAME: &str = "/dev/cu.usbserial-2150";
    pub(crate) const BAUD_RATE: u32 = 19_200;
    pub(crate) const FAN_ADDRESS: u8 = 0x02;
    pub(crate) const DURATION: Duration = Duration::from_secs(1);
    pub(crate) const DATA_BITS: DataBits = DataBits::Eight;
    pub(crate) const PARITY: Parity = Parity::Even;
    pub(crate) const STOP_BITS: StopBits = StopBits::One;
}

#[derive(Clone)]
struct AppState {
    fan: Fan,
    /// The speed of the motor in RPM
    /// None if no value has been read yet
    set_point: Arc<Mutex<Option<u16>>>,
    set_point_sender: Arc<tokio::sync::watch::Sender<Option<u16>>>,
}

fn open_serial_port() -> serialport::Result<Box<dyn SerialPort>> {
    use fan_defaults::*;
    serialport::new(PORT_NAME, BAUD_RATE)
        .timeout(DURATION)
        .data_bits(DATA_BITS)
        .stop_bits(STOP_BITS)
        .parity(PARITY)
        .open()
}

mod api {
    use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

    use crate::fan::MAX_SET_POINT;
    use crate::{
        fan::{self, UpdateSetPointError},
        AppState,
    };

    pub async fn get_current_set_point(
        State(state): State<AppState>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let mut value = state.set_point.lock().unwrap();

        if let Some(value) = value.as_ref() {
            return Ok(value.to_string());
        }

        // Else load and set value as we are the master for modbus and the only ones that can change it on the device

        let new_value = match state.fan.get_current_set_point() {
            Ok(value) => value,
            Err(error) => match error {
                fan::GetSetPointError::SerialPortError(error) => {
                    log::error!("Failed to read current set point: {}", error);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
                fan::GetSetPointError::PoisonError(error) => {
                    log::error!("Failed to access serial port: {}", error);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            },
        };

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

        let result = state.fan.set_current_set_point(value);
        if let Err(error) = result {
            return match error {
                UpdateSetPointError::SerialPortError(error) => {
                    log::error!("Failed to update set point: {}", error);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to update set point",
                    )
                        .into_response()
                }
                UpdateSetPointError::ValueTooLarge => (
                    StatusCode::BAD_REQUEST,
                    format!("Value needs to be {MAX_SET_POINT} or less"),
                )
                    .into_response(),
                UpdateSetPointError::PoisonError(error) => {
                    log::error!("Failed to access serial port: {}", error);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to update set point",
                    )
                        .into_response()
                }
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

    let fan = Fan::new(fan_defaults::FAN_ADDRESS, port);
    let app_state = AppState {
        fan,
        set_point: Arc::new(Mutex::new(None)),
        set_point_sender: Arc::new(sender),
    };

    // Set up logging
    simple_logger::init_with_level(log::Level::Info).expect("couldn't initialize logging");

    // SPA setup
    // Not used during development where we use vite for serving the client and reverse proxying API requests to the server
    let serve_dir = ServeDir::new("../client/dist")
        // If the route is a client side navigation route, this will serve the app and let the app router take over the
        // path handling after the app is loaded
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
