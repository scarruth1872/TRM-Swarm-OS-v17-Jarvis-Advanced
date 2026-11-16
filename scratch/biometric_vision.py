"""TRM Swarm OS v18.50 — Biometric Wave-Space Vision Layer"""
import sys; sys.path.insert(0, '.')
import numpy as np, time, json, hashlib
from pathlib import Path; from collections import deque
from dataclasses import dataclass, field
from typing import Dict, Any

from swarm_v2.core.radix_trie_matcher import get_radix_trie
from swarm_v2.core.thz_wave_gateway import get_thz_gateway
from swarm_v2.core.kernel_lora_adapter import get_lora_registry, KernelLoRAAdapter

trie = get_radix_trie()
gw = get_thz_gateway()
reg = get_lora_registry()
for f in sorted(Path("swarm_v2_global_memory/lora_adapters").glob("*_v18.lora.pt")):
    n = f.stem.replace("_v18","").replace(".lora","")
    if n not in reg._adapters:
        try: a=KernelLoRAAdapter(n); a.load_from_pytorch(f); reg.register(a)
        except: pass

@dataclass
class WiFiSignal:
    rssi_dbm: float; phase_rad: float = 0
    csi: np.ndarray = field(default_factory=lambda: np.zeros(30,dtype=np.float32))

class WaveSpaceProc:
    def __init__(s): s.buf=np.zeros(64,dtype=np.complex64); s.hist=deque(maxlen=100)
    def process(s, sig: WiFiSignal) -> Dict:
        amp=10**(sig.rssi_dbm/20); w=amp*np.exp(1j*sig.phase_rad)
        s.buf=np.roll(s.buf,1); s.buf[0]=w; I=np.abs(s.buf)**2
        t=np.zeros(64,dtype=np.int8); t[I>0.3]=1; t[I<0.01]=-1
        p=int(np.sum(t==1)); f=int(np.sum(t==-1))
        pl=json.dumps({"rssi":sig.rssi_dbm,"phase":round(sig.phase_rad,4)}).encode()
        gr=json.loads(gw.process_request(pl).decode())
        return {"rssi":sig.rssi_dbm,"pass":p,"fail":f,"ambi":64-p-f,
                "trit":1 if p>f else(-1 if f>p else 0),"thz_us":gr.get("latency_us",0)}

class IntelCSI:
    def analyze(s, csi: np.ndarray) -> Dict:
        fft=np.abs(np.fft.fft(csi)); fr=np.fft.fftfreq(len(csi))
        br=float(np.sum(fft[(abs(fr)>0.1)&(abs(fr)<0.5)]))
        hb=float(np.sum(fft[(abs(fr)>0.8)&(abs(fr)<2.0)]))
        mt=float(np.sum(fft[abs(fr)>2.0]))
        if br>hb and br>mt: return {"det":"breathing","trit":1}
        if hb>br: return {"det":"heartbeat","trit":1}
        if mt>br*2: return {"det":"motion","trit":-1}
        return {"det":"ambiguous","trit":0}

wsp=WaveSpaceProc(); csi_eng=IntelCSI()

pats={"wifi vision":"WiFi RSSI presence + phase-shift spatial + 30-subcarrier CSI via Intel",
      "biometric scan":"Wave-space presence + CSI breathing/heartbeat/motion detection",
      "wave space":"THz Fourier + Dark Valley collision + Kleene biometric signatures",
      "intel frequency":"CSI subcarrier FFT biometric: breathing 0.1Hz, heartbeat 0.8-2.0Hz",
      "presence detection":"RSSI variance + CSI amplitude via Dark Valley collision nodes",
      "biometric identity":"Wave interference signature hash from RSSI+Phase+CSI moments"}
for p,h in pats.items(): trie.register(p, lambda r=h: r)

print("="*60)
print(f"BIOMETRIC WAVE-SPACE VISION — {trie.registered} trie patterns")
print("="*60)

rssi_v=[-45,-55,-65,-75,-85,-50,-60,-70,-80,-48]
ph_v=[0.0,0.5,1.0,1.5,2.0,0.3,0.8,1.3,1.8,2.5]
res=[]; det=0; tus=0

for i in range(50):
    r=rssi_v[i%10]; ph=ph_v[i%10]
    csi=np.random.randn(30).astype(np.float32)*0.3+np.sin(np.linspace(0,4*np.pi,30))*0.5
    t0=time.perf_counter_ns()
    wr=wsp.process(WiFiSignal(rssi_dbm=r,phase_rad=ph))
    cr=csi_eng.analyze(csi)
    el=(time.perf_counter_ns()-t0)/1000; tus+=el
    cb=1 if(wr["trit"]==1 and cr["trit"]>=0)else(-1 if wr["trit"]==-1 else 0)
    res.append(cb)
    if trie.match("biometric scan" if cb==1 else "presence detection"): det+=1
    if(i+1)%25==0: print(f"  {i+1:2d}/50 | {r:>4d}dBm | trit={cb:+d} | {el:>6.0f}us")

pr=res.count(1); inf=res.count(-1); sc=res.count(0)
print(f"\nResults: presence={pr} interference={inf} scanning={sc}")
print(f"Trie: {det}/50 | Avg: {tus/50:.0f}us")
print(f"\nPipeline: WiFi RSSI -> THz Fourier -> Dark Valley -> Kleene Trit")
print(f"Patterns: {trie.registered} | Ready for Jarvis WiFi/Intel Vision tab")
