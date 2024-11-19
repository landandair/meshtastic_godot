// use crate::ipc::IPCMessage;
// use crate::DEVICE_CONFIG;
// use anyhow::{bail, Result};
use meshtastic::protobufs::{Neighbor, NodeInfo};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[derive(Debug, Clone, Default)]
pub struct ComprehensiveNode {
    pub id: u32,
    pub node_info: NodeInfo,
    pub neighbors: Vec<Neighbor>,
    pub last_seen: u64,
    pub last_snr: f32,
    pub last_rssi: i32,
}


impl ComprehensiveNode {
    pub fn with_id(id: u32) -> Self {
        ComprehensiveNode {
            id,
            ..Default::default()
        }
    }
}

// pub fn get_channel_from_id(id: u32) -> Option<Channel> {
//     match DEVICE_CONFIG.try_read() {
//         Ok(device_config) => {
//             let devcfg = device_config.clone();
//             if let Some(cfg) = devcfg {
//                 let channel = cfg.channels.get(&(id as i32));
//                 return channel.cloned();
//             };
//             None
//         }
//         Err(_e) => {
//             warn!("Couldn't lock config for shared read, so, channel lookup failed.");
//             None
//         }
//     }
// }
//
// pub async fn send_to_radio(ipc: IPCMessage) -> Result<()> {
//     let trm = crate::TO_RADIO_MPSC.write().await.clone().unwrap();
//     if let Err(e) = trm.clone().send(ipc).await {
//         bail!(e);
//     }
//     Ok(())
// }