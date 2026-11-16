import json, math, random, urllib.request
E='http://127.0.0.1:8593/v1/embeddings'
D=r'F:/Development sites/TRM-Swarm-OS-v2/kaggle/combined/arc-agi_training2_challenges.json'

def emb(t):
    b=json.dumps({'input':t}).encode()
    r=urllib.request.Request(E,data=b,headers={'Content-Type':'application/json'})
    with urllib.request.urlopen(r,timeout=60) as h:
        d=json.loads(h.read())
    v=d['data'][0]['embedding']
    return v

def cos(a,b):
    s=sum(x*y for x,y in zip(a,b))
    na=math.sqrt(sum(x*x for x in a)); nb=math.sqrt(sum(y*y for y in b))
    return s/(na*nb) if na and nb else 0.0

ch=json.load(open(D))
ids=list(ch)[:120]
texts={}; sig={}
for t in ids:
    d=ch[t]; tr=d['train'][0]
    gi=tr['input']; go=tr['output']
    rows=lambda g:' '.join(''.join(str(c) for c in r) for r in g)
    texts[t]=(str(len(gi))+'x'+str(len(gi[0]))+' in '+str(len(go))+'x'+str(len(go[0]))+' out '+rows(gi)+' -> '+rows(go))
    sig[t]=(len(go),len(go[0]),tuple(sorted({c for r in go for c in r})))

V={}
for t in ids:
    try: V[t]=emb(texts[t])
    except Exception as e: V[t]=None
ok=[t for t in ids if V.get(t)]
print('embedded',len(ok),'of',len(ids),'dim',len(V[ok[0]]))

hit_shape=0; hit_cols=0
for t in ok:
    best=None; bv=-2
    for u in ok:
        if u==t: continue
        c=cos(V[t],V[u])
        if c>bv: bv=c; best=u
    a=sig[t]; b=sig[best]
    hit_shape += (a[0]==b[0] and a[1]==b[1])
    hit_cols += (a[2]==b[2])
n=len(ok)
print('top1 same output shape:',hit_shape,'/',n, round(100*hit_shape/n,1),'%')
print('top1 same colour set  :',hit_cols,'/',n, round(100*hit_cols/n,1),'%')
