"""Run Jarvis knowledge fine-tuning on Gemma 270M via DirectML."""
import sys
sys.path.insert(0, r"F:\Development sites\TRM-Swarm-OS-v2")

# Patch the config BEFORE importing the training script
import finetune_gemma
finetune_gemma.DATASET_PATH = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\jarvis_train_dataset\jarvis_knowledge_chat.json"
finetune_gemma.OUTPUT_DIR = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\jarvis_gemma_adapter"
finetune_gemma.NUM_EPOCHS = 5

finetune_gemma.main()
