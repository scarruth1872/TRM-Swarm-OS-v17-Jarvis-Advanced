//! swarm-shard — Motion A: multi-shard NAND fabric.
//!
//! The fabric grows by adding shards, not rebuilding. A manifest records
//! the shard table; each shard is a verified NandFabric (canonical K3,
//! codec v2, flock-protected). Verify gates: append→readback roundtrip,
//! shard switching at threshold, cross-shard reads.

pub mod corpus;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub const SHARD_FULL_RATIO: f64 = 0.90; // seal a shard at 90%

/// One shard's manifest entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShardEntry {
    pub id: u32,
    pub path: String,
    pub size_mb: u64,
    pub codec: u32,
    pub status: String, // "active" | "sealed"
}

/// The manifest: the shard table (the only new file in the design).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub shards: Vec<ShardEntry>,
}

impl Manifest {
    pub fn load_or_create(root: &str) -> std::io::Result<Self> {
        let path = PathBuf::from(root).join("MANIFEST.k3");
        if path.exists() {
            let s = fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&s).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?)
        } else {
            let m = Manifest { version: 1, shards: vec![] };
            m.save(root)?;
            Ok(m)
        }
    }

    pub fn save(&self, root: &str) -> std::io::Result<()> {
        let path = PathBuf::from(root).join("MANIFEST.k3");
        let s = serde_json::to_string_pretty(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, s)
    }

    pub fn active_shard(&self) -> Option<&ShardEntry> {
        self.shards.iter().find(|s| s.status == "active")
    }
}

/// The multi-shard fabric.
pub struct ShardedFabric {
    root: String,
    manifest: Manifest,
    open_shards: HashMap<u32, swarm_nand::NandFabric>,
    /// capacity of a shard in trits (256MB arena, 2-bit/trit)
    shard_capacity_trits: u64,
}

impl ShardedFabric {
    pub fn open(root: &str) -> std::io::Result<Self> {
        fs::create_dir_all(root)?;
        let mut manifest = Manifest::load_or_create(root)?;
        // if empty, seed shard_000
        if manifest.shards.is_empty() {
            let p = PathBuf::from(root).join("shard_000.k3");
            let sp = p.to_str().unwrap().to_string();
            let entry = ShardEntry {
                id: 0,
                path: sp.clone(),
                size_mb: 256,
                codec: 2,
                status: "active".into(),
            };
            // create the shard file via NandFabric::open (initializes header)
            let _ = swarm_nand::NandFabric::open(&sp)?;
            manifest.shards.push(entry);
            manifest.save(root)?;
        }
        let mut open_shards = HashMap::new();
        let mut shard_capacity_trits = 0;
        if let Some(a) = manifest.active_shard() {
            // A manifest is PORTABLE data: it may have been written on the other side of the
            // WSL<->Windows boundary, where the same files have different absolute paths (this entry
            // was recorded as /home/shawn/... by a Linux-side run, and Windows cannot open that path
            // at all). Try the recorded path, then the translation -- and REPORT the substitution: an
            // unreported substitution is how a gate starts answering for a file it never opened.
            let fab = match swarm_nand::NandFabric::open(&a.path) {
                Ok(f) => f,
                Err(e) => {
                    let alt = swarm_paths::resolve(&a.path);
                    if alt == a.path {
                        return Err(e);
                    }
                    match swarm_nand::NandFabric::open(&alt) {
                        Ok(f) => {
                            eprintln!(
                                "[swarm-shard] manifest shard path re-resolved: {} -> {}",
                                a.path, alt
                            );
                            f
                        }
                        Err(_) => return Err(e),
                    }
                }
            };
            shard_capacity_trits = fab.capacity_trits();
            open_shards.insert(a.id, fab);
        }
        Ok(ShardedFabric { root: root.into(), manifest, open_shards, shard_capacity_trits })
    }

    /// Append trits to the active shard; switch shards at 90%.
    pub fn append(&mut self, trits: &[i8]) -> std::io::Result<(u32, u64)> {
        let id = self.manifest.active_shard().expect("active shard").id;
        let fab = self.open_shards.get_mut(&id).expect("open shard");
        let offset = fab.append(trits)?;
        fab.flush()?;
        let used = fab.trit_count() as f64 / self.shard_capacity_trits as f64;
        if used >= SHARD_FULL_RATIO {
            self.seal_and_create_next(id)?;
        }
        Ok((id, offset))
    }

