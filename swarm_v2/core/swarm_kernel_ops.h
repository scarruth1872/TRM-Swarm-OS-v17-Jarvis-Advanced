#ifndef SWARM_KERNEL_OPS_H
#define SWARM_KERNEL_OPS_H

#include <cstdint>
#include <cstdbool>

#ifdef __cplusplus
extern "C" {
#endif

// ─── MAMBA3STEP: Cache-line aligned persistent state ───
constexpr size_t MAMBA_STATE_DIM = 32;
constexpr size_t MAX_MAMBA_SLOTS = 16;  // 15 agents + 1 reserve

struct alignas(64) Mamba3State {
    float h[MAMBA_STATE_DIM];     // Continuous hidden state vector (128B)
    float A[MAMBA_STATE_DIM];     // Diagonal transition matrix (128B)
    float B[MAMBA_STATE_DIM];     // Input projection (128B)
    float C[MAMBA_STATE_DIM];     // Output projection (128B)
    float dt;                     // Time-delta step parameter (4B)
    uint64_t step_count;          // Monotonic sequence step (8B)
    uint64_t magic_header;        // Validation: 0x4D414D4241335353 "MAMBA3SS"
    uint8_t  padding[20];         // Align to 512B total
};

// ─── INSTELLA_GATE: Expert routing structures ───
constexpr size_t TOTAL_EXPERTS = 64;
constexpr size_t MAX_TOP_K = 8;

struct alignas(64) InstellaGateInput {
    float hidden_states[384];
    float expert_weights[TOTAL_EXPERTS][384];
};

struct alignas(64) InstellaGateOutput {
    uint8_t  selected_expert_ids[MAX_TOP_K];
    float    expert_gate_weights[MAX_TOP_K];
    uint8_t  active_experts_count;
};

// ─── NVMe Persistent Memory Region ───
struct alignas(64) MambaPersistentRegion {
    uint64_t global_magic;        // 0x535741524D5F4E56 "SWARM_NV"
    uint32_t active_slots;
    uint8_t  reserved[52];
    Mamba3State slots[MAX_MAMBA_SLOTS];
};

// ─── C-ABI Standard API ───
typedef uint64_t mamba_handle_t;

mamba_handle_t c_mamba3_create_state(const float* dt_init);
void          c_mamba3_destroy_state(mamba_handle_t handle);
bool          c_mamba3_step(mamba_handle_t handle, const float* input_x, float* output_y);
bool          c_instella_gate_topk(const InstellaGateInput* input, uint8_t top_k, InstellaGateOutput* output);

// ─── C-ABI Persistent NVMe API ───
int       c_nvme_mamba_init(const char* filepath);
uint64_t  c_nvme_mamba_bind_slot(uint32_t agent_slot_id, float default_dt);
bool      c_mamba3_step_persistent(uint64_t handle, const float* input_x, float* output_y);
void      c_nvme_mamba_flush();

#ifdef __cplusplus
}
#endif

#endif // SWARM_KERNEL_OPS_H
