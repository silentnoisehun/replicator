@echo off
title HOPE-OS — The Replicator
cd /d "%~dp0"

echo.
echo  ╔══════════════════════════════════════════╗
echo  ║     HOPE-OS  —  The Replicator  v0.1    ║
echo  ╚══════════════════════════════════════════╝
echo.

:: --- Cargo keresés ---
set CARGO=cargo
where cargo >nul 2>&1
if errorlevel 1 (
    if exist "%USERPROFILE%\.cargo\bin\cargo.exe" (
        set CARGO=%USERPROFILE%\.cargo\bin\cargo.exe
    ) else (
        echo [ERROR] cargo nem talalhato. Telepitsd a Rust-ot: https://rustup.rs
        pause
        exit /b 1
    )
)

:: --- .env betöltés ha van ---
if exist ".env" (
    echo [HOPE] .env betoltve
    for /f "usebackq tokens=1,* delims==" %%A in (".env") do (
        if not "%%A"=="" if not "%%A:~0,1%"=="#" set %%A=%%B
    )
)

:: --- Build ---
echo [HOPE] Build folyamatban...
%CARGO% build --release 2>&1
if errorlevel 1 (
    echo [ERROR] Build sikertelen!
    pause
    exit /b 1
)
echo [HOPE] Build OK
echo.

:: --- Agent indítás háttérben ---
echo [HOPE] Agent indul (hatterben)...
start "HOPE-AGENT" /min %CARGO% run --release -- --agent

timeout /t 2 /nobreak >nul

:: --- Vizuális HUD megnyitása ---
if exist "hope.html" (
    echo [HOPE] Visual HUD megnyitva...
    start "" "%~dp0hope.html"
)

:: --- TUI foreground ---
echo [HOPE] TUI indul...
echo.
%CARGO% run --release -- --tui
