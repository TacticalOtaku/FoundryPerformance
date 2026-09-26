$ErrorActionPreference = "Stop"
$version = (Get-Content src-tauri/tauri.conf.json -Raw | ConvertFrom-Json).version
$exe = "src-tauri/target/release/foundry-performance.exe"
if (-not (Test-Path $exe)) { throw "Build first: npm run tauri build" }

New-Item -ItemType Directory -Force dist-release | Out-Null

# The exe is the installer: run it and it installs itself.
$setup = "dist-release/FoundryPerformance-$version.exe"
Copy-Item $exe $setup -Force

# Portable: the same exe plus an empty "portable" marker next to it (no installer screen).
$tmp = Join-Path $env:TEMP "fp-portable-$version"
if (Test-Path $tmp) { Remove-Item $tmp -Recurse -Force }
New-Item -ItemType Directory $tmp | Out-Null
Copy-Item $exe (Join-Path $tmp "FoundryPerformance.exe")
New-Item -ItemType File (Join-Path $tmp "portable") | Out-Null
$zip = "dist-release/FoundryPerformance-$version-portable.zip"
if (Test-Path $zip) { Remove-Item $zip }
Compress-Archive -Path (Join-Path $tmp "*") -DestinationPath $zip
Remove-Item $tmp -Recurse -Force

Write-Output "Created $setup"
Write-Output "Created $zip"
