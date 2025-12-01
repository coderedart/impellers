param(
  [switch]$FailIfMissing,
  [string]$SdkVersion = '10.0.22621.0'
)

# Normalize FailIfMissing (default true)
if ($PSBoundParameters.ContainsKey('FailIfMissing')) { $FailIfMissingEffective = [bool]$FailIfMissing } else { $FailIfMissingEffective = $true }

$ErrorActionPreference = 'Stop'

Write-Host "Ensure Windows SDK $SdkVersion and Debugging Tools are installed (via Visual Studio Installer)."

# Paths to check
$sdkIncludePath = Join-Path 'C:\Program Files (x86)\Windows Kits\10\include' $SdkVersion
$dbghelpPath = 'C:\Program Files (x86)\Windows Kits\10\Debuggers\x64\dbghelp.dll'

Write-Host "Checking for SDK include: $sdkIncludePath"
Write-Host "Checking for dbghelp.dll: $dbghelpPath"

if ((Test-Path $sdkIncludePath) -and (Test-Path $dbghelpPath)) {
  Write-Host "Required SDK and Debugging Tools already present."
  exit 0
}

Write-Host "Required files missing. Attempting to add SDK via Visual Studio Installer..."

# Robustly obtain Program Files (x86)
$programFilesX86 = [System.Environment]::GetEnvironmentVariable('ProgramFiles(x86)')
if (-not $programFilesX86) { $programFilesX86 = [System.Environment]::GetEnvironmentVariable('ProgramFiles') }
if (-not $programFilesX86) { $programFilesX86 = 'C:\Program Files (x86)' }

$vsWherePath = Join-Path $programFilesX86 'Microsoft Visual Studio\Installer\vswhere.exe'
$vsInstallerPath = Join-Path $programFilesX86 'Microsoft Visual Studio\Installer\vs_installer.exe'

if (-not (Test-Path $vsWherePath)) { Write-Error "vswhere.exe not found under $programFilesX86"; if ($FailIfMissingEffective) { exit 1 } else { return } }
if (-not (Test-Path $vsInstallerPath)) { Write-Error "vs_installer.exe not found under $programFilesX86"; if ($FailIfMissingEffective) { exit 1 } else { return } }

# Locate latest Visual Studio installation
$instPath = & $vsWherePath -latest -products * -property installationPath 2>$null | Select-Object -First 1
if (-not $instPath) { Write-Error "Could not locate Visual Studio installation via vswhere.exe"; if ($FailIfMissingEffective) { exit 1 } else { return } }

Write-Host "Found Visual Studio at: $instPath"

# Attempt to add the Windows 10 SDK component
$componentId = 'Microsoft.VisualStudio.Component.Windows10SDK'
$modifyArgs = @('modify', '--installPath', $instPath, '--add', $componentId, '--passive', '--wait')

Write-Host "Running Visual Studio Installer to add component: $componentId"
Write-Host "Command: $vsInstallerPath $($modifyArgs -join ' ')"

& $vsInstallerPath @modifyArgs

# Re-check presence
$sdkExists = Test-Path $sdkIncludePath
$dbgExists = Test-Path $dbghelpPath

Write-Host "Post-install SDK include present: $sdkExists at $sdkIncludePath"
Write-Host "Post-install dbghelp.dll present: $dbgExists at $dbghelpPath"

if ($FailIfMissingEffective -and (-not $sdkExists -or -not $dbgExists)) {
  Write-Error "Required SDK or Debugging Tools missing after installer."
  exit 2
}

Write-Host "Install/verification finished."
param(
    [switch]$FailIfMissing,
    [string]$SdkVersion = '10.0.22621.0'
)

# Normalize FailIfMissing: default to $true when not explicitly provided
if ($PSBoundParameters.ContainsKey('FailIfMissing')) {
    $FailIfMissingEffective = [bool]$FailIfMissing
} else {
    $FailIfMissingEffective = $true
}

$ErrorActionPreference = 'Stop'

Write-Host "Ensure Windows SDK $SdkVersion and Debugging Tools are installed (via Visual Studio Installer)."

# Paths to check
$sdkIncludePath = Join-Path 'C:\Program Files (x86)\Windows Kits\10\include' $SdkVersion
$dbghelpPath = 'C:\Program Files (x86)\Windows Kits\10\Debuggers\x64\dbghelp.dll'

Write-Host "Checking for SDK include: $sdkIncludePath"
Write-Host "Checking for dbghelp.dll: $dbghelpPath"

$sdkExists = Test-Path $sdkIncludePath
$dbgExists = Test-Path $dbghelpPath

if ($sdkExists -and $dbgExists) {
  Write-Host "Required SDK and Debugging Tools already present."
  return
}

Write-Host "Required files missing. Attempting to add SDK via Visual Studio Installer..."

# Locate Program Files (x86)
$programFilesX86 = $env['ProgramFiles(x86)']
if (-not $programFilesX86) { $programFilesX86 = 'C:\Program Files (x86)' }

$vsInstallerPath = Join-Path $programFilesX86 'Microsoft Visual Studio\Installer\vs_installer.exe'
$vsWherePath = Join-Path $programFilesX86 'Microsoft Visual Studio\Installer\vswhere.exe'

if (-not (Test-Path $vsWherePath) -or -not (Test-Path $vsInstallerPath)) {
  Write-Error "vswhere.exe or vs_installer.exe not found under $programFilesX86. Cannot install SDK via Visual Studio Installer."
  if ($FailIfMissingEffective) { exit 1 } else { return }
}

