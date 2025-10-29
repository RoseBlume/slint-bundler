# build-rust.ps1 - helper to build the Rust native lib for Android and copy the .so into the Android project's jniLibs
# Usage: Open an Android NDK-enabled PowerShell and run: .\build-rust.ps1

# Requirements (Windows):
# - Rust toolchain (rustup)
# - Android NDK installed and ANDROID_NDK_HOME environment variable set (or configured for cargo-ndk)
# - cargo-ndk installed (cargo install cargo-ndk)
# - Gradle (or use Android Studio)

$ErrorActionPreference = 'Stop'

$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Definition
$crateDir = $projectRoot
$jniDir = Join-Path $projectRoot 'android-app\app\src\main\jniLibs'

# ABIs to build. Adjust as needed.
$abis = @('armeabi-v7a','arm64-v8a','x86_64', 'x86')

# Ensure cargo-ndk exists
if (-not (Get-Command cargo-ndk -ErrorAction SilentlyContinue)) {
    Write-Host "cargo-ndk not found. Installing..."
    cargo install cargo-ndk
}

# Create jniLibs root
New-Item -ItemType Directory -Force -Path $jniDir | Out-Null

foreach ($abi in $abis) {
    Write-Host "Building for ABI: $abi"
    Push-Location $crateDir
    # cargo-ndk accepts --target or -t values: usein the abi name is supported by cargo-ndk.
    # This command will build the cdylib into target/<target-triple>/release/libmain.so
    cargo ndk -t $abi -o target build --release
    Pop-Location

    # Attempt to find produced lib in crate target directory (several target triples may be used)
    $possibleTargets = @(
        "aarch64-linux-android",
        "armv7-linux-androideabi",
        "x86_64-linux-android",
        "i686-linux-android"
    )

    $found = $false
    foreach ($t in $possibleTargets) {
        $src = Join-Path $crateDir "target\$t\release\libmain.so"
        if (Test-Path $src) {
            $destDir = Join-Path $jniDir $abi
            New-Item -ItemType Directory -Force -Path $destDir | Out-Null
            Copy-Item $src -Destination (Join-Path $destDir 'libmain.so') -Force
            Write-Host "Copied $src -> $destDir\libmain.so"
            $found = $true
            break
        }
    }

    if (-not $found) {
        Write-Warning "Could not find libmain.so for ABI $abi in expected target locations. Build may have failed or used a different target triple."
    }
}

Write-Host "Rust build finished. Next: open android-app/ in Android Studio or run Gradle to build the APK (it will include the .so files from app/src/main/jniLibs)."