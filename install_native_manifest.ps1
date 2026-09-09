param(
    [Parameter(Mandatory=$true)]
    [string]$ExtensionId
)

$ManifestPath = Join-Path $PSScriptRoot "com.zeroproductivity.desktop.json"
$ExePath = Join-Path $PSScriptRoot "apps\desktop\src-tauri\target\debug\desktop.exe"
$BatPath = Join-Path $PSScriptRoot "native_host.bat"

$BatContent = @"
@echo off
"$ExePath" --native-messaging
"@
Set-Content -Path $BatPath -Value $BatContent

$ManifestContent = @"
{
  "name": "com.zeroproductivity.desktop",
  "description": "Zero Productivity Desktop integration",
  "path": "$($BatPath.Replace('\', '\\'))",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://$ExtensionId/"
  ]
}
"@

Set-Content -Path $ManifestPath -Value $ManifestContent

$RegistryPath = "HKCU:\Software\Google\Chrome\NativeMessagingHosts\com.zeroproductivity.desktop"
if (!(Test-Path $RegistryPath)) {
    New-Item -Path $RegistryPath -Force | Out-Null
}

Set-ItemProperty -Path $RegistryPath -Name "(Default)" -Value $ManifestPath
Write-Host "Native Messaging Manifest installed successfully."
Write-Host "Manifest Path: $ManifestPath"
Write-Host "Wrapper Path: $BatPath"
Write-Host "Extension ID: $ExtensionId"
