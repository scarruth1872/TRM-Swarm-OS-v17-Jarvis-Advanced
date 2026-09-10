"""Accurate inventory of resident-bridge processes + the one holding UDP 8022."""
import subprocess, re

def sh(args):
    return subprocess.run(args, capture_output=True, text=True).stdout

# server-side filter -> one PID per line, no CSV field-splitting problems
out = sh(['wmic', 'process', 'where',
          "CommandLine like '%run_resident_bridge%'", 'get', 'ProcessId'])
pids = sorted({int(x) for x in re.findall(r'\d+', out)})

# who owns UDP 8022
hold = []
for line in sh(['netstat', '-ano']).splitlines():
    if 'UDP' in line and ':8022' in line:
        hold.append(line.strip().split()[-1])

print('run_resident_bridge processes :', pids)
print('UDP 8022 owner pid            :', hold)
names = {p: sh(['tasklist', '/FI', f'PID eq {p}', '/FO', 'CSV', '/NH']).strip().split(',')[0].strip('"')
         for p in pids + hold}
for p, n in names.items():
    print(f'  pid {p:>7}  {n}')
