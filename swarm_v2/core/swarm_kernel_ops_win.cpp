/**
 * swarm_kernel_ops_win.cpp — TRM kernel operations with CCX affinity pinning.
 * Compiles via Zig: zig c++ -O3 -shared -mcpu=znver2 -mavx2 -mfma -o libswarm_kernel_ops.dll
 */

#include <cstdint>
#include <cstdbool>
#include <cstring>
#include <cmath>
#include <vector>
#include <algorithm>
#include <unordered_map>
#include <immintrin.h>
#include <windows.h>
#include <chrono>
#include <thread>
#include <atomic>

constexpr size_t MAMBA_STATE_DIM = 32;
constexpr size_t TOTAL_EXPERTS = 64;
constexpr size_t MAX_TOP_K = 8;
constexpr size_t MAX_SLOTS = 16;

struct alignas(64) BatchLoopResult {
    uint64_t total_steps_executed;
    double   elapsed_seconds;
    double   steps_per_second;
    float    final_checksum;
};

struct alignas(64) Mamba3State {
    float h[MAMBA_STATE_DIM], A[MAMBA_STATE_DIM], B[MAMBA_STATE_DIM], C[MAMBA_STATE_DIM];
    float dt; uint32_t slot_id; uint64_t step_count; uint64_t magic_header;
};

struct alignas(64) InstellaGateInput {
    float hidden_states[384]; float expert_weights[TOTAL_EXPERTS][384];
};

struct alignas(64) InstellaGateOutput {
    uint8_t selected_expert_ids[MAX_TOP_K];
    float   expert_gate_weights[MAX_TOP_K];
    uint8_t active_experts_count;
};

typedef uint64_t mamba_handle_t;

// Global state
static std::unordered_map<mamba_handle_t, Mamba3State*> g_states;
static uint64_t g_handle_counter = 1000;
static void* g_persistent_view = nullptr;
static HANDLE g_persistent_handle = nullptr;

// Zen 2 CCX affinity masks
constexpr DWORD_PTR ZEN2_CORE_MASKS[16] = {
    0x00000001ULL, 0x00000004ULL, 0x00000010ULL, 0x00000040ULL,
    0x00000100ULL, 0x00000400ULL, 0x00001000ULL, 0x00004000ULL,
    0x00010000ULL, 0x00040000ULL, 0x00100000ULL, 0x00400000ULL,
    0x00000002ULL, 0x00000008ULL, 0x00000020ULL, 0x00000080ULL,
};

