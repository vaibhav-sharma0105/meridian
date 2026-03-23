# update.ps1 — Build and package Meridian on Windows
# Usage: .\scripts\update.ps1 [-Version "x.y.z"] [-SkipFrontend]
param(
    [string]$Version = "",
    [switch]$SkipFrontend
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot

Push-Location $Root

try {
    if ($Version -ne "") {
        Write-Host "-> Bumping version to $Version"

        # Update Cargo.toml
        $cargo = Get-Content "$Root\src-tauri\Cargo.toml" -Raw
        $cargo = $cargo -replace '^version = ".*"', "version = `"$Version`""
        Set-Content "$Root\src-tauri\Cargo.toml" $cargo

        # Update tauri.conf.json
        $conf = Get-Content "$Root\src-tauri\tauri.conf.json" -Raw | ConvertFrom-Json
        $conf.version = $Version
        $conf | ConvertTo-Json -Depth 20 | Set-Content "$Root\src-tauri\tauri.conf.json"

        # Update package.json
        $pkg = Get-Content "$Root\package.json" -Raw | ConvertFrom-Json
        $pkg.version = $Version
        $pkg | ConvertTo-Json -Depth 10 | Set-Content "$Root\package.json"

        Write-Host "  Version updated to $Version"
    }

    if (-not $SkipFrontend) {
        Write-Host "-> Installing npm dependencies"
        npm ci

        Write-Host "-> Type-checking TypeScript"
        npx tsc --noEmit

        Write-Host "-> Building frontend"
        npm run build
    }

    Write-Host "-> Building Tauri (release)"
    npm run tauri build

    Write-Host ""
    Write-Host "Build complete. Artifacts:"
    $bundleDir = "$Root\src-tauri\target\release\bundle"
    if (Test-Path $bundleDir) {
        Get-ChildItem $bundleDir | ForEach-Object { Write-Host "  $_" }
    }
} finally {
    Pop-Location
}
