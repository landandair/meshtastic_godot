use crate::connection::Connection;
use crate::ipc::IPCMessage;
use crate::mt_node::meshtastic_loop;


use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use anyhow::{Result};

pub fn create_thread_ipc() -> (Sender<IPCMessage>, Receiver<IPCMessage>, Sender<IPCMessage>, Receiver<IPCMessage>) {
    let (mut fromradio_thread_tx, mut fromradio_thread_rx) =
        channel::<IPCMessage>(100);
    let (mut toradio_thread_tx, mut toradio_thread_rx) =
        channel::<IPCMessage>(100);
    (fromradio_thread_tx, fromradio_thread_rx, toradio_thread_tx, toradio_thread_rx)
}

pub fn start_meshtastic_loop(conn:Connection, fromradio_tx:Sender<IPCMessage>, toradio_thread_rx:Receiver<IPCMessage>) -> JoinHandle<Result<()>> {
    let mut join_handle: JoinHandle<Result<()>> = tokio::task::spawn(async move {
        meshtastic_loop(conn, fromradio_tx, toradio_thread_rx).await
    });
    join_handle
}



