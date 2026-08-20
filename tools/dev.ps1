[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [ValidateSet("audit", "validation-plan", "g8c-matrix", "session-start", "session-span", "session-phase-start", "session-phase-end", "measure", "session-end")]
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

function Test-DevelopmentTimerTestMode {
    [Environment]::GetEnvironmentVariable("POWDERGAME_DEV_TIMER_TEST_MODE", "Process") -eq "1"
}

function Get-DevelopmentUtcNow {
    $testValue = [Environment]::GetEnvironmentVariable("POWDERGAME_DEV_TEST_UTC_NOW", "Process")
    if (-not [string]::IsNullOrWhiteSpace($testValue)) {
        if (-not (Test-DevelopmentTimerTestMode)) {
            throw "POWDERGAME_DEV_TEST_UTC_NOW requires POWDERGAME_DEV_TIMER_TEST_MODE=1"
        }
        $parsed = [DateTimeOffset]::Parse(
            $testValue,
            [Globalization.CultureInfo]::InvariantCulture,
            [Globalization.DateTimeStyles]::RoundtripKind
        )
        return $parsed.ToUniversalTime()
    }
    [DateTimeOffset]::UtcNow
}

function Get-DevelopmentMonotonicTick {
    $testValue = [Environment]::GetEnvironmentVariable("POWDERGAME_DEV_TEST_MONOTONIC_TICK", "Process")
    if (-not [string]::IsNullOrWhiteSpace($testValue)) {
        if (-not (Test-DevelopmentTimerTestMode)) {
            throw "POWDERGAME_DEV_TEST_MONOTONIC_TICK requires POWDERGAME_DEV_TIMER_TEST_MODE=1"
        }
        return [int64]::Parse($testValue, [Globalization.CultureInfo]::InvariantCulture)
    }
    [Diagnostics.Stopwatch]::GetTimestamp()
}

function Get-DevelopmentStopwatchFrequency {
    $testValue = [Environment]::GetEnvironmentVariable("POWDERGAME_DEV_TEST_STOPWATCH_FREQUENCY", "Process")
    if (-not [string]::IsNullOrWhiteSpace($testValue)) {
        if (-not (Test-DevelopmentTimerTestMode)) {
            throw "POWDERGAME_DEV_TEST_STOPWATCH_FREQUENCY requires POWDERGAME_DEV_TIMER_TEST_MODE=1"
        }
        $frequency = [int64]::Parse($testValue, [Globalization.CultureInfo]::InvariantCulture)
        if ($frequency -le 0) { throw "Test stopwatch frequency must be positive" }
        return $frequency
    }
    [Diagnostics.Stopwatch]::Frequency
}

function Format-DevelopmentUtc {
    param([Parameter(Mandatory)][DateTimeOffset]$Value)
    $Value.ToUniversalTime().ToString(
        "yyyy-MM-dd'T'HH:mm:ss.fffffff'Z'",
        [Globalization.CultureInfo]::InvariantCulture
    )
}

function ConvertTo-DevelopmentUtc {
    param([Parameter(Mandatory)]$Value)
    if ($Value -is [DateTimeOffset]) {
        return ([DateTimeOffset]$Value).ToUniversalTime()
    }
    if ($Value -is [DateTime]) {
        $dateTime = [DateTime]$Value
        if ($dateTime.Kind -eq [DateTimeKind]::Unspecified) {
            throw "UTC timestamp lost its offset: $Value"
        }
        return ([DateTimeOffset]$dateTime).ToUniversalTime()
    }
    $text = [string]$Value
    if ($text -notmatch "(?:Z|[+-][0-9]{2}:[0-9]{2})$") {
        throw "Timestamp lacks an RFC3339 offset: $text"
    }
    [DateTimeOffset]::Parse(
        $text,
        [Globalization.CultureInfo]::InvariantCulture,
        [Globalization.DateTimeStyles]::RoundtripKind
    ).ToUniversalTime()
}

function Invoke-RepoGit {
    param(
        [Parameter(Mandatory)]
        [string[]]$GitArgs,
        [switch]$AllowFailure
    )
    $safe = $RepoRoot.Replace("\", "/")
    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $GitExe
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    foreach ($argument in @("-c", "safe.directory=$safe") + $GitArgs) {
        [void]$startInfo.ArgumentList.Add($argument)
    }
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    try {
        if (-not $process.Start()) { throw "Unable to start git process" }
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        $process.WaitForExit()
        $text = $stdoutTask.GetAwaiter().GetResult().TrimEnd()
        $errorText = $stderrTask.GetAwaiter().GetResult().TrimEnd()
        $rc = $process.ExitCode
    } finally {
        $process.Dispose()
    }
    $testError = [Environment]::GetEnvironmentVariable("POWDERGAME_DEV_TEST_GIT_STDERR", "Process")
    if (-not [string]::IsNullOrWhiteSpace($testError)) {
        if (-not (Test-DevelopmentTimerTestMode)) {
            throw "POWDERGAME_DEV_TEST_GIT_STDERR requires POWDERGAME_DEV_TIMER_TEST_MODE=1"
        }
        $errorText = @($errorText, $testError) |
            Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
            Join-String -Separator ([Environment]::NewLine)
    }
    if ($rc -ne 0 -and -not $AllowFailure) {
        $diagnostic = @($errorText, $text) |
            Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
            Join-String -Separator ([Environment]::NewLine)
        throw "git $($GitArgs -join ' ') failed ($rc)`n$diagnostic"
    }
    [pscustomobject]@{ ExitCode = $rc; Text = $text; ErrorText = $errorText }
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
        timestamp_utc = Format-DevelopmentUtc (Get-DevelopmentUtcNow)
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
    $override = [Environment]::GetEnvironmentVariable("POWDERGAME_DEV_TEST_SESSIONS_ROOT", "Process")
    if (-not [string]::IsNullOrWhiteSpace($override)) {
        if (-not (Test-DevelopmentTimerTestMode)) {
            throw "POWDERGAME_DEV_TEST_SESSIONS_ROOT requires POWDERGAME_DEV_TIMER_TEST_MODE=1"
        }
        $path = [IO.Path]::GetFullPath($override)
    } else {
        $path = Join-Path ([string]$Policy.artifacts.root) ([string]$Policy.artifacts.session_subdir)
    }
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
        throw "Invalid BaseRef: $FromRef`n$($verified.ErrorText)"
    }
    $result = Invoke-RepoGit -GitArgs @("diff", "--name-only", "--diff-filter=ACMRD", "$FromRef...HEAD") -AllowFailure
    if ($result.ExitCode -ne 0) {
        throw "Unable to compare BaseRef $FromRef with HEAD`n$($result.ErrorText)"
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
        generated_utc = Format-DevelopmentUtc (Get-DevelopmentUtcNow)
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

function Invoke-LauncherAuditProbe {
    param(
        [Parameter(Mandatory)][string]$LauncherPath,
        [string[]]$LauncherArgs = @(),
        [Parameter(Mandatory)][string]$ProbeEnvironment,
        [Parameter(Mandatory)][string]$ProbeNonceEnvironment,
        [Parameter(Mandatory)][string]$ProbeNonce,
        [Parameter(Mandatory)][int]$TimeoutMs
    )

    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $(if ($env:ComSpec) { $env:ComSpec } else { "cmd.exe" })
    $startInfo.WorkingDirectory = $RepoRoot
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $startInfo.Environment[$ProbeEnvironment] = $ProbeNonce
    $startInfo.Environment[$ProbeNonceEnvironment] = $ProbeNonce
    [void]$startInfo.ArgumentList.Add("/d")
    [void]$startInfo.ArgumentList.Add("/c")
    [void]$startInfo.ArgumentList.Add($LauncherPath)
    foreach ($argument in $LauncherArgs) {
        [void]$startInfo.ArgumentList.Add([string]$argument)
    }

    $process = [Diagnostics.Process]::Start($startInfo)
    try {
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        if (-not $process.WaitForExit($TimeoutMs)) {
            try {
                $process.Kill($true)
            } finally {
                $process.WaitForExit()
            }
            [void]$stdoutTask.GetAwaiter().GetResult()
            [void]$stderrTask.GetAwaiter().GetResult()
            throw "Launcher audit probe timed out after $TimeoutMs ms"
        }
        $process.WaitForExit()
        $stdout = $stdoutTask.GetAwaiter().GetResult()
        $stderr = $stderrTask.GetAwaiter().GetResult()
        [pscustomobject]@{
            exit_code = $process.ExitCode
            stdout = $stdout
            stderr = $stderr
        }
    } finally {
        $process.Dispose()
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
    $canonicalLauncherPath = Join-Path $RepoRoot ([string]$Policy.canonical_app.normal_launcher)
    if ($null -eq $Policy.launcher_contract) {
        $errors.Add("Canonical launcher contract missing from development policy")
    } elseif (Test-Path -LiteralPath $canonicalLauncherPath -PathType Leaf) {
        $contract = $Policy.launcher_contract
        $requiredAliases = [ordered]@{
            normal = "--benchmark-gallery"
            gallery = "--benchmark-gallery"
            runtime = "--runtime-baseline"
            g0 = "--runtime-baseline"
            movement = "--movement-demo"
            density = "--density-demo"
            thermal = "--thermal-demo"
            "thermal-environment" = "--thermal-environment-candidate"
            te2 = "--thermal-environment-candidate"
            pressure = "--pressure-demo"
            "parallel-integrity" = "--parallel-integrity-demo"
            activity = "--activity-demo"
            sandbox = "--sandbox"
            play = "--sandbox"
        }
        if ([string]$contract.default_args -cne "--sandbox") {
            $errors.Add("Canonical launcher policy default must be --sandbox")
        }
        $declaredAliases = @($contract.aliases.PSObject.Properties)
        if ($declaredAliases.Count -ne $requiredAliases.Count) {
            $errors.Add("Canonical launcher policy must declare exactly $($requiredAliases.Count) aliases")
        }
        foreach ($requiredAlias in $requiredAliases.Keys) {
            $matches = @($declaredAliases | Where-Object { $_.Name -ceq $requiredAlias })
            if ($matches.Count -ne 1 -or [string]$matches[0].Value -cne [string]$requiredAliases[$requiredAlias]) {
                $errors.Add("Canonical launcher policy mapping missing or changed: $requiredAlias -> $($requiredAliases[$requiredAlias])")
            }
        }
        if ([string]$contract.raw_cli_prefix -cne "--") {
            $errors.Add("Canonical launcher raw CLI prefix must remain --")
        }
        $requiredRawProbeArgs = @("--smoke-frames", "3", "--benchmark-gallery")
        $declaredRawProbeArgs = @($contract.raw_cli_probe_args | ForEach-Object { [string]$_ })
        if ($declaredRawProbeArgs.Count -ne $requiredRawProbeArgs.Count -or
            ($declaredRawProbeArgs -join "`0") -cne ($requiredRawProbeArgs -join "`0")) {
            $errors.Add("Canonical launcher raw CLI probe must preserve the three-argument passthrough contract")
        }
        if ([int]$contract.invalid_alias_exit_code -ne 2) {
            $errors.Add("Canonical launcher invalid alias exit code must remain 2")
        }
        if ([string]$contract.probe_environment -cne "POWDERGAME_LAUNCHER_AUDIT_ONLY" -or
            [string]$contract.probe_nonce_environment -cne "POWDERGAME_LAUNCHER_AUDIT_NONCE") {
            $errors.Add("Canonical launcher audit must retain its two-variable nonce guard")
        }
        if ([int]$contract.probe_timeout_ms -ne 5000) {
            $errors.Add("Canonical launcher audit probe timeout must remain 5000 ms")
        }
        if ([string]$contract.probe_stdout_prefix -cne "POWDERGAME_LAUNCHER_AUDIT_ARGS=") {
            $errors.Add("Canonical launcher audit stdout prefix changed")
        }
        $requiredUsageTerms = @(
            "default = G9-A first playable Sandbox",
            "normal/gallery = G8-B Benchmark Gallery",
            "sandbox/play = G9-A first playable Sandbox",
            "thermal-environment/te2 = TE-2 passive Thermal Environment candidate",
            "runtime/g0 = technical empty G0 baseline"
        )
        $declaredUsageTerms = @($contract.usage_required_terms | ForEach-Object { [string]$_ })
        foreach ($requiredUsageTerm in $requiredUsageTerms) {
            if ($declaredUsageTerms -cnotcontains $requiredUsageTerm) {
                $errors.Add("Canonical launcher usage requirement missing from policy: $requiredUsageTerm")
            }
        }
        $probeEnvironment = [string]$contract.probe_environment
        $probeNonceEnvironment = [string]$contract.probe_nonce_environment
        $probeNonce = [Guid]::NewGuid().ToString("N")
        $probeTimeoutMs = [int]$contract.probe_timeout_ms
        $probePrefix = [string]$contract.probe_stdout_prefix
        $launcherText = Get-Content -LiteralPath $canonicalLauncherPath -Raw
        $probeHookSnippets = @(
            "if defined $probeEnvironment (",
            ('if "%{0}%"=="%{1}%" goto launcher_audit' -f $probeEnvironment, $probeNonceEnvironment),
            ":launcher_audit",
            "echo $probePrefix%APP_ARGS%"
        )
        $probeHookReady = $true
        foreach ($snippet in $probeHookSnippets) {
            if ($launcherText.IndexOf($snippet, [StringComparison]::Ordinal) -lt 0) {
                $errors.Add("Canonical launcher audit hook missing: $snippet")
                $probeHookReady = $false
            }
        }
        $successCases = [Collections.Generic.List[object]]::new()
        $successCases.Add([pscustomobject]@{
            name = "no-argument"
            args = @()
            expected_args = [string]$contract.default_args
        })
        foreach ($alias in $contract.aliases.PSObject.Properties) {
            $successCases.Add([pscustomobject]@{
                name = "alias:$($alias.Name)"
                args = @($alias.Name)
                expected_args = [string]$alias.Value
            })
        }
        $rawArgs = $declaredRawProbeArgs
        if ($rawArgs.Count -eq 0 -or -not $rawArgs[0].StartsWith([string]$contract.raw_cli_prefix)) {
            $errors.Add("Canonical launcher raw CLI probe is missing or does not use the declared prefix")
        } else {
            $successCases.Add([pscustomobject]@{
                name = "raw-cli"
                args = $rawArgs
                expected_args = $rawArgs -join " "
            })
        }
        $requiredSandboxSmokeProbeArgs = @("sandbox", "--smoke-frames", "3")
        $declaredSandboxSmokeProbeArgs = @($contract.sandbox_smoke_probe_args | ForEach-Object { [string]$_ })
        if ($declaredSandboxSmokeProbeArgs.Count -ne $requiredSandboxSmokeProbeArgs.Count -or
            ($declaredSandboxSmokeProbeArgs -join "`0") -cne ($requiredSandboxSmokeProbeArgs -join "`0")) {
            $errors.Add("Canonical launcher Sandbox smoke probe must remain sandbox --smoke-frames 3")
        } else {
            $successCases.Add([pscustomobject]@{
                name = "sandbox-smoke"
                args = $declaredSandboxSmokeProbeArgs
                expected_args = "--sandbox --smoke-frames 3"
            })
        }

        foreach ($case in $(if ($probeHookReady) { $successCases } else { @() })) {
            try {
                $probe = Invoke-LauncherAuditProbe `
                    -LauncherPath $canonicalLauncherPath `
                    -LauncherArgs @($case.args) `
                    -ProbeEnvironment $probeEnvironment `
                    -ProbeNonceEnvironment $probeNonceEnvironment `
                    -ProbeNonce $probeNonce `
                    -TimeoutMs $probeTimeoutMs
                $stdout = $probe.stdout.TrimEnd("`r", "`n")
                $expectedStdout = "$probePrefix$($case.expected_args)"
                if ($probe.exit_code -ne 0) {
                    $errors.Add("Canonical launcher probe $($case.name) exited $($probe.exit_code), expected 0")
                }
                if ($stdout -cne $expectedStdout) {
                    $errors.Add("Canonical launcher probe $($case.name) stdout mismatch: '$stdout'")
                }
                if ($probe.stderr.Length -ne 0) {
                    $errors.Add("Canonical launcher probe $($case.name) wrote stderr: '$($probe.stderr.Trim())'")
                }
            } catch {
                $errors.Add("Canonical launcher probe $($case.name) failed to execute: $($_.Exception.Message)")
                break
            }
        }

        try {
            if (-not $probeHookReady) { throw "Launcher audit hook preflight failed" }
            $invalidProbe = Invoke-LauncherAuditProbe `
                -LauncherPath $canonicalLauncherPath `
                -LauncherArgs @([string]$contract.invalid_alias_probe) `
                -ProbeEnvironment $probeEnvironment `
                -ProbeNonceEnvironment $probeNonceEnvironment `
                -ProbeNonce $probeNonce `
                -TimeoutMs $probeTimeoutMs
            if ($invalidProbe.exit_code -ne [int]$contract.invalid_alias_exit_code) {
                $errors.Add("Canonical launcher invalid-alias probe exited $($invalidProbe.exit_code), expected $($contract.invalid_alias_exit_code)")
            }
            if ($invalidProbe.stdout.Length -ne 0) {
                $errors.Add("Canonical launcher invalid-alias probe wrote stdout: '$($invalidProbe.stdout.Trim())'")
            }
            if ($invalidProbe.stderr.IndexOf("Usage:", [StringComparison]::OrdinalIgnoreCase) -lt 0) {
                $errors.Add("Canonical launcher invalid-alias probe did not write Usage to stderr")
            }
            foreach ($term in @($contract.usage_required_terms)) {
                if ($invalidProbe.stderr.IndexOf([string]$term, [StringComparison]::OrdinalIgnoreCase) -lt 0) {
                    $errors.Add("Canonical launcher invalid-alias stderr missing: $term")
                }
            }
        } catch {
            $errors.Add("Canonical launcher invalid-alias probe failed to execute: $($_.Exception.Message)")
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

function Get-RequiredEventProperty {
    param(
        [Parameter(Mandatory)]$Event,
        [Parameter(Mandatory)][string]$PropertyName,
        [Parameter(Mandatory)][string]$Context
    )
    $property = $Event.PSObject.Properties[$PropertyName]
    if ($null -eq $property -or $null -eq $property.Value) {
        throw "$Context is missing $PropertyName"
    }
    $property.Value
}

function Get-DevelopmentSessionEvents {
    param([Parameter(Mandatory)][string]$Directory)
    $path = Join-Path $Directory "SESSION.jsonl"
    $events = [Collections.Generic.List[object]]::new()
    $lineNumber = 0
    foreach ($line in Get-Content -LiteralPath $path -Encoding UTF8) {
        $lineNumber += 1
        if ([string]::IsNullOrWhiteSpace($line)) { continue }
        try {
            $events.Add(($line | ConvertFrom-Json -ErrorAction Stop))
        } catch {
            throw "MALFORMED_SESSION_EVENT: line $lineNumber is not valid JSON: $($_.Exception.Message)"
        }
    }
    @($events)
}

function Measure-DevelopmentIntervalUnionTicks {
    param([object[]]$Intervals)
    $sorted = @($Intervals | Sort-Object start_tick, end_tick)
    if ($sorted.Count -eq 0) { return [int64]0 }
    [int64]$currentStart = $sorted[0].start_tick
    [int64]$currentEnd = $sorted[0].end_tick
    [int64]$total = 0
    foreach ($interval in @($sorted | Select-Object -Skip 1)) {
        [int64]$nextStart = $interval.start_tick
        [int64]$nextEnd = $interval.end_tick
        if ($nextStart -le $currentEnd) {
            if ($nextEnd -gt $currentEnd) { $currentEnd = $nextEnd }
        } else {
            $total += $currentEnd - $currentStart
            $currentStart = $nextStart
            $currentEnd = $nextEnd
        }
    }
    $total + ($currentEnd - $currentStart)
}

function Get-DevelopmentPhaseTimingState {
    param(
        [Parameter(Mandatory)][object[]]$Events,
        [switch]$AllowOpen
    )
    $activeById = @{}
    $activeNameToId = @{}
    $intervals = [Collections.Generic.List[object]]::new()
    foreach ($event in $Events) {
        if ($event.event -eq "phase_start") {
            $id = [string](Get-RequiredEventProperty $event "phase_id" "phase_start")
            $name = [string](Get-RequiredEventProperty $event "name" "phase_start")
            if ($activeById.ContainsKey($id) -or $activeNameToId.ContainsKey($name)) {
                throw "DUPLICATE_PHASE: phase '$name' is already open"
            }
            $activeById[$id] = $event
            $activeNameToId[$name] = $id
        } elseif ($event.event -eq "phase_end") {
            $id = [string](Get-RequiredEventProperty $event "phase_id" "phase_end")
            $name = [string](Get-RequiredEventProperty $event "name" "phase_end")
            if (-not $activeById.ContainsKey($id)) {
                throw "PHASE_NOT_OPEN: phase '$name' has no matching start"
            }
            $started = $activeById[$id]
            if ([string]$started.name -ne $name) {
                throw "MALFORMED_PHASE: phase '$name' does not match its start"
            }
            [int64]$startTick = Get-RequiredEventProperty $started "stopwatch_start_tick" "phase_start '$name'"
            [int64]$endTick = Get-RequiredEventProperty $event "stopwatch_end_tick" "phase_end '$name'"
            [int64]$startFrequency = Get-RequiredEventProperty $started "stopwatch_frequency" "phase_start '$name'"
            [int64]$endFrequency = Get-RequiredEventProperty $event "stopwatch_frequency" "phase_end '$name'"
            if ($startFrequency -le 0 -or $startFrequency -ne $endFrequency -or $endTick -lt $startTick) {
                throw "MALFORMED_PHASE: phase '$name' has invalid monotonic timing"
            }
            $intervals.Add([pscustomobject]@{
                kind = "phase"
                name = $name
                start_tick = $startTick
                end_tick = $endTick
                duration_seconds = [double]($endTick - $startTick) / [double]$startFrequency
            })
            $activeById.Remove($id)
            $activeNameToId.Remove($name)
        }
    }
    if (-not $AllowOpen -and $activeById.Count -gt 0) {
        $names = @($activeById.Values | ForEach-Object { [string]$_.name } | Sort-Object)
        throw "OPEN_PHASE: session has unterminated phase(s): $($names -join ', ')"
    }
    [pscustomobject]@{
        intervals = @($intervals)
        open_phases = @($activeById.Values)
    }
}

function Write-DevelopmentSessionError {
    param(
        [Parameter(Mandatory)][string]$Directory,
        [Parameter(Mandatory)][string]$Code,
        [Parameter(Mandatory)][string]$Message,
        [Parameter(Mandatory)][DateTimeOffset]$EndUtc,
        [Parameter(Mandatory)][int64]$EndTick
    )
    $value = [ordered]@{
        status = "ERROR"
        error = $Code
        message = $Message
        end_utc = Format-DevelopmentUtc $EndUtc
        stopwatch_end_tick = $EndTick
    }
    $value | ConvertTo-Json -Depth 8 |
        Set-Content -LiteralPath (Join-Path $Directory "TIMER_ERROR.json") -Encoding UTF8
    Add-JsonLine -Path (Join-Path $Directory "SESSION.jsonl") -Value ([ordered]@{
        event = "session_error"
        error = $Code
        message = $Message
        end_utc = Format-DevelopmentUtc $EndUtc
        stopwatch_end_tick = $EndTick
    })
}

function Start-DevelopmentSession {
    if (-not $Task) { throw "session-start requires -Task" }
    $started = Get-DevelopmentUtcNow
    [int64]$startTick = Get-DevelopmentMonotonicTick
    [int64]$frequency = Get-DevelopmentStopwatchFrequency
    $snapshot = Get-DevelopmentSnapshot
    $slug = [regex]::Replace($Task.ToLowerInvariant(), "[^a-z0-9]+", "-").Trim("-")
    if (-not $slug) { $slug = "task" }
    if (-not $SessionId) {
        $SessionId = "{0}-{1}-{2}" -f $started.ToString("yyyyMMddTHHmmssfff'Z'"), $slug, ([string]$snapshot.source_sha).Substring(0, 8)
    }
    if ($SessionId -notmatch "^[A-Za-z0-9][A-Za-z0-9._-]*$") { throw "Unsafe SessionId: $SessionId" }
    $directory = Join-Path (Get-SessionsRoot) $SessionId
    New-Item -ItemType Directory -Path $directory -ErrorAction Stop | Out-Null
    "started_utc,ended_utc,stopwatch_start_tick,stopwatch_end_tick,stopwatch_frequency,elapsed_seconds,exit_code,category,argv_json" |
        Set-Content -LiteralPath (Join-Path $directory "COMMAND_TIMINGS.csv") -Encoding UTF8
    $timestamp = Format-DevelopmentUtc $started
    Add-JsonLine -Path (Join-Path $directory "SESSION.jsonl") -Value ([ordered]@{
        event = "session_start"
        schema_version = 2
        session_id = $SessionId
        task = $Task
        timing_confidence = "utc-and-monotonic"
        start_utc = $timestamp
        started_utc = $timestamp
        stopwatch_start_tick = $startTick
        monotonic_start_tick = $startTick
        stopwatch_frequency = $frequency
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
        recorded_utc = Format-DevelopmentUtc (Get-DevelopmentUtcNow)
        timing_note = "declared legacy span; excluded from interval-union coverage"
    })
}

function Start-DevelopmentPhase {
    if (-not $SessionId) { throw "session-phase-start requires -SessionId" }
    if ([string]::IsNullOrWhiteSpace($Name)) { throw "session-phase-start requires -Name" }
    if ($Name -match "[\r\n]") { throw "MALFORMED_PHASE: phase name contains a newline" }
    $directory = Get-SessionDirectory -Id $SessionId
    $events = @(Get-DevelopmentSessionEvents -Directory $directory)
    $state = Get-DevelopmentPhaseTimingState -Events $events -AllowOpen
    if (@($state.open_phases | Where-Object { [string]$_.name -eq $Name }).Count -gt 0) {
        throw "DUPLICATE_PHASE: phase '$Name' is already open"
    }
    $started = Get-DevelopmentUtcNow
    [int64]$tick = Get-DevelopmentMonotonicTick
    [int64]$frequency = Get-DevelopmentStopwatchFrequency
    $phaseId = [Guid]::NewGuid().ToString("N")
    Add-JsonLine -Path (Join-Path $directory "SESSION.jsonl") -Value ([ordered]@{
        event = "phase_start"
        phase_id = $phaseId
        name = $Name
        start_utc = Format-DevelopmentUtc $started
        stopwatch_start_tick = $tick
        stopwatch_frequency = $frequency
    })
    Write-Host "PHASE_ID=$phaseId"
    Write-Host "Phase started: $Name"
}

function Stop-DevelopmentPhase {
    if (-not $SessionId) { throw "session-phase-end requires -SessionId" }
    if ([string]::IsNullOrWhiteSpace($Name)) { throw "session-phase-end requires -Name" }
    $directory = Get-SessionDirectory -Id $SessionId
    $events = @(Get-DevelopmentSessionEvents -Directory $directory)
    $state = Get-DevelopmentPhaseTimingState -Events $events -AllowOpen
    $matches = @($state.open_phases | Where-Object { [string]$_.name -eq $Name })
    if ($matches.Count -eq 0) { throw "PHASE_NOT_OPEN: phase '$Name' has no matching start" }
    if ($matches.Count -gt 1) { throw "MALFORMED_PHASE: phase '$Name' has multiple starts" }
    $started = $matches[0]
    $ended = Get-DevelopmentUtcNow
    [int64]$tick = Get-DevelopmentMonotonicTick
    [int64]$frequency = Get-DevelopmentStopwatchFrequency
    if ([int64]$started.stopwatch_frequency -ne $frequency) {
        throw "MALFORMED_PHASE: stopwatch frequency changed during phase '$Name'"
    }
    if ($tick -lt [int64]$started.stopwatch_start_tick) {
        throw "MALFORMED_PHASE: monotonic time moved backwards during phase '$Name'"
    }
    Add-JsonLine -Path (Join-Path $directory "SESSION.jsonl") -Value ([ordered]@{
        event = "phase_end"
        phase_id = [string]$started.phase_id
        name = $Name
        end_utc = Format-DevelopmentUtc $ended
        stopwatch_end_tick = $tick
        stopwatch_frequency = $frequency
    })
    Write-Host "Phase ended: $Name"
}

function Measure-DevelopmentCommand {
    if (-not $SessionId) { throw "measure requires -SessionId" }
    $directory = Get-SessionDirectory -Id $SessionId
    $argv = @($RemainingArgs)
    if ($argv.Count -gt 0 -and $argv[0] -eq "--") { $argv = @($argv | Select-Object -Skip 1) }
    if ($argv.Count -eq 0) { throw "measure requires a command after --" }
    $executable = $argv[0]
    $arguments = $(if ($argv.Count -gt 1) { @($argv[1..($argv.Count - 1)]) } else { @() })
    $started = Get-DevelopmentUtcNow
    [int64]$startTick = Get-DevelopmentMonotonicTick
    [int64]$frequency = Get-DevelopmentStopwatchFrequency
    & $executable @arguments
    $rc = $LASTEXITCODE
    [int64]$endTick = Get-DevelopmentMonotonicTick
    $ended = Get-DevelopmentUtcNow
    $elapsed = [double]($endTick - $startTick) / [double]$frequency
    if ($elapsed -lt 0) { throw "Command monotonic duration is negative" }
    $startTimestamp = Format-DevelopmentUtc $started
    $endTimestamp = Format-DevelopmentUtc $ended
    [pscustomobject]@{
        started_utc = $startTimestamp
        ended_utc = $endTimestamp
        stopwatch_start_tick = $startTick
        stopwatch_end_tick = $endTick
        stopwatch_frequency = $frequency
        elapsed_seconds = [Math]::Round($elapsed, 6)
        exit_code = $rc
        category = $Category
        argv_json = ConvertTo-Json -InputObject (@($executable) + $arguments) -Compress
    } | Export-Csv -LiteralPath (Join-Path $directory "COMMAND_TIMINGS.csv") -Append -NoTypeInformation -Encoding UTF8
    Add-JsonLine -Path (Join-Path $directory "SESSION.jsonl") -Value ([ordered]@{
        event = "command"
        category = $Category
        argv = @($executable) + $arguments
        start_utc = $startTimestamp
        end_utc = $endTimestamp
        started_utc = $startTimestamp
        ended_utc = $endTimestamp
        stopwatch_start_tick = $startTick
        stopwatch_end_tick = $endTick
        stopwatch_frequency = $frequency
        duration_seconds = [Math]::Round($elapsed, 6)
        exit_code = $rc
    })
    exit $rc
}

function Stop-DevelopmentSession {
    if (-not $SessionId) { throw "session-end requires -SessionId" }
    $directory = Get-SessionDirectory -Id $SessionId
    if (Test-Path -LiteralPath (Join-Path $directory "SUMMARY.json")) {
        throw "Session already has a published summary: $SessionId"
    }
    $events = @(Get-DevelopmentSessionEvents -Directory $directory)
    $starts = @($events | Where-Object { $_.event -eq "session_start" })
    if ($starts.Count -ne 1) { throw "Session must contain exactly one session_start" }
    $start = $starts[0]
    if ([int](Get-RequiredEventProperty $start "schema_version" "session_start") -lt 2) {
        throw "UNSUPPORTED_LEGACY_TIMER: session_start lacks the UTC and monotonic v2 contract"
    }
    $startUtc = ConvertTo-DevelopmentUtc (Get-RequiredEventProperty $start "start_utc" "session_start")
    [int64]$startTick = Get-RequiredEventProperty $start "stopwatch_start_tick" "session_start"
    [int64]$frequency = Get-RequiredEventProperty $start "stopwatch_frequency" "session_start"
    $endUtc = Get-DevelopmentUtcNow
    [int64]$endTick = Get-DevelopmentMonotonicTick
    [int64]$endFrequency = Get-DevelopmentStopwatchFrequency
    if ($frequency -le 0 -or $endFrequency -ne $frequency -or $endTick -lt $startTick) {
        $message = "Session monotonic clock metadata is invalid or changed"
        Write-DevelopmentSessionError $directory "TIMER_INCONSISTENCY" $message $endUtc $endTick
        throw "TIMER_INCONSISTENCY: $message"
    }
    $wallUtc = ($endUtc - $startUtc).TotalSeconds
    $wallMonotonic = [double]($endTick - $startTick) / [double]$frequency
    $wallDifference = [Math]::Abs($wallUtc - $wallMonotonic)
    if ($wallUtc -lt 0 -or $wallDifference -gt 5.0) {
        $message = "UTC wall $([Math]::Round($wallUtc, 6)) s and monotonic wall $([Math]::Round($wallMonotonic, 6)) s differ by $([Math]::Round($wallDifference, 6)) s"
        Write-DevelopmentSessionError $directory "TIMER_INCONSISTENCY" $message $endUtc $endTick
        throw "TIMER_INCONSISTENCY: $message"
    }

    try {
        $phaseState = Get-DevelopmentPhaseTimingState -Events $events
    } catch {
        Write-DevelopmentSessionError $directory "SESSION_PHASE_ERROR" $_.Exception.Message $endUtc $endTick
        throw
    }
    $phaseIntervals = @($phaseState.intervals)
    $commands = @($events | Where-Object { $_.event -eq "command" })
    $commandIntervals = [Collections.Generic.List[object]]::new()
    foreach ($command in $commands) {
        [int64]$commandStart = Get-RequiredEventProperty $command "stopwatch_start_tick" "command"
        [int64]$commandEnd = Get-RequiredEventProperty $command "stopwatch_end_tick" "command"
        [int64]$commandFrequency = Get-RequiredEventProperty $command "stopwatch_frequency" "command"
        if ($commandFrequency -ne $frequency -or $commandStart -lt $startTick -or $commandEnd -lt $commandStart -or $commandEnd -gt $endTick) {
            throw "MALFORMED_COMMAND_TIMING: command interval is outside the session monotonic bounds"
        }
        $commandIntervals.Add([pscustomobject]@{
            kind = "command"
            name = [string]$command.category
            start_tick = $commandStart
            end_tick = $commandEnd
        })
    }
    foreach ($phase in $phaseIntervals) {
        if ($phase.start_tick -lt $startTick -or $phase.end_tick -gt $endTick) {
            throw "MALFORMED_PHASE: phase '$($phase.name)' is outside the session monotonic bounds"
        }
    }

    [int64]$commandUnionTicks = Measure-DevelopmentIntervalUnionTicks -Intervals @($commandIntervals)
    [int64]$phaseUnionTicks = Measure-DevelopmentIntervalUnionTicks -Intervals $phaseIntervals
    [int64]$classifiedUnionTicks = Measure-DevelopmentIntervalUnionTicks -Intervals (@($commandIntervals) + @($phaseIntervals))
    $measuredCommandSeconds = [double]$commandUnionTicks / [double]$frequency
    $measuredPhaseSeconds = [double]$phaseUnionTicks / [double]$frequency
    $classifiedSeconds = [double]$classifiedUnionTicks / [double]$frequency
    $unclassifiedSeconds = [Math]::Max(0.0, $wallMonotonic - $classifiedSeconds)
    $commandRatio = $(if ($wallMonotonic -gt 0) { $measuredCommandSeconds / $wallMonotonic } else { 0.0 })
    $phaseRatio = $(if ($wallMonotonic -gt 0) { $measuredPhaseSeconds / $wallMonotonic } else { 0.0 })

    $spans = @($events | Where-Object { $_.event -eq "span" })
    $legacySpanSeconds = $(if ($spans.Count -eq 0) {
        0.0
    } else {
        [double](($spans | Measure-Object duration_seconds -Sum).Sum)
    })
    $timingWarnings = @()
    if ($spans.Count -gt 0) {
        $timingWarnings += "Legacy session-span declarations are reported separately and excluded from interval-union coverage."
    }
    $fullCount = @($commands | Where-Object { ($_.argv -join " ") -match "cargo test --workspace" }).Count
    $candidateCount = @($commands | Where-Object {
        $text = $_.argv -join " "
        $text -match "run_experiment\.bat" -and $text -notmatch "--mode scratch"
    }).Count
    $finalSnapshot = Get-DevelopmentSnapshot
    $initialSnapshot = $start.snapshot
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
    $longestPhases = @($phaseIntervals |
        Sort-Object @{ Expression = "duration_seconds"; Descending = $true }, @{ Expression = "name"; Descending = $false } |
        Select-Object -First 5 |
        ForEach-Object {
            [ordered]@{
                seconds = [Math]::Round([double]$_.duration_seconds, 6)
                name = [string]$_.name
            }
        })
    $phaseTotals = [ordered]@{}
    foreach ($phase in @($phaseIntervals | Sort-Object name)) {
        $key = [string]$phase.name
        if (-not $phaseTotals.Contains($key)) { $phaseTotals[$key] = 0.0 }
        $phaseTotals[$key] += [double]$phase.duration_seconds
    }
    $startTimestamp = Format-DevelopmentUtc $startUtc
    $endTimestamp = Format-DevelopmentUtc $endUtc
    $summary = [ordered]@{
        schema_version = 2
        status = "PASS"
        session_id = $SessionId
        task = [string]$start.task
        start_utc = $startTimestamp
        end_utc = $endTimestamp
        started_utc = $startTimestamp
        ended_utc = $endTimestamp
        stopwatch_start_tick = $startTick
        stopwatch_end_tick = $endTick
        stopwatch_frequency = $frequency
        wall_seconds_utc = [Math]::Round($wallUtc, 6)
        wall_seconds_monotonic = [Math]::Round($wallMonotonic, 6)
        wall_clock_difference_seconds = [Math]::Round($wallDifference, 6)
        wall_seconds = [Math]::Round($wallMonotonic, 6)
        measured_command_seconds = [Math]::Round($measuredCommandSeconds, 6)
        measured_phase_seconds = [Math]::Round($measuredPhaseSeconds, 6)
        measured_classified_union_seconds = [Math]::Round($classifiedSeconds, 6)
        unclassified_seconds = [Math]::Round($unclassifiedSeconds, 6)
        command_to_wall_ratio = [Math]::Round($commandRatio, 6)
        phase_to_wall_ratio = [Math]::Round($phaseRatio, 6)
        command_seconds = [Math]::Round($measuredCommandSeconds, 6)
        recorded_phase_seconds = [Math]::Round($measuredPhaseSeconds, 6)
        legacy_declared_span_seconds = [Math]::Round($legacySpanSeconds, 6)
        phase_totals = $phaseTotals
        timing_warnings = $timingWarnings
        full_count = $fullCount
        candidate_count = $candidateCount
        longest_commands = $longest
        longest_phases = $longestPhases
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
        "- Status: $($summary.status)",
        "- Session: ``$SessionId``",
        "- Task: $($summary.task)",
        "- Wall (UTC): $($summary.wall_seconds_utc) s",
        "- Wall (monotonic): $($summary.wall_seconds_monotonic) s",
        "- Clock difference: $($summary.wall_clock_difference_seconds) s",
        "- Measured command union: $($summary.measured_command_seconds) s ($($summary.command_to_wall_ratio))",
        "- Measured phase union: $($summary.measured_phase_seconds) s ($($summary.phase_to_wall_ratio))",
        "- Unclassified: $($summary.unclassified_seconds) s",
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
    $markdown += @("", "## Longest phases", "")
    foreach ($item in $longestPhases) {
        $markdown += "- $($item.seconds) s · $($item.name)"
    }
    $markdown -join [Environment]::NewLine |
        Set-Content -LiteralPath (Join-Path $directory "SUMMARY.md") -Encoding UTF8
    Add-JsonLine -Path (Join-Path $directory "SESSION.jsonl") -Value ([ordered]@{
        event = "session_end"
        status = "PASS"
        end_utc = $endTimestamp
        ended_utc = $endTimestamp
        stopwatch_end_tick = $endTick
        monotonic_end_tick = $endTick
        stopwatch_frequency = $frequency
        wall_seconds_utc = $summary.wall_seconds_utc
        wall_seconds_monotonic = $summary.wall_seconds_monotonic
        wall_clock_difference_seconds = $summary.wall_clock_difference_seconds
        full_count = $fullCount
        candidate_count = $candidateCount
    })
    Write-Host "Session complete: $directory"
}

switch ($Command) {
    "audit" { Invoke-DevelopmentAudit }
    "g8c-matrix" {
        $pythonCommand = Get-Command python -CommandType Application -ErrorAction SilentlyContinue |
            Select-Object -First 1
        if ($null -eq $pythonCommand) {
            throw "g8c-matrix requires a Python 3 interpreter named 'python' on PATH"
        }
        & $pythonCommand.Source -B (Join-Path $RepoRoot "tools\g8c_matrix.py") @RemainingArgs
        if ($LASTEXITCODE -ne 0) {
            exit $LASTEXITCODE
        }
    }
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
    "session-phase-start" { Start-DevelopmentPhase }
    "session-phase-end" { Stop-DevelopmentPhase }
    "measure" { Measure-DevelopmentCommand }
    "session-end" { Stop-DevelopmentSession }
}
