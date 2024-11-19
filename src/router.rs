use meshtastic::packet::PacketRouter;
use meshtastic::protobufs::{FromRadio, MeshPacket};
use meshtastic::types::NodeId;

use std::{error::Error, fmt};
use crate::router::DeviceUpdateError::PacketNotSupported;

#[derive(Clone, Debug)]
pub enum DeviceUpdateError {
    PacketNotSupported(String),
    RadioMessageNotSupported(String),
    DecodeFailure(String),
    GeneralFailure(String),
    EventDispatchFailure(String),
    NotificationDispatchFailure(String),
}

impl fmt::Display for DeviceUpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to update device: ")?;

        match *self {
            DeviceUpdateError::DecodeFailure(ref decode_error) => {
                f.write_fmt(format_args!("failed to decode packet:\n{}", decode_error))?;
            }
            DeviceUpdateError::PacketNotSupported(ref packet_type) => {
                f.write_fmt(format_args!(
                    "packet of type \"{}\" not supported",
                    packet_type
                ))?;
            }
            DeviceUpdateError::RadioMessageNotSupported(ref message_type) => {
                f.write_fmt(format_args!(
                    "radio message of type \"{}\" not supported",
                    message_type
                ))?;
            }
            DeviceUpdateError::GeneralFailure(ref failure_reason) => {
                f.write_fmt(format_args!(
                    "General device update failure:\n{}",
                    failure_reason
                ))?;
            }
            DeviceUpdateError::EventDispatchFailure(ref dispatch_error) => {
                f.write_fmt(format_args!(
                    "Failed to dispatch device to client:\n{}",
                    dispatch_error
                ))?;
            }
            DeviceUpdateError::NotificationDispatchFailure(ref notification_error) => {
                f.write_fmt(format_args!(
                    "Failed to send system-level notification:\n{}",
                    notification_error
                ))?;
            }
        }

        Ok(())
    }
}

impl Error for DeviceUpdateError {}

pub struct MeshPacketApi;

impl PacketRouter<(), DeviceUpdateError> for MeshPacketApi{
    fn handle_packet_from_radio(&mut self, packet: FromRadio) -> Result<(), DeviceUpdateError> {
        todo!()
    }

    fn handle_mesh_packet(&mut self, packet: MeshPacket) -> Result<(), DeviceUpdateError> {
        todo!()
    }

    fn source_node_id(&self) -> NodeId {
        todo!()
    }
}
