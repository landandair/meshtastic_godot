use godot::prelude::*;
use godot::classes::{RefCounted, IRefCounted};

use meshtastic::api::ConnectedStreamApi;
use meshtastic::utils;
use meshtastic::api::StreamApi;
use meshtastic::packet::PacketReceiver;
use meshtastic::types::MeshChannel;
use meshtastic::packet::PacketDestination;

use serialport::SerialPortType;
use tokio::runtime::{Builder, Runtime};

#[derive(GodotClass)]
#[class(base=RefCounted)]
struct MeshtasticNode{
    stream_api: Option<ConnectedStreamApi>,
    listener: Option<PacketReceiver>,
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
            stream_api: None,
            listener: None,
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

    // /// Open a MeshtasticNode port with the specified name and baud rate
    // #[func]
    // fn open(&mut self, name: GString, baud_rate: u32) -> bool {
    //     let stream_api = StreamApi::new();
    //
    //     match utils::stream::build_serial_stream(name.to_string(), None, None, None) {
    //         Ok(serial_stream) => {
    //             let (mut decoded_listener, stream_api) =
    //                 self.runtime.block_on(stream_api.connect(serial_stream));
    //             let config_id = utils::generate_rand_id();
    //             let stream_api = self.runtime.block_on(stream_api.configure(config_id)).unwrap();
    //             self.listener = Some(decoded_listener);
    //             self.stream_api = Some(stream_api);
    //             true
    //         },
    //         Err(e) => {
    //             godot_error!("Failed to open serial port: {}", e);
    //             false
    //         }
    //     }
    // }

    /// Is MeshtasticNodeopen or not
    #[func]
    fn is_open(&self) -> bool {
        match self.stream_api {
            Some(_) => true,
            None => false,
        }
    }

    /// Close the MeshtasticNode port
    #[func]
    fn close(&mut self) {
        self.listener = None;
        self.stream_api = None;
    }

    // /// Write data to the MeshtasticNodeport.
    // let channel = MeshChannel::new(0).unwrap();
    //
    // let _ = self.runtime.block_on(stream_api.send_text(&mut router, "Hello world!".to_string(), PacketDestination::Broadcast, true, channel)).expect("TODO: panic message");
    // /// Return n as bytes written, or -1 when failed
    // #[func]
    // fn write(&mut self, data: PackedByteArray) -> i32 {
    //     if let Some(port) = &mut self.port {
    //         match port.write(&data.to_vec()) {
    //             Ok(n) => n as i32,
    //             Err(e) => {
    //                 godot_error!("Failed to write to MeshtasticNodeport: {}", e);
    //                 -1
    //             }
    //         }
    //     } else {
    //         godot_error!("MeshtasticNodeport not open");
    //         -1
    //     }
    // }
    //
    // /// Write string to the MeshtasticNodeport
    // ///
    // /// Return n as bytes written, or -1 when failed
    // #[func]
    // fn write_str(&mut self, string: GString) -> i32 {
    //     if let Some(port) = &mut self.port {
    //         match port.write(&string.to_string().as_bytes()) {
    //             Ok(n) => n as i32,
    //             Err(e) => {
    //                 godot_error!("Failed to write to MeshtasticNodeport: {}", e);
    //                 -1
    //             }
    //         }
    //     } else {
    //         godot_error!("MeshtasticNodeport not open");
    //         -1
    //     }
    // }
    //
    // /// Read data from the MeshtasticNodeport.
    // #[func]
    // fn read(&mut self) -> PackedByteArray {
    //     if let Some(port) = &mut self.port {
    //         let mut buf = vec![0u8; port.bytes_to_read().unwrap_or(0) as usize];
    //         match port.read(&mut buf) {
    //             Ok(_) => buf.as_slice().into(),
    //             Err(e) => {
    //                 godot_error!("Failed to write to MeshtasticNodeport: {}", e);
    //                 PackedByteArray::new()
    //             }
    //         }
    //     } else {
    //         godot_error!("MeshtasticNodeport not open");
    //         PackedByteArray::new()
    //     }
    // }
    //
    // /// Read exact number of bytes from the MeshtasticNodeport.
    // #[func]
    // fn read_exact(&mut self, size: i32) -> PackedByteArray {
    //     if let Some(port) = &mut self.port {
    //         let mut buf = vec![0u8; size as usize];
    //         match port.read_exact(&mut buf) {
    //             Ok(_) => buf.as_slice().into(),
    //             Err(e) => {
    //                 godot_error!("Failed to read from MeshtasticNodeport: {}", e);
    //                 PackedByteArray::new()
    //             }
    //         }
    //     } else {
    //         godot_error!("MeshtasticNodeport not open");
    //         PackedByteArray::new()
    //     }
    // }
    //
    // /// Read string form the seral port[broken]
    // // #[func]
    // // fn read_str(&mut self, utf8_encoding: bool) -> GString {
    // //     if let Some(port) = &mut self.port {
    // //         let mut buf = vec![0u8; port.bytes_to_read().unwrap() as usize];
    // //         match port.read_exact(&mut buf) {
    // //             Ok(_) => {
    // //                 if utf8_encoding {
    // //                     String::from_utf8_lossy(&buf).into()
    // //                 } else {
    // //                     unsafe {
    // //                         String::from_raw_parts(buf.as_mut_ptr(), buf.len(), buf.len()).into()
    // //                     }
    // //                 }
    // //             }
    // //             Err(e) => {
    // //                 godot_error!("Failed to read from MeshtasticNodeport: {}", e);
    // //                 GString::new()
    // //             }
    // //         }
    // //     } else {
    // //         godot_error!("MeshtasticNodeport not open");
    // //         GString::new()
    // //     }
    // // }
    //
    // /// Gets the number of bytes available to be read from the input buffer.
    // #[func]
    // fn available(&self) -> i32 {
    //     if let Some(port) = &self.port {
    //         match port.bytes_to_read() {
    //             Ok(bytes) => bytes as i32,
    //             Err(e) => {
    //                 godot_error!("Failed to get bytes read: {}", e);
    //                 0
    //             }
    //         }
    //     } else {
    //         godot_error!("MeshtasticNodeport not open");
    //         0
    //     }
    // }
    //
    // #[func]
    // fn remains(&self) -> i32 {
    //     if let Some(port) = &self.port {
    //         match port.bytes_to_write() {
    //             Ok(bytes) => bytes as i32,
    //             Err(e) => {
    //                 godot_error!("Failed to get bytes remains to write: {}", e);
    //                 0
    //             }
    //         }
    //     } else {
    //         godot_error!("MeshtasticNodeport not open");
    //         0
    //     }
    // }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert!(true)
    }
}

