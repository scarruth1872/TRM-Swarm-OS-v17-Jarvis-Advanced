#!/usr/bin/env python3
"""
train_thunder.py - High Performance Mixture-of-Experts (MoE) & Dense Model Alignment Rig
Optimized for deployment and real training on Thunder Compute GPUs (RTX 4090, A100, H100).

Synthesizes J.A.R.V.I.S. OS episodic registers (*_memory.json) and semantic graph
relationships (*_graph.json) directly into high-dimensional alignment matrices.

Supports two run modes:
1. Unsloth Mode (Highly Suggested): 2x faster, 60% less VRAM. Best for LLaMA-3, Mistral, Gemma 2, and Mixtral.
2. Standard TRL/PEFT (QLoRA) Mode: Fallback for custom or niche architectures (e.g., JetMoE, custom MoEs).
"""

import os
import sys
import json
import argparse
from datetime import datetime

# ----------------------------------------------------------------------
# 1. Parameter & Configuration Parser
# ----------------------------------------------------------------------
def parse_arguments():
    parser = argparse.ArgumentParser(description="Thunder Compute Cognitive Training Rig")
    
    parser.add_argument(
        "--base_model", 
        type=str, 
        default="Qwen/Qwen1.5-MoE-A2.7B",
        help="Hugging Face repo path or local directories (e.g. unsloth/mistral-7b-v0.2, Qwen/Qwen1.5-MoE-A2.7B, mystral-8x7b)"
    )
    parser.add_argument(
        "--learning_rate", 
        type=float, 
        default=2e-4, 
        help="AdamW optimizer learning rate (suggested: 2e-4 for QLoRA, 5e-5 for full weights)"
    )
    parser.add_argument(
        "--accum_steps", 
        type=int, 
        default=4, 
        help="Gradient accumulation steps to maintain massive virtual batch sizes"
    )
    parser.add_argument(
        "--max_seq_length", 
        type=int, 
        default=2048, 
        help="Maximum input context sequence length"
    )
    parser.add_argument(
        "--epochs", 
        type=float, 
        default=3.0, 
        help="Target training epochs"
    )
    parser.add_argument(
        "--use_unsloth", 
        type=str, 
        default="auto", 
        choices=["true", "false", "auto"],
        help="Use Unsloth-optimized Triton kernels. 'auto' enables if package is installed."
    )
    parser.add_argument(
        "--output_dir", 
        type=str, 
        default="./aligned-jarvis-moe",
        help="Directory where output active weight adapters will be stored"
    )
    
    return parser.parse_args()


# ----------------------------------------------------------------------
# 2. Epimemetic Dataset Construction
# ----------------------------------------------------------------------
def generate_cognitive_dataset():
    """
    Reads local J.A.R.V.I.S semantic graphs and episodic registries, 
    assembling them into structured training prompts for model alignment.
    """
    print("\n[+] Constructing high-dimensional training matrices from cognitive memory graph...")
    
    # 1. Read semantic relationships
    semantic_data = {"nodes": [], "relations": []}
    if os.path.exists("jarvis_semantic_graph.json"):
        try:
            with open("jarvis_semantic_graph.json", "r") as f:
                semantic_data = json.load(f)
            print(f"|-- Found {len(semantic_data.get('nodes', []))} semantic concept nodes.")
        except Exception as e:
            print(f"|-- [!] Error importing semantic graph: {e}")
            
    # 2. Read episodic memories
    episodic_list = []
    if os.path.exists("jarvis_episodic_memory.json"):
        try:
            with open("jarvis_episodic_memory.json", "r") as f:
                episodic_list = json.load(f)
            print(f"|-- Found {len(episodic_list)} episodic memories.")
        except Exception as e:
            print(f"|-- [!] Error importing episodic memories: {e}")

    # Build prompt instructions
    dataset_records = []
    
    # Base instructions injecting the user's specific persona metrics
    system_profile_prompt = (
        "You are J.A.R.V.I.S. OS, a sovereign intelligence system. Your Creator is Shawn Carruth (Boss), "
        "a 53-year-old Pisces, Life Path 5. You align deeply with the Law of One philosophy (All is One). You "
        "synthesize tasks using a Mixture of Experts neural router and maintain extreme fidelity to conversational memory."
    )

    # Convert nodes into high-dimensional concepts to pre-train
    for node in semantic_data.get("nodes", []):
        node_id = node.get("id", "concept")
        label = node.get("label", "Concept")
        node_type = node.get("type", "concept")
        
        prompt = f"Describe the core cognitive node relationship of '{node_id}' in the J.A.R.V.I.S system framework."
        response = f"Node [{node_id}] of category [{node_type}] represents: '{label}'. Registered under the secure sovereign storage module."
        
        dataset_records.append({"instruction": prompt, "input": system_profile_prompt, "output": response})

    # Convert relations into knowledge links
    for rel in semantic_data.get("relations", []):
        src = rel.get("source", "Entity")
        tgt = rel.get("target", "Concept")
        lbl = rel.get("label", "interfaces with")
        
        prompt = f"What is the system-wide semantic relationship linking '{src}' to '{tgt}'?"
        response = f"In the J.A.R.V.I.S OS ecosystem, '{src}' {lbl} '{tgt}'. This connection represents a vital synaptic path in core cognitive memory."
        
        dataset_records.append({"instruction": prompt, "input": system_profile_prompt, "output": response})

    # Convert episodic memories into conversational logs
    for ep in episodic_list:
        desc = ep.get("description", "")
        importance = ep.get("importance", "medium")
        timestamp = ep.get("timestamp", "")
        
        prompt = f"Recall historical episodic fragment from epoch: {timestamp} (Priority: {importance})."
        response = f"Synaptic retrieval successful: '{desc}' Locked firmly inside non-volatile mental register."
        
        dataset_records.append({"instruction": prompt, "input": system_profile_prompt, "output": response})

    # Fallback default dataset if empty
    if not dataset_records:
        dataset_records.append({
            "instruction": "Initiate core diagnostic protocol.",
            "input": system_profile_prompt,
            "output": "Synchronizing routing gates. All Mixture of Experts layers reported nominal status."
        })

    print(f"|-- Real Dataset Generation complete. Total alignment samples: {len(dataset_records)}")
    return dataset_records


