"""
Memory-Mapped Inter-Process Communication (IPC) Bridge
Phase 3: High-Speed Ternary State Synchronization
Provides sub-millisecond shared memory passage of swarm node states.
"""

import mmap
import os
import json
import logging
import asyncio
import concurrent.futures

logger = logging.getLogger("MemoryMappedIPC")

class MemoryMappedIPC:
    def __init__(self, filename=".trm_shared_memory", size=65536):
        self.filename = filename
        self.size = size
        self.mmap_file = None
        self.fd = None
        self._executor = concurrent.futures.ThreadPoolExecutor(max_workers=2, thread_name_prefix="trm_ipc_pool")
        
        # Ensure the backing file exists and matches the required size
        if not os.path.exists(self.filename) or os.path.getsize(self.filename) < self.size:
            try:
                with open(self.filename, "wb") as f:
                    f.write(b"\x00" * self.size)
            except IOError as e:
                logger.error(f"Failed to create/expand IPC backing file: {e}")
                return
                
        try:
            # Open the file for reading and writing (with O_BINARY on Windows)
            flags = os.O_RDWR
            if hasattr(os, "O_BINARY"):
                flags |= os.O_BINARY
                
            self.fd = os.open(self.filename, flags)
            self.mmap_file = mmap.mmap(self.fd, self.size, access=mmap.ACCESS_WRITE)
            logger.info(f"Memory-Mapped IPC active on file: {self.filename} ({self.size} bytes)")
        except Exception as e:
            logger.error(f"Failed to map shared memory backing file: {e}")
            
    def write_state(self, state_dict: dict) -> bool:
        """Serializes and writes the state dictionary to shared memory."""
        if not self.mmap_file:
            return False
        try:
            self.mmap_file.seek(0)
            data = json.dumps(state_dict).encode("utf-8")
            if len(data) > self.size - 4:
                logger.error(f"State data size ({len(data)} bytes) exceeds IPC size limits.")
                return False
            # Write 4-byte big-endian length prefix
            self.mmap_file.write(len(data).to_bytes(4, byteorder="big"))
            self.mmap_file.write(data)
            # Clear remaining bytes to avoid stale read pollution
            remaining = self.size - 4 - len(data)
            self.mmap_file.write(b"\x00" * remaining)
            self.mmap_file.flush()
            return True
        except Exception as e:
            logger.error(f"IPC memory write failure: {e}")
            return False
            
    def read_state(self) -> dict:
        """Reads and deserializes the state dictionary from shared memory."""
        if not self.mmap_file:
            return {}
        try:
            self.mmap_file.seek(0)
            length_bytes = self.mmap_file.read(4)
            length = int.from_bytes(length_bytes, byteorder="big")
            if length <= 0 or length > self.size - 4:
                return {}
            data_bytes = self.mmap_file.read(length)
            return json.loads(data_bytes.decode("utf-8"))
        except Exception as e:
            logger.debug(f"IPC memory read failure (expected during active writes): {e}")
            return {}
            
    def close(self):
        """Releases shared memory locks and file handles."""
        try:
            self._executor.shutdown(wait=False)
        except Exception:
            pass
        if self.mmap_file:
            try:
                self.mmap_file.close()
            except Exception:
                pass
        if self.fd:
            try:
                os.close(self.fd)
            except Exception:
                pass

    async def write_state_async(self, state_dict: dict) -> bool:
        """Asynchronously writes the state dictionary using a thread pool."""
        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(self._executor, self.write_state, state_dict)

    async def read_state_async(self) -> dict:
        """Asynchronously reads the state dictionary using a thread pool."""
        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(self._executor, self.read_state)
