// capture_hidden.cpp — Build #1 reference: read the qwen3moe hidden state
// (voltage representation) before the output softmax, via llama.cpp.
//
// Usage: capture_hidden <model.gguf> <n_tokens> <token0> <token1> ...
// Prints the final-layer hidden state (llama_get_embeddings_ith) as f32.
#include "llama.h"
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>

int main(int argc, char** argv) {
    if (argc < 4) {
        fprintf(stderr, "usage: %s <model.gguf> <n_tokens> <tok>...\n", argv[0]);
        return 1;
    }
    const char* model_path = argv[1];
    int n_tokens = atoi(argv[2]);
    if (argc < 3 + n_tokens) {
        fprintf(stderr, "need %d tokens\n", n_tokens);
        return 1;
    }

    llama_model_params mparams = llama_model_default_params();
    // CPU inference for deterministic reference (no CUDA variance)
    mparams.n_gpu_layers = 0;
    mparams.use_mmap = false;
    llama_model* model = llama_load_model_from_file(model_path, mparams);
    if (!model) {
        fprintf(stderr, "failed to load model\n");
        return 1;
    }

    llama_context_params cparams = llama_context_default_params();
    cparams.n_ctx = 2048;
    cparams.n_batch = 32;
    cparams.n_ubatch = 32;
    cparams.no_perf = true;
    cparams.embeddings = true;   // extract hidden states (with logits)
    llama_context* ctx = llama_init_from_model(model, cparams);
    if (!ctx) {
        fprintf(stderr, "failed to init context\n");
        llama_free_model(model);
        return 1;
    }
    llama_set_embeddings(ctx, true);

    std::vector<llama_token> tokens;
    for (int i = 0; i < n_tokens; i++) {
        tokens.push_back((llama_token)atoi(argv[3 + i]));
    }

    // single batch: all tokens in one decode
    llama_batch batch = llama_batch_init(tokens.size(), 0, 1);
    for (int i = 0; i < (int)tokens.size(); i++) {
        batch.token[i] = tokens[i];
        batch.pos[i] = i;
        batch.n_seq_id[i] = 1;
        batch.seq_id[i][0] = 0;
        batch.logits[i] = false;
    }
    batch.n_tokens = tokens.size();

    if (llama_decode(ctx, batch) != 0) {
        fprintf(stderr, "decode failed\n");
        llama_batch_free(batch);
        llama_free(ctx);
        llama_free_model(model);
        return 1;
    }

    // hidden state of the LAST token (the voltage representation)
    int n_embd = llama_n_embd(model);
    float* emb = llama_get_embeddings_ith(ctx, tokens.size() - 1);
    if (!emb) {
        fprintf(stderr, "no embeddings (pooling disabled?)\n");
        llama_batch_free(batch);
        llama_free(ctx);
        llama_free_model(model);
        return 1;
    }

    // print as binary f32 (for the Rust verify harness)
    fwrite(&n_embd, sizeof(int), 1, stdout);
    fwrite(emb, sizeof(float), n_embd, stdout);

    llama_batch_free(batch);
    llama_free(ctx);
    llama_free_model(model);
    return 0;
}
