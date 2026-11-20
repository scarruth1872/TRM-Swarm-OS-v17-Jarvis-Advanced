#!/usr/bin/env bash
# Run a system Python with the caller's PYTHONPATH stripped.
#
# Why: execute_code (the Hermes tool kernel) runs Python 3.14 and exports a PYTHONPATH pointing at its
# own venv. A child py3.11/py3.12 process inherits it, finds numpy 2.5.4 built for cpython-3.14, and
# dies with "No module named 'numpy._core._multiarray_umath'". The interpreters are fine; the inherited
# path is the fault.
#
#   tools/py.sh 312 -c "import torch; print(torch.__version__)"
#   tools/py.sh 311 script.py arg1        # or: tools/py.sh -c "..."  (defaults to 312)
set -u

unset PYTHONPATH PYTHONHOME

TAG="${1:-}"
case "$TAG" in
  311) EXE="C:/Users/shawn/AppData/Local/Programs/Python/Python311/python.exe"; shift ;;
  312) EXE="C:/Users/shawn/AppData/Local/Programs/Python/Python312/python.exe"; shift ;;
  313) EXE="C:/Users/shawn/AppData/Local/Programs/Python/Python313/python.exe"; shift ;;
  *)   EXE="C:/Users/shawn/AppData/Local/Programs/Python/Python312/python.exe" ;;
esac

if [ ! -f "$EXE" ]; then
  echo "py.sh: interpreter not found: $EXE" >&2
  exit 127
fi

exec "$EXE" "$@"
