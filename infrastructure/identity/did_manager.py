"""
Swarm OS v12 - Decentralized Identity (DID) Manager
Phase 7 component for self-sovereign agent identities.
Simulates W3C DID Core standards and IPFS-based document storage.
"""

import json
import logging
import time
import hashlib
from typing import Dict, List, Optional

logger = logging.getLogger("DIDManager")

class DidDocument:
    def __init__(self, did: str, public_keys: List[str]):
        self.id = did
        self.public_keys = public_keys
        self.authentication = [f"{did}#key-1"]
        self.created = int(time.time())
        self.updated = self.created
        
    def to_dict(self) -> dict:
        return {
            "id": self.id,
            "verificationMethod": [{
                "id": f"{self.id}#key-{i+1}",
                "type": "JsonWebKey2020",
                "controller": self.id,
                "publicKeyJwk": {"kid": key}
            } for i, key in enumerate(self.public_keys)],
            "authentication": self.authentication,
            "created": self.created,
            "updated": self.updated
        }

class DIDManager:
    def __init__(self):
        # Simulate an IPFS node / decentralized storage
        self._ipfs_storage: Dict[str, dict] = {}
        # Simulate an on-chain anchor registry mapping DID -> CID
        self._registry_contract: Dict[str, str] = {}
        logger.info("[DID] Initialized Decentralized Identity Manager.")

    async def create_did(self, agent_id: str, public_keys: List[str]) -> str:
        """
        Create a new DID document for an agent and publish it to simulated IPFS.
        Returns the DID URI.
        """
        did = f"did:swarm:{agent_id}"
        doc = DidDocument(did, public_keys)
        doc_dict = doc.to_dict()
        
        # Simulate IPFS add (CID generation)
        doc_json = json.dumps(doc_dict, sort_keys=True)
        cid = "Qm" + hashlib.sha256(doc_json.encode()).hexdigest()[:44]
        
        # Store in simulated IPFS
        self._ipfs_storage[cid] = doc_dict
        
        # Anchor on-chain
        await self._anchor_did_on_chain(did, cid)
        
        logger.info(f"[DID] Created DID for agent {agent_id}: {did}")
        return did

    async def resolve_did(self, did: str) -> Optional[dict]:
        """
        Resolve a DID to its document by querying the on-chain registry and IPFS.
        """
        cid = await self._get_cid_from_chain(did)
        if not cid:
            logger.warning(f"[DID] Failed to resolve DID {did}: Not found in registry.")
            return None
            
        doc = self._ipfs_storage.get(cid)
        if not doc:
            logger.error(f"[DID] Failed to fetch document for CID {cid} from IPFS.")
            return None
            
        logger.debug(f"[DID] Resolved document for {did}")
        return doc

    async def _anchor_did_on_chain(self, did: str, cid: str):
        # Simulate smart contract interaction
        self._registry_contract[did] = cid
        logger.debug(f"[DID] Anchored {did} to CID {cid} on simulated ledger.")

    async def _get_cid_from_chain(self, did: str) -> Optional[str]:
        return self._registry_contract.get(did)

# Singleton instance
_did_manager = DIDManager()

def get_did_manager() -> DIDManager:
    return _did_manager
