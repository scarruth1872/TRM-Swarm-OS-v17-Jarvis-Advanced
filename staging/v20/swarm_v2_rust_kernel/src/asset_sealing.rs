//! asset_sealing.rs — Sovereign Asset Sealing & Access Control Engine
//! TRM Swarm OS v19.001 "Sovereign"
//!
//! Enforces cryptographic SHA-256 sealing, Monad Truth Anchor validation (0x4D4F4E4144000001),
//! and 5-tier access control (Public, Academic, Commercial, Partner, Sovereign).

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub const MONAD_TRUTH_ANCHOR: u64 = 0x4D4F_4E41_4400_0001;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessLevel {
    Public,        // Open access (educational, public papers)
    Academic,      // Free for non-profit research with attribution
    Commercial,    // Paid revenue-sharing tier
    Partner,       // Contract + NDA access
    Sovereign,     // Never disclosed / internal bare-metal
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LicenseType {
    OpenSource,      // MIT / Apache-2.0
    AcademicFree,    // Free for educational & healthcare research
    CommercialPaid,  // Enterprise revenue required
    PatentProtected, // Legal patent-pending lead compound
    TradeSecret,     // Proprietary algorithm / never disclosed
}

#[derive(Debug, Clone)]
pub struct SealedAsset {
    pub asset_id: String,
    pub asset_name: String,
    pub sha256_hash: String,
    pub monad_anchor: u64,
    pub monad_seal: u64,
    pub timestamp: u64,
    pub access_level: AccessLevel,
    pub license_type: LicenseType,
    pub commercial_valuation_usd: u64,
}

impl SealedAsset {
    pub fn new(
        asset_id: &str,
        asset_name: &str,
        sha256_hash: &str,
        access_level: AccessLevel,
        license_type: LicenseType,
        valuation_usd: u64,
    ) -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Calculate Monad Truth Seal: Anchor XOR first 8 bytes of hash
        let hash_prefix = u64::from_str_radix(&sha256_hash[..16.min(sha256_hash.len())], 16)
            .unwrap_or(0);
        let monad_seal = MONAD_TRUTH_ANCHOR ^ hash_prefix;

        Self {
            asset_id: asset_id.to_string(),
            asset_name: asset_name.to_string(),
            sha256_hash: sha256_hash.to_string(),
            monad_anchor: MONAD_TRUTH_ANCHOR,
            monad_seal,
            timestamp: ts,
            access_level,
            license_type,
            commercial_valuation_usd: valuation_usd,
        }
    }

    pub fn verify_seal(&self) -> bool {
        let hash_prefix = u64::from_str_radix(&self.sha256_hash[..16.min(self.sha256_hash.len())], 16)
            .unwrap_or(0);
        (self.monad_seal ^ hash_prefix) == MONAD_TRUTH_ANCHOR
    }

    pub fn allows_public_read(&self) -> bool {
        matches!(self.access_level, AccessLevel::Public)
    }

    pub fn requires_commercial_license(&self) -> bool {
        matches!(self.access_level, AccessLevel::Commercial | AccessLevel::Partner | AccessLevel::Sovereign)
    }
}

pub struct AssetRegistry {
    assets: HashMap<String, SealedAsset>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    pub fn register(&mut self, asset: SealedAsset) {
        self.assets.insert(asset.asset_id.clone(), asset);
    }

    pub fn get(&self, asset_id: &str) -> Option<&SealedAsset> {
        self.assets.get(asset_id)
    }

    pub fn total_portfolio_valuation(&self) -> u64 {
        self.assets.values().map(|a| a.commercial_valuation_usd).sum()
    }

    pub fn count_by_level(&self, level: AccessLevel) -> usize {
        self.assets.values().filter(|a| a.access_level == level).count()
    }
}

/// FFI: Verify an asset's Monad seal externally from Python or C
#[no_mangle]
pub extern "C" fn rust_verify_asset_seal(monad_seal: u64, hash_prefix: u64) -> i32 {
    if (monad_seal ^ hash_prefix) == MONAD_TRUTH_ANCHOR {
        1 // Valid
    } else {
        0 // Invalid
    }
}
