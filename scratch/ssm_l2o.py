
import json, collections, itertools, statistics
import numpy as np
BASE=r"F:/Development sites/TRM-Swarm-OS-v2"
d=json.load(open(BASE+"/scratch/council_ledger_snapshot.json",encoding="utf-8"))
rows=[dict(topic=p["topic"],pid=p["id"],**v) for p in d for v in p.get("votes",[]) if "ssm_trit" in v]
cells=[]
for k,g in itertools.groupby(sorted(rows,key=lambda r:(r["topic"],r["agent"])), key=lambda r:(r["topic"],r["agent"])):
    g=list(g); cells.append(dict(topic=k[0],agent=k[1],
        ssm_mean=statistics.mean(float(x["ssm_mean"]) for x in g),
        confidence=statistics.mean(float(x["confidence"]) for x in g),
        weight=statistics.mean(float(x["weight"]) for x in g),
        y=int(collections.Counter(int(x["llm_trit"]) for x in g).most_common(1)[0][0])))
def onehot(y):
    M=np.zeros((len(y),3)); idx={-1:0,0:1,1:2}
    for i,v in enumerate(y): M[i,idx[int(v)]]=1
    return M
def ridge_fit(X,Y,lam=10.0):
    mu=X.mean(0); sd=X.std(0); sd[sd==0]=1.0
    Z=(X-mu)/sd; Z=np.hstack([Z,np.ones((len(Z),1))])
    A=Z.T@Z+lam*np.eye(Z.shape[1]); A[-1,-1]-=lam
    return mu,sd,np.linalg.solve(A,Z.T@Y)
def pred(m,X):
    mu,sd,W=m; Z=(X-mu)/sd; Z=np.hstack([Z,np.ones((len(Z),1))]); s=Z@W
    return np.array([[-1,0,1][i] for i in s.argmax(1)])
C=cells; tops=sorted(set(c["topic"] for c in C))
def run(feats,holdout):
    X=np.array([[c[f] for f in feats] for c in C]); y=np.array([c["y"] for c in C])
    correct=[]; base=[]
    for hs in holdout:
        te=np.array([c["topic"] in hs for c in C]); tr=~te
        p=pred(ridge_fit(X[tr],onehot(y[tr])),X[te])
        correct+=list(p==y[te])
        cc=collections.Counter(y[tr]); base+=[cc.most_common(1)[0][0]]*int(te.sum())
    return float(np.mean(correct)),float(np.mean(np.array(correct)==np.array(correct)))if False else float(np.mean(np.array(base)==np.array([c["y"] for c in C])[np.array([c["topic"] in set().union(*holdout) for c in C])]) if False else 0), float(np.mean(correct))
F2=[tuple(x) for x in itertools.combinations(tops,2)]
print("leave-two-topics-out, %d folds"%len(F2))
for name,f in [("ssm_mean",["ssm_mean"]),("confidence",["confidence"]),("ssm+conf+weight",["ssm_mean","confidence","weight"]),("ssm+conf",["ssm_mean","confidence"])]:
    X=np.array([[c[f_] for f_ in f] for c in C]); y=np.array([c["y"] for c in C]); accs=[];bs=[]
    for hs in F2:
        te=np.array([c["topic"] in hs for c in C]); tr=~te
        p=pred(ridge_fit(X[tr],onehot(y[tr])),X[te]); accs.append(np.mean(p==y[te]))
        cc=collections.Counter(y[tr]); bs.append(np.mean(y[te]==cc.most_common(1)[0][0]))
    print("  %-16s mean held-out %.4f  (min %.3f max %.3f)  train-majority baseline %.4f"%(name,np.mean(accs),min(accs),max(accs),np.mean(bs)))
print("  %-16s %.4f"%("global-majority",collections.Counter(c["y"] for c in C).most_common(1)[0][1]/len(C)))
