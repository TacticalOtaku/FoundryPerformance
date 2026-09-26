$ErrorActionPreference = "Stop"
$version = (Get-Content src-tauri/tauri.conf.json -Raw | ConvertFrom-Json).version
$exe = "src-tauri/target/release/foundry-performance.exe"
if (-not (Test-Path $exe)) { throw "Build first: npm run tauri build" }
New-Item -ItemType Directory -Force dist-portable | Out-Null
$zip = "dist-portable/FoundryPerformance-$version-portable.zip"
if (Test-Path $zip) { Remove-Item $zip }
Compress-Archive -Path $exe -DestinationPath $zip
Write-Output "Created $zip"
