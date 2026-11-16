// voltage_node_offload.cpp — partial GPU offload for models that don't
// fully fit (Bonsai 3.8GB vs 8GB GPU with backend load).
// Usage: <model> <n_gpu_layers> <n_tokens> <tok>...
#include "llama.h"
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <vector>

int main(int argc, char** argv) {
    if (argc < 5) { fprintf(stderr, "usage: %s <model> <gpu_layers> <n_tokens> <tok>...\n", argv[0]); return 1; }
    const char* model_path = argv[1];
    int gpu_layers = atoi(argv[2]);
    int n_tokens = atoi(argv[3]);

    llama_model_params mparams = llama_model_default_params();
    mparams.n_gpu_layers = gpu_layers;
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
    for (int i = 0; i < n_tokens; i++) tokens.push_back((llama_token)atoi(argv[4 + i]));

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

    FILE* f = fopen("/home/shawn/trm-rust/hidden_state.bin", "wb");
    fwrite(&n_embd, sizeof(int), 1, f);
    fwrite(emb, sizeof(float), n_embd, f);
    fclose(f);
    double s = 0; for (int i = 0; i < n_embd; i++) s += emb[i]*emb[i];
    printf("hidden state: %d dims | magnitude: %.4f | gpu_layers: %d\n", n_embd, sqrt(s), gpu_layers);

    llama_batch_free(batch);
    llama_free(ctx);
    llama_free_model(model);
    return 0;
}