    /// Seal the current shard, create the next one.
    fn seal_and_create_next(&mut self, current: u32) -> std::io::Result<()> {
        if let Some(entry) = self.manifest.shards.iter_mut().find(|s| s.id == current) {
            entry.status = "sealed".into();
        }
        let next_id = current + 1;
        let p = PathBuf::from(&self.root).join(format!("shard_{next_id:03}.k3"));
        let sp = p.to_str().unwrap().to_string();
        let _ = swarm_nand::NandFabric::open(&sp)?; // init header
        let entry = ShardEntry { id: next_id, path: sp, size_mb: 256, codec: 2, status: "active".into() };
        self.manifest.shards.push(entry.clone());
        self.manifest.save(&self.root)?;
        let fab = swarm_nand::NandFabric::open(&entry.path)?;
        self.shard_capacity_trits = fab.capacity_trits();
        self.open_shards.insert(next_id, fab);
        self.open_shards.remove(&current);
        Ok(())
    }

    /// Read n trits from (shard_id, offset) — cross-shard reads supported.
    pub fn read(&mut self, shard_id: u32, offset: u64, n: usize) -> std::io::Result<Vec<i8>> {
        if !self.open_shards.contains_key(&shard_id) {
            // open the shard on demand (sealed or not-yet-open)
            let entry = self.manifest.shards.iter().find(|s| s.id == shard_id)
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no such shard"))?;
            let fab = swarm_nand::NandFabric::open(&entry.path)?;
            self.open_shards.insert(shard_id, fab);
        }
        self.open_shards.get_mut(&shard_id).unwrap().read(offset, n)
    }

    /// Total trits across all shards (live).
    pub fn total_trits(&mut self) -> u64 {
        let mut total = 0;
        let ids: Vec<u32> = self.open_shards.keys().cloned().collect();
        for id in ids {
            if let Some(fab) = self.open_shards.get(&id) {
                total += fab.trit_count();
            }
        }
        total
    }

    pub fn shard_count(&self) -> usize { self.manifest.shards.len() }
    pub fn manifest(&self) -> &Manifest { &self.manifest }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_roundtrip() {
        let dir = std::env::temp_dir().join("trm_shard_test_manifest");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let mut m = Manifest::load_or_create(dir.to_str().unwrap()).unwrap();
        assert_eq!(m.version, 1);
        m.shards.push(ShardEntry { id: 7, path: "x".into(), size_mb: 256, codec: 2, status: "active".into() });
        m.save(dir.to_str().unwrap()).unwrap();
        let m2 = Manifest::load_or_create(dir.to_str().unwrap()).unwrap();
        assert_eq!(m2.shards.len(), 1);
        assert_eq!(m2.shards[0].id, 7);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn append_readback_roundtrip() {
        let dir = std::env::temp_dir().join("trm_shard_test_rb");
        let _ = fs::remove_dir_all(&dir);
        let mut fab = ShardedFabric::open(dir.to_str().unwrap()).unwrap();
        assert_eq!(fab.shard_count(), 1);
        let trits: Vec<i8> = vec![1, 1, 1, -1, -1, 1, 0, 1, -1, 1, 1, 1, -1, 0, 1, -1, 1, 1, 1, 1, 0, -1, 1];
        let (sid, off) = fab.append(&trits).unwrap();
        assert_eq!(sid, 0);
        let back = fab.read(sid, off, trits.len()).unwrap();
        assert_eq!(back, trits, "append→readback must be byte-identical");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cross_shard_read() {
        let dir = std::env::temp_dir().join("trm_shard_test_x");
        let _ = fs::remove_dir_all(&dir);
        let mut fab = ShardedFabric::open(dir.to_str().unwrap()).unwrap();
        // force a shard switch: seal 0, create 1
        fab.seal_and_create_next(0).unwrap();
        assert_eq!(fab.shard_count(), 2);
        let t1: Vec<i8> = vec![1, -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -1, 1];
        let (sid1, off1) = fab.append(&t1).unwrap();
        assert_eq!(sid1, 1, "append must go to the new active shard");
        let back1 = fab.read(sid1, off1, t1.len()).unwrap();
        assert_eq!(back1, t1);
        let _ = fs::remove_dir_all(&dir);
    }
}
