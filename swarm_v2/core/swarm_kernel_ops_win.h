#ifndef SWARM_KERNEL_OPS_WIN_H
#define SWARM_KERNEL_OPS_WIN_H

#include <cstdint>
#include <cstdbool>

#ifdef __cplusplus
extern "C" {
#endif

constexpr size_t MAMBA_STATE_DIM = 32;
constexpr size_t MAX_MAMBA_SLOTS = 16;

struct alignas(64) Mamba3State {
    float h[MAMBA_STATE_DIM];
    float A[MAMBA_STATE_DIM];
    float B[MAMBA_STATE_DIM];
    float C[MAMBA_STATE_DIM];
    float dt;
    uint32_t slot_id;
    uint64_t step_count;
    uint64_t magic_header;
    uint8_t  padding[16];
};

struct alignas(64) MambaPersistentRegion {
    uint64_t global_magic;
    uint32_t active_slots;
    uint8_t  reserved[52];
    Mamba3State slots[MAX_MAMBA_SLOTS];
};

struct alignas(64) BatchLoopResult {
    uint64_t total_steps_executed;
    double   elapsed_seconds;
    double   steps_per_second;
    float    final_checksum;
};

// Existing single-step API
__declspec(dllexport) uint64_t c_mamba3_create_state(const float* dt_init);
__declspec(dllexport) void     c_mamba3_destroy_state(uint64_t handle);
__declspec(dllexport) bool     c_mamba3_step(uint64_t handle, const float* input_x, float* output_y);

// Batch loop: entire 100M iteration inside C++ — zero ctypes overhead
__declspec(dllexport) bool     c_run_batch_mamba3_loop(
    uint64_t total_iterations,
    uint8_t  active_agents_count,
    BatchLoopResult* out_result
);

#ifdef __cplusplus
}
#endif

#endif
