/**
 * swarm_kernel_ops.cpp — Native C++ AVX2/FMA kernel operations.
 * Combines standard MAMBA3STEP + INSTELLA_GATE + NVMe persistent state.
 * 
 * Compile: ./build_kernel_ops.sh
 *   g++ -O3 -shared -fPIC -march=native -mavx2 -mfma -flto
 *       -o libswarm_kernel_ops.so swarm_kernel_ops.cpp
 */

#include "swarm_kernel_ops.h"
#include <immintrin.h>
#include <fcntl.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>
#include <cmath>
#include <cstring>
#include <algorithm>
#include <vector>
#include <unordered_map>

// ─── Standard API: In-memory MAMBA3 states ───
static std::unordered_map<mamba_handle_t, Mamba3State*> g_mamba_states;
static uint64_t g_handle_counter = 1000;

extern "C" {

mamba_handle_t c_mamba3_create_state(const float* dt_init) {
    Mamba3State* state = new (std::align_val_t(64)) Mamba3State();
    std::memset(state, 0, sizeof(Mamba3State));
    
    state->dt = (dt_init) ? *dt_init : 0.001f;
    state->magic_header = 0x4D414D4241335353ULL;
    
    for (size_t i = 0; i < MAMBA_STATE_DIM; ++i) {
        state->A[i] = -0.5f;
        state->B[i] = 1.0f;
        state->C[i] = 1.0f;
    }
    
    mamba_handle_t handle = g_handle_counter++;
    g_mamba_states[handle] = state;
    return handle;
}

void c_mamba3_destroy_state(mamba_handle_t handle) {
    auto it = g_mamba_states.find(handle);
    if (it != g_mamba_states.end()) {
        operator delete(it->second, std::align_val_t(64));
        g_mamba_states.erase(it);
    }
}

bool c_mamba3_step(mamba_handle_t handle, const float* input_x, float* output_y) {
    auto it = g_mamba_states.find(handle);
    if (it == g_mamba_states.end() || !input_x || !output_y) return false;

    Mamba3State* s = it->second;
    float dt = s->dt;
    __m256 v_dt = _mm256_set1_ps(dt);
    __m256 v_input = _mm256_set1_ps(*input_x);
    float y_accum = 0.0f;

    for (size_t i = 0; i < MAMBA_STATE_DIM; i += 8) {
        __m256 v_h  = _mm256_load_ps(&s->h[i]);
        __m256 v_A  = _mm256_load_ps(&s->A[i]);
        __m256 v_B  = _mm256_load_ps(&s->B[i]);
        __m256 v_C  = _mm256_load_ps(&s->C[i]);

        // dA = 1.0 + A*dt  (first-order Taylor)
        __m256 v_dA = _mm256_fmadd_ps(v_A, v_dt, _mm256_set1_ps(1.0f));
        // h_new = h*dA + B*input*dt
        __m256 v_dB = _mm256_mul_ps(v_B, v_input);
        __m256 v_h_new = _mm256_fmadd_ps(v_dB, v_dt, _mm256_mul_ps(v_h, v_dA));

        _mm256_store_ps(&s->h[i], v_h_new);

        // y += h_new * C
        __m256 v_y = _mm256_mul_ps(v_h_new, v_C);
        alignas(32) float temp[8];
        _mm256_store_ps(temp, v_y);
        for (int k = 0; k < 8; ++k) y_accum += temp[k];
    }

    *output_y = y_accum;
    s->step_count++;
    return true;
}

bool c_instella_gate_topk(const InstellaGateInput* input, uint8_t top_k, InstellaGateOutput* output) {
    if (!input || !output || top_k == 0 || top_k > MAX_TOP_K) return false;

    float logits[TOTAL_EXPERTS];

    // Dot product: W[e] · x across 64 experts
    for (size_t e = 0; e < TOTAL_EXPERTS; ++e) {
        __m256 v_acc = _mm256_setzero_ps();
        for (size_t i = 0; i < 384; i += 8) {
            __m256 v_in = _mm256_loadu_ps(&input->hidden_states[i]);
            __m256 v_w  = _mm256_loadu_ps(&input->expert_weights[e][i]);
            v_acc = _mm256_fmadd_ps(v_in, v_w, v_acc);
        }
        alignas(32) float temp[8];
        _mm256_store_ps(temp, v_acc);
        logits[e] = temp[0]+temp[1]+temp[2]+temp[3]+temp[4]+temp[5]+temp[6]+temp[7];
    }

    // Top-K partial sort
    std::vector<std::pair<float, uint8_t>> scores(TOTAL_EXPERTS);
    for (uint8_t i = 0; i < TOTAL_EXPERTS; ++i)
        scores[i] = {logits[i], i};

    std::partial_sort(scores.begin(), scores.begin()+top_k, scores.end(),
        [](const auto& a, const auto& b) { return a.first > b.first; });

    // Softmax over top-K
    float max_logit = scores[0].first, sum_exp = 0.0f;
    for (size_t i = 0; i < top_k; ++i) {
        float exp_v = std::exp(scores[i].first - max_logit);
        output->expert_gate_weights[i] = exp_v;
        output->selected_expert_ids[i] = scores[i].second;
        sum_exp += exp_v;
    }
    for (size_t i = 0; i < top_k; ++i)
        output->expert_gate_weights[i] /= sum_exp;

    output->active_experts_count = top_k;
    return true;
}

// ─── NVMe Persistent Memory API ───

static MambaPersistentRegion* g_nvme_region = nullptr;
static int g_shm_fd = -1;

int c_nvme_mamba_init(const char* filepath) {
    const char* path = filepath ? filepath : "/var/shm/swarm_mamba_persistent.shm";
    
    g_shm_fd = open(path, O_CREAT | O_RDWR, 0666);
    if (g_shm_fd == -1) return -1;

    size_t size = sizeof(MambaPersistentRegion);
    struct stat st;
    fstat(g_shm_fd, &st);
    
    if (st.st_size < (off_t)size) {
        if (ftruncate(g_shm_fd, size) == -1) { close(g_shm_fd); return -2; }
    }

    void* ptr = mmap(nullptr, size, PROT_READ | PROT_WRITE, MAP_SHARED, g_shm_fd, 0);
    if (ptr == MAP_FAILED) { close(g_shm_fd); return -3; }

    g_nvme_region = static_cast<MambaPersistentRegion*>(ptr);

    if (g_nvme_region->global_magic != 0x535741524D5F4E56ULL) {
        g_nvme_region->global_magic = 0x535741524D5F4E56ULL;
        g_nvme_region->active_slots = 0;
        std::memset(g_nvme_region->slots, 0, sizeof(g_nvme_region->slots));
    }

    return 0;
}

uint64_t c_nvme_mamba_bind_slot(uint32_t agent_slot_id, float default_dt) {
    if (!g_nvme_region || agent_slot_id >= MAX_MAMBA_SLOTS) return 0;

    Mamba3State& slot = g_nvme_region->slots[agent_slot_id];

    if (slot.magic_header == 0x4D414D4241335353ULL) {
        // Cold-start recovery: state already persisted
    } else {
        std::memset(&slot, 0, sizeof(Mamba3State));
        slot.magic_header = 0x4D414D4241335353ULL;
        slot.dt = default_dt;
        for (size_t i = 0; i < MAMBA_STATE_DIM; ++i) {
            slot.A[i] = -0.5f; slot.B[i] = 1.0f; slot.C[i] = 1.0f;
        }
        g_nvme_region->active_slots++;
    }

    return reinterpret_cast<uint64_t>(&slot);
}

bool c_mamba3_step_persistent(uint64_t handle, const float* input_x, float* output_y) {
    if (!handle || !input_x || !output_y) return false;

    Mamba3State* s = reinterpret_cast<Mamba3State*>(handle);
    if (s->magic_header != 0x4D414D4241335353ULL) return false;

    float dt = s->dt;
    __m256 v_dt = _mm256_set1_ps(dt);
    __m256 v_input = _mm256_set1_ps(*input_x);
    float y_accum = 0.0f;

    for (size_t i = 0; i < MAMBA_STATE_DIM; i += 8) {
        __m256 v_h  = _mm256_load_ps(&s->h[i]);
        __m256 v_A  = _mm256_load_ps(&s->A[i]);
        __m256 v_B  = _mm256_load_ps(&s->B[i]);
        __m256 v_C  = _mm256_load_ps(&s->C[i]);

        __m256 v_dA = _mm256_fmadd_ps(v_A, v_dt, _mm256_set1_ps(1.0f));
        __m256 v_dB = _mm256_mul_ps(v_B, v_input);
        __m256 v_h_new = _mm256_fmadd_ps(v_dB, v_dt, _mm256_mul_ps(v_h, v_dA));

        _mm256_store_ps(&s->h[i], v_h_new);  // Direct NVMe write

        __m256 v_y = _mm256_mul_ps(v_h_new, v_C);
        alignas(32) float temp[8];
        _mm256_store_ps(temp, v_y);
        for (int k = 0; k < 8; ++k) y_accum += temp[k];
    }

    *output_y = y_accum;
    s->step_count++;
    return true;
}

void c_nvme_mamba_flush() {
    if (g_nvme_region) {
        msync(reinterpret_cast<void*>(g_nvme_region), sizeof(MambaPersistentRegion), MS_ASYNC);
    }
}

} // extern "C"
