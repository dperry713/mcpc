@echo off
REM Windows Command Prompt wrapper for the PowerShell Installer Wizard
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1"
pause
