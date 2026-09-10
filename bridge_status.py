"""Resident-bridge inventory + cleanup (UDP 8022).

  python bridge_status.py          -> show who is alive and who owns 8022
  python bridge_status.py --kill   -> kill every bridge python.exe, wait for
                                      the port to free (bash wrappers are inert
                                      and deliberately left alone: killing the
                                      parent risks the live child)

Why this exists: the bridge does a plain bind() (no SO_REUSEADDR), so a stale
instance holding 8022 makes a patched relaunch exit 1 with WinError 10048 while
the OLD code keeps serving. Always --kill and confirm the port is FREE before
launching exactly one instance.
"""
import subprocess
import re
import sys
import time


def sh(args):
    return subprocess.run(args, capture_output=True, text=True).stdout


def bridge_pids():
    """Server-side wmic filter -> avoids CSV splitting on command lines with commas."""
    out = sh(['wmic', 'process', 'where',
              "CommandLine like '%run_resident_bridge%'", 'get', 'ProcessId'])
    return sorted({int(x) for x in re.findall(r'\d+', out)})


def port_owner():
    return [l.strip().split()[-1] for l in sh(['netstat', '-ano']).splitlines()
            if 'UDP' in l and ':8022' in l]


def proc_name(pid):
    row = sh(['tasklist', '/FI', f'PID eq {pid}', '/FO', 'CSV', '/NH']).strip()
    return row.split(',')[0].strip('"') if row else ''


if __name__ == '__main__':
    if '--kill' in sys.argv:
        for _ in range(8):
            pythons = [p for p in bridge_pids() if proc_name(p) == 'python.exe']
            for p in pythons:
                subprocess.run(['taskkill', '/PID', str(p), '/F'], capture_output=True)
            time.sleep(1)
            if not port_owner():
                break
        print('UDP :8022', 'FREE' if not port_owner() else f'STILL HELD by {port_owner()}')

    pids = bridge_pids()
    hold = port_owner()
    print('bridge processes :', pids)
    print('UDP 8022 owner   :', hold)
    for p in pids + hold:
        print(f'  pid {p:>7}  {proc_name(p) or "(exited)"}')
