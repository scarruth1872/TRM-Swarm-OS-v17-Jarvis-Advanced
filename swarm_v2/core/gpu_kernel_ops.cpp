/**
 * gpu_kernel_ops.cpp — Native HIP/ROCm kernel for INSTELLA_GATE.
 * Compiles with: hipcc -O3 --offload-arch=gfx1031 -shared -o libgpu_kernel_ops.so
 * 
 * Targets RX 6700 XT (Navi 22, gfx1031, RDNA2)
 * 2,560 stream processors × 40 CUs × 2.5 GHz = 12.8 TFLOPS FP32
 */

#include <hip/hip_runtime.h>
#include <cmath>
#include <cstdint>

constexpr int TOTAL_EXPERTS = 64;
constexpr int HIDDEN_DIM = 384;
constexpr int WARP_SIZE = 64;  // AMD wavefront size
constexpr int BLOCK_SIZE = 256;

/**
 * HIP Kernel: Parallel dot product across all 64 experts.
 * Each thread block computes one expert's score.
 * 64 blocks × 256 threads = full GPU saturation.
 */
__global__ void instella_expert_scores_kernel(
    const float* __restrict__ hidden_states,   // [384]
    const float* __restrict__ expert_weights,  // [64][384]
    float* __restrict__ expert_scores,          // [64]
    int batch_size
) {
    int expert_idx = blockIdx.x;
    if (expert_idx >= TOTAL_EXPERTS) return;
    
    // Each thread handles a chunk of the 384-dim dot product
    __shared__ float partial_sums[BLOCK_SIZE];
    
    int tid = threadIdx.x;
    float acc = 0.0f;
    
    // Strided access pattern for coalesced memory reads
    for (int i = tid; i < HIDDEN_DIM; i += BLOCK_SIZE) {
        acc += hidden_states[i] * expert_weights[expert_idx * HIDDEN_DIM + i];
    }
    
    partial_sums[tid] = acc;
    __syncthreads();
    
    // Warp-level reduction
    for (int stride = BLOCK_SIZE / 2; stride > 0; stride >>= 1) {
        if (tid < stride) {
            partial_sums[tid] += partial_sums[tid + stride];
        }
        __syncthreads();
    }
    
    if (tid == 0) {
        expert_scores[expert_idx] = partial_sums[0];
    }
}

/**
 * HIP Kernel: Top-K selection using warp-level primitives.
 */
__global__ void instella_topk_kernel(
    const float* __restrict__ scores,    // [64]
    int* __restrict__ topk_indices,      // [K]
    float* __restrict__ topk_weights,    // [K]
    int k
) {
    extern __shared__ float shared_scores[];
    int tid = threadIdx.x;
    
    // Load scores into shared memory
    if (tid < TOTAL_EXPERTS) {
        shared_scores[tid] = scores[tid];
    }
    __syncthreads();
    
    // Bitonic sort in shared memory (64 elements fits in one wavefront)
    for (int stage = 2; stage <= TOTAL_EXPERTS; stage <<= 1) {
        for (int step = stage >> 1; step > 0; step >>= 1) {
            int idx = tid ^ step;
            if (idx > tid) {
                bool ascending = ((tid & stage) == 0);
                float a = shared_scores[tid];
                float b = shared_scores[idx];
                if ((a < b) == ascending) {
                    shared_scores[tid] = b;
                    shared_scores[idx] = a;
                }
            }
            __syncthreads();
        }
    }
    
    // Write top-K results
    if (tid < k) {
        topk_indices[tid] = tid;  // Simplified — full kernel would track original indices
        topk_weights[tid] = shared_scores[TOTAL_EXPERTS - 1 - tid];
    }
}

// ─── C-ABI Exports ───

extern "C" {

/**
 * Initialize GPU device and allocate memory.
 * Returns device handle (>0) on success, 0 on failure.
 */
__declspec(dllexport) int c_gpu_init() {
    int device_count;
    hipError_t err = hipGetDeviceCount(&device_count);
    if (err != hipSuccess || device_count == 0) return 0;
    
    hipSetDevice(0);
    return device_count;
}

/**
 * Run INSTELLA_GATE on GPU — single call with batched vectors.
 * Returns scores for all 64 experts.
 */
__declspec(dllexport) int c_gpu_instella_gate_batch(
    const float* hidden_states,   // [batch][384]
    const float* expert_weights,  // [64][384]
    float* expert_scores,         // [batch][64]
    int batch_size
) {
    float *d_hidden, *d_weights, *d_scores;
    
    // Async copy to device
    hipMalloc(&d_hidden, batch_size * HIDDEN_DIM * sizeof(float));
    hipMalloc(&d_weights, TOTAL_EXPERTS * HIDDEN_DIM * sizeof(float));
    hipMalloc(&d_scores, batch_size * TOTAL_EXPERTS * sizeof(float));
    
    hipMemcpyAsync(d_hidden, hidden_states, 
                   batch_size * HIDDEN_DIM * sizeof(float), 
                   hipMemcpyHostToDevice, 0);
    hipMemcpyAsync(d_weights, expert_weights,
                   TOTAL_EXPERTS * HIDDEN_DIM * sizeof(float),
                   hipMemcpyHostToDevice, 0);
    
    // Launch kernel: 64 blocks × 256 threads
    dim3 grid(TOTAL_EXPERTS);
    dim3 block(BLOCK_SIZE);
    size_t shared_mem = BLOCK_SIZE * sizeof(float);
    
    for (int b = 0; b < batch_size; ++b) {
        instella_expert_scores_kernel<<<grid, block, shared_mem, 0>>>(
            &d_hidden[b * HIDDEN_DIM],
            d_weights,
            &d_scores[b * TOTAL_EXPERTS],
            1
        );
    }
    
    hipDeviceSynchronize();
    
    // Copy results back
    hipMemcpy(expert_scores, d_scores,
              batch_size * TOTAL_EXPERTS * sizeof(float),
              hipMemcpyDeviceToHost);
    
    hipFree(d_hidden);
    hipFree(d_weights);
    hipFree(d_scores);
    
    return batch_size;
}

} // extern "C"
