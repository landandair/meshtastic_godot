use crate::mesh_connection::packet_handler::MessageEnvelope;
use meshtastic::protobufs::{FromRadio, ToRadio};
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
pub enum IPCMessage {
    FromRadio(FromRadio),
    ToRadio(ToRadio),
    SendMessage(MessageEnvelope),
}


pub struct RadioIPC{
    pub(crate) from_radio_tx: Sender<IPCMessage>,
    pub(crate) to_radio_rx: Receiver<IPCMessage>
}

pub struct InterfaceIPC{
    pub(crate) from_radio_rx: Receiver<IPCMessage>,
    pub(crate) to_radio_tx: Sender<IPCMessage>,
}

