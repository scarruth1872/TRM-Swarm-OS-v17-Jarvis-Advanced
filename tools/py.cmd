@echo off
rem Run a system Python with the caller's PYTHONPATH stripped.
rem
rem Why: execute_code (the Hermes tool kernel) runs Python 3.14 and exports a PYTHONPATH pointing at
rem its own venv. A child py3.11/py3.12 process inherits that path, finds numpy 2.5.4 built for
rem cpython-3.14, and dies with "No module named 'numpy._core._multiarray_umath'". The interpreters
rem themselves are fine - the inherited path is the fault.
rem
rem   tools\py.cmd 312 -c "import torch; print(torch.__version__)"
rem   tools\py.cmd 311 script.py arg1
rem
rem Also works with no version: tools\py.cmd -c "...", which defaults to 312 (torch + rdkit).
setlocal
set "PYTHONPATH="
set "PYTHONHOME="
rem Dispatch on %~1 (safe: it is a bare token), strip it from %* by substring. `shift` cannot be used
rem because it does not modify %*. %ALL% must never appear inside an `if ""==""` test: if the args
rem themselves contain quotes the comparison becomes unbalanced and cmd aborts.
set "TAG=%~1"
set "ALL=%*"
set "REST=%ALL%"
set "EXE=C:\Users\shawn\AppData\Local\Programs\Python\Python312\python.exe"
if "%TAG%"=="311" set "EXE=C:\Users\shawn\AppData\Local\Programs\Python\Python311\python.exe"
if "%TAG%"=="313" set "EXE=C:\Users\shawn\AppData\Local\Programs\Python\Python313\python.exe"
if "%TAG%"=="311" set "REST=%ALL:~4%"
if "%TAG%"=="312" set "REST=%ALL:~4%"
if "%TAG%"=="313" set "REST=%ALL:~4%"
goto :run

:run
if not exist "%EXE%" (
  echo py.cmd: interpreter not found: %EXE% 1>&2
  exit /b 127
)
"%EXE%" %REST%
exit /b %ERRORLEVEL%
