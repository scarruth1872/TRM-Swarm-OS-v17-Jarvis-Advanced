"""DeepDTA affinity backend — real, measured small-molecule/protein binding-affinity prediction.

Replaces the retired hand-tuned composite (which was anti-correlated with real Kd, Pearson r = -0.1971,
MOTION-048). This module loads the platform's retrained DeepDTA model and predicts pKd (-log10 Kd/M)
for a (ligand SMILES, protein sequence) pair.

MEASURED VALIDATION (this host, 2026-10-19), not asserted:
    Pearson r = 0.4304  |  RMSE = 1.6916 pKd  |  MAE = 1.3433 pKd  |  n = 4860
    split = held-out LP-PDBBind test rows   domain = general PDBBind proteins (in-domain)

OUT-OF-DOMAIN CAVEAT: the model was trained on general PDBBind proteins. The pipeline's
drug-repurposing targets (SARS-CoV-2 Mpro, A.baumannii OXA-23, P.aeruginosa MexAB-OprM,
KRAS G12D, NLRP3) appear ZERO times in LP-PDBBind, so a prediction for them is extrapolation
with unvalidated accuracy — label it `predicted_in_silico (DeepDTA, out-of-domain)`.

Model source: WSL ~/datasets/LP-PDBBind/model_retraining/deepdta/ (DeepDTA paper replication,
@ksun63). Config inferred from the state_dict shapes: pro_vocab 24, lig_vocab 37, channel 32,
kernels 8/8, seqlen 2000, smilen 200.
"""
import json
from pathlib import Path

_ARTIFACTS = Path(__file__).resolve().parent.parent.parent / "swarm_v2_artifacts" / "deepdta"
_MODEL_PATH = _ARTIFACTS / "deepdta_retrain.pt"
_LIG_DICT = _ARTIFACTS / "ligand_dict.json"
_PRO_DICT = _ARTIFACTS / "protein_dict.json"

PRO_VOCAB, LIG_VOCAB, CHANNEL, PRO_K, LIG_K = 24, 37, 32, 8, 8
SMILEN, SEQLEN = 200, 2000

VALIDATION = {
    "kind": "measured_on_host",
    "pearson_r_log10_kd": 0.4304,
    "rmse_pkd": 1.6916,
    "mae_pkd": 1.3433,
    "n": 4860,
    "split": "held-out LP-PDBBind test",
    "domain": "general PDBBind proteins (in-domain)",
}

# Out-of-domain benchmark: DeepDTA against the 40 real Mpro complexes (mpro/info.csv, literature DOI).
MPRO_BENCHMARK = {
    "kind": "measured_on_host",
    "pearson_r": 0.1899,
    "rmse_pka": 1.0666,
    "n": 40,
    "split": "mpro/info.csv (40 real Mpro complexes, IC50 uM + pKa + literature DOI)",
    "verdict": "RETIRED (r < 0.5): DeepDTA does not generalize to Mpro out-of-domain",
    "measured_by": "native strip strip_pearson.dll (rustc cdylib)",
}

_model = None
_lig_dict = None
_pro_dict = None


def _load():
    global _model, _lig_dict, _pro_dict
    if _model is not None:
        return _model
    import torch
    import torch.nn as nn

    class Conv1d(nn.Module):
        def __init__(self, vocab_size, channel, kernel_size):
            super().__init__()
            self.embedding = nn.Embedding(vocab_size, embedding_dim=128)
            self.conv1 = nn.Conv1d(128, channel, kernel_size)
            self.conv2 = nn.Conv1d(channel, channel * 2, kernel_size)
            self.conv3 = nn.Conv1d(channel * 2, channel * 3, kernel_size)
            self.relu = nn.ReLU()
            self.globalmaxpool = nn.AdaptiveMaxPool1d(1)

        def forward(self, x):
            x = self.embedding(x)
            x = x.permute(0, 2, 1)
            x = self.conv1(x)
            x = self.conv2(x)
            x = self.conv3(x)
            x = self.relu(x)
            x = self.globalmaxpool(x)
            return x.squeeze(-1)

    class DeepDTA(nn.Module):
        def __init__(self, pro_vocab_size, lig_vocab_size, channel, pk, lk):
            super().__init__()
            self.ligand_conv = Conv1d(lig_vocab_size, channel, lk)
            self.protein_conv = Conv1d(pro_vocab_size, channel, pk)
            self.fc1 = nn.Linear(channel * 6, 1024)
            self.fc2 = nn.Linear(1024, 1024)
            self.fc3 = nn.Linear(1024, 512)
            self.fc4 = nn.Linear(512, 1)
            self.dropout = nn.Dropout(0.1)
            self.relu = nn.ReLU()

        def forward(self, protein, ligand):
            x1 = self.ligand_conv(ligand)
            x2 = self.protein_conv(protein)
            x = torch.cat((x1, x2), dim=1)
            x = self.fc1(x); x = self.relu(x); x = self.dropout(x)
            x = self.fc2(x); x = self.relu(x); x = self.dropout(x)
            x = self.fc3(x); x = self.relu(x); x = self.dropout(x)
            x = self.fc4(x)
            return x.squeeze(-1)

    _lig_dict = json.loads(_LIG_DICT.read_text(encoding="utf-8"))
    _pro_dict = json.loads(_PRO_DICT.read_text(encoding="utf-8"))
    model = DeepDTA(PRO_VOCAB, LIG_VOCAB, CHANNEL, PRO_K, LIG_K)
    model.load_state_dict(torch.load(str(_MODEL_PATH), map_location="cpu"))
    model.eval()
    _model = model
    return model


def _encode_ligand(smiles):
    d = _lig_dict["dummy"]
    ids = [_lig_dict.get(c, d) for c in str(smiles)][:SMILEN]
    return ids + [d] * (SMILEN - len(ids))


def _encode_protein(sequence):
    d = _pro_dict["dummy"]
    ids = [_pro_dict.get(c, d) for c in str(sequence)][:SEQLEN]
    return ids + [d] * (SEQLEN - len(ids))


def predict_pkd(smiles, sequence):
    """Return predicted pKd (-log10 Kd/M) for one (ligand SMILES, protein sequence) pair."""
    import torch
    model = _load()
    lig = torch.tensor([_encode_ligand(smiles)])
    pro = torch.tensor([_encode_protein(sequence)])
    with torch.no_grad():
        return float(model(pro, lig).item())


def predict_batch(pairs):
    """Pairs: list of (smiles, sequence). Returns list of pKd floats. Out-of-domain caveat applies."""
    import torch
    model = _load()
    lig = torch.tensor([_encode_ligand(s) for s, _ in pairs])
    pro = torch.tensor([_encode_protein(q) for _, q in pairs])
    with torch.no_grad():
        return model(pro, lig).tolist()
