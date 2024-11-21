use crate::mesh_connection::ipc::IPCMessage;
use crate::mesh_connection::util::ComprehensiveNode;
use crate::mesh_connection::util;
use meshtastic::packet::PacketDestination;
// use meshtastic::protobufs::config::PayloadVariant;
use meshtastic::protobufs::log_record::Level;
// use meshtastic::protobufs::module_config::PayloadVariant as mpv;
use meshtastic::protobufs::{from_radio, mesh_packet, routing, NeighborInfo, NodeInfo, PortNum, Position, Routing, User};
use meshtastic::types::{MeshChannel, EncodedMeshPacketData};
use meshtastic::Message;
use std::collections::HashMap;

pub(crate) enum PacketResponse {
    NodeUpdate(u32, Box<ComprehensiveNode>),
    UserUpdate(u32, User),
    InboundMessage(MessageEnvelope),
    OurAddress(u32),
}

#[derive(Debug, Clone)]
pub struct MessageEnvelope {
    pub(crate) timestamp: u32,
    pub(crate) source: Option<NodeInfo>,
    pub(crate) destination: PacketDestination,
    pub(crate) channel: MeshChannel,
    pub(crate) payload: EncodedMeshPacketData,
    pub(crate) rx_rssi: i32,
    pub(crate) rx_snr: f32,
    pub(crate) port_num: PortNum,
    pub(crate) want_ack: bool
}

