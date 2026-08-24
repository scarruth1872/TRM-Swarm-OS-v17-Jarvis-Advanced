//! TRM Swarm OS v19.001 - Native trm:// QUIC Protocol Transport (no_std port)
//! Zero-RTT multiplexed binary frame codec. Ported from
//! swarm_v2_rust_kernel/src/quic_protocol.rs - Instant -> crate::time::now_ns.

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

pub const TRM_MAGIC_HEADER: u32 = 0x54524D30; // ASCII "TRM0"

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct TRMQuicFrameHeader {
    pub magic: u32,
    pub agent_id: u8,
    pub intent_id: u8,
    pub k3_trit: i8,
    pub payload_len: u32,
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
        let start = crate::time::now_ns();
        let packet = TRMQuicPacket::parse(frame_bytes).unwrap_or_else(|_| {
            TRMQuicPacket::new(0, 0, 0, b"ERROR_INVALID_FRAME".to_vec())
        });
        let lat_ns = crate::time::now_ns().wrapping_sub(start);
        (packet, lat_ns)
    }
}
