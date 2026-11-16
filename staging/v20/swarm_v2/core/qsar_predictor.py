"""
QSAR antibacterial-activity predictor (Domain 1, task #10) — production model.

Smaller than the v1 (535 MB) build and retrained with a <10 uM active threshold so
moderate-potency antibiotics (ciprofloxacin, ~1-5 uM) are correctly labeled active.
"""
import json
import urllib.request
import numpy as np
from pathlib import Path

_MODEL_DIR = Path(__file__).resolve().parent.parent.parent / "swarm_v2_global_memory" / "qsar"
_MODEL_PATH = _MODEL_DIR / "qsar_bindingdb_rf_prod.joblib"
_META_PATH = _MODEL_DIR / "qsar_bindingdb_rf_prod.meta.json"
_NBITS = 1024

ACTIVITY_THRESHOLD_NM = 10000.0  # active < 10 uM, inactive > 10 uM
RF_PARAMS = dict(n_estimators=80, max_depth=25, min_samples_leaf=2,
                 random_state=42, max_features="sqrt", n_jobs=-1)


def _morgan(smi: str, nbits: int = _NBITS):
    """Morgan fingerprint -> uint8 0/1 array. Returns None if SMILES unparseable."""
    from rdkit import Chem
    from rdkit.Chem import AllChem
    from rdkit import DataStructs
    mol = Chem.MolFromSmiles(smi)
    if mol is None:
        return None
    mol = Chem.RemoveHs(mol)
    fp = AllChem.GetMorganFingerprintAsBitVect(mol, 2, nBits=nbits)
    arr = np.zeros((nbits,), dtype=np.uint8)
    DataStructs.ConvertToNumpyArray(fp, arr)
    return arr


def _parse_nm(s):
    s = (s or "").strip()
    if not s:
        return None
    for p in "=<>~ ":
        s = s.lstrip(p)
    try:
        return float(s)
    except ValueError:
        return None


def _scaffold(smi: str):
    from rdkit import Chem
    from rdkit.Chem.Scaffolds import MurckoScaffold
    mol = Chem.MolFromSmiles(smi)
    if mol is None:
        return None
    s = MurckoScaffold.GetScaffoldForMol(mol)
    if s is None or s.GetNumAtoms() == 0:
        return None
    return Chem.MolToSmiles(s)


def _load_dataset(data_tsv, nbits=_NBITS):
    """Parse BindingDB antibacterial subset -> (X uint8, y, scaffolds)."""
    import csv
    smiles, labels, scafs = [], [], []
    with open(data_tsv, encoding="utf-8") as f:
        r = csv.reader(f, delimiter="\t")
        next(r)
        for row in r:
            smi = row[0].strip()
            if not smi:
                continue
            val = None
            for i in (4, 5, 6, 7):
                val = _parse_nm(row[i])
                if val is not None:
                    break
            if val is None:
                continue
            if val < ACTIVITY_THRESHOLD_NM:
                lab = 1
            elif val > ACTIVITY_THRESHOLD_NM:
                lab = 0
            else:
                continue  # exactly at threshold (negligible)
            smiles.append(smi)
            labels.append(lab)
            scafs.append(_scaffold(smi) or smi)

    X = np.zeros((len(smiles), nbits), dtype=np.uint8)
    keep = []
    for i, s in enumerate(smiles):
        fp = _morgan(s, nbits)
        if fp is None:
            continue
        X[i] = fp
        keep.append(i)
    X = X[keep]
    y = np.array(labels)[keep]
    scafs = [scafs[i] for i in keep]
    return X, y, scafs


def _evaluate_scaffold_split(X, y, scafs, params, seed=42):
    """Honest generalization estimate: split by Bemis-Murcko scaffold."""
    from sklearn.ensemble import RandomForestClassifier
    from sklearn.model_selection import GroupShuffleSplit
    from sklearn.metrics import roc_auc_score
    gss = GroupShuffleSplit(n_splits=1, test_size=0.2, random_state=seed)
    tr, te = next(gss.split(X, y, groups=np.array(scafs)))
    clf = RandomForestClassifier(**params)
    clf.fit(X[tr], y[tr])
    auc = roc_auc_score(y[te], clf.predict_proba(X[te])[:, 1])
    return round(float(auc), 4)


