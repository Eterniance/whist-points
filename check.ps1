#!/usr/bin/env pwsh
# This script runs various CI-like checks in a convenient way.

# Equivalent of: set -eux
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$PSNativeCommandUseErrorActionPreference = $true

# Echo commands as they are executed
function Invoke-CommandLogged {
    param ([string]$Command)
    Write-Host "+ $Command"
    Invoke-Expression $Command
}

Invoke-CommandLogged "cargo check --quiet --workspace --all-targets"
Invoke-CommandLogged "cargo check --quiet --workspace --all-features --lib --target wasm32-unknown-unknown"
Invoke-CommandLogged "cargo fmt --all -- --check"
Invoke-CommandLogged "cargo clippy --quiet --workspace --all-targets --all-features -- -D warnings -W clippy::all"
Invoke-CommandLogged "cargo test --quiet --workspace --all-targets --all-features"
Invoke-CommandLogged "cargo test --quiet --workspace --doc"
Invoke-CommandLogged "trunk build"
