Set-StrictMode -Version 2
$ErrorActionPreference = 'Stop'

vcpkg integrate install
vcpkg install openssl:x64-windows-static-md
'OPENSSL_DIR=C:\vcpkg\installed\x64-windows-static-md' >> $env:GITHUB_ENV
'OPENSSL_STATIC=Yes' >> $env:GITHUB_ENV
