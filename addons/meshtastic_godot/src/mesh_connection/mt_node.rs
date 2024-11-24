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
				//panic!("ahh")
			}
		}
		if let Ok(inbound) = rx.try_recv() {
			match inbound {
				IPCMessage::SendMessage(message) => {
					println!("Sending Message: {}", message.payload);
					if let Err(e) = _stream_api
						.send_mesh_packet(
							&mut packet_router,
							message.payload,
							message.port_num,
							message.destination,
							message.channel,
							message.want_ack,
							false,
							false,
							None,
							None
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
