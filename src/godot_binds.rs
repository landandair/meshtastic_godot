use godot::prelude::*;
use godot::classes::{RefCounted, IRefCounted};

use crate::mesh_connection::ipc::InterfaceIPC;
use crate::mesh_connection::util::{get_secs, ComprehensiveNode};
use crate::mesh_connection::packet_handler::{process_packet, MessageEnvelope, PacketResponse};
use crate::api::{create_thread_ipc, start_meshtastic_loop};
use crate::mesh_connection::connection::Connection;

use meshtastic::types::MeshChannel;
use meshtastic::packet::PacketDestination;

use std::collections::HashMap;
use serialport::SerialPortType;
use tokio::runtime::{Builder, Runtime};
use tokio::task::JoinHandle;
use anyhow::Result;
use godot::sys::join;


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
        Self { runtime: Builder::new_current_thread()
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

    /// Open a MeshtasticNode serial port with the specified name and baud rate
    #[func]
    fn open_serial_node(&mut self, name: GString) {
        self.connection = Connection::Serial(name.to_string());
        let (iface, radio) = create_thread_ipc();
        self.mpsc_channels = Box::new(Some(iface));
        self.mt_loop_join_handle = Some(start_meshtastic_loop(self.connection.clone(), radio));
    }

    /// Open a MeshtasticNode TCP port
    #[func]
    fn open_tcp_node(&mut self, ip: GString, port: u16) {
        let ip = ip.to_string();
        self.connection = Connection::TCP(ip, port);
        let (iface, radio) = create_thread_ipc();
        self.mpsc_channels = Box::new(Some(iface));
        self.mt_loop_join_handle = Some(start_meshtastic_loop(self.connection.clone(), radio));
    }

    /// Poll the interface and update the status and service mpsc
    #[func]
    fn poll(&mut self) {
        match self.mpsc_channels.as_mut(){
            Some(iface) => {
                while iface.from_radio_rx.is_empty() {
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
                        }
                    }
                }
            },
            None => {
                ()
            }
        }
    }

    #[func]
    fn available_message(&self) -> i16 {
        self.message_queue.len() as i16
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
            let _ = dict.insert("portnum", msg.port_num.as_str_name());
            let _ = dict.insert("portnum", msg.rx_rssi);
            let _ = dict.insert("portnum", msg.rx_snr);
        }
        dict
    }

    /// return open files
    #[func]
    fn is_open(&self) -> bool {
        match &self.mt_loop_join_handle {
            Some(join) => {
                !join.is_finished()
            },
            None => false,
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
                self.connection = Connection::None
            },
            None => (),
        }
    }
}

