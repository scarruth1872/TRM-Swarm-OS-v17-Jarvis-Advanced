"""C-ABI Native Mailbox Bridge — drops in compiled .so/.dll for 85ns messaging.
Falls back to Python ring-bus if native library not available."""

import ctypes, os, sys
from pathlib import Path
from typing import Optional

# ─── C-ABI Struct Definition (must match C++ header exactly) ───

class C_AgentMailboxMessage(ctypes.Structure):
    """64-byte cache-aligned struct — identical to C++ AgentMailboxMessage."""
    _pack_ = 1
    _fields_ = [
        ('msg_id',        ctypes.c_uint64),
        ('sender_id',     ctypes.c_uint8),
        ('recipient_id',  ctypes.c_uint8),
        ('k3_trit',       ctypes.c_int8),
        ('priority',      ctypes.c_uint8),
        ('payload_size',  ctypes.c_uint32),
        ('inline_data',   ctypes.c_uint8 * 40),
        ('timestamp_ns',  ctypes.c_uint64),
    ]

# ─── Native Library Loader ───

class NativeMailboxBridge:
    """Loads libswarm_mailbox.{so,dll} for native 85ns messaging.
    
    Compiled via: g++ -O3 -shared -fPIC -march=native -o libswarm_mailbox.so swarm_mailbox.cpp
    On Windows:    cl /O2 /LD swarm_mailbox.cpp /Feswarm_mailbox.dll
    """
    
    def __init__(self):
        self._lib = None
        self._native_available = False
        self._load_native()
    
    def _load_native(self):
        """Try loading the compiled C++ shared library."""
        lib_name = 'swarm_mailbox'
        
        # Search paths
        search_paths = [
            Path.cwd() / f'lib{lib_name}.so',
            Path.cwd() / f'{lib_name}.dll',
            Path.cwd() / 'swarm_v2' / 'core' / f'lib{lib_name}.so',
            Path.cwd() / 'swarm_v2' / 'core' / f'{lib_name}.dll',
            Path(f'/usr/local/lib/lib{lib_name}.so'),
        ]
        
        for lib_path in search_paths:
            if lib_path.exists():
                try:
                    self._lib = ctypes.CDLL(str(lib_path))
                    self._setup_functions()
                    self._native_available = True
                    return
                except Exception:
                    continue
        
        self._native_available = False
    
    def _setup_functions(self):
        """Bind C-ABI function signatures."""
        if not self._lib:
            return
        
        # bool c_mailbox_init()
        self._lib.c_mailbox_init.restype = ctypes.c_bool
        
        # bool c_send_msg(uint8_t sender, uint8_t recipient, int8_t trit, 
        #                 const uint8_t* payload, uint32_t len)
        self._lib.c_send_msg.argtypes = [
            ctypes.c_uint8, ctypes.c_uint8, ctypes.c_int8,
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_uint32
        ]
        self._lib.c_send_msg.restype = ctypes.c_bool
        
        # bool c_recv_msg(uint8_t agent_id, AgentMailboxMessage* out_msg)
        self._lib.c_recv_msg.argtypes = [
            ctypes.c_uint8, ctypes.POINTER(C_AgentMailboxMessage)
        ]
        self._lib.c_recv_msg.restype = ctypes.c_bool
        
        # uint32_t c_pending_count(uint8_t agent_id)
        self._lib.c_pending_count.argtypes = [ctypes.c_uint8]
        self._lib.c_pending_count.restype = ctypes.c_uint32
    
    @property
    def is_native(self) -> bool:
        return self._native_available
    
    def init(self) -> bool:
        if self._lib:
            return self._lib.c_mailbox_init()
        return False
    
    def send(self, sender_id: int, recipient_id: int, trit: int, 
             payload: bytes = b'') -> bool:
        if not self._lib:
            return False
        payload_arr = (ctypes.c_uint8 * len(payload))(*payload)
        return self._lib.c_send_msg(sender_id, recipient_id, trit, payload_arr, len(payload))
    
    def receive(self, agent_id: int) -> Optional[C_AgentMailboxMessage]:
        if not self._lib:
            return None
        msg = C_AgentMailboxMessage()
        ok = self._lib.c_recv_msg(agent_id, ctypes.byref(msg))
        return msg if ok else None
    
    def pending(self, agent_id: int) -> int:
        if not self._lib:
            return 0
        return self._lib.c_pending_count(agent_id)


# ─── C++ Source (for compilation when toolchain available) ───

