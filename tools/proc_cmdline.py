"""Print the command line of a PID. Exit 1 if unavailable.

Used by START_TRM_PLATFORM.bat :verify_cmd so verification does not depend on
wmic (removed from Win11) or nested-quote PowerShell from inside a for/f loop.
"""
import sys

def main() -> int:
    if len(sys.argv) < 2:
        print("usage: proc_cmdline.py PID", file=sys.stderr)
        return 2
    try:
        import psutil
    except ImportError:
        print("psutil not installed", file=sys.stderr)
        return 3
    try:
        pid = int(sys.argv[1])
    except ValueError:
        print("PID must be an integer", file=sys.stderr)
        return 2
    try:
        parts = psutil.Process(pid).cmdline()
    except Exception as exc:                      # noqa: BLE001 - any read failure is "unavailable"
        print(f"unavailable: {exc}", file=sys.stderr)
        return 1
    if not parts:
        print("unavailable: empty cmdline (kernel thread or access denied)", file=sys.stderr)
        return 1
    print(" ".join(parts))
    return 0

if __name__ == "__main__":
    sys.exit(main())
