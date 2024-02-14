use std::sync::Arc;

use axum::{routing::get, Router};
use serialport::{DataBits, SerialPortInfo, SerialPortType, UsbPortInfo};
use tokio::sync::Mutex;
use tower_http::services::{ServeDir, ServeFile};

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
#[derive(Clone)]
struct AppState {
    port: Arc<Mutex<SerialPortInfo>>,
    speed: Arc<Mutex<u32>>,
}

#[tokio::main]
async fn main() {
    let port = serialport::available_ports()
        .expect("Expected to get serial ports")
        // Use this if the previous vector is no longer needed
        .into_iter()
        .find(is_usb_serial_adapter)
        .expect("Could not find serial port");

    let port = Arc::new(Mutex::new(port));
    let app_state = AppState {
        port,
        speed: Arc::new(Mutex::new(9600)),
    };

    let serve_dir = ServeDir::new("../client/dist")
        .not_found_service(ServeFile::new("../client/dist/index.html"));

    let app = Router::new()
        .route("/api", get(|| async { "Hello, World!" }))
        .fallback_service(serve_dir)
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
