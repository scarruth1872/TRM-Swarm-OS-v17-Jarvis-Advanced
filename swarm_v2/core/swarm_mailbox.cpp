/**
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
