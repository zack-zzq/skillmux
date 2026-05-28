param(
    [Parameter(Position = 0)]
    [string]$Bump,

    [switch]$Yes,
    [switch]$DryRun,
    [switch]$SkipChecks,
    [string]$Remote = "origin"
)

$ErrorActionPreference = "Stop"

$releaseScript = Join-Path $PSScriptRoot "release.py"
$python = Get-Command python -ErrorAction SilentlyContinue
if (-not $python) {
    throw "Python was not found on PATH."
}

$argsList = @($releaseScript)
if ($Bump) {
    $argsList += $Bump
}
if ($Yes) {
    $argsList += "--yes"
}
if ($DryRun) {
    $argsList += "--dry-run"
}
if ($SkipChecks) {
    $argsList += "--skip-checks"
}
if ($Remote -and $Remote -ne "origin") {
    $argsList += "--remote"
    $argsList += $Remote
}

& $python.Source @argsList
exit $LASTEXITCODE