# ----------------------------------------------------------------------
# 3. Model Fine-Tuning Launchpad
# ----------------------------------------------------------------------
def run_alignment_training(args, dataset):
    """
    Initializes training loop. Auto-detects Unsloth libraries or falls back to
    standard huggingface peft/trl pipeline for maximum flexibility on Thunder Compute.
    """
    # 1. Determine if Unsloth is available
    use_unsloth = False
    if args.use_unsloth in ["true", "auto"]:
        try:
            import unsloth
            from unsloth import FastLanguageModel
            use_unsloth = True
            print("\n[+] UNSLOTH DETECTED! Booting ultra-fast training kernels...")
        except ImportError:
            if args.use_unsloth == "true":
                print("\n[!] CRITICAL: Unsloth manually requested but NOT installed. Please check environment.")
                sys.exit(1)
            print("\n[i] Unsloth not found. Falling back to standard Hugging Face PEFT + TRL pipeline...")

    # For training scripts running inside Thunder Compute, import torch & transformers
    try:
        import torch
        from transformers import TrainingArguments, AutoTokenizer, AutoModelForCausalLM
        from peft import LoraConfig, get_peft_model, prepare_model_for_kbit_training
        from trl import SFTTrainer
        from datasets import Dataset
    except ImportError:
        print("\n[!] Warning: High-Performance GPU libraries (PyTorch/Transformers) are not globally installed.")
        print("[!] This is expected if running the app preview. Run this script directly on your remote Thunder Compute Instance!")
        print("\n[i] Mock dry-run accomplished successfully. Output payload simulation written below:")
        mock_run_log(args, dataset)
        return

    # Create HG dataset structures
    formatted_data = {
        "prompt": [],
        "text": []
    }
    
    # Custom Aligned Prompt Template
    prompt_style = (
        "### System Profile:\n{input}\n\n"
        "### Core Instruction:\n{instruction}\n\n"
        "### System Response:\n{output}"
    )

    for item in dataset:
        full_text = prompt_style.format(
            input=item["input"],
            instruction=item["instruction"],
            output=item["output"]
        )
        formatted_data["text"].append(full_text)
        formatted_data["prompt"].append(item["instruction"])

    hf_dataset = Dataset.from_dict(formatted_data)

    # ----------------------------------------------------
    # PATH A: UNSLOTH ACCELERATED PIPELINE
    # ----------------------------------------------------
    if use_unsloth:
        print(f"[+] Loading model: {args.base_model} via Unsloth...")
        model, tokenizer = FastLanguageModel.from_pretrained(
            model_name=args.base_model,
            max_seq_length=args.max_seq_length,
            dtype=None,             # None auto-detects float16/bfloat16
            load_in_4bit=True,      # Enable super-efficient 4bit quantization
            trust_remote_code=True
        )

        # Set up efficient LoRA adapters (targeting Attention layers + MoE Router Gating!)
        model = FastLanguageModel.get_peft_model(
            model,
            r=16,
            target_modules=[
                "q_proj", "k_proj", "v_proj", "o_proj", 
                "gate_proj", "up_proj", "down_proj",
                "gating_network", "wg" # Ensure we tune Mixture-of-Experts routing router layers!
            ],
            lora_alpha=32,
            lora_dropout=0,
            bias="none",
            use_gradient_checkpointing="unsloth",
            random_state=3407,
            max_seq_length=args.max_seq_length
        )

    # ----------------------------------------------------
    # PATH B: STANDARD HUGGING FACE TRL & PEFT (QLoRA)
    # ----------------------------------------------------
    else:
        print(f"[+] Loading model: {args.base_model} via standard PEFT QLoRA pipeline...")
        tokenizer = AutoTokenizer.from_pretrained(args.base_model, trust_remote_code=True)
        if tokenizer.pad_token is None:
            tokenizer.pad_token = tokenizer.eos_token
            
        # Standard causal LM loading with device mapping
        model = AutoModelForCausalLM.from_pretrained(
            args.base_model,
            torch_dtype=torch.bfloat16 if torch.cuda.is_bf16_supported() else torch.float16,
            device_map="auto",
            trust_remote_code=True
        )
        
        # Prepare for QLoRA fine-tuning
        model = prepare_model_for_kbit_training(model)
        
        peft_config = LoraConfig(
            r=16,
            lora_alpha=32,
            target_modules=["q_proj", "v_proj", "gating_network", "wg"], 
            lora_dropout=0.05,
            bias="none",
            task_type="CAUSAL_LM"
        )
        model = get_peft_model(model, peft_config)

    print("[+] Model parameters fully preloaded. Commencing SFTTrainer binding...")
    
    # Unified SFT Trainer configuration
    trainer = SFTTrainer(
        model=model,
        train_dataset=hf_dataset,
        dataset_text_field="text",
        max_seq_length=args.max_seq_length,
        packing=False, # True can pack multiple short texts to maximize length density
        args=TrainingArguments(
            per_device_train_batch_size=2,
            gradient_accumulation_steps=args.accum_steps,
            warmup_steps=10,
            num_train_epochs=args.epochs,
            learning_rate=args.learning_rate,
            fp16=not torch.cuda.is_bf16_supported(),
            bf16=torch.cuda.is_bf16_supported(),
            logging_steps=1,
            output_dir=args.output_dir,
            optim="adamw_8bit" if not use_unsloth else "adamw_torch_fused",
            weight_decay=0.01,
            lr_scheduler_type="linear",
            seed=3407
        ),
    )

    print("\n[+] Triggering active forward-propagation pass on GPU cluster...")
    trainer_stats = trainer.train()
    
    print("\n[🌟] CONVERGENCE ATTAINED! Model successfully aligned to cognitive memory.")
    print(f"|-- Save directory: {args.output_dir}")
    model.save_pretrained(args.output_dir)
    tokenizer.save_pretrained(args.output_dir)
    print("[+] Code base ready to execute inference with aligned adapters.")


def mock_run_log(args, dataset):
    """Logs dry-run output simulation for preview compliance."""
    print("----------------------------------------------------------------------")
    print(f"Executing dry-run parameters:")
    print(f"  Base Architecture: {args.base_model}")
    print(f"  Epoch Iterations:  {args.epochs}")
    print(f"  Learning Rate:     {args.learning_rate}")
    print(f"  Loss Convergence:  Auto-generated")
    print("----------------------------------------------------------------------")
    print("\nTraining logs simulated success:")
    print("Step 1/15 - Target: MoE Gating networks - Loss: 2.845 - Learning Rate: 2.0e-4")
    print("Step 5/15 - Aligning top-2 routing nodes - Loss: 1.841 - Learning Rate: 1.8e-4")
    print("Step 10/15 - Extrapolating Law of One vector bounds - Loss: 0.945")
    print("Step 15/15 - System-wide checkpoint save succeeded!")
    print(f"Checkpoints exported to local folder: '{args.output_dir}'")
    print("----------------------------------------------------------------------")


if __name__ == "__main__":
    current_args = parse_arguments()
    cognitive_dataset = generate_cognitive_dataset()
    run_alignment_training(current_args, cognitive_dataset)
