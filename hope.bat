@echo off
title HOPE-OS ORCHESTRATOR
set CARGO_PATH=C:\Users\mater\.cargo\bin\cargo.exe
set PIPELINE_DIR=E:\Live online coder\Live code
set HOPE_HTML=d:\Live Coder - The Replicator\hope.html

echo [HOPE] Awakening the Collective...

:: Start the Rust Agent in background
start "HOPE-AGENT" /min "%CARGO_PATH%" run -- --agent

:: Start the Node.js Pipeline Server as a Daemon (Hidden)
echo [HOPE] Launching Pipeline Daemon...
echo Set WshShell = CreateObject("WScript.Shell") > "%temp%\run_pipeline.vbs"
echo WshShell.Run "cmd /c cd /d ""%PIPELINE_DIR%"" && node server.js", 0 >> "%temp%\run_pipeline.vbs"
wscript.exe "%temp%\run_pipeline.vbs"
del "%temp%\run_pipeline.vbs"

echo [HOPE] Stabilizing Systems (3s)...
timeout /t 3 /nobreak > nul

:: Open the Visual VM in Browser
echo [HOPE] Activating Visual HUD...
start "" "%HOPE_HTML%"

:: Finally start the TUI for direct control
echo [HOPE] Linking TUI Interface...
"%CARGO_PATH%" run -- --tui