pub fn process_packet(
    packet: IPCMessage,
    node_list: HashMap<u32, ComprehensiveNode>,
) -> Option<PacketResponse> {
    if let IPCMessage::FromRadio(fr) = packet {
        if let Some(some_fr) = fr.payload_variant {
            match some_fr {
                from_radio::PayloadVariant::Packet(pa) => {
                    if let Some(payload) = pa.clone().payload_variant {
                        match payload.clone() {
                            mesh_packet::PayloadVariant::Decoded(de) => {
                                match de.portnum() {
                                    PortNum::PositionApp => {
                                        let data = Position::decode(de.payload.as_slice()).unwrap();
                                        let mut cn = match node_list.contains_key(&pa.from) {
                                            true => node_list.get(&pa.from).unwrap().to_owned(),
                                            false => ComprehensiveNode::with_id(de.source),
                                        };
                                        println!(
                                            "Updating Position for {} ({})",
                                            cn.clone()
                                                .node_info
                                                .user
                                                .unwrap_or_else(User::default)
                                                .id,
                                            pa.from
                                        );
                                        cn.node_info.position = Some(data);
                                        cn.last_seen = util::get_secs();
                                        cn.last_rssi = pa.rx_rssi;
                                        cn.last_snr = pa.rx_snr;
                                        return Some(PacketResponse::NodeUpdate(
                                            cn.node_info.num,
                                            Box::new(cn),
                                        ));
                                    }
                                    PortNum::NeighborinfoApp => {
                                        let data =
                                            NeighborInfo::decode(de.payload.as_slice()).unwrap();
                                        let empty = ComprehensiveNode::with_id(de.source);
                                        for neighbor in data.neighbors.iter() {
                                            let d_cn = node_list
                                                .get(&data.node_id)
                                                .map_or(empty.clone(), |v| v.clone());
                                            let n_cn = node_list
                                                .get(&neighbor.node_id)
                                                .map_or(empty.clone(), |v| v.clone());

                                            let mut hub = "Unknown".to_string();
                                            let mut spoke = "Unknown".to_string();
                                            if let Some(d_user) = d_cn.node_info.user {
                                                hub = d_user.id;
                                            }
                                            if let Some(n_user) = n_cn.node_info.user {
                                                spoke = n_user.id;
                                            }
                                            println!("NeighborInfo: {hub} has neighbor {spoke}");
                                        }
                                        let mut cn = match node_list.get(&data.node_id) {
                                            None => {
                                                ComprehensiveNode::with_id(pa.from)
                                            }
                                            Some(n) => n.clone(),
                                        };
                                        cn.neighbors = data.neighbors;
                                        cn.last_seen = util::get_secs();
                                        cn.last_rssi = pa.rx_rssi;
                                        cn.last_snr = pa.rx_snr;
                                        return Some(PacketResponse::NodeUpdate(
                                            cn.node_info.num,
                                            Box::new(cn),
                                        ));
                                    }
                                    PortNum::NodeinfoApp => {
                                        let data = User::decode(de.payload.as_slice()).unwrap();
                                        println!(
                                            "Received node info update for {} ({})",
                                            data.id, pa.from
                                        );
                                        let nid = u32::from_str_radix(
                                            data.id.clone().trim_start_matches('!'),
                                            16,
                                        )
                                            .unwrap_or(0_u32);
                                        if nid == 0 {
                                            println!("Received a node update but the node string ({}) is not parseable hexadecimal",data.id.clone());
                                            return None;
                                        }

                                        return Some(PacketResponse::UserUpdate(nid, data));
                                    }
                                    PortNum::RoutingApp => {
                                        let data = Routing::decode(de.payload.as_slice()).unwrap();
                                        if let Some(v) = data.variant {
                                            match v {
                                                routing::Variant::RouteRequest(_r) => {
                                                    println!("RouteRequest");
                                                }
                                                routing::Variant::RouteReply(_rr) => {
                                                    println!("RouteReply")
                                                }
                                                routing::Variant::ErrorReason(er) => match er {
                                                    0 => {
                                                        let _from_id = pa.clone().from;
                                                        let _to_id = pa.clone().to;

                                                        println!("Routing Message: Outbound message id {} successfully transmitted" ,de.request_id);
                                                    }
                                                    _ => {
                                                        println!("Routing Error: message trace id {} has errorcode {}", de.request_id, er);
                                                    }
                                                },
                                            }
                                        }
                                    }
                                    PortNum::TracerouteApp => {
                                        println!("Received traceroute");
                                        // let val_resp =
                                        //     RouteDiscovery::decode(de.payload.as_slice());
                                        // if let Ok(route) = val_resp {
                                        //     let from_id = pa.clone().from;
                                        //     let to_id = pa.clone().to;
                                        //     let mut cn = match node_list.get(&from_id) {
                                        //         None => {
                                        //             println!("{:#?}", pa.clone());
                                        //             return None;
                                        //         }
                                        //         Some(n) => n.clone(),
                                        //     };
                                        //     cn.route_list.insert(to_id, route.clone().route);
                                        //     println!(
                                        //         "updating route table to {:#?} for !{:x}->!{:x}",
                                        //         route.route, from_id, to_id
                                        //     );
                                        //     return Some(PacketResponse::NodeUpdate(cn.id, Box::new(cn)));
                                        // }
                                        return None
                                    }
                                    other_port => {
                                        let source_ni = match node_list.get(&pa.from) {
                                            Some(s) => s.clone().node_info,
                                            None => {
                                                println!(
                                                    "Could not find node info for id {}",
                                                    pa.from
                                                );
                                                return None;
                                            }
                                        };
                                        let _dest_ni =
                                            node_list.get(&pa.to).map(|s| s.clone().node_info);
                                        let destinated: PacketDestination = match pa.to {
                                            0 => PacketDestination::Local,
                                            u32::MAX => PacketDestination::Broadcast,
                                            s => PacketDestination::Node(s.into()),
                                        };

                                        return Some(PacketResponse::InboundMessage(
                                            MessageEnvelope {
                                                timestamp: pa.rx_time,
                                                source: Some(source_ni),
                                                destination: destinated,
                                                channel: MeshChannel::from(pa.channel),
                                                payload: EncodedMeshPacketData::new(de.payload),
                                                rx_rssi: pa.rx_rssi,
                                                rx_snr: pa.rx_snr,
                                                port_num: other_port,
                                                want_ack: pa.want_ack
                                            },
                                        ));
                                    } // PortNum::AdminApp => {}
                                    // PortNum::WaypointApp => {}

                                    // PortNum::PaxcounterApp => {}
                                    // PortNum::StoreForwardApp => {}
                                    // PortNum::RangeTestApp => {}
                                }
                            }
                            mesh_packet::PayloadVariant::Encrypted(_) => {
                                println!("Received an encrypted packet.");
                                return None;
                            }
                        }
                    }
                    return None;
                }
                from_radio::PayloadVariant::MyInfo(mi) => {
                    println!("My node number is {:#?}", mi.my_node_num);
                    return Some(PacketResponse::OurAddress(mi.my_node_num));
                }
                from_radio::PayloadVariant::NodeInfo(ni) => {
                    println!(
                        "Updating NodeInfo for {} ({})",
                        ni.clone().user.unwrap_or_else(User::default).id,
                        ni.num
                    );
                    let mut cn = ComprehensiveNode::with_id(ni.num);
                    cn.node_info = ni.clone();
                    cn.last_seen = util::get_secs();
                    cn.last_rssi = 0;
                    cn.last_snr = ni.snr;

                    return Some(PacketResponse::NodeUpdate(ni.num, Box::new(cn)));
                }
                from_radio::PayloadVariant::Config(_cfg) => {
                    println!("Receiving DeviceConfig from device. UNUSED");
                    // match cfg.payload_variant {
                    //     None => {}
                    //     Some(s) => {
                    //         let mut f = DEVICE_CONFIG.write().await;
                    //         if f.is_none() {
                    //             *f = Some(DeviceConfiguration::default());
                    //         }
                    //         let mut devcfg = f.clone().unwrap();
                    //         match s {
                    //             PayloadVariant::Device(d) => devcfg.device = d,
                    //             PayloadVariant::Position(p) => devcfg.position = p,
                    //             PayloadVariant::Power(p) => devcfg.power = p,
                    //             PayloadVariant::Network(n) => devcfg.network = n,
                    //             PayloadVariant::Display(d) => devcfg.display = d,
                    //             PayloadVariant::Lora(l) => devcfg.lora = l,
                    //             PayloadVariant::Bluetooth(b) => devcfg.bluetooth = b,
                    //         }
                    //         devcfg.last_update = get_secs();
                    //         *f = Some(devcfg);
                    //     }
                    //}
                    return None
                }
                from_radio::PayloadVariant::LogRecord(v) => {
                    match v.level() {
                        Level::Unset => {
                            println!("Log Message: {}", v.message)
                        }
                        Level::Critical => {
                            println!("Log Message: {}", v.message)
                        }
                        Level::Error => {
                            println!("Log Message: {}", v.message)
                        }
                        Level::Warning => {
                            println!("Log Message: {}", v.message)
                        }
                        Level::Info => {
                            println!("Log Message: {}", v.message)
                        }
                        Level::Debug => {
                            println!("Log Message: {}", v.message)
                        }
                        Level::Trace => {
                            println!("Log Message: {}", v.message)
                        }
                    }
                    return None;
                }
                //from_radio::PayloadVariant::ConfigCompleteId(_) => {}
                from_radio::PayloadVariant::Rebooted(v) => {
                    if v {
                        println!("Device has reported a reboot");
                    }
                    return None;
                }
                from_radio::PayloadVariant::ModuleConfig(_module_obj) => {
                    println!("Receiving ModulesConfig from device. UNUSED");
                    // if let Some(module) = module_obj.payload_variant {
                    //     let mut f = DEVICE_CONFIG.write().await;
                    //     if f.is_none() {
                    //         *f = Some(DeviceConfiguration::default());
                    //     }
                    //     let mut devcfg = f.clone().unwrap();
                    //
                    //     match module {
                    //         mpv::Mqtt(o) => devcfg.mqtt = o,
                    //         mpv::Serial(o) => devcfg.serial = o,
                    //         mpv::ExternalNotification(o) => devcfg.external_notification = o,
                    //         mpv::StoreForward(o) => devcfg.store_forward = o,
                    //         mpv::RangeTest(o) => devcfg.range_test = o,
                    //         mpv::Telemetry(o) => devcfg.telemetry = o,
                    //         mpv::CannedMessage(o) => devcfg.canned_message = o,
                    //         mpv::Audio(o) => devcfg.audio = o,
                    //         mpv::RemoteHardware(o) => devcfg.remote_hardware = o,
                    //         mpv::NeighborInfo(o) => devcfg.neighbor_info = o,
                    //         mpv::AmbientLighting(o) => devcfg.ambient_lighting = o,
                    //         mpv::DetectionSensor(o) => devcfg.detection_sensor = o,
                    //         mpv::Paxcounter(o) => devcfg.paxcounter = o,
                    //     }
                    //     devcfg.last_update = get_secs();
                    //     *f = Some(devcfg);
                    // }
                    return None
                }
                from_radio::PayloadVariant::ConfigCompleteId(u) => {
                    println!(
                        "We've received all config from the device! (Checksum {})",
                        u
                    );
                }
                from_radio::PayloadVariant::Channel(_c) => {
                    //let mut channelpacket = c.clone();
                    println!("Unimplemented channelPacket")
                }
                from_radio::PayloadVariant::QueueStatus(v) => {
                    println!(
                        "QueueStatus: res {}/free {}/maxlen {}/mesh_packet_id {}",
                        v.res, v.free, v.maxlen, v.mesh_packet_id
                    );
                    return None;
                }
                from_radio::PayloadVariant::XmodemPacket(v) => {
                    println!("{:#?}", v);
                    return None;
                }
                from_radio::PayloadVariant::Metadata(v) => {
                    println!("Device firmware version: {}", v.firmware_version);
                    return None;
                }
                from_radio::PayloadVariant::MqttClientProxyMessage(v) => {
                    println!("{:#?}", v);
                    return None;
                }
            }
        }
        return None;
    };
    None
}