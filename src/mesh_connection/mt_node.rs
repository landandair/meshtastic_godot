use crate::mesh_connection::connection::Connection;
use crate::mesh_connection::ipc::IPCMessage;
use anyhow::{bail, Result};

use meshtastic::packet::PacketRouter;
use meshtastic::protobufs::{FromRadio, MeshPacket};
use meshtastic::types::NodeId;
use meshtastic::{api::StreamApi, utils};

use strum_macros::Display;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Display, Clone, Debug, Error)]
pub enum DeviceUpdateError {
	PacketNotSupported(String),
	RadioMessageNotSupported(String),
	DecodeFailure(String),
	GeneralFailure(String),
	EventDispatchFailure(String),
	NotificationDispatchFailure(String),
}

#[derive(Default)]
struct MyPacketRouter {
	_source_node_id: NodeId,
}

impl MyPacketRouter {
	fn new(node_id: u32) -> Self {
		MyPacketRouter {
			_source_node_id: node_id.into(),
		}
	}
}

impl PacketRouter<(), DeviceUpdateError> for MyPacketRouter {
	fn handle_packet_from_radio(
		&mut self,
		_packet: FromRadio,
	) -> std::result::Result<(), DeviceUpdateError> {
		println!("handle_packet_from_radio called but not sure what to do");
		Ok(())
	}

	fn handle_mesh_packet(
		&mut self,
		_packet: MeshPacket,
	) -> std::result::Result<(), DeviceUpdateError> {
		println!("handle_mesh_packet called but not sure what to do here");
		Ok(())
	}

	fn source_node_id(&self) -> NodeId {
		self._source_node_id
	}
}

pub(crate) async fn meshtastic_loop(
	connection: Connection,
	tx: tokio::sync::mpsc::Sender<IPCMessage>,
	mut rx: tokio::sync::mpsc::Receiver<IPCMessage>,
) -> Result<()> {
	let stream_api = StreamApi::new();
	let mut decoded_listener;
	let connected_stream_api;
	match connection {
		Connection::TCP(ip, port) => {
			let tcp_stream = match utils::stream::build_tcp_stream(format!("{ip}:{port}")).await {
				Ok(sh) => sh,
				Err(e) => {
					bail!(e);
				}
			};
			(decoded_listener, connected_stream_api) = stream_api.connect(tcp_stream).await;
		}
		Connection::Serial(device) => {
			let serial_stream = utils::stream::build_serial_stream(device, None, None, None)
				.expect("Unable to open serial port.");
			(decoded_listener, connected_stream_api) = stream_api.connect(serial_stream).await;
		},
		Connection::BLE(_address) => {
			panic!("BLE is not yet implemented into meshtastic-rust. Make a pr or request change when it is")
		},
		Connection::None => {
			panic!("Neither tcp nor serial selected for connection.");
		}
	}
	let config_id = utils::generate_rand_id();
	let mut _stream_api = connected_stream_api.configure(config_id).await?;
	println!("Connected to meshtastic node!");
	let mut packet_router = MyPacketRouter::new(0);
	loop {
		if let Ok(fr) = decoded_listener.try_recv() {
			if let Err(e) = tx.send(IPCMessage::FromRadio(fr)).await {
				println!("Couldn't send FromRadio packet to mpsc: {e}");
			}
		}
		if let Ok(inbound) = rx.try_recv() {
			match inbound {
				IPCMessage::SendMessage(message) => {
					println!("Sending Message: {}", message.message);
					if let Err(e) = _stream_api
						.send_text(
							&mut packet_router,
							message.message,
							message.destination,
							true,
							message.channel,
						)
						.await
					{
						println!("We tried to send a message but... nope: {e}");
					}
				}
				IPCMessage::ToRadio(tr) => {
					if let Err(e) = _stream_api.send_to_radio_packet(tr.payload_variant).await {
						println!("We tried to send a ToRadio message directly but errored: {e}");
					}
				}
				_ => {
					println!("Unknown ipc message sent into comms thread.");
				}
			}
		}
		tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
	}
}


#[cfg(test)]
mod test {
	use crate::mesh_connection::ipc::IPCMessage;
	use crate::mesh_connection::mt_node::*;
	use crate::mesh_connection::packet_handler::{process_packet, MessageEnvelope, PacketResponse};
	use crate::mesh_connection::util::{get_secs, ComprehensiveNode};

	use tokio::sync::mpsc;
	use tokio::task::JoinHandle;

	use meshtastic::utils::stream::available_serial_ports;
	use meshtastic::packet::PacketDestination;
	use meshtastic::types::MeshChannel;

	use std::collections::HashMap;

	#[tokio::test]
	async fn my_test() {
		for p in available_serial_ports().unwrap(){
			println!("{}\n", p);
		}
		let (mut fromradio_thread_tx, mut fromradio_thread_rx) =
			mpsc::channel::<IPCMessage>(100);
		let (mut toradio_thread_tx, mut toradio_thread_rx) =
			mpsc::channel::<IPCMessage>(100);

		let fromradio_tx = fromradio_thread_tx.clone();
		let conn = Connection::Serial("/dev/tty.usbserial-54760041581".to_string());

		let mut join_handle: JoinHandle<Result<()>> = tokio::task::spawn(async move {
			meshtastic_loop(conn, fromradio_tx, toradio_thread_rx).await
		});
		
		let t = get_secs();
		let message = IPCMessage::SendMessage(MessageEnvelope{
			timestamp: t as u32,
			source: None,
			destination: PacketDestination::Broadcast,
			channel: MeshChannel::new(0).unwrap(),
			message: "Test".to_string(),
			rx_rssi: 0,
			rx_snr: 0.0
		});
		toradio_thread_tx.send(message).await.expect("TODO: panic message");
		let mut node_list:HashMap<u32, ComprehensiveNode> = HashMap::new();

		tokio::time::sleep(tokio::time::Duration::from_millis(20000)).await;

		while !fromradio_thread_rx.is_empty() {
			match fromradio_thread_rx.try_recv() {
				Ok(msg) => {
					let update = process_packet(msg, node_list.clone()).await;
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
								println!("New message: {} from {}\n", msg.message, msg.source.unwrap().user.unwrap().short_name);
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

