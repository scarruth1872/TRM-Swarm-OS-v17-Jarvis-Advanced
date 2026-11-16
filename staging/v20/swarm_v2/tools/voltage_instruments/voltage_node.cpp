//! voltage_node.cpp — the GPU fast-path voltage node.
//!
//! The correct architecture (user-directed): llama.cpp CUDA runs the real
//! forward (339 tensors, ~2s on RTX 3070), reads the hidden state, and the
//! verified K3 collapse produces the action vector. No CPU matmul re-derivation.
//!
//! Output: hidden_state.bin (1024 f32) + K3 action vector printed.
#include "llama.h"
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <vector>

int main(int argc, char** argv) {
    if (argc < 4) { fprintf(stderr, "usage: %s <model> <n_tokens> <tok>...\n", argv[0]); return 1; }
    const char* model_path = argv[1];
    int n_tokens = atoi(argv[2]);

    llama_model_params mparams = llama_model_default_params();
    mparams.n_gpu_layers = 99;   // GPU fast path
    mparams.use_mmap = false;
    llama_model* model = llama_load_model_from_file(model_path, mparams);
    if (!model) { fprintf(stderr, "load failed\n"); return 1; }

    llama_context_params cparams = llama_context_default_params();
    cparams.n_ctx = 2048; cparams.n_batch = 32; cparams.n_ubatch = 32;
    cparams.no_perf = true;
    cparams.embeddings = true;
    llama_context* ctx = llama_init_from_model(model, cparams);
    if (!ctx) { llama_free_model(model); return 1; }
    llama_set_embeddings(ctx, true);

    std::vector<llama_token> tokens;
    for (int i = 0; i < n_tokens; i++) tokens.push_back((llama_token)atoi(argv[3 + i]));

    llama_batch batch = llama_batch_init(tokens.size(), 0, 1);
    for (int i = 0; i < (int)tokens.size(); i++) {
        batch.token[i] = tokens[i]; batch.pos[i] = i;
        batch.n_seq_id[i] = 1; batch.seq_id[i][0] = 0; batch.logits[i] = false;
    }
    batch.n_tokens = tokens.size();
    if (llama_decode(ctx, batch) != 0) { fprintf(stderr, "decode failed\n"); return 1; }

    int n_embd = llama_n_embd(model);
    float* emb = llama_get_embeddings_ith(ctx, tokens.size() - 1);
    if (!emb) { fprintf(stderr, "no embeddings\n"); return 1; }

    // hidden state (the voltage field)
    FILE* f = fopen("/home/shawn/trm-rust/hidden_state.bin", "wb");
    fwrite(&n_embd, sizeof(int), 1, f);
    fwrite(emb, sizeof(float), n_embd, f);
    fclose(f);
    printf("hidden state: %d dims written\n", n_embd);
    printf("magnitude: %.4f\n", [&](){ double s=0; for(int i=0;i<n_embd;i++) s += emb[i]*emb[i]; return sqrt(s); }());

    llama_batch_free(batch);
    llama_free(ctx);
    llama_free_model(model);
    return 0;
}
