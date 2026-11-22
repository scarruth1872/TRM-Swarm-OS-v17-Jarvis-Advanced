@echo off
REM :8593 = nomic-embed-text, served from the ollama BLOB by llama-server (no ollama daemon).
REM 768-dim, mean pooling, Vulkan offload, 4 slots. The kernel's memory/evidence paths call this
REM port; without it /memory/search degrades and says so.
REM Foreground on purpose: `start` returned before the child had a console and the task reported
REM 0x80070002 with nothing listening.
set LOCAL_BIN=F:\Development sites\llama-vulkan-b12360
set MODEL=C:\Users\shawn\.ollama\models\blobs\sha256-970aa74c0a90ef7482477cf803618e776e173c007bf957f635f1015bfcfef0e6
set LOG=F:\Development sites\TRM-Swarm-OS-v2\logs\embed_8593.out.log
if not exist "%LOCAL_BIN%\llama-server.exe" exit /b 1
if not exist "%MODEL%" exit /b 1
"%LOCAL_BIN%\llama-server.exe" -m "%MODEL%" --embeddings --pooling mean -c 2048 -b 2048 -ub 2048 --host 127.0.0.1 --port 8593 -ngl 99 --no-webui >> "%LOG%" 2>&1
exit /b %ERRORLEVEL%
