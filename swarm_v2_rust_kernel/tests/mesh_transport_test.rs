//! Transport tests against the REAL mesh_discovery + quic_protocol code.
//! These call the platform's own serialize/deserialize, not a reimplementation.

use swarm_rust_kernel::mesh_discovery::{MeshBeaconPacket, MESH_MAGIC, MESH_DISCOVERY_PORT};
use swarm_rust_kernel::quic_protocol::{TRMQuicPacket, TRM_MAGIC_HEADER};
use swarm_rust_kernel::MONAD_TRUTH_ANCHOR;

#[test]
fn beacon_roundtrip_preserves_fields() {
    let b = MeshBeaconPacket::new_local(0x5452_4D00_0000_0001);
    let bytes = b.serialize();
    println!("BEACON_BYTES={}", bytes.iter().map(|x| format!("{:02x}", x)).collect::<String>());
    println!("BEACON_LEN={}", bytes.len());
    let back = MeshBeaconPacket::deserialize(&bytes).expect("deserialize failed");
    assert_eq!(back.magic, MESH_MAGIC);
    assert_eq!(back.node_id, b.node_id);
    assert_eq!(back.truth_anchor, MONAD_TRUTH_ANCHOR);
    println!("MAGIC={:#x}", back.magic);
    println!("TRUTH_ANCHOR={:#x}", back.truth_anchor);
    println!("AGENT_COUNT={}", back.agent_count);
    println!("TRIT_COUNT={}", back.trit_count);
    println!("PORT={}", MESH_DISCOVERY_PORT);
}

#[test]
fn beacon_rejects_bad_magic_and_short_input() {
    assert!(MeshBeaconPacket::deserialize(&[0u8; 35]).is_none(), "short buffer must be rejected");
    let mut b = MeshBeaconPacket::new_local(1).serialize();
    b[0] ^= 0xFF; // corrupt magic
    assert!(MeshBeaconPacket::deserialize(&b).is_none(), "bad magic must be rejected");
}

#[test]
fn quic_frame_roundtrip_all_trits() {
    println!("QUIC_MAGIC={:#x}", TRM_MAGIC_HEADER);
    for trit in [-1i8, 0, 1] {
        let p = TRMQuicPacket::new(14, 186, trit, b"K3_PAYLOAD".to_vec());
        let bytes = p.serialize();
        let back = TRMQuicPacket::parse(&bytes).expect("parse failed");
        assert_eq!(back.header.k3_trit, trit, "trit must survive i8 round-trip");
        assert_eq!(back.payload, b"K3_PAYLOAD");
        println!("QLEN={} TRIT={}", bytes.len(), back.header.k3_trit);
    }
    assert!(TRMQuicPacket::parse(&[0u8; 5]).is_err(), "short frame must error");
}
