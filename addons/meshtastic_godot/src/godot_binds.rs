use godot::prelude::*;
use godot::classes::{RefCounted, IRefCounted};

use crate::mesh_connection::ipc::InterfaceIPC;
use crate::mesh_connection::util::{get_secs, ComprehensiveNode};
use crate::mesh_connection::packet_handler::{process_packet, MessageEnvelope, PacketResponse};
use crate::api::{create_thread_ipc, send_raw_message, send_text_message, start_meshtastic_loop};
use crate::mesh_connection::connection::Connection;

use meshtastic::types::{MeshChannel, NodeId};
use meshtastic::packet::PacketDestination;

use std::collections::HashMap;
use serialport::SerialPortType;
use tokio::runtime::{Builder, Runtime};
use tokio::task::JoinHandle;
use anyhow::Result;
use godot::meta::AsArg;
use meshtastic::protobufs::{PortNum, User};

#[derive(GodotClass)]
#[class(base=RefCounted)]
struct MeshtasticNode{
    mt_loop_join_handle: Option<JoinHandle<Result<()>>>,
    mpsc_channels: Box<Option<InterfaceIPC>>,
    connection: Connection,
    node_list: HashMap<u32, ComprehensiveNode>,
    message_queue: Vec<MessageEnvelope>,
    our_node: Option<u32>,
    runtime: Runtime,

    #[base]
    base: Base<RefCounted>,
}


#[godot_api]
impl IRefCounted for MeshtasticNode{
    fn init(base: Base<RefCounted>) -> Self {
        Self { runtime: Builder::new_multi_thread()
            .enable_io() 	// optional, depending on your needs
            .enable_time() 	// optional, depending on your needs
            .build()
            .unwrap(),
            mt_loop_join_handle: None,
            mpsc_channels: Box::new(None),
            connection: Connection::None,
            node_list: HashMap::new(),
            message_queue: Vec::new(),
            our_node: None,
            base }
    }
}

#[godot_api]
impl MeshtasticNode{
    /// Returns an array of all MeshtasticNode ports on system
    ///
    /// It is not guaranteed that these ports exist or are available even if they're
    /// returned by this function.
    #[func]
    fn list_ports() -> Array<Dictionary> {
        if let Ok(infos) = serialport::available_ports() {
            infos
                .into_iter()
                .map(|info| {
                    let mut dict = Dictionary::new();
                    let _ = dict.insert("name", info.port_name);
                    if let SerialPortType::UsbPort(usb) = info.port_type {
                        let _ = dict.insert("type", "usb");
                        let _ = dict.insert("vid", usb.vid);
                        let _ = dict.insert("pid", usb.pid);
                        let _ = dict.insert("sn", usb.serial_number.unwrap_or("".to_string()));
                        let _ = dict.insert("manufacture", usb.manufacturer.unwrap_or("".to_string()));
                        let _ = dict.insert("product", usb.product.unwrap_or("".to_string()));
                    }
                    dict
                })
                .collect()
        } else {
            godot_error!("Failed to list MeshtasticNodeports");
            Array::new()
        }
    }

    fn start_node_loop(&mut self)  {
        if self.is_open() {
            self.close()
        }
        let cloned_connection = self.connection.clone();
        let (iface, radio) = create_thread_ipc();
        self.mpsc_channels = Box::new(Some(iface));
        self.mt_loop_join_handle = Some(self.runtime.spawn( {
            start_meshtastic_loop(cloned_connection, radio)
        }));
    }

    /// Open a MeshtasticNode serial port with the specified name and baud rate
    #[func]
    fn open_serial_node(&mut self, name: GString) {
        self.connection = Connection::Serial(name.to_string());
        self.start_node_loop()
    }

    /// Open a MeshtasticNode TCP port
    #[func]
    fn open_tcp_node(&mut self, ip: GString, port: u16) {
        let ip = ip.to_string();
        self.connection = Connection::TCP(ip, port);
        self.start_node_loop()
    }

