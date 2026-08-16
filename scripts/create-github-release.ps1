# create-github-release.ps1
# Create prerelease v0.1.0-alpha on xf2214/api-router and upload 2 assets.
# Uses Git Credential Manager via `git credential fill` for HTTPS auth.

$ErrorActionPreference = 'Stop'

$Owner     = 'xf2214'
$Repo      = 'api-router'
$Tag       = 'v0.1.0-alpha'
$Title     = 'API Router v0.1.0-alpha · Windows x64 首个公开预览版'

$RepoRoot  = Split-Path -Parent $PSScriptRoot
$NotesFile = Join-Path $RepoRoot 'release-notes-v0.1.0-alpha.md'
$BundleDir = Join-Path $RepoRoot 'src-tauri\target\release\bundle'
$ExePath   = Join-Path $BundleDir 'nsis\API Router_0.1.0_x64-setup.exe'
$MsiPath   = Join-Path $BundleDir 'msi\API Router_0.1.0_x64_en-US.msi'

# ---------------------------------------------------------------------------
# 1. Acquire GitHub token via Git Credential Manager (GCM)
# ---------------------------------------------------------------------------
Write-Host '[1/4] Acquiring GitHub token via git-credential (GCM) ...' -ForegroundColor Cyan

$credIn = "protocol=https`r`nhost=github.com`r`n`r`n"
$cred = $credIn | & git credential fill 2>&1 | Out-String
if ($LASTEXITCODE -ne 0) {
    Write-Host 'First fill failed — retrying to allow GCM interactive login ...' -ForegroundColor Yellow
    Start-Sleep -Seconds 1
    $cred = $credIn | & git credential fill 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0) { throw "git credential fill failed: $cred" }
}

$pwLine = @($cred -split "`r?`n" | Where-Object { $_ -match '^password=' })[0]
if (-not $pwLine) { throw 'No token returned by git credential. Ensure GCM is installed and you logged into github.com.' }
$Token = $pwLine.Substring('password='.Length)

$headers = @{
    'Accept'               = 'application/vnd.github+json'
    'Authorization'        = 'Bearer ' + $Token
    'X-GitHub-Api-Version' = '2022-11-28'
}

# ---------------------------------------------------------------------------
# 2. Create prerelease via REST API
# ---------------------------------------------------------------------------
Write-Host "[2/4] Creating prerelease $Tag on $Owner/$Repo ..." -ForegroundColor Cyan
if (-not (Test-Path -LiteralPath $NotesFile)) { throw "Notes file not found: $NotesFile" }

$bodyObj = @{
    tag_name               = $Tag
    target_commitish       = 'master'
    name                   = $Title
    body                   = [System.IO.File]::ReadAllText($NotesFile, [System.Text.Encoding]::UTF8)
    draft                  = $false
    prerelease             = $true
    generate_release_notes = $false
    make_latest            = 'true'
}
$bodyJson = $bodyObj | ConvertTo-Json -Depth 6

$resp = Invoke-RestMethod -Method Post `
    -Uri ("https://api.github.com/repos/$Owner/$Repo/releases") `
    -Headers $headers -ContentType 'application/json' -Body $bodyJson
$releaseId = [string]$resp.id
$uploadBase = ($resp.upload_url -replace '\{\?name,label\}$','')
Write-Host ('   Release ID = ' + $releaseId) -ForegroundColor Green

# ---------------------------------------------------------------------------
# 3. Upload assets
# ---------------------------------------------------------------------------
function UploadOne($path, $label) {
    $f = Get-Item -LiteralPath $path
    $sizeMb = [math]::Round($f.Length / 1MB, 2)
    $escName = [System.Uri]::EscapeDataString($f.Name)
    $escLab  = [System.Uri]::EscapeDataString($label)
    $uri = $uploadBase + '?name=' + $escName + '&label=' + $escLab

    Write-Host ('   -> ' + $f.Name + ' (' + $sizeMb + ' MB) ... ') -NoNewline
    $bytes = [System.IO.File]::ReadAllBytes($f.FullName)
    $ah = $headers.Clone()
    $ah['Content-Type'] = 'application/octet-stream'
    $r = Invoke-RestMethod -Method Post -Uri $uri -Headers $ah -Body $bytes
    Write-Host 'done' -ForegroundColor Green
    return [string]$r.browser_download_url
}

Write-Host '[3/4] Uploading 2 assets ...' -ForegroundColor Cyan
UploadOne $ExePath 'Windows x64 NSIS installer (recommended) - API Router_0.1.0_x64-setup.exe'
UploadOne $MsiPath 'Windows x64 WiX installer - API Router_0.1.0_x64_en-US.msi'

# ---------------------------------------------------------------------------
# 4. Summary
# ---------------------------------------------------------------------------
Write-Host ''
Write-Host '[4/4] Release created successfully.' -ForegroundColor Green
Write-Host ('   URL: https://github.com/' + $Owner + '/' + $Repo + '/releases/tag/' + $Tag)
