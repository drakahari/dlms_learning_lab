# Optional per-user installation. No elevation, policy changes or progress removal.
$ErrorActionPreference = 'Stop'
$Target = Join-Path $env:LOCALAPPDATA 'Programs\DLMS Learning Lab'
$Existing = Get-Item -LiteralPath $Target -Force -ErrorAction SilentlyContinue
if ($null -ne $Existing) {
    throw 'Existing application folder retained. Close the app and use the documented manual upgrade path.'
}
$Shortcut = Join-Path ([Environment]::GetFolderPath('Programs')) 'DLMS Learning Lab.lnk'
$Shell = New-Object -ComObject WScript.Shell
$ExistingLink = Get-Item -LiteralPath $Shortcut -Force -ErrorAction SilentlyContinue
if ($null -ne $ExistingLink) {
    if (($ExistingLink.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'Existing shortcut link retained.' }
    $Old = $Shell.CreateShortcut($Shortcut)
    if ($Old.TargetPath -ne (Join-Path $Target 'dlms-learning-lab.exe')) { throw 'Existing unrelated Start-menu shortcut retained.' }
}
[IO.Directory]::CreateDirectory($Target) | Out-Null
Get-ChildItem -LiteralPath $PSScriptRoot -Force | ForEach-Object {
    Copy-Item -LiteralPath $_.FullName -Destination $Target -Recurse
}
Set-Content -LiteralPath (Join-Path $Target '.dlms-installation') -Value 'DLMS Learning Lab application resources'
$Link = $Shell.CreateShortcut($Shortcut)
$Link.TargetPath = Join-Path $Target 'dlms-learning-lab.exe'
$Link.Save()
Write-Output 'Installed for this user. Local learner progress stays in its separate application-data folder.'
