[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [ValidateSet("audit", "validation-plan", "session-start", "session-span", "measure", "session-end")]
    [string]$Command = "audit",
    [string]$BaseRef = "HEAD~1",
    [string]$Task = "",
    [string]$SessionId = "",
    [string]$Category = "command",
    [string]$Name = "",
    [double]$DurationSeconds = 0,
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
    param(
        [Parameter(Mandatory)]
        [string[]]$GitArgs,
        [switch]$AllowFailure
    )
    $safe = $RepoRoot.Replace("\", "/")
    $lines = @(& $GitExe -c "safe.directory=$safe" @GitArgs 2>&1)
    $rc = $LASTEXITCODE
    $text = ($lines -join [Environment]::NewLine).TrimEnd()
    if ($rc -ne 0 -and -not $AllowFailure) {
        throw "git $($GitArgs -join ' ') failed ($rc)`n$text"
    }
    [pscustomobject]@{ ExitCode = $rc; Text = $text }
}

function Get-RepoGitText {
    param([Parameter(Mandatory)][string[]]$GitArgs)
    (Invoke-RepoGit -GitArgs $GitArgs).Text.Trim()
}

function Get-DirectorySizeBytes {
    param([string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) { return [int64]0 }
    $sum = (Get-ChildItem -LiteralPath $Path -File -Recurse -Force -ErrorAction SilentlyContinue |
        Measure-Object Length -Sum).Sum
    if ($null -eq $sum) { return [int64]0 }
    [int64]$sum
}

function Get-WorktreeCount {
    $result = Invoke-RepoGit -GitArgs @("worktree", "list", "--porcelain") -AllowFailure
    if ($result.ExitCode -ne 0) { return $null }
    @($result.Text -split "`r?`n" | Where-Object { $_ -match "^worktree " }).Count
}

function Get-DevelopmentSnapshot {
    $status = Get-RepoGitText -GitArgs @("status", "--porcelain", "--untracked-files=all")
    [ordered]@{
        timestamp_utc = [DateTime]::UtcNow.ToString("o")
        repo_root = $RepoRoot
        branch = Get-RepoGitText -GitArgs @("branch", "--show-current")
        source_sha = Get-RepoGitText -GitArgs @("rev-parse", "HEAD")
        git_state = $(if ([string]::IsNullOrWhiteSpace($status)) { "clean" } else { "dirty" })
        worktree_count = Get-WorktreeCount
        target_bytes = Get-DirectorySizeBytes (Join-Path $RepoRoot "target")
        artifact_bytes = Get-DirectorySizeBytes ([string]$Policy.artifacts.root)
    }
}

function Get-SessionsRoot {
    $path = Join-Path ([string]$Policy.artifacts.root) ([string]$Policy.artifacts.session_subdir)
    New-Item -ItemType Directory -Path $path -Force | Out-Null
    $path
}

function Get-SessionDirectory {
    param([Parameter(Mandatory)][string]$Id)
    if ($Id -notmatch "^[A-Za-z0-9][A-Za-z0-9._-]*$") { throw "Unsafe SessionId: $Id" }
    $path = Join-Path (Get-SessionsRoot) $Id
    if (-not (Test-Path -LiteralPath $path -PathType Container)) { throw "Unknown session: $Id" }
    $path
}

function Add-JsonLine {
    param([Parameter(Mandatory)][string]$Path, [Parameter(Mandatory)]$Value)
    Add-Content -LiteralPath $Path -Encoding UTF8 -Value ($Value | ConvertTo-Json -Depth 14 -Compress)
}

function Get-ChangedPaths {
    param([Parameter(Mandatory)][string]$FromRef)
    $paths = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    $verified = Invoke-RepoGit -GitArgs @("rev-parse", "--verify", "$FromRef^{commit}") -AllowFailure
    if ($verified.ExitCode -ne 0) {
        throw "Invalid BaseRef: $FromRef"
    }
    $result = Invoke-RepoGit -GitArgs @("diff", "--name-only", "--diff-filter=ACMRD", "$FromRef...HEAD") -AllowFailure
    if ($result.ExitCode -ne 0) {
        throw "Unable to compare BaseRef $FromRef with HEAD"
    }
    foreach ($line in $result.Text -split "`r?`n") {
        if ($line) { [void]$paths.Add($line.Replace("\", "/")) }
    }
    $probes = @(
        @("diff", "--name-only", "--diff-filter=ACMRD"),
        @("diff", "--cached", "--name-only", "--diff-filter=ACMRD"),
        @("ls-files", "--others", "--exclude-standard")
    )
    foreach ($probe in $probes) {
        $result = Invoke-RepoGit -GitArgs $probe -AllowFailure
        if ($result.ExitCode -eq 0) {
            foreach ($line in $result.Text -split "`r?`n") {
                if ($line) { [void]$paths.Add($line.Replace("\", "/")) }
            }
        }
    }
    @($paths | Sort-Object)
}

function Get-FullRank {
    param([string]$Value)
    switch ($Value) {
        "required" { 3 }
        "recommended" { 2 }
        default { 1 }
    }
}

function Get-SmokeRank {
    param([string]$Value)
    switch ($Value) {
        "review-required" { 5 }
        "required" { 4 }
        "scenario-bounded" { 3 }
        "minimal-launcher" { 2 }
        default { 1 }
    }
}

function Get-CandidateRank {
    param([string]$Value)
    switch ($Value) {
        "review-required" { 3 }
        "when-task-output" { 2 }
        default { 1 }
    }
}

function Get-ValidationPlan {
    $files = @(Get-ChangedPaths -FromRef $BaseRef)
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
                if ($file -match [string]$pattern) {
                    $selected = $class
                    break
                }
            }
            if ($null -ne $selected) { break }
        }
        if ($null -eq $selected) {
            $selected = [pscustomobject]@{
                id = "unknown"
                full = "recommended"
                smoke = "review-required"
                candidate = "review-required"
                commands = @(
                    "cargo fmt --all -- --check",
                    "cargo check --workspace --all-targets",
                    "git diff --check"
                )
            }
        }
        if ((Get-FullRank ([string]$selected.full)) -gt (Get-FullRank $full)) {
            $full = [string]$selected.full
        }
        if ((Get-SmokeRank ([string]$selected.smoke)) -gt (Get-SmokeRank $smoke)) {
            $smoke = [string]$selected.smoke
        }
        if ((Get-CandidateRank ([string]$selected.candidate)) -gt (Get-CandidateRank $candidate)) {
            $candidate = [string]$selected.candidate
        }
        foreach ($item in @($selected.commands)) {
            $value = [string]$item
            if ($seen.Add($value)) { $commands.Add($value) }
        }
        $rows += [pscustomobject]@{
            path = $file
            class = [string]$selected.id
            full = [string]$selected.full
            smoke = [string]$selected.smoke
            candidate = [string]$selected.candidate
        }
    }

    $strictAudit = "pwsh -NoProfile -File tools/dev.ps1 audit -Strict"
    $plainAudit = "pwsh -NoProfile -File tools/dev.ps1 audit"
    if ($seen.Contains($strictAudit) -and $seen.Contains($plainAudit)) {
        [void]$commands.Remove($plainAudit)
    }

    [ordered]@{
        schema_version = 1
        generated_utc = [DateTime]::UtcNow.ToString("o")
        base_ref = $BaseRef
        source_sha = Get-RepoGitText -GitArgs @("rev-parse", "HEAD")
        changed_file_count = $files.Count
        changed_files = $rows
        full = $full
        smoke = $smoke
        candidate = $candidate
        commands = @($commands)
    }
}

function Invoke-DevelopmentAudit {
    $errors = [Collections.Generic.List[string]]::new()
    $warnings = [Collections.Generic.List[string]]::new()
    $allowed = @($Policy.launchers.allowed_root | ForEach-Object { [string]$_ })
    $legacy = @($Policy.launchers.legacy_root | ForEach-Object { [string]$_.path })
    $known = @($allowed + $legacy)
    $rootFiles = @(Get-ChildItem -LiteralPath $RepoRoot -File -Force)
    $launchers = @($rootFiles |
        Where-Object { $_.Name -match "^run_.*\.(bat|cmd|ps1)$" } |
        Select-Object -ExpandProperty Name |
        Sort-Object)

    foreach ($launcher in $launchers) {
        if ($known -notcontains $launcher) { $errors.Add("Unapproved root launcher: $launcher") }
    }
    foreach ($file in $rootFiles) {
        foreach ($pattern in @($Policy.launchers.deny_new_root_patterns)) {
            if ($file.Name -match [string]$pattern -and $known -notcontains $file.Name) {
                $errors.Add("Denied root entrypoint variant: $($file.Name)")
                break
            }
        }
    }
    foreach ($launcher in $allowed) {
        if (-not (Test-Path -LiteralPath (Join-Path $RepoRoot $launcher) -PathType Leaf)) {
            $errors.Add("Required launcher missing: $launcher")
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
    $trackedExe = Get-RepoGitText -GitArgs @("ls-files", "--", "*.exe")
    if ($trackedExe) {
        foreach ($path in $trackedExe -split "`r?`n") {
            if ($path) { $errors.Add("EXE committed to Git: $path") }
        }
    }

    $worktrees = $null
    if (-not $Ci) {
        $worktrees = Get-WorktreeCount
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
    if ($Json) {
        $result | ConvertTo-Json -Depth 8
    } else {
        Write-Host "Development policy audit: $($result.status)"
        Write-Host "Canonical app: $($result.canonical_binary)"
        Write-Host "Launchers: $($launchers -join ', ')"
        foreach ($warning in $warnings) { Write-Warning $warning }
        foreach ($auditError in $errors) { Write-Host "ERROR: $auditError" -ForegroundColor Red }
    }
    if ($errors.Count -gt 0) { exit 1 }
    if ($Strict -and $warnings.Count -gt 0) {
        Write-Host "Declared migration debt remains warning-only; new violations fail."
    }
}

function Start-DevelopmentSession {
    if (-not $Task) { throw "session-start requires -Task" }
    $started = [DateTime]::UtcNow
    $snapshot = Get-DevelopmentSnapshot
    $slug = [regex]::Replace($Task.ToLowerInvariant(), "[^a-z0-9]+", "-").Trim("-")
    if (-not $slug) { $slug = "task" }
    if (-not $SessionId) {
        $SessionId = "{0}-{1}-{2}" -f $started.ToString("yyyyMMddTHHmmssfffZ"), $slug, ([string]$snapshot.source_sha).Substring(0, 8)
    }
    if ($SessionId -notmatch "^[A-Za-z0-9][A-Za-z0-9._-]*$") { throw "Unsafe SessionId: $SessionId" }
    $directory = Join-Path (Get-SessionsRoot) $SessionId
    New-Item -ItemType Directory -Path $directory -ErrorAction Stop | Out-Null
    "started_utc,ended_utc,elapsed_seconds,exit_code,category,argv_json" |
        Set-Content -LiteralPath (Join-Path $directory "COMMAND_TIMINGS.csv") -Encoding UTF8
    Add-JsonLine -Path (Join-Path $directory "SESSION.jsonl") -Value ([ordered]@{
        event = "session_start"
        session_id = $SessionId
        task = $Task
        timing_confidence = "observed"
        started_utc = $started.ToString("o")
        snapshot = $snapshot
    })
    Write-Host "SESSION_ID=$SessionId"
    Write-Host "SESSION_DIR=$directory"
}

function Add-DevelopmentSpan {
    if (-not $SessionId) { throw "session-span requires -SessionId" }
    if (-not $Name) { throw "session-span requires -Name" }
    if ($DurationSeconds -lt 0) { throw "DurationSeconds must be nonnegative" }
    $directory = Get-SessionDirectory -Id $SessionId
    Add-JsonLine -Path (Join-Path $directory "SESSION.jsonl") -Value ([ordered]@{
        event = "span"
        name = $Name
        duration_seconds = [Math]::Round($DurationSeconds, 6)
        recorded_utc = [DateTime]::UtcNow.ToString("o")
    })
}

function Measure-DevelopmentCommand {
    if (-not $SessionId) { throw "measure requires -SessionId" }
    $directory = Get-SessionDirectory -Id $SessionId
    $argv = @($RemainingArgs)
    if ($argv.Count -gt 0 -and $argv[0] -eq "--") { $argv = @($argv | Select-Object -Skip 1) }
    if ($argv.Count -eq 0) { throw "measure requires a command after --" }
    $executable = $argv[0]
    $arguments = $(if ($argv.Count -gt 1) { @($argv[1..($argv.Count - 1)]) } else { @() })
    $started = [DateTime]::UtcNow
    $watch = [Diagnostics.Stopwatch]::StartNew()
    & $executable @arguments
    $rc = $LASTEXITCODE
    $watch.Stop()
    $ended = [DateTime]::UtcNow
    [pscustomobject]@{
        started_utc = $started.ToString("o")
        ended_utc = $ended.ToString("o")
        elapsed_seconds = [Math]::Round($watch.Elapsed.TotalSeconds, 6)
        exit_code = $rc
        category = $Category
        argv_json = ConvertTo-Json -InputObject (@($executable) + $arguments) -Compress
    } | Export-Csv -LiteralPath (Join-Path $directory "COMMAND_TIMINGS.csv") -Append -NoTypeInformation -Encoding UTF8
    Add-JsonLine -Path (Join-Path $directory "SESSION.jsonl") -Value ([ordered]@{
        event = "command"
        category = $Category
        argv = @($executable) + $arguments
        started_utc = $started.ToString("o")
        ended_utc = $ended.ToString("o")
        duration_seconds = [Math]::Round($watch.Elapsed.TotalSeconds, 6)
        exit_code = $rc
    })
    exit $rc
}

function Stop-DevelopmentSession {
    if (-not $SessionId) { throw "session-end requires -SessionId" }
    $directory = Get-SessionDirectory -Id $SessionId
    $events = @(Get-Content -LiteralPath (Join-Path $directory "SESSION.jsonl") -Encoding UTF8 |
        ForEach-Object { $_ | ConvertFrom-Json })
    $start = @($events | Where-Object { $_.event -eq "session_start" } | Select-Object -First 1)
    if ($start.Count -eq 0) { throw "Missing session_start" }
    $startUtc = [DateTime]::Parse([string]$start[0].started_utc).ToUniversalTime()
    $endUtc = [DateTime]::UtcNow
    $commands = @($events | Where-Object { $_.event -eq "command" })
    $spans = @($events | Where-Object { $_.event -eq "span" })
    $commandSum = ($commands | Measure-Object duration_seconds -Sum).Sum
    $spanSum = ($spans | Measure-Object duration_seconds -Sum).Sum
    $commandSeconds = $(if ($null -eq $commandSum) { 0.0 } else { [double]$commandSum })
    $spanSeconds = $(if ($null -eq $spanSum) { 0.0 } else { [double]$spanSum })
    $fullCount = @($commands | Where-Object { ($_.argv -join " ") -match "cargo test --workspace" }).Count
    $candidateCount = @($commands | Where-Object {
        $text = $_.argv -join " "
        $text -match "run_experiment\.bat" -and $text -notmatch "--mode scratch"
    }).Count
    $finalSnapshot = Get-DevelopmentSnapshot
    $initialSnapshot = $start[0].snapshot
    $longest = @($commands |
        Sort-Object duration_seconds -Descending |
        Select-Object -First 5 |
        ForEach-Object {
            [ordered]@{
                seconds = [double]$_.duration_seconds
                exit_code = [int]$_.exit_code
                command = ($_.argv -join " ")
            }
        })
    $phaseTotals = @{}
    foreach ($span in $spans) {
        $key = [string]$span.name
        if (-not $phaseTotals.ContainsKey($key)) { $phaseTotals[$key] = 0.0 }
        $phaseTotals[$key] += [double]$span.duration_seconds
    }
    $summary = [ordered]@{
        schema_version = 1
        session_id = $SessionId
        task = [string]$start[0].task
        started_utc = $startUtc.ToString("o")
        ended_utc = $endUtc.ToString("o")
        wall_seconds = [Math]::Round(($endUtc - $startUtc).TotalSeconds, 3)
        command_seconds = [Math]::Round($commandSeconds, 3)
        recorded_phase_seconds = [Math]::Round($spanSeconds, 3)
        phase_totals = $phaseTotals
        full_count = $fullCount
        candidate_count = $candidateCount
        longest_commands = $longest
        initial_snapshot = $initialSnapshot
        final_snapshot = $finalSnapshot
        target_delta_bytes = [int64]$finalSnapshot.target_bytes - [int64]$initialSnapshot.target_bytes
        artifact_delta_bytes = [int64]$finalSnapshot.artifact_bytes - [int64]$initialSnapshot.artifact_bytes
    }
    $summary | ConvertTo-Json -Depth 14 |
        Set-Content -LiteralPath (Join-Path $directory "SUMMARY.json") -Encoding UTF8
    $markdown = @(
        "# Development Session Summary",
        "",
        "- Session: ``$SessionId``",
        "- Task: $($summary.task)",
        "- Wall: $($summary.wall_seconds) s",
        "- Command time: $($summary.command_seconds) s",
        "- FULL count: $fullCount",
        "- Candidate count: $candidateCount",
        "- Target delta: $($summary.target_delta_bytes) bytes",
        "- Artifact delta: $($summary.artifact_delta_bytes) bytes",
        "",
        "## Longest commands",
        ""
    )
    foreach ($item in $longest) {
        $markdown += "- $($item.seconds) s · exit $($item.exit_code) · ``$($item.command)``"
    }
    $markdown -join [Environment]::NewLine |
        Set-Content -LiteralPath (Join-Path $directory "SUMMARY.md") -Encoding UTF8
    Add-JsonLine -Path (Join-Path $directory "SESSION.jsonl") -Value ([ordered]@{
        event = "session_end"
        ended_utc = $endUtc.ToString("o")
        wall_seconds = $summary.wall_seconds
        full_count = $fullCount
        candidate_count = $candidateCount
    })
    Write-Host "Session complete: $directory"
}

switch ($Command) {
    "audit" { Invoke-DevelopmentAudit }
    "validation-plan" {
        $plan = Get-ValidationPlan
        if ($Json) {
            $plan | ConvertTo-Json -Depth 12
        } else {
            Write-Host "Changed files: $($plan.changed_file_count)"
            Write-Host "FULL: $($plan.full) | Smoke: $($plan.smoke) | Candidate: $($plan.candidate)"
            foreach ($row in $plan.changed_files) {
                Write-Host "- $($row.path) -> $($row.class) (FULL $($row.full))"
            }
            Write-Host "Suggested commands:"
            foreach ($item in $plan.commands) { Write-Host "  $item" }
        }
    }
    "session-start" { Start-DevelopmentSession }
    "session-span" { Add-DevelopmentSpan }
    "measure" { Measure-DevelopmentCommand }
    "session-end" { Stop-DevelopmentSession }
}
