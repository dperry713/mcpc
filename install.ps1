# MCPC Enterprise Multi-Platform Installation Wizard (Windows PowerShell)
# Automates compiling and registering the production orchestrator

Clear-Host
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "         🔒 MCPC ENTERPRISE INSTALLATION WIZARD           " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "Target: Windows Production Release" -ForegroundColor Gray
Write-Host ""

# 1. Prerequisite Verification
function Verify-Tool ($name, $checkCommand) {
    Write-Host "Checking for $name... " -NoNewline
    try {
        $result = Invoke-Expression "$checkCommand" -ErrorAction SilentlyContinue
        if ($LASTEXITCODE -eq 0 -or $result) {
            Write-Host "Found." -ForegroundColor Green
            return $true
        }
    } catch {}
    Write-Host "NOT FOUND. Please install $name." -ForegroundColor Red
    return $false
}

$prereqs = $true
$prereqs = $prereqs -and (Verify-Tool "Rust/Cargo" "cargo --version")
$prereqs = $prereqs -and (Verify-Tool "Node.js" "node --version")
$prereqs = $prereqs -and (Verify-Tool "npm" "npm --version")
$prereqs = $prereqs -and (Verify-Tool "Docker" "docker --version")

if (-not $prereqs) {
    Write-Host "`n[Error] Prerequisites check failed. Please install missing tools and try again." -ForegroundColor Red
    Exit 1
}

# 2. Compile Rust Backend
Write-Host "`n[1/4] Compiling Rust backend in Release Mode..." -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "[Error] Rust compilation failed." -ForegroundColor Red
    Exit 1
}
Write-Host "Rust backend successfully compiled." -ForegroundColor Green

# 3. Compile GUI wrapper
Write-Host "`n[2/4] Initializing React/Tauri GUI..." -ForegroundColor Cyan
Push-Location mcpc-gui
Write-Host "Installing GUI dependencies..."
npm install
if ($LASTEXITCODE -ne 0) {
    Write-Host "[Error] GUI dependencies installation failed." -ForegroundColor Red
    Pop-Location
    Exit 1
}
Write-Host "Building GUI client..."
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "[Error] GUI compilation failed." -ForegroundColor Red
    Pop-Location
    Exit 1
}
Pop-Location
Write-Host "GUI successfully compiled." -ForegroundColor Green

# 4. Install Binary & Register Path
Write-Host "`n[3/4] Installing binary to user home directory..." -ForegroundColor Cyan
$installDir = Join-Path $HOME ".mcpc\bin"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir | Out-Null
}

$sourceBinary = "target\release\mcpc.exe"
$destBinary = Join-Path $installDir "mcpc.exe"
Copy-Item -Path $sourceBinary -Destination $destBinary -Force

Write-Host "Binary copied to $destBinary" -ForegroundColor Green

# 5. Add to PATH env
Write-Host "`n[4/4] Registering binary path in system Environment..." -ForegroundColor Cyan
$userPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($userPath -notlike "*$installDir*") {
    $newUserPath = "$userPath;$installDir"
    [Environment]::SetEnvironmentVariable("Path", $newUserPath, [EnvironmentVariableTarget]::User)
    $env:Path = "$env:Path;$installDir"
    Write-Host "Path successfully updated. Please restart your shell to apply changes." -ForegroundColor Green
} else {
    Write-Host "Path is already registered." -ForegroundColor Gray
}

# 6. Bootstrap Directories
$pluginsDir = Join-Path $HOME ".mcpc\plugins"
if (-not (Test-Path $pluginsDir)) {
    New-Item -ItemType Directory -Path $pluginsDir | Out-Null
}
$cacheDir = Join-Path $HOME ".mcpc\cache"
if (-not (Test-Path $cacheDir)) {
    New-Item -ItemType Directory -Path $cacheDir | Out-Null
}

Write-Host "`n==========================================================" -ForegroundColor Cyan
Write-Host "🔒 MCPC INSTALLATION COMPLETED SUCCESSFULLY!" -ForegroundColor Green
Write-Host "You can now run 'mcpc --help' to get started." -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Cyan
