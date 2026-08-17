[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [ValidateSet("audit", "validation-plan", "session-start", "measure", "session-end")]
    [string]$Command = "audit",
    [string]$BaseRef = "HEAD~1",
    [string]$Task = "",
    [string]$SessionId = "",
    [string]$Category = "command",
    [switch]$Json,
    [switch]$Ci,
    [switch]$Strict,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$RemainingArgs
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$PolicyPath = Join-Path $RepoRoot "config\development-policy.json"
if (-not (Test-Path -LiteralPath $PolicyPath -PathType Leaf)) {
    throw "Missing development policy: $PolicyPath"
}
$Policy = Get-Content -LiteralPath $PolicyPath -Raw -Encoding UTF8 | ConvertFrom-Json
$GitExe = (Get-Command git -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source

function Invoke-RepoGit {
    param([string[]]$Args, [switch]$AllowFailure)
    $safe = $RepoRoot.Replace("\", "/")
    $lines = @(& $GitExe -c "safe.directory=$safe" @Args 2>&1)
    $rc = $LASTEXITCODE
    $text = ($lines -join [Environment]::NewLine).TrimEnd()
    if ($rc -ne 0 -and -not $AllowFailure) {
        throw "git $($Args -join ' ') failed ($rc)`n$text"
    }
    [pscustomobject]@{ ExitCode = $rc; Text = $text }
}

function GitText {
    param([string[]]$Args)
    (Invoke-RepoGit -Args $Args).Text.Trim()
}

function SizeBytes {
    param([string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) { return [int64]0 }
    $sum = (Get-ChildItem -LiteralPath $Path -File -Recurse -Force -ErrorAction SilentlyContinue |
        Measure-Object Length -Sum).Sum
    if ($null -eq $sum) { return [int64]0 }
    [int64]$sum
}

function WorktreeCount {
    $r = Invoke-RepoGit -Args @("worktree", "list", "--porcelain") -AllowFailure
    if ($r.ExitCode -ne 0) { return $null }
    @($r.Text -split "`r?`n" | Where-Object { $_ -match "^worktree " }).Count
}

function Snapshot {
    $status = GitText @("status", "--porcelain", "--untracked-files=all")
    [ordered]@{
        timestamp_utc = [DateTime]::UtcNow.ToString("o")
        repo_root = $RepoRoot
        branch = GitText @("branch", "--show-current")
        source_sha = GitText @("rev-parse", "HEAD")
        git_state = $(if ([string]::IsNullOrWhiteSpace($status)) { "clean" } else { "dirty" })
        worktree_count = WorktreeCount
        target_bytes = SizeBytes (Join-Path $RepoRoot "target")
        artifact_bytes = SizeBytes ([string]$Policy.artifacts.root)
    }
}

function SessionsRoot {
    $path = Join-Path ([string]$Policy.artifacts.root) ([string]$Policy.artifacts.session_subdir)
    New-Item -ItemType Directory -Path $path -Force | Out-Null
    $path
}

function SessionDir {
    param([string]$Id)
    if ($Id -notmatch "^[A-Za-z0-9][A-Za-z0-9._-]*$") { throw "Unsafe SessionId: $Id" }
    $path = Join-Path (SessionsRoot) $Id
    if (-not (Test-Path -LiteralPath $path -PathType Container)) { throw "Unknown session: $Id" }
    $path
}

function JsonLine {
    param([string]$Path, $Value)
    Add-Content -LiteralPath $Path -Encoding UTF8 -Value ($Value | ConvertTo-Json -Depth 12 -Compress)
}

function ChangedPaths {
    param([string]$FromRef)
    $set = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    $gotRange = $false
    foreach ($range in @("$FromRef...HEAD", "$FromRef..HEAD")) {
        $r = Invoke-RepoGit -Args @("diff", "--name-only", "--diff-filter=ACMR", $range) -AllowFailure
        if ($r.ExitCode -eq 0) {
            foreach ($line in $r.Text -split "`r?`n") {
                if ($line) { [void]$set.Add($line.Replace("\", "/")) }
            }
            $gotRange = $true
            break
        }
    }
    if (-not $gotRange) {
        $r = Invoke-RepoGit -Args @("diff", "--name-only", "--diff-filter=ACMR", "HEAD^", "HEAD") -AllowFailure
        if ($r.ExitCode -eq 0) {
            foreach ($line in $r.Text -split "`r?`n") {
                if ($line) { [void]$set.Add($line.Replace("\", "/")) }
            }
        }
    }
    foreach ($args in @(
        @("diff", "--name-only", "--diff-filter=ACMR"),
        @("diff", "--cached", "--name-only", "--diff-filter=ACMR"),
        @("ls-files", "--others", "--exclude-standard")
    )) {
        $r = Invoke-RepoGit -Args $args -AllowFailure
        if ($r.ExitCode -eq 0) {
            foreach ($line in $r.Text -split "`r?`n") {
                if ($line) { [void]$set.Add($line.Replace("\", "/")) }
            }
        }
    }
    @($set | Sort-Object)
}

function FullRank {
    param([string]$Value)
    if ($Value -eq "required") { return 3 }
    if ($Value -eq "recommended") { return 2 }
    1
}

function ValidationPlan {
    $files = @(ChangedPaths $BaseRef)
    $classes = @($Policy.validation_classes | Sort-Object priority -Descending)
    $rows = @()
    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $commands = [Collections.Generic.List[string]]::new()
    $full = "not-required"
    $smoke = "not-required"
    $candidate = "not-required"

    foreach ($file in $files) {
        $selected = $null
        foreach ($class in $classes) {
            foreach ($pattern in @($class.patterns)) {
                if ($file -match [string]$pattern) { $selected = $class; break }
            }
            if ($null -ne $selected) { break }
        }
        if ($null -eq $selected) {
            $selected = [pscustomobject]@{
                id = "unknown"; priority = 75; full = "recommended"
                smoke = "review-required"; candidate = "review-required"
                commands = @("cargo fmt --all -- --check", "cargo check --workspace --all-targets", "git diff --check")
            }
        }
        if ((FullRank ([string]$selected.full)) -gt (FullRank $full)) { $full = [string]$selected.full }
        if ([string]$selected.smoke -ne "not-required") { $smoke = [string]$selected.smoke }
        if ([string]$selected.candidate -ne "not-required") { $candidate = [string]$selected.candidate }
        foreach ($item in @($selected.commands)) {
            $value = [string]$item
            if ($seen.Add($value)) { $commands.Add($value) }
        }
        $rows += [pscustomobject]@{
            path = $file; class = [string]$selected.id
            full = [string]$selected.full; smoke = [string]$selected.smoke
            candidate = [string]$selected.candidate
        }
    }

    [ordered]@{
        schema_version = 1
        generated_utc = [DateTime]::UtcNow.ToString("o")
        base_ref = $BaseRef
        source_sha = GitText @("rev-parse", "HEAD")
        changed_file_count = $files.Count
        changed_files = $rows
        full = $full
        smoke = $smoke
        candidate = $candidate
        commands = @($commands)
    }
}

function Audit {
    $errors = [Collections.Generic.List[string]]::new()
    $warnings = [Collections.Generic.List[string]]::new()
    $allowed = @($Policy.launchers.allowed_root | ForEach-Object { [string]$_ })
    $legacy = @($Policy.launchers.legacy_root | ForEach-Object { [string]$_.path })
    $known = @($allowed + $legacy)
    $rootFiles = @(Get-ChildItem -LiteralPath $RepoRoot -File -Force)
    $launchers = @($rootFiles | Where-Object { $_.Name -match "^run_.*\.(bat|cmd|ps1)$" } |
        Select-Object -ExpandProperty Name | Sort-Object)

    foreach ($name in $launchers) {
        if ($known -notcontains $name) { $errors.Add("Unapproved root launcher: $name") }
    }
    foreach ($file in $rootFiles) {
        foreach ($pattern in @($Policy.launchers.deny_new_root_patterns)) {
            if ($file.Name -match [string]$pattern -and $known -notcontains $file.Name) {
                $errors.Add("Denied root entrypoint variant: $($file.Name)")
                break
            }
        }
    }
    foreach ($name in $allowed) {
        if (-not (Test-Path -LiteralPath (Join-Path $RepoRoot $name) -PathType Leaf)) {
            $errors.Add("Required launcher missing: $name")
        }
    }
    foreach ($entry in @($Policy.launchers.legacy_root)) {
        if (Test-Path -LiteralPath (Join-Path $RepoRoot ([string]$entry.path))) {
            $warnings.Add("Legacy launcher: $($entry.path) -> $($entry.replacement); remove by $($entry.remove_by)")
        }
    }
    foreach ($path in @($Policy.required_policy_files)) {
        if (-not (Test-Path -LiteralPath (Join-Path $RepoRoot ([string]$path)) -PathType Leaf)) {
            $errors.Add("Required policy file missing: $path")
        }
    }
    $trackedExe = GitText @("ls-files", "--", "*.exe")
    if ($trackedExe) {
        foreach ($path in $trackedExe -split "`r?`n") { if ($path) { $errors.Add("EXE committed to Git: $path") } }
    }

    $worktrees = $null
    if (-not $Ci) {
        $worktrees = WorktreeCount
        if ($null -ne $worktrees -and $worktrees -gt [int]$Policy.worktrees.max_total) {
            $warnings.Add("Worktrees $worktrees exceed max $($Policy.worktrees.max_total)")
        }
        $artifact = [IO.Path]::GetFullPath([string]$Policy.artifacts.root)
        $repo = [IO.Path]::GetFullPath($RepoRoot)
        if ($artifact.StartsWith($repo, [StringComparison]::OrdinalIgnoreCase)) {
            $errors.Add("Artifact root must be outside repository: $artifact")
        }
    }

    $result = [ordered]@{
        status = $(if ($errors.Count -eq 0) { "PASS" } else { "FAIL" })
        canonical_binary = [string]$Policy.canonical_app.binary
        launchers = $launchers
        worktree_count = $worktrees
        errors = @($errors)
        warnings = @($warnings)
    }
    if ($Json) { $result | ConvertTo-Json -Depth 8 }
    else {
        Write-Host "Development policy audit: $($result.status)"
        Write-Host "Canonical app: $($result.canonical_binary)"
        Write-Host "Launchers: $($launchers -join ', ')"
        foreach ($w in $warnings) { Write-Warning $w }
        foreach ($e in $errors) { Write-Error $e -ErrorAction Continue }
    }
    if ($errors.Count -gt 0) { exit 1 }
    if ($Strict -and $warnings.Count -gt 0) {
        Write-Host "Declared migration debt remains warning-only; new violations fail."
    }
}

function StartSession {
    if (-not $Task) { throw "session-start requires -Task" }
    $started = [DateTime]::UtcNow
    $snap = Snapshot
    $slug = [regex]::Replace($Task.ToLowerInvariant(), "[^a-z0-9]+", "-").Trim("-")
    if (-not $slug) { $slug = "task" }
    if (-not $SessionId) {
        $SessionId = "{0}-{1}-{2}" -f $started.ToString("yyyyMMddTHHmmssfffZ"), $slug, ([string]$snap.source_sha).Substring(0, 8)
    }
    if ($SessionId -notmatch "^[A-Za-z0-9][A-Za-z0-9._-]*$") { throw "Unsafe SessionId: $SessionId" }
    $dir = Join-Path (SessionsRoot) $SessionId
    New-Item -ItemType Directory -Path $dir -ErrorAction Stop | Out-Null
    "started_utc,ended_utc,elapsed_seconds,exit_code,category,argv_json" |
        Set-Content -LiteralPath (Join-Path $dir "COMMAND_TIMINGS.csv") -Encoding UTF8
    JsonLine (Join-Path $dir "SESSION.jsonl") ([ordered]@{
        event = "session_start"; session_id = $SessionId; task = $Task
        timing_confidence = "observed"; started_utc = $started.ToString("o"); snapshot = $snap
    })
    Write-Host "SESSION_ID=$SessionId"
    Write-Host "SESSION_DIR=$dir"
}

function MeasureCommand {
    $dir = SessionDir $SessionId
    $argv = @($RemainingArgs)
    if ($argv.Count -gt 0 -and $argv[0] -eq "--") { $argv = @($argv | Select-Object -Skip 1) }
    if ($argv.Count -eq 0) { throw "measure requires a command" }
    $exe = $argv[0]
    $args = $(if ($argv.Count -gt 1) { @($argv[1..($argv.Count - 1)]) } else { @() })
    $start = [DateTime]::UtcNow
    $watch = [Diagnostics.Stopwatch]::StartNew()
    & $exe @args
    $rc = $LASTEXITCODE
    $watch.Stop()
    $end = [DateTime]::UtcNow
    [pscustomobject]@{
        started_utc = $start.ToString("o"); ended_utc = $end.ToString("o")
        elapsed_seconds = [Math]::Round($watch.Elapsed.TotalSeconds, 6)
        exit_code = $rc; category = $Category
        argv_json = ConvertTo-Json -InputObject (@($exe) + $args) -Compress
    } | Export-Csv -LiteralPath (Join-Path $dir "COMMAND_TIMINGS.csv") -Append -NoTypeInformation -Encoding UTF8
    JsonLine (Join-Path $dir "SESSION.jsonl") ([ordered]@{
        event = "command"; category = $Category; argv = @($exe) + $args
        started_utc = $start.ToString("o"); ended_utc = $end.ToString("o")
        duration_seconds = [Math]::Round($watch.Elapsed.TotalSeconds, 6); exit_code = $rc
    })
    exit $rc
}

function EndSession {
    $dir = SessionDir $SessionId
    $events = @(Get-Content -LiteralPath (Join-Path $dir "SESSION.jsonl") -Encoding UTF8 |
        ForEach-Object { $_ | ConvertFrom-Json })
    $start = @($events | Where-Object { $_.event -eq "session_start" } | Select-Object -First 1)
    if ($start.Count -eq 0) { throw "Missing session_start" }
    $startUtc = [DateTime]::Parse([string]$start[0].started_utc).ToUniversalTime()
    $endUtc = [DateTime]::UtcNow
    $commands = @($events | Where-Object { $_.event -eq "command" })
    $commandSeconds = [double](($commands | Measure-Object duration_seconds -Sum).Sum)
    $fullCount = @($commands | Where-Object { ($_.argv -join " ") -match "cargo test --workspace" }).Count
    $candidateCount = @($commands | Where-Object {
        $t = $_.argv -join " "; $t -match "run_experiment\.bat" -and $t -notmatch "--mode scratch"
    }).Count
    $final = Snapshot
    $initial = $start[0].snapshot
    $longest = @($commands | Sort-Object duration_seconds -Descending | Select-Object -First 5 | ForEach-Object {
        [ordered]@{ seconds = [double]$_.duration_seconds; exit_code = [int]$_.exit_code; command = ($_.argv -join " ") }
    })
    $summary = [ordered]@{
        schema_version = 1; session_id = $SessionId; task = [string]$start[0].task
        started_utc = $startUtc.ToString("o"); ended_utc = $endUtc.ToString("o")
        wall_seconds = [Math]::Round(($endUtc - $startUtc).TotalSeconds, 3)
        command_seconds = [Math]::Round($commandSeconds, 3)
        full_count = $fullCount; candidate_count = $candidateCount
        longest_commands = $longest; initial_snapshot = $initial; final_snapshot = $final
        target_delta_bytes = [int64]$final.target_bytes - [int64]$initial.target_bytes
        artifact_delta_bytes = [int64]$final.artifact_bytes - [int64]$initial.artifact_bytes
    }
    $summary | ConvertTo-Json -Depth 14 | Set-Content -LiteralPath (Join-Path $dir "SUMMARY.json") -Encoding UTF8
    $md = @(
        "# Development Session Summary", "",
        "- Session: ``$SessionId``", "- Task: $($summary.task)",
        "- Wall: $($summary.wall_seconds) s", "- Command time: $($summary.command_seconds) s",
        "- FULL count: $fullCount", "- Candidate count: $candidateCount",
        "- Target delta: $($summary.target_delta_bytes) bytes",
        "- Artifact delta: $($summary.artifact_delta_bytes) bytes", "", "## Longest commands", ""
    )
    foreach ($item in $longest) { $md += "- $($item.seconds) s · exit $($item.exit_code) · ``$($item.command)``" }
    $md -join [Environment]::NewLine | Set-Content -LiteralPath (Join-Path $dir "SUMMARY.md") -Encoding UTF8
    JsonLine (Join-Path $dir "SESSION.jsonl") ([ordered]@{
        event = "session_end"; ended_utc = $endUtc.ToString("o")
        wall_seconds = $summary.wall_seconds; full_count = $fullCount; candidate_count = $candidateCount
    })
    Write-Host "Session complete: $dir"
}

switch ($Command) {
    "audit" { Audit }
    "validation-plan" {
        $plan = ValidationPlan
        if ($Json) { $plan | ConvertTo-Json -Depth 12 }
        else {
            Write-Host "Changed files: $($plan.changed_file_count)"
            Write-Host "FULL: $($plan.full) | Smoke: $($plan.smoke) | Candidate: $($plan.candidate)"
            foreach ($row in $plan.changed_files) { Write-Host "- $($row.path) -> $($row.class) (FULL $($row.full))" }
            Write-Host "Suggested commands:"
            foreach ($item in $plan.commands) { Write-Host "  $item" }
        }
    }
    "session-start" { StartSession }
    "measure" { MeasureCommand }
    "session-end" { EndSession }
}
