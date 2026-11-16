//! TRM Swarm OS — trm:// datagram frame parser.
//! This is NOT QUIC: it defines the 11-byte TRM0 header and a payload, with no handshake, streams,
//! congestion control or TLS. The file name is historical; the protocol is a custom UDP framing.

use std::time::Instant;

pub const TRM_MAGIC_HEADER: u32 = 0x54524D30; // ASCII "TRM0"

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct TRMQuicFrameHeader {
    pub magic: u32,       // 0x54524D30 ("TRM0")
    pub agent_id: u8,     // 0..13
    pub intent_id: u8,    // 0..3
    pub k3_trit: i8,      // +1 (TRUE), 0 (UNKNOWN), -1 (FALSE)
    pub payload_len: u32, // bytes count
}

pub struct TRMQuicPacket {
    pub header: TRMQuicFrameHeader,
    pub payload: Vec<u8>,
}

impl TRMQuicPacket {
    pub fn new(agent_id: u8, intent_id: u8, k3_trit: i8, payload: Vec<u8>) -> Self {
        let header = TRMQuicFrameHeader {
            magic: TRM_MAGIC_HEADER,
            agent_id,
            intent_id,
            k3_trit,
            payload_len: payload.len() as u32,
        };
        TRMQuicPacket { header, payload }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let header_bytes: [u8; 11] = [
            (self.header.magic >> 24) as u8,
            (self.header.magic >> 16) as u8,
            (self.header.magic >> 8) as u8,
            self.header.magic as u8,
            self.header.agent_id,
            self.header.intent_id,
            self.header.k3_trit as u8,
            (self.header.payload_len >> 24) as u8,
            (self.header.payload_len >> 16) as u8,
            (self.header.payload_len >> 8) as u8,
            self.header.payload_len as u8,
        ];
        
        let mut out = Vec::with_capacity(11 + self.payload.len());
        out.extend_from_slice(&header_bytes);
        out.extend_from_slice(&self.payload);
        out
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 11 {
            return Err("Packet too short");
        }
        
        let magic = ((bytes[0] as u32) << 24) | ((bytes[1] as u32) << 16) | ((bytes[2] as u32) << 8) | (bytes[3] as u32);
        if magic != TRM_MAGIC_HEADER {
            return Err("Invalid TRM QUIC magic header");
        }
        
        let agent_id = bytes[4];
        let intent_id = bytes[5];
        let k3_trit = bytes[6] as i8;
        let payload_len = ((bytes[7] as u32) << 24) | ((bytes[8] as u32) << 16) | ((bytes[9] as u32) << 8) | (bytes[10] as u32);
        
        let payload = if bytes.len() >= 11 + payload_len as usize {
            bytes[11..11 + payload_len as usize].to_vec()
        } else {
            bytes[11..].to_vec()
        };
        
        Ok(TRMQuicPacket {
            header: TRMQuicFrameHeader { magic, agent_id, intent_id, k3_trit, payload_len },
            payload,
        })
    }
}

pub struct TRMQuicServer {
    pub port: u16,
    pub active: bool,
}

impl TRMQuicServer {
    pub fn new(port: u16) -> Self {
        TRMQuicServer { port, active: true }
    }

    pub fn handle_binary_frame(&self, frame_bytes: &[u8]) -> (TRMQuicPacket, u64) {
        let start = Instant::now();
        let packet = TRMQuicPacket::parse(frame_bytes).unwrap_or_else(|_| {
            TRMQuicPacket::new(0, 0, 0, b"ERROR_INVALID_FRAME".to_vec())
        });
        let lat_ns = start.elapsed().as_nanos() as u64;
        (packet, lat_ns)
    }
}

#[no_mangle]
pub extern "C" fn rust_trm_quic_parse_frame(
    bytes_ptr: *const u8,
    len: usize,
    out_agent: *mut u8,
    out_intent: *mut u8,
    out_trit: *mut i8,
) -> i32 {
    if bytes_ptr.is_null() || len < 11 || out_agent.is_null() || out_intent.is_null() || out_trit.is_null() {
        return -1;
    }
    
    unsafe {
        let slice = std::slice::from_raw_parts(bytes_ptr, len);
        match TRMQuicPacket::parse(slice) {
            Ok(pkt) => {
                *out_agent = pkt.header.agent_id;
                *out_intent = pkt.header.intent_id;
                *out_trit = pkt.header.k3_trit;
                0
            }
            Err(_) => -2,
        }
    }
}
