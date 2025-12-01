param(
    [switch]$FailIfMissing = $true,
    [string]$SdkVersion = '10.0.22621.0'
)

$ErrorActionPreference = 'Stop'

Write-Host "Install Windows SDK $SdkVersion and Windows ADK (Debugging Tools) via winget"

function Try-WingetInstall($id, $version = $null) {
  try {
    if ($version) {
      Write-Host "winget install $id --version $version"
      winget install --accept-package-agreements --accept-source-agreements --id $id --version $version -e
    } else {
      Write-Host "winget install $id"
      winget install --accept-package-agreements --accept-source-agreements --id $id -e
    }
    return $true
  } catch {
    Write-Host "winget install for $id failed or already installed: $($_.Exception.Message)"
    return $false
  }
}

# Ensure winget exists
if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
  Write-Error "winget not found on runner. Ensure winget is available on the Windows runner."
  if ($FailIfMissing) { exit 1 } else { return }
}

# Install SDK and ADK (Debugging Tools)
Try-WingetInstall -id 'Microsoft.WindowsSDK' -version $SdkVersion | Out-Null
Try-WingetInstall -id 'Microsoft.WindowsADK' | Out-Null

# Verification
$sdkIncludePath = Join-Path 'C:\Program Files (x86)\Windows Kits\10\include' $SdkVersion
$dbghelpPath = 'C:\Program Files (x86)\Windows Kits\10\Debuggers\x64\dbghelp.dll'

$sdkExists = Test-Path $sdkIncludePath
$dbgExists = Test-Path $dbghelpPath

Write-Host "Windows SDK include present: $sdkExists at $sdkIncludePath"
Write-Host "dbghelp.dll present: $dbgExists at $dbghelpPath"

if (-not $sdkExists) { Write-Warning "Windows SDK $SdkVersion not found under Windows Kits include dir" }
if (-not $dbgExists) { Write-Warning "dbghelp.dll not found. Debugging Tools may be missing." }

if ($FailIfMissing -and (-not $sdkExists -or -not $dbgExists)) {
  Write-Error "Required SDK or Debugging Tools missing. Failing as requested."
  exit 2
}

Write-Host "Install/verification finished." 
