[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$DevScript = Join-Path $RepoRoot "tools\dev.ps1"
$Pwsh = (Get-Command pwsh -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source
$TestRoot = Join-Path ([IO.Path]::GetTempPath()) ("powdergame-dev-timer-tests-" + [Guid]::NewGuid().ToString("N"))
$Frequency = [int64]1000
$script:Passed = 0

$TimerEnvironmentNames = @(
    "POWDERGAME_DEV_TIMER_TEST_MODE",
    "POWDERGAME_DEV_TEST_SESSIONS_ROOT",
    "POWDERGAME_DEV_TEST_STOPWATCH_FREQUENCY",
    "POWDERGAME_DEV_TEST_UTC_NOW",
    "POWDERGAME_DEV_TEST_MONOTONIC_TICK",
    "POWDERGAME_DEV_TEST_GIT_STDERR"
)
$SavedEnvironment = @{}
foreach ($environmentName in $TimerEnvironmentNames) {
    $SavedEnvironment[$environmentName] = [Environment]::GetEnvironmentVariable($environmentName, "Process")
}

function Assert-True {
    param([Parameter(Mandatory)][bool]$Condition, [Parameter(Mandatory)][string]$Message)
    if (-not $Condition) { throw "ASSERTION FAILED: $Message" }
}

function Assert-Equal {
    param($Expected, $Actual, [Parameter(Mandatory)][string]$Message)
    if ($Expected -ne $Actual) {
        throw "ASSERTION FAILED: $Message (expected '$Expected', actual '$Actual')"
    }
}

function Assert-Near {
    param(
        [Parameter(Mandatory)][double]$Expected,
        [Parameter(Mandatory)][double]$Actual,
        [double]$Tolerance = 0.000001,
        [Parameter(Mandatory)][string]$Message
    )
    if ([Math]::Abs($Expected - $Actual) -gt $Tolerance) {
        throw "ASSERTION FAILED: $Message (expected $Expected +/- $Tolerance, actual $Actual)"
    }
}

function Invoke-DevTimer {
    param(
        [Parameter(Mandatory)][string[]]$DevArgs,
        [Parameter(Mandatory)][string]$UtcNow,
        [Parameter(Mandatory)][int64]$MonotonicTick
    )
    $env:POWDERGAME_DEV_TEST_UTC_NOW = $UtcNow
    $env:POWDERGAME_DEV_TEST_MONOTONIC_TICK = $MonotonicTick.ToString([Globalization.CultureInfo]::InvariantCulture)
    $output = @(& $Pwsh -NoProfile -File $DevScript @DevArgs 2>&1 | ForEach-Object { $_.ToString() })
    [pscustomobject]@{
        exit_code = $LASTEXITCODE
        output = $output -join [Environment]::NewLine
    }
}

function Start-TestSession {
    param(
        [Parameter(Mandatory)][string]$Id,
        [Parameter(Mandatory)][string]$UtcNow,
        [Parameter(Mandatory)][int64]$MonotonicTick
    )
    $result = Invoke-DevTimer -DevArgs @(
        "session-start", "-Task", "timer-test-$Id", "-SessionId", $Id
    ) -UtcNow $UtcNow -MonotonicTick $MonotonicTick
    Assert-Equal 0 $result.exit_code "session-start should succeed for $Id"
    Join-Path $TestRoot $Id
}

function Read-TestJson {
    param([Parameter(Mandatory)][string]$Path)
    Get-Content -LiteralPath $Path -Raw -Encoding UTF8 | ConvertFrom-Json -DateKind String
}

function Read-TestEvents {
    param([Parameter(Mandatory)][string]$Directory)
    @(
        Get-Content -LiteralPath (Join-Path $Directory "SESSION.jsonl") -Encoding UTF8 |
            Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
            ForEach-Object { $_ | ConvertFrom-Json -DateKind String }
    )
}

function Add-TestEvent {
    param(
        [Parameter(Mandatory)][string]$Directory,
        [Parameter(Mandatory)]$Value
    )
    Add-Content -LiteralPath (Join-Path $Directory "SESSION.jsonl") -Encoding UTF8 -Value (
        $Value | ConvertTo-Json -Depth 12 -Compress
    )
}

function Complete-TestSession {
    param(
        [Parameter(Mandatory)][string]$Id,
        [Parameter(Mandatory)][string]$UtcNow,
        [Parameter(Mandatory)][int64]$MonotonicTick
    )
    Invoke-DevTimer -DevArgs @("session-end", "-SessionId", $Id) -UtcNow $UtcNow -MonotonicTick $MonotonicTick
}

function Invoke-TestCase {
    param([Parameter(Mandatory)][string]$Name, [Parameter(Mandatory)][scriptblock]$Body)
    & $Body
    $script:Passed += 1
    Write-Host "PASS $Name"
}

try {
    New-Item -ItemType Directory -Path $TestRoot -ErrorAction Stop | Out-Null
    $env:POWDERGAME_DEV_TIMER_TEST_MODE = "1"
    $env:POWDERGAME_DEV_TEST_SESSIONS_ROOT = $TestRoot
    $env:POWDERGAME_DEV_TEST_STOPWATCH_FREQUENCY = $Frequency.ToString([Globalization.CultureInfo]::InvariantCulture)

    Invoke-TestCase "timer test has launcher-policy validation classification" {
        $policy = Get-Content -LiteralPath (Join-Path $RepoRoot "config\development-policy.json") -Raw -Encoding UTF8 |
            ConvertFrom-Json
        $toolingClass = @($policy.validation_classes | Where-Object { $_.id -eq "launcher-policy-tooling" })[0]
        Assert-True ($null -ne $toolingClass) "launcher-policy-tooling class must exist"
        Assert-True (@($toolingClass.patterns) -contains '^tools/test-dev-timer\.ps1$') "timer test must map to no-FULL tooling validation"
        Assert-Equal "not-required" ([string]$toolingClass.full) "timer test tooling class must not require FULL"
    }

    Invoke-TestCase "Git stderr cannot become a changed path" {
        $marker = "warning: TEST_STDERR_MUST_NOT_BECOME_A_CHANGED_PATH"
        $env:POWDERGAME_DEV_TEST_GIT_STDERR = $marker
        try {
            $valid = Invoke-DevTimer @("validation-plan", "-BaseRef", "HEAD", "-Json") "2026-08-18T00:00:00Z" 90000
            Assert-Equal 0 $valid.exit_code "validation-plan should tolerate warning-like Git stderr"
            $plan = $valid.output | ConvertFrom-Json -DateKind String
            foreach ($row in @($plan.changed_files)) {
                Assert-True ([string]$row.path -notmatch "TEST_STDERR") "Git stderr must not enter changed paths"
            }
            Assert-True ($valid.output -notmatch "TEST_STDERR") "successful Git stderr should stay out of structured stdout"

            $invalid = Invoke-DevTimer @(
                "validation-plan", "-BaseRef", "refs/powdergame-timer-test/missing", "-Json"
            ) "2026-08-18T00:00:00Z" 90000
            Assert-True ($invalid.exit_code -ne 0) "invalid BaseRef must fail"
            Assert-True ($invalid.output -match "TEST_STDERR_MUST_NOT_BECOME_A_CHANGED_PATH") "Git stderr must remain available in failure diagnostics"
        } finally {
            Remove-Item Env:POWDERGAME_DEV_TEST_GIT_STDERR -ErrorAction SilentlyContinue
        }
    }

    Invoke-TestCase "UTC start/end, KST normalization, double-offset regression, cross-process end" {
        $id = "utc-kst-cross-process"
        $directory = Start-TestSession $id "2026-08-18T09:00:00+09:00" 100000
        $events = Read-TestEvents $directory
        $start = @($events | Where-Object { $_.event -eq "session_start" })[0]
        Assert-Equal "2026-08-18T00:00:00.0000000Z" $start.start_utc "session start must persist normalized RFC3339 Z"
        Assert-Equal 100000 ([int64]$start.stopwatch_start_tick) "session start must persist Stopwatch.GetTimestamp"
        Assert-Equal $Frequency ([int64]$start.stopwatch_frequency) "session start must persist Stopwatch.Frequency"

        $result = Complete-TestSession $id "2026-08-18T09:00:10+09:00" 110000
        Assert-Equal 0 $result.exit_code "cross-process session-end should succeed"
        $summary = Read-TestJson (Join-Path $directory "SUMMARY.json")
        Assert-Equal "PASS" $summary.status "consistent timer summary should be successful"
        Assert-Equal "2026-08-18T00:00:00.0000000Z" $summary.start_utc "summary start must stay UTC"
        Assert-Equal "2026-08-18T00:00:10.0000000Z" $summary.end_utc "summary end must stay UTC"
        Assert-Near 10 ([double]$summary.wall_seconds_utc) -Message "KST display must not alter UTC duration"
        Assert-Near 10 ([double]$summary.wall_seconds_monotonic) -Message "monotonic duration should be independent"
        Assert-Near 0 ([double]$summary.wall_clock_difference_seconds) -Message "UTC and monotonic clocks should agree"
        Assert-Near 10 ([double]$summary.unclassified_seconds) -Message "unmeasured wall time should be unclassified"
        $ended = @((Read-TestEvents $directory) | Where-Object { $_.event -eq "session_end" })[0]
        Assert-Equal 110000 ([int64]$ended.stopwatch_end_tick) "session end must persist monotonic end tick"
        Assert-Equal "2026-08-18T00:00:10.0000000Z" $ended.end_utc "session end event must persist RFC3339 Z"
    }

    Invoke-TestCase "overlapping command/phase union and unclassified ratios" {
        $id = "interval-union"
        $directory = Start-TestSession $id "2026-08-18T00:00:00Z" 200000
        $phaseAStart = Invoke-DevTimer @("session-phase-start", "-SessionId", $id, "-Name", "implementation") "2026-08-18T00:00:01Z" 201000
        Assert-Equal 0 $phaseAStart.exit_code "phase A should start"
        $phaseBStart = Invoke-DevTimer @("session-phase-start", "-SessionId", $id, "-Name", "analysis") "2026-08-18T00:00:02Z" 202000
        Assert-Equal 0 $phaseBStart.exit_code "overlapping phase B should start"
        $phaseAEnd = Invoke-DevTimer @("session-phase-end", "-SessionId", $id, "-Name", "implementation") "2026-08-18T00:00:06Z" 206000
        Assert-Equal 0 $phaseAEnd.exit_code "phase A should end"
        $phaseBEnd = Invoke-DevTimer @("session-phase-end", "-SessionId", $id, "-Name", "analysis") "2026-08-18T00:00:08Z" 208000
        Assert-Equal 0 $phaseBEnd.exit_code "phase B should end"

        Add-TestEvent $directory ([ordered]@{
            event = "command"; category = "test-a"; argv = @("test-a")
            start_utc = "2026-08-18T00:00:03.0000000Z"; end_utc = "2026-08-18T00:00:07.0000000Z"
            stopwatch_start_tick = 203000; stopwatch_end_tick = 207000; stopwatch_frequency = $Frequency
            duration_seconds = 4.0; exit_code = 0
        })
        Add-TestEvent $directory ([ordered]@{
            event = "command"; category = "test-b"; argv = @("test-b")
            start_utc = "2026-08-18T00:00:06.0000000Z"; end_utc = "2026-08-18T00:00:09.0000000Z"
            stopwatch_start_tick = 206000; stopwatch_end_tick = 209000; stopwatch_frequency = $Frequency
            duration_seconds = 3.0; exit_code = 0
        })
        $result = Complete-TestSession $id "2026-08-18T00:00:10Z" 210000
        Assert-Equal 0 $result.exit_code "union session should end"
        $summary = Read-TestJson (Join-Path $directory "SUMMARY.json")
        Assert-Near 6 ([double]$summary.measured_command_seconds) -Message "overlapping command intervals must be unioned"
        Assert-Near 7 ([double]$summary.measured_phase_seconds) -Message "overlapping phase intervals must be unioned"
        Assert-Near 8 ([double]$summary.measured_classified_union_seconds) -Message "command/phase overlap must not be double counted"
        Assert-Near 2 ([double]$summary.unclassified_seconds) -Message "unclassified time must use the combined union"
        Assert-Near 0.6 ([double]$summary.command_to_wall_ratio) -Message "command ratio should use command union"
        Assert-Near 0.7 ([double]$summary.phase_to_wall_ratio) -Message "phase ratio should use phase union"
        Assert-Near 5 ([double]$summary.phase_totals.implementation) -Message "implementation phase duration should be retained"
        Assert-Near 6 ([double]$summary.phase_totals.analysis) -Message "analysis phase duration should be retained"
    }

    Invoke-TestCase "timer inconsistency blocks successful summary" {
        $id = "timer-inconsistency"
        $directory = Start-TestSession $id "2026-08-18T00:00:00Z" 300000
        $result = Complete-TestSession $id "2026-08-18T09:00:10Z" 310000
        Assert-True ($result.exit_code -ne 0) "nine-hour UTC/monotonic mismatch must fail"
        Assert-True ($result.output -match "TIMER_INCONSISTENCY") "failure must name TIMER_INCONSISTENCY"
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $directory "SUMMARY.json"))) "inconsistent timer must not publish SUMMARY.json"
        $errorValue = Read-TestJson (Join-Path $directory "TIMER_ERROR.json")
        Assert-Equal "TIMER_INCONSISTENCY" $errorValue.error "timer error artifact should be explicit"
    }

    Invoke-TestCase "duplicate, missing, and open phases are rejected" {
        $id = "phase-errors"
        $directory = Start-TestSession $id "2026-08-18T00:00:00Z" 400000
        $first = Invoke-DevTimer @("session-phase-start", "-SessionId", $id, "-Name", "implementation") "2026-08-18T00:00:01Z" 401000
        Assert-Equal 0 $first.exit_code "first phase start should succeed"
        $duplicate = Invoke-DevTimer @("session-phase-start", "-SessionId", $id, "-Name", "implementation") "2026-08-18T00:00:02Z" 402000
        Assert-True ($duplicate.exit_code -ne 0) "duplicate open phase must fail"
        Assert-True ($duplicate.output -match "DUPLICATE_PHASE") "duplicate failure should be explicit"
        $backwards = Invoke-DevTimer @("session-phase-end", "-SessionId", $id, "-Name", "implementation") "2026-08-18T00:00:00Z" 399000
        Assert-True ($backwards.exit_code -ne 0) "backwards phase timing must fail"
        Assert-True ($backwards.output -match "MALFORMED_PHASE") "backwards timing failure should be explicit"
        $missing = Invoke-DevTimer @("session-phase-end", "-SessionId", $id, "-Name", "missing") "2026-08-18T00:00:02Z" 402000
        Assert-True ($missing.exit_code -ne 0) "ending a missing phase must fail"
        Assert-True ($missing.output -match "PHASE_NOT_OPEN") "missing phase failure should be explicit"
        $ended = Complete-TestSession $id "2026-08-18T00:00:03Z" 403000
        Assert-True ($ended.exit_code -ne 0) "session with an open phase must fail"
        Assert-True ($ended.output -match "OPEN_PHASE") "open phase failure should be explicit"
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $directory "SUMMARY.json"))) "open phase must not publish a successful summary"
    }

    Invoke-TestCase "malformed session event is rejected" {
        $id = "malformed-event"
        $directory = Start-TestSession $id "2026-08-18T00:00:00Z" 500000
        Add-Content -LiteralPath (Join-Path $directory "SESSION.jsonl") -Encoding UTF8 -Value "{not-json"
        $result = Complete-TestSession $id "2026-08-18T00:00:01Z" 501000
        Assert-True ($result.exit_code -ne 0) "malformed JSONL must fail"
        Assert-True ($result.output -match "MALFORMED_SESSION_EVENT") "malformed event failure should be explicit"
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $directory "SUMMARY.json"))) "malformed session must not publish summary"
    }

    Write-Host "Timer tests passed: $script:Passed"
} finally {
    foreach ($environmentName in $TimerEnvironmentNames) {
        [Environment]::SetEnvironmentVariable($environmentName, $SavedEnvironment[$environmentName], "Process")
    }
    $fullTestRoot = [IO.Path]::GetFullPath($TestRoot)
    $fullTempRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    if ($fullTestRoot.StartsWith($fullTempRoot, [StringComparison]::OrdinalIgnoreCase) -and
        (Split-Path -Leaf $fullTestRoot) -like "powdergame-dev-timer-tests-*") {
        Remove-Item -LiteralPath $fullTestRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}