    /// Poll the interface and update the status and service mpsc
    #[func]
    fn poll(&mut self) -> bool{
        match self.mpsc_channels.as_mut(){
            Some(iface) => {
                while !iface.from_radio_rx.is_empty() {
                    match iface.from_radio_rx.try_recv() {
                        Ok(msg) => {
                            let update = process_packet(msg, self.node_list.clone());
                            if update.is_some() {
                                match update.unwrap() {
                                    PacketResponse::NodeUpdate(id, cn) => {
                                        println!("update for: {}\n", id);
                                        self.node_list.insert(id, *cn);
                                    }
                                    PacketResponse::UserUpdate(id, user) => {
                                        println!("Update for user {}\n", id);
                                        if let Some(cn) = self.node_list.get(&id) {
                                            let mut ncn = cn.clone();
                                            ncn.node_info.user = Some(user);
                                            ncn.last_seen = get_secs();
                                            self.node_list.insert(id, ncn);
                                        } else {
                                            let mut cn = ComprehensiveNode::with_id(id);
                                            cn.node_info.user = Some(user);
                                            cn.last_seen = get_secs();
                                            self.node_list.insert(id, cn);
                                        }
                                    }
                                    PacketResponse::InboundMessage(msg) => {
                                        // Put message on queue
                                        self.message_queue.push(msg);
                                    }
                                    PacketResponse::OurAddress(id) => {
                                        self.our_node = Some(id);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            godot_error!("{}", e);
                            return false
                        }
                    }
                }
            },
            None => {
                return false
            }
        }
        if !self.is_open(){  // Something went wrong, restart loop
            self.start_node_loop();
            godot_error!("Something went wrong with the meshtastic Connection: Restarting");
            return false
        }
        true
    }

    /// Get first message on queue
    #[func]
    fn get_message(&mut self) -> Dictionary {
        let mut dict = Dictionary::new();
        if !self.message_queue.is_empty(){
            let msg = self.message_queue.remove(0);
            let src = msg.source.unwrap().user.unwrap().id;
            let dest = match msg.destination{
                PacketDestination::Local => {"Local".to_string()}
                PacketDestination::Broadcast => {"broadcast".to_string()}
                PacketDestination::Node(id) => {id.to_string()}
            };

            let _ = dict.insert("source", src);
            let _ = dict.insert("payload", msg.payload.data_vec());
            let _ = dict.insert("channel", msg.channel.channel());
            let _ = dict.insert("destination", dest);
            let _ = dict.insert("port_num", msg.port_num.as_str_name());
            let _ = dict.insert("sub_port", msg.sub_port);
            let _ = dict.insert("rx_rssi", msg.rx_rssi);
            let _ = dict.insert("rx_snr", msg.rx_snr);
            let _ = dict.insert("time", msg.timestamp);
        }
        dict
    }

    /// Gets length of message queue as int
    #[func]
    fn get_available_messages(&mut self) -> i64 {
        self.message_queue.len() as i64
    }

    /// Get length of message queue and return integer length
    #[func]
    fn send_text_message(&mut self, text: GString, channel_num: i64, packet_destination_id: i64, want_ack: bool) {
        let destination:Option<PacketDestination> = match packet_destination_id  {
            0 => {
                None
            },
            _ => {
                Some(PacketDestination::Node(NodeId::new(packet_destination_id as u32)))
            }
        };
        let channel = MeshChannel::new(channel_num as u32).unwrap_or(MeshChannel::new(0).unwrap());

        match self.mpsc_channels.as_ref() {
            Some(interface) => {
                let to_radio = interface.to_radio_tx.clone();
                self.runtime.block_on({
                        send_text_message(to_radio, text.to_string(), destination, channel, want_ack)
                    }
                )
            }
            None => {
                godot_error!("No meshtastic interface connected, message: '{text}' failed to send")
            }
        };
    }

    /// Get length of message queue and return integer length
    #[func]
    fn send_raw_message(&mut self, data: PackedByteArray, channel_num: i64, packet_destination_id: i64, sub_port: i64, want_ack: bool) {
        let destination:Option<PacketDestination> = match packet_destination_id  {
            0 => {
                None
            },
            _ => {
                Some(PacketDestination::Node(NodeId::new(packet_destination_id as u32)))
            }
        };
        let mut message_vec = (sub_port as u16).to_le_bytes().to_vec();
        message_vec.append(&mut data.to_vec());
        let channel = MeshChannel::new(channel_num as u32).unwrap_or(MeshChannel::new(0).unwrap());
        match self.mpsc_channels.as_ref() {
            Some(interface) => {
                self.runtime.block_on({
                    send_raw_message(interface.to_radio_tx.clone(), message_vec, destination, channel, PortNum::PrivateApp, sub_port as u16, want_ack)
                }
                )
            }
            None => {
                godot_error!("No meshtastic interface connected, raw message failed to send")
            }
        };
    }

    /// List all nodes the radio knows about
    #[func]
    fn get_node_list(&mut self) -> Array<GString>{
        let mut node_array:Array<GString> = Array::new();
        let vec = self.node_list.keys().cloned().collect::<Vec<u32>>();
        for key in vec{
            let str = GString::from(format!("!{key:x}"));
            godot_print!("key: {}", str);
            node_array.push(str.into_arg());
        }
        node_array
    }

    /// List all nodes the radio knows about
    #[func]
    fn get_node_info(&mut self, id_hex: GString) -> Dictionary {
        let id_str = id_hex.to_string().replace("!", "");
        let mut dict = Dictionary::new();
        let id:u32 = u32::from_str_radix(&id_str, 16).unwrap();
        match self.node_list.get(&id) {
            None => {godot_error!("No node id: id")}
            Some(node) => {
                let info = node.node_info.clone();
                let user = info.user.unwrap_or(User::default());
                let _ = dict.insert("short_name", user.short_name);
                let _ = dict.insert("long_name", user.long_name);
                let _ = dict.insert("hw_model", user.hw_model);
                let _ = dict.insert("role_int", user.role);
                let _ = dict.insert("last_heard", node.last_seen);
                let _ = dict.insert("device_num", info.num);
                let _ = dict.insert("is_favorite", info.is_favorite);
                let _ = dict.insert("snr", info.snr);
                let _ = dict.insert("hops", info.hops_away);
                let _ = dict.insert("is_mqtt", info.via_mqtt);
                let _ = dict.insert("last_heard", node.last_seen);
            }
        };
        dict
    }

    /// return if meshtastic loop is running(node is connected)
    #[func]
    fn is_open(&self) -> bool {
        match &self.mt_loop_join_handle {
            Some(join) => {
                !join.is_finished()
            },
            None => false
        }
    }

    /// return if meshtastic loop is running(node is connected)
    #[func]
    fn get_connected_id(&self) -> GString {
        match self.our_node {
            Some(our_id) => {
                GString::from(format!("!{our_id:x}"))
            }
            None => {
                GString::new()
            }
        }
    }

    /// Close the MeshtasticNode port
    #[func]
    fn close(&mut self) {
        match &self.mt_loop_join_handle {
            Some(join) => {
                join.abort();
                self.our_node = None;
                self.message_queue = Vec::new();
                self.mpsc_channels = Box::new(None);
            },
            None => (),
        }
    }
}