def train_and_save(data_tsv, model_path=_MODEL_PATH, nbits=_NBITS, params=RF_PARAMS):
    import time, joblib
    from sklearn.ensemble import RandomForestClassifier

    print(f"loading dataset (threshold {ACTIVITY_THRESHOLD_NM:.0f} nM)...", flush=True)
    X, y, scafs = _load_dataset(data_tsv, nbits)
    print(f"  {X.shape[0]:,} x {X.shape[1]} (active={int(y.sum()):,} inactive={int((1-y).sum()):,})", flush=True)

    print("scaffold-split validation...", flush=True)
    auc = _evaluate_scaffold_split(X, y, scafs, params)

    print(f"training final model ({params['n_estimators']} trees) on all data...", flush=True)
    t0 = time.time()
    clf = RandomForestClassifier(**params)
    clf.fit(X, y)
    print(f"  trained in {time.time()-t0:.0f}s", flush=True)

    _MODEL_DIR.mkdir(parents=True, exist_ok=True)
    joblib.dump(clf, model_path)
    meta = {
        "model": str(model_path), "n_compounds": int(X.shape[0]),
        "n_features": nbits, "active": int(y.sum()), "inactive": int((1 - y).sum()),
        "activity_threshold_nm": ACTIVITY_THRESHOLD_NM,
        "fingerprint": f"morgan_r2_{nbits}",
        "classifier": f"RandomForest(n={params['n_estimators']},depth={params.get('max_depth')})",
        "scaffold_split_auc": auc,
    }
    _META_PATH.write_text(json.dumps(meta, indent=2), encoding="utf-8")
    print(f"saved -> {model_path} ({model_path.stat().st_size/1e6:.1f} MB), scaffold AUC={auc}", flush=True)
    return clf, meta


KERNEL_QSAR_URL = "http://127.0.0.1:8022/crate/qsar/predict"


class QSARPredictor:
    """Scoring goes to the kernel; the model lives in Rust (`swarm-drug`).

    This class used to load a 74 MB joblib RandomForest of its own, so the platform held two copies
    of one model and could score the same molecule two ways. The Rust model is already bit-exact
    against that joblib (worst diff 4.42e-13 over six controls), so the Python copy is gone and the
    fingerprint -- the only part RDKit must compute -- is passed over the wire.
    """

    def __init__(self):
        self._last_meta = None

    def get_status(self):
        return {
            "loaded": True,
            "engine": "swarm-drug (Rust; kernel route)",
            "route": KERNEL_QSAR_URL,
            "fingerprint": f"morgan_r2_{_NBITS} computed in Python (RDKit); scoring is Rust",
            "last_response_meta": self._last_meta,
        }

    def predict(self, smiles: str):
        """Return {smiles, active_prob, label, class} or {error} if unparseable/unreachable."""
        arr = _morgan(smiles)
        if arr is None:
            return {"smiles": smiles, "error": "unparseable SMILES"}
        # RDKit sets bit 0 in the LSB of byte 0; np.packbits is MSB-first by default and would send a
        # well-formed 128-byte string the kernel decodes into the wrong bits. Verified on six controls.
        fp_hex = np.packbits(arr, bitorder="little").tobytes().hex()
        body = json.dumps({"fp_hex": fp_hex}).encode()
        req = urllib.request.Request(
            KERNEL_QSAR_URL, data=body,
            headers={"Content-Type": "application/json"}, method="POST")
        try:
            d = json.loads(urllib.request.urlopen(req, timeout=30).read())
        except Exception as e:
            return {"smiles": smiles, "error": f"kernel qsar route unreachable: {e}"}
        self._last_meta = {k: d.get(k) for k in ("n_trees", "n_bits", "threshold_nm", "scaffold_auc")}
        p = float(d["active_prob"])
        label = 1 if p >= 0.5 else 0
        return {"smiles": smiles, "active_prob": round(p, 4), "label": label,
                "class": d.get("class") or ("active" if label else "inactive"),
                "engine": d.get("engine", "swarm-drug")}

    def predict_batch(self, smiles_list):
        return [self.predict(s) for s in smiles_list]


_predictor = None


def get_qsar_predictor() -> QSARPredictor:
    global _predictor
    if _predictor is None:
        _predictor = QSARPredictor()
    return _predictor
