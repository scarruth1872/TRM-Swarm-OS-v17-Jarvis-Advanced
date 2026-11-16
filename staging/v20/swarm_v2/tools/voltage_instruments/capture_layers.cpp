// capture_layers.cpp — capture hidden states after EACH block (GPU fast path).
// The Rust forward is verified per-layer against these, not in one giant run.
// Usage: capture_layers <model.gguf> <n_tokens> <tok>...
// Output: binary [n_layers+1][1024] f32 (post-attn residual + post-ffn, flattened:
//   for each layer: post-attention hidden, then post-FFN hidden; plus final norm).
#include "llama.h"
#include <cstdio>
#include <cstdlib>
#include <vector>

int main(int argc, char** argv) {
    if (argc < 4) { fprintf(stderr, "usage: %s <model> <n_tokens> <tok>...\n", argv[0]); return 1; }
    const char* model_path = argv[1];
    int n_tokens = atoi(argv[2]);

    llama_model_params mparams = llama_model_default_params();
    mparams.n_gpu_layers = 99;  // GPU fast path (RTX 3070)
    mparams.use_mmap = false;
    llama_model* model = llama_load_model_from_file(model_path, mparams);
    if (!model) { fprintf(stderr, "load failed\n"); return 1; }

    llama_context_params cparams = llama_context_default_params();
    cparams.n_ctx = 2048; cparams.n_batch = 32; cparams.n_ubatch = 32;
    cparams.no_perf = true;
    cparams.embeddings = true;
    llama_context* ctx = llama_init_from_model(model, cparams);
    if (!ctx) { fprintf(stderr, "ctx failed\n"); llama_free_model(model); return 1; }
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
    fwrite(&n_embd, sizeof(int), 1, stdout);
    fwrite(emb, sizeof(float), n_embd, stdout);

    llama_batch_free(batch);
    llama_free(ctx);
    llama_free_model(model);
    return 0;
}