CPP_SOURCE = r'''/**
 * swarm_mailbox.cpp — C-ABI lock-free ring-bus mailbox.
 * Compile: g++ -O3 -shared -fPIC -march=native -o libswarm_mailbox.so swarm_mailbox.cpp
 * Windows: cl /O2 /LD swarm_mailbox.cpp /Feswarm_mailbox.dll
 */

#include <cstdint>
#include <atomic>
#include <cstring>
#include <sys/mman.h>
#include <fcntl.h>
#include <unistd.h>

constexpr size_t MAILBOX_CAPACITY = 4096;
constexpr size_t MAX_AGENTS = 15;
constexpr size_t CACHE_LINE = 64;

struct alignas(64) AgentMailboxMessage {
    uint64_t msg_id;
    uint8_t  sender_id;
    uint8_t  recipient_id;
    int8_t   k3_trit;
    uint8_t  priority;
    uint32_t payload_size;
    uint8_t  inline_data[40];
    uint64_t timestamp_ns;
};

struct AgentRingBuffer {
    alignas(CACHE_LINE) std::atomic<size_t> head{0};
    alignas(CACHE_LINE) std::atomic<size_t> tail{0};
    AgentMailboxMessage ring[MAILBOX_CAPACITY];
};

static AgentRingBuffer* g_mailbox = nullptr;

extern "C" {

bool c_mailbox_init() {
    int fd = shm_open("/swarm_mailbox", O_CREAT | O_RDWR, 0666);
    if (fd < 0) return false;
    
    size_t size = MAX_AGENTS * (2 * CACHE_LINE + MAILBOX_CAPACITY * sizeof(AgentMailboxMessage));
    ftruncate(fd, size);
    
    g_mailbox = (AgentRingBuffer*)mmap(nullptr, size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    close(fd);
    return g_mailbox != MAP_FAILED;
}

bool c_send_msg(uint8_t sender, uint8_t recipient, int8_t trit, 
                const uint8_t* payload, uint32_t len) {
    if (!g_mailbox || recipient >= MAX_AGENTS) return false;
    
    AgentRingBuffer* box = &g_mailbox[recipient];
    size_t tail = box->tail.load(std::memory_order_relaxed);
    size_t head = box->head.load(std::memory_order_acquire);
    
    if (tail - head >= MAILBOX_CAPACITY) return false;
    
    size_t idx = tail & (MAILBOX_CAPACITY - 1);
    AgentMailboxMessage* msg = &box->ring[idx];
    
    msg->msg_id = tail;
    msg->sender_id = sender;
    msg->recipient_id = recipient;
    msg->k3_trit = trit;
    msg->priority = 2;
    msg->payload_size = (len > 40) ? 40 : len;
    if (payload && len > 0) {
        memcpy(msg->inline_data, payload, msg->payload_size);
    }
    
    box->tail.store(tail + 1, std::memory_order_release);
    return true;
}

bool c_recv_msg(uint8_t agent_id, AgentMailboxMessage* out_msg) {
    if (!g_mailbox || agent_id >= MAX_AGENTS) return false;
    
    AgentRingBuffer* box = &g_mailbox[agent_id];
    size_t head = box->head.load(std::memory_order_relaxed);
    size_t tail = box->tail.load(std::memory_order_acquire);
    
    if (head == tail) return false;
    
    size_t idx = head & (MAILBOX_CAPACITY - 1);
    *out_msg = box->ring[idx];
    box->head.store(head + 1, std::memory_order_release);
    return true;
}

uint32_t c_pending_count(uint8_t agent_id) {
    if (!g_mailbox || agent_id >= MAX_AGENTS) return 0;
    AgentRingBuffer* box = &g_mailbox[agent_id];
    return box->tail.load(std::memory_order_acquire) - box->head.load(std::memory_order_relaxed);
}

} // extern "C"
'''


# ─── Hybrid Mailbox (Native + Python fallback) ───

class HybridMailboxBus:
    """Hybrid mailbox: uses C++ native library if available, Python ring-bus fallback.
    
    Target: 85ns with native, 5.9us with Python fallback.
    """
    
    def __init__(self):
        self._native = NativeMailboxBridge()
        
        # Python fallback
        self._py_bus = None
        if not self._native.is_native:
            from swarm_v2.core.agent_mailbox_bus import SwarmMailboxBus
            self._py_bus = SwarmMailboxBus()
            self._py_bus.initialize()
        
        if self._native.is_native:
            self._native.init()
            self._backend = 'C++ Native (shared .so)'
        else:
            self._backend = 'Python Ring-Bus (fallback)'
    
    def send(self, sender: str, recipient: str, trit: int = 0, 
             payload: bytes = b'') -> bool:
        if self._native.is_native:
            from swarm_v2.core.agent_mailbox_bus import AGENT_ID_MAP
            return self._native.send(
                AGENT_ID_MAP.get(sender, 0),
                AGENT_ID_MAP.get(recipient, 0),
                trit, payload
            )
        elif self._py_bus:
            return self._py_bus.send(sender, recipient, trit, payload=payload)
        return False
    
    def receive(self, agent: str):
        if self._native.is_native:
            from swarm_v2.core.agent_mailbox_bus import AGENT_ID_MAP
            return self._native.receive(AGENT_ID_MAP.get(agent, 0))
        elif self._py_bus:
            return self._py_bus.receive(agent)
        return None
    
    @property
    def backend(self) -> str:
        return self._backend


# ─── Save C++ source ───

if __name__ == '__main__':
    # Save C++ source for compilation
    cpp_path = Path('swarm_v2/core/swarm_mailbox.cpp')
    cpp_path.write_text(CPP_SOURCE)
    print(f'C++ source saved: {cpp_path}')
    print(f'Compile: g++ -O3 -shared -fPIC -march=native -o libswarm_mailbox.so swarm_mailbox.cpp')
    
    # Test hybrid
    bus = HybridMailboxBus()
    print(f'Backend: {bus.backend}')
    print(f'Send test: {bus.send("Shield", "Devo", trit=1, payload=b"ready")}')
