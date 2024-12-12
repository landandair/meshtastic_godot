use crate::mesh_connection::connection::Connection;
use crate::mesh_connection::ipc::{IPCMessage, InterfaceIPC, RadioIPC};
use crate::mesh_connection::mt_node::meshtastic_loop;
use crate::mesh_connection::util::get_secs;
use crate::mesh_connection::packet_handler::MessageEnvelope;

use meshtastic::packet::PacketDestination;
use meshtastic::types::{EncodedMeshPacketData, MeshChannel};

use tokio::sync::mpsc::{channel, Sender};
use anyhow::{Result};
use meshtastic::protobufs::PortNum;

pub fn create_thread_ipc() -> (InterfaceIPC, RadioIPC) {
    let (fromradio_thread_tx, fromradio_thread_rx) =
        channel::<IPCMessage>(1000);
    let (toradio_thread_tx, toradio_thread_rx) =
        channel::<IPCMessage>(1000);
    let iface = InterfaceIPC {
        from_radio_rx: fromradio_thread_rx,
        to_radio_tx: toradio_thread_tx,
    };
    let radio = RadioIPC {
        from_radio_tx: fromradio_thread_tx,
        to_radio_rx: toradio_thread_rx
    };
    (iface, radio)
}

pub async fn start_meshtastic_loop(conn: Connection, radio_ipc: RadioIPC) -> Result<()> {
    meshtastic_loop(conn, radio_ipc.from_radio_tx, radio_ipc.to_radio_rx).await
}

///Function intended to be used to send text messages with the TextMessageApp
pub async fn send_text_message(toradio_thread_tx: Sender<IPCMessage>, message:String, packet_destination: Option<PacketDestination>, channel: MeshChannel, want_ack: bool){
    let packet_destination = packet_destination.unwrap_or(PacketDestination::Broadcast);
    let t = get_secs();
    let payload = EncodedMeshPacketData::new(message.as_bytes().to_vec());
    let message = IPCMessage::SendMessage(MessageEnvelope{
        timestamp: t as u32,
        source: None,
        destination: packet_destination,
        channel,
        payload,
        rx_rssi: 0,
        rx_snr: 0.0,
        port_num: PortNum::TextMessageApp,
        sub_port: 0,
        want_ack
    });
    toradio_thread_tx.send(message).await.expect("TODO: panic message");
    ()
}

/// Function intended to be used to send raw binary data messages from the radio.
pub async fn send_raw_message(toradio_thread_tx: Sender<IPCMessage>, message:Vec<u8>,
                              packet_destination: Option<PacketDestination>, channel: MeshChannel,
                              port_num: PortNum, sub_port: u16, want_ack: bool){
    let packet_destination = packet_destination.unwrap_or(PacketDestination::Broadcast);
    let t = get_secs();
    let payload = EncodedMeshPacketData::new(message.to_vec());
    let message = IPCMessage::SendMessage(MessageEnvelope{
        timestamp: t as u32,
        source: None,
        destination: packet_destination,
        channel,
        payload,
        rx_rssi: 0,
        rx_snr: 0.0,
        port_num,
        sub_port,  // Unused, sub-port should be pre-pended to payload vector prior to sending
        want_ack
    });
    toradio_thread_tx.send(message).await.expect("TODO: panic message");
    ()
}


#[cfg(test)]
mod test {
    use crate::mesh_connection::packet_handler::{process_packet, PacketResponse};
    use crate::mesh_connection::util::{get_secs, ComprehensiveNode};
    use crate::mesh_connection::connection::Connection;
    use crate::api::*;

    use meshtastic::utils::stream::available_serial_ports;

    use std::collections::HashMap;

    #[tokio::test]
    async fn test_api() {
        for p in available_serial_ports().unwrap() {
            println!("{}\n", p);
        }
        let (mut interface_ipc, radio_ipc) = create_thread_ipc();

        let conn = Connection::Serial("/dev/tty.usbserial-54760041581".to_string());

        let join_handle = tokio::task::spawn(start_meshtastic_loop(conn, radio_ipc));

        send_text_message(interface_ipc.to_radio_tx.clone(), "string_val: hi".to_string(), None, 0.into(), true).await;
        let mut node_list: HashMap<u32, ComprehensiveNode> = HashMap::new();

        tokio::time::sleep(tokio::time::Duration::from_millis(20000)).await;

        while !interface_ipc.from_radio_rx.is_empty() {
            match interface_ipc.from_radio_rx.try_recv() {
                Ok(msg) => {
                    let update = process_packet(msg, node_list.clone());
                    if update.is_some() {
                        match update.unwrap() {
                            PacketResponse::NodeUpdate(id, cn) => {
                                println!("update for: {}\n", id);
                                node_list.insert(id, *cn);
                            }
                            PacketResponse::UserUpdate(id, user) => {
                                println!("Update for user {}\n", id);
                                if let Some(cn) = node_list.get(&id) {
                                    let mut ncn = cn.clone();
                                    ncn.node_info.user = Some(user);
                                    ncn.last_seen = get_secs();
                                    node_list.insert(id, ncn);
                                } else {
                                    let mut cn = ComprehensiveNode::with_id(id);
                                    cn.node_info.user = Some(user);
                                    cn.last_seen = get_secs();
                                    node_list.insert(id, cn);
                                }
                            }
                            PacketResponse::InboundMessage(msg) => {
                                println!("New message: {} from {}\n", msg.payload, msg.source.unwrap().user.unwrap().short_name);
                                // Put message on queue
                            }
                            PacketResponse::OurAddress(id) => {
                                println!("Our Address: {id}");
                                // Use this to set our address for display
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("{}", e)
                }
            }
        }

        join_handle.abort();
        assert!(true);
    }
}
