param(
    [switch]$FailIfMissing = $true,
    [string]$SdkVersion = '10.0.22621.0'
)

$ErrorActionPreference = 'Stop'

Write-Host "Install Windows SDK $SdkVersion and Windows ADK (Debugging Tools) via winget"

# Prepare logs directory
$logDir = Join-Path $env:TEMP 'install-windows-sdk-logs'
if (-not (Test-Path $logDir)) { New-Item -ItemType Directory -Path $logDir | Out-Null }

function Try-WingetInstall($id, $version = $null) {
  $safeId = ($id -replace '[^A-Za-z0-9_-]', '_')
  $logFile = Join-Path $logDir "$safeId-install.log"
  try {
    if ($version) {
      Write-Host "winget install $id --version $version (logging to $logFile)"
      & winget install --accept-package-agreements --accept-source-agreements --id $id --version $version -e *> $logFile 2>&1
    } else {
      Write-Host "winget install $id (logging to $logFile)"
      & winget install --accept-package-agreements --accept-source-agreements --id $id -e *> $logFile 2>&1
    }
    Write-Host "--- Begin winget log: $logFile ---"
    Get-Content -Path $logFile -ErrorAction SilentlyContinue | ForEach-Object { Write-Host $_ }
    Write-Host "--- End winget log ---"
    return $true
  } catch {
    Write-Host "winget install for $id failed or already installed: $($_.Exception.Message)"
    if (Test-Path $logFile) {
      Write-Host "--- winget log (on failure): $logFile ---"
      Get-Content -Path $logFile -ErrorAction SilentlyContinue | ForEach-Object { Write-Host $_ }
      Write-Host "--- end winget log ---"
    }
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

# Initial verification
$sdkExists = Test-Path $sdkIncludePath
$dbgExists = Test-Path $dbghelpPath

Write-Host "Windows SDK include present: $sdkExists at $sdkIncludePath"
Write-Host "dbghelp.dll present: $dbgExists at $dbghelpPath"

if (-not $sdkExists) { Write-Warning "Windows SDK $SdkVersion not found under Windows Kits include dir" }
if (-not $dbgExists) { Write-Warning "dbghelp.dll not found. Debugging Tools may be missing." }

# If the exact SDK is missing, try to add it via Visual Studio Installer CLI (more reliable on hosted runners)
if (-not $sdkExists) {
  $programFilesX86 = $env['ProgramFiles(x86)']
  if (-not $programFilesX86) { $programFilesX86 = 'C:\Program Files (x86)' }
  $vsInstallerPath = Join-Path $programFilesX86 'Microsoft Visual Studio\Installer\vs_installer.exe'
  $vsWherePath = Join-Path $programFilesX86 'Microsoft Visual Studio\Installer\vswhere.exe'
  $vsInstallerExists = Test-Path -Path $vsInstallerPath -PathType Leaf -ErrorAction SilentlyContinue
  $vsWhereExists = Test-Path -Path $vsWherePath -PathType Leaf -ErrorAction SilentlyContinue
  if ($vsInstallerExists -and $vsWhereExists) {
    Write-Host "Attempting to install Windows SDK $SdkVersion via Visual Studio Installer..."
    try {
      # Find the default installation path of the latest Visual Studio instance
      $instPath = & $vsWherePath -latest -products * -property installationPath 2>$null | Select-Object -First 1
      if (-not $instPath) { throw "Could not locate Visual Studio installation via vswhere" }

      # Component IDs: try generic and specific SDK component IDs
      $components = @("Microsoft.VisualStudio.Component.Windows10SDK", "Microsoft.VisualStudio.Component.Windows10SDK.$($SdkVersion -replace '[^0-9]','')")
      $modifyArgs = @('modify', "--installPath", $instPath)
      foreach ($c in $components) { $modifyArgs += @('--add', $c) }
      $modifyArgs += @('--passive', '--wait')

      $vsLog = Join-Path $logDir 'vs_installer.log'
      Write-Host "Running Visual Studio Installer: $vsInstallerPath $($modifyArgs -join ' ') (logging to $vsLog)"
      & $vsInstallerPath @modifyArgs *> $vsLog 2>&1
      Write-Host "--- Begin vs_installer log ---"
      Get-Content -Path $vsLog -ErrorAction SilentlyContinue | ForEach-Object { Write-Host $_ }
      Write-Host "--- End vs_installer log ---"
    } catch {
      Write-Warning "Visual Studio Installer approach failed: $($_.Exception.Message)"
    }

    # Re-check presence after attempting install
    $sdkExists = Test-Path $sdkIncludePath
    Write-Host "Post-install check: Windows SDK include present: $sdkExists at $sdkIncludePath"
  } else {
    Write-Warning "Visual Studio Installer (vs_installer.exe) or vswhere.exe not found; cannot attempt installer-based SDK add."
  }
}

if ($FailIfMissing -and (-not $sdkExists -or -not $dbgExists)) {
  Write-Error "Required SDK or Debugging Tools missing. Failing as requested."
  exit 2
}

Write-Host "Install/verification finished." 