extern "C" {

// ─── Standard API ───
__declspec(dllexport) mamba_handle_t c_mamba3_create_state(const float* dt_init) {
    Mamba3State* s = (Mamba3State*)_aligned_malloc(sizeof(Mamba3State), 64);
    std::memset(s, 0, sizeof(Mamba3State));
    s->dt = dt_init ? *dt_init : 0.001f; s->magic_header = 0x4D414D4241335353ULL;
    for (size_t i = 0; i < MAMBA_STATE_DIM; ++i) { s->A[i] = -0.5f; s->B[i] = 1.0f; s->C[i] = 1.0f; }
    mamba_handle_t h = g_handle_counter++; g_states[h] = s;
    return h;
}

__declspec(dllexport) void c_mamba3_destroy_state(mamba_handle_t h) {
    auto it = g_states.find(h);
    if (it != g_states.end()) { _aligned_free(it->second); g_states.erase(it); }
}

__declspec(dllexport) bool c_mamba3_step(mamba_handle_t h, const float* x, float* y) {
    auto it = g_states.find(h);
    if (it == g_states.end() || !x || !y) return false;
    Mamba3State* s = it->second; float dt = s->dt, ya = 0.0f;
    for (size_t i = 0; i < MAMBA_STATE_DIM; ++i) {
        float dA = 1.0f + s->A[i] * dt;
        s->h[i] = s->h[i] * dA + s->B[i] * (*x) * dt;
        ya += s->h[i] * s->C[i];
    }
    *y = ya; s->step_count++; return true;
}

__declspec(dllexport) bool c_instella_gate_topk(const InstellaGateInput* in, uint8_t k, InstellaGateOutput* out) {
    if (!in || !out || k == 0 || k > MAX_TOP_K) return false;
    float logits[TOTAL_EXPERTS];
    for (size_t e = 0; e < TOTAL_EXPERTS; ++e) {
        __m256 acc = _mm256_setzero_ps();
        for (size_t i = 0; i < 384; i += 8)
            acc = _mm256_fmadd_ps(_mm256_loadu_ps(&in->hidden_states[i]), _mm256_loadu_ps(&in->expert_weights[e][i]), acc);
        float t[8]; _mm256_storeu_ps(t, acc);
        logits[e] = t[0]+t[1]+t[2]+t[3]+t[4]+t[5]+t[6]+t[7];
    }
    std::vector<std::pair<float,uint8_t>> sc(TOTAL_EXPERTS);
    for (uint8_t i = 0; i < TOTAL_EXPERTS; ++i) sc[i] = {logits[i], i};
    std::partial_sort(sc.begin(), sc.begin()+k, sc.end(), [](auto& a, auto& b){return a.first>b.first;});
    float ml = sc[0].first, se = 0.0f;
    for (size_t i = 0; i < k; ++i) { float ev = std::exp(sc[i].first-ml); out->expert_gate_weights[i]=ev; out->selected_expert_ids[i]=sc[i].second; se+=ev; }
    for (size_t i = 0; i < k; ++i) out->expert_gate_weights[i] /= se;
    out->active_experts_count = k; return true;
}

// ─── Persistent state ───
__declspec(dllexport) int c_persistent_state_init(const char* path) {
    size_t sz = sizeof(Mamba3State) * 16;
    g_persistent_handle = CreateFileMappingA(INVALID_HANDLE_VALUE, NULL, PAGE_READWRITE, 0, (DWORD)sz, path?path:"swarm_persist.shm");
    if (!g_persistent_handle) return -1;
    g_persistent_view = MapViewOfFile(g_persistent_handle, FILE_MAP_ALL_ACCESS, 0, 0, sz);
    return g_persistent_view ? 0 : -2;
}

__declspec(dllexport) uint64_t c_persistent_state_get_slot(uint32_t id) {
    if (!g_persistent_view || id >= 16) return 0;
    Mamba3State* s = (Mamba3State*)g_persistent_view + id;
    if (s->magic_header != 0x4D414D4241335353ULL) {
        std::memset(s, 0, sizeof(Mamba3State)); s->magic_header = 0x4D414D4241335353ULL; s->dt = 0.001f;
        for (size_t i = 0; i < MAMBA_STATE_DIM; ++i) { s->A[i]=-0.5f; s->B[i]=1.0f; s->C[i]=1.0f; }
    }
    return (uint64_t)s;
}

__declspec(dllexport) void c_persistent_state_flush() {
    if (g_persistent_view) FlushViewOfFile(g_persistent_view, sizeof(Mamba3State)*16);
}

// ─── CCX Affinity Pinned Batch Loop ───
__declspec(dllexport) bool c_run_pinned_batch_loop(uint64_t total, uint8_t threads, BatchLoopResult* out) {
    if (!out || threads == 0 || threads > 16) return false;
    HANDLE hf = CreateFileMappingA(INVALID_HANDLE_VALUE, NULL, PAGE_READWRITE, 0, sizeof(Mamba3State)*16, "Local\\swarm_pinned.shm");
    if (!hf) return false;
    Mamba3State* slots = (Mamba3State*)MapViewOfFile(hf, FILE_MAP_ALL_ACCESS, 0, 0, sizeof(Mamba3State)*16);
    if (!slots) { CloseHandle(hf); return false; }
    for (uint8_t a = 0; a < 15; ++a) {
        Mamba3State& s = slots[a];
        if (s.magic_header != 0x4D414D4241335353ULL) {
            std::memset(&s, 0, sizeof(Mamba3State)); s.magic_header = 0x4D414D4241335353ULL; s.slot_id = a;
            s.dt = 0.001f + a * 0.0001f;
            for (size_t i = 0; i < MAMBA_STATE_DIM; ++i) { s.A[i]=-0.5f; s.B[i]=1.0f; s.C[i]=1.0f; }
        }
    }
    auto t0 = std::chrono::high_resolution_clock::now();
    auto worker = [&](uint8_t tid) {
        if (tid < 16) { SetThreadAffinityMask(GetCurrentThread(), ZEN2_CORE_MASKS[tid]); SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST); }
        uint64_t per = total / threads, st = tid * per, en = (tid+1==threads)?total:st+per;
        float sig = 0.85f + tid * 0.01f;
        for (uint64_t step = st; step < en; ++step) {
            Mamba3State& s = slots[(step+tid)%15];
            __m256 vd = _mm256_set1_ps(s.dt), vi = _mm256_set1_ps(sig); float ya = 0.0f;
            for (size_t i = 0; i < MAMBA_STATE_DIM; i += 8) {
                __m256 vh=_mm256_load_ps(&s.h[i]), vA=_mm256_load_ps(&s.A[i]), vB=_mm256_load_ps(&s.B[i]), vC=_mm256_load_ps(&s.C[i]);
                __m256 vdA=_mm256_fmadd_ps(vA,vd,_mm256_set1_ps(1.0f)), vdB=_mm256_mul_ps(vB,vi);
                __m256 vhn=_mm256_fmadd_ps(vdB,vd,_mm256_mul_ps(vh,vdA));
                _mm256_store_ps(&s.h[i], vhn);
                volatile float* vs=&s.h[0]; *vs=s.h[0];
                __m256 vy=_mm256_mul_ps(vhn,vC); float t[8]; _mm256_store_ps(t,vy);
                for(int k=0;k<8;++k) ya+=t[k];
            }
            s.step_count++;
            if((step&0xFFFFF)==0) sig=(ya>0.0f)?0.85f:-0.85f;
        }
    };
    std::vector<std::thread> th; for (uint8_t t=0;t<threads;++t) th.emplace_back(worker,t);
    for (auto& t:th) t.join();
    auto t1 = std::chrono::high_resolution_clock::now();
    std::chrono::duration<double> d = t1-t0;
    out->total_steps_executed=total; out->elapsed_seconds=d.count();
    out->steps_per_second=(double)total/d.count(); out->final_checksum=0.0f;
    FlushViewOfFile(slots,sizeof(Mamba3State)*16); UnmapViewOfFile(slots); CloseHandle(hf);
    return true;
}

} // extern "C"