# Find latest Visual Studio installation path
$instPath = & $vsWherePath -latest -products * -property installationPath 2>$null | Select-Object -First 1
if (-not $instPath) {
  Write-Error "Could not locate Visual Studio installation via vswhere.exe"
  if ($FailIfMissingEffective) { exit 1 } else { return }
}

Write-Host "Found Visual Studio at: $instPath"

# Try to add the Windows 10 SDK component
$componentId = 'Microsoft.VisualStudio.Component.Windows10SDK'
$modifyArgs = @('modify', '--installPath', $instPath, '--add', $componentId, '--passive', '--wait')

Write-Host "Running Visual Studio Installer to add component: $componentId"
Write-Host "Command: $vsInstallerPath $($modifyArgs -join ' ')"

& $vsInstallerPath @modifyArgs

# Re-check presence
$sdkExists = Test-Path $sdkIncludePath
$dbgExists = Test-Path $dbghelpPath

Write-Host "Post-install SDK include present: $sdkExists at $sdkIncludePath"
Write-Host "Post-install dbghelp.dll present: $dbgExists at $dbghelpPath"

if ($FailIfMissingEffective -and (-not $sdkExists -or -not $dbgExists)) {
  Write-Error "Required SDK or Debugging Tools missing after installer. Failing as requested."
  exit 2
}

Write-Host "Install/verification finished."

& $vsinst modify --installPath $inst --add Microsoft.VisualStudio.Component.Windows10SDK --passive --wait
if ((Test-Path $sdk) -and (Test-Path $dbg)) { Write-Host 'Installed SDK+dbg'; exit 0 }
Write-Error 'SDK or dbghelp missing after install'; exit 2
param(
  [switch]$FailIfMissing,
  [string]$SdkVersion = '10.0.22621.0'
)

# Normalize FailIfMissing: default to $true when not explicitly provided
if ($PSBoundParameters.ContainsKey('FailIfMissing')) {
  $FailIfMissingEffective = [bool]$FailIfMissing
} else {
  $FailIfMissingEffective = $true
}

$ErrorActionPreference = 'Stop'

Write-Host "Install Windows SDK $SdkVersion and Windows ADK (Debugging Tools) via winget"

# Prepare logs directory
$logDir = Join-Path $env:TEMP 'install-windows-sdk-logs'
if (-not (Test-Path $logDir)) { New-Item -ItemType Directory -Path $logDir | Out-Null }
param(
    [switch]$FailIfMissing = $true,
    [string]$SdkVersion = '10.0.22621.0'
)

$ErrorActionPreference = 'Stop'

Write-Host "Ensure Windows SDK $SdkVersion and Debugging Tools are installed (via Visual Studio Installer)."

# Paths to check
$sdkIncludePath = Join-Path 'C:\Program Files (x86)\Windows Kits\10\include' $SdkVersion
$dbghelpPath = 'C:\Program Files (x86)\Windows Kits\10\Debuggers\x64\dbghelp.dll'

Write-Host "Checking for SDK include: $sdkIncludePath"
Write-Host "Checking for dbghelp.dll: $dbghelpPath"

$sdkExists = Test-Path $sdkIncludePath
$dbgExists = Test-Path $dbghelpPath

if ($sdkExists -and $dbgExists) {
  Write-Host "Required SDK and Debugging Tools already present."
  return
}

Write-Host "Required files missing. Attempting to add SDK via Visual Studio Installer..."

# Locate Program Files (x86)
$programFilesX86 = $env['ProgramFiles(x86)']
if (-not $programFilesX86) { $programFilesX86 = 'C:\Program Files (x86)' }

$vsInstallerPath = Join-Path $programFilesX86 'Microsoft Visual Studio\Installer\vs_installer.exe'
$vsWherePath = Join-Path $programFilesX86 'Microsoft Visual Studio\Installer\vswhere.exe'

if (-not (Test-Path $vsWherePath) -or -not (Test-Path $vsInstallerPath)) {
  Write-Error "vswhere.exe or vs_installer.exe not found under $programFilesX86. Cannot install SDK via Visual Studio Installer."
  if ($FailIfMissing) { exit 1 } else { return }
}

# Find latest Visual Studio installation path
$instPath = & $vsWherePath -latest -products * -property installationPath 2>$null | Select-Object -First 1
if (-not $instPath) {
  Write-Error "Could not locate Visual Studio installation via vswhere.exe"
  if ($FailIfMissing) { exit 1 } else { return }
}

Write-Host "Found Visual Studio at: $instPath"

# Try to add the Windows 10 SDK component
$componentId = 'Microsoft.VisualStudio.Component.Windows10SDK'
$modifyArgs = @('modify', '--installPath', $instPath, '--add', $componentId, '--passive', '--wait')

Write-Host "Running Visual Studio Installer to add component: $componentId"
Write-Host "Command: $vsInstallerPath $($modifyArgs -join ' ')"

& $vsInstallerPath @modifyArgs

# Re-check presence
$sdkExists = Test-Path $sdkIncludePath
$dbgExists = Test-Path $dbghelpPath

Write-Host "Post-install SDK include present: $sdkExists at $sdkIncludePath"
Write-Host "Post-install dbghelp.dll present: $dbgExists at $dbghelpPath"

if ($FailIfMissingEffective -and (-not $sdkExists -or -not $dbgExists)) {
  Write-Error "Required SDK or Debugging Tools missing after installer. Failing as requested."
  exit 2
}

Write-Host "Install/verification finished."
  $vsWhereExists = Test-Path -Path $vsWherePath -PathType Leaf -ErrorAction SilentlyContinue
