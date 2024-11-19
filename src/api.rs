use crate::connection::Connection;
use crate::ipc::IPCMessage;

use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

pub fn create_serial_connection(port: &String) -> Connection {
    let conn = Connection::Serial(port.clone());
    conn
}

pub fn create_thread_ipc() -> (Sender<IPCMessage>, Receiver<IPCMessage>, Sender<IPCMessage>, Receiver<IPCMessage>) {
    let (mut fromradio_thread_tx, mut fromradio_thread_rx) =
        channel::<IPCMessage>(100);
    let (mut toradio_thread_tx, mut toradio_thread_rx) =
        channel::<IPCMessage>(100);
    (fromradio_thread_tx, fromradio_thread_rx, toradio_thread_tx, toradio_thread_rx)
}
