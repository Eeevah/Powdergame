[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($PSVersionTable.PSVersion.Major -lt 7) {
    throw 'test-verify-evidence.ps1 requires PowerShell 7 or newer (pwsh.exe).'
}

Add-Type -AssemblyName System.IO.Compression.FileSystem

$verifierPath = Join-Path $PSScriptRoot 'verify-evidence.ps1'
$testRoot = Join-Path ([IO.Path]::GetTempPath()) ('powdergame-verifier-tests-' + [Guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($testRoot) | Out-Null

function Write-Utf8NoBom {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][AllowEmptyString()][string]$Text
    )
    $parent = Split-Path -Parent $Path
    if (-not [string]::IsNullOrWhiteSpace($parent)) { [IO.Directory]::CreateDirectory($parent) | Out-Null }
    [IO.File]::WriteAllText($Path, $Text, [Text.UTF8Encoding]::new($false))
}

function Get-Sha256Hex {
    param([Parameter(Mandatory)][string]$Path)
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-RelativeForwardPath {
    param([string]$Root, [string]$Path)
    return [IO.Path]::GetRelativePath($Root, $Path).Replace([char]92, [char]47)
}

function New-FileRecord {
    param(
        [Parameter(Mandatory)][string]$CaptureRoot,
        [Parameter(Mandatory)][string]$Path
    )
    return [ordered]@{
        path = Get-RelativeForwardPath $CaptureRoot $Path
        size_bytes = (Get-Item -LiteralPath $Path).Length
        sha256 = Get-Sha256Hex $Path
    }
}

function New-SummaryRow {
    param(
        [string]$MetricType,
        [string]$Name,
        [string]$Trial = 'n/a',
        [string]$Value = '',
        [string]$Count = '',
        [string]$P50 = '',
        [string]$P95 = '',
        [string]$Mean = '',
        [string]$Min = '',
        [string]$Max = '',
        [string]$TickStart = '0',
        [string]$TickEnd = '0',
        [string]$RunId = 'run-fixture'
    )
    return [ordered]@{
        schema_version = 'powdergame-g8a-v5'
        run_id = $RunId
        commit_sha = '1111111111111111111111111111111111111111'
        git_state = 'clean'
        width = '2'
        height = '2'
        chunk_size = '2'
        trial = $Trial
        tick_start = $TickStart
        tick_end = $TickEnd
        metric_type = $MetricType
        name = $Name
        value = $Value
        count = $Count
        p50 = $P50
        p95 = $P95
        mean = $Mean
        min = $Min
        max = $Max
    }
}

function Add-StatsSummaryRow {
    param(
        [Collections.Generic.List[object]]$Rows,
        [string]$MetricType,
        [string]$Name,
        [string]$Trial,
        [double[]]$Values
    )
    $sorted = @($Values | Sort-Object)
    $sum = 0.0
    foreach ($value in $sorted) { $sum += $value }
    $p50 = $sorted[[int][Math]::Floor((0.5 * ($sorted.Count - 1)) + 0.5)]
    $p95 = $sorted[[int][Math]::Floor((0.95 * ($sorted.Count - 1)) + 0.5)]
    $Rows.Add((New-SummaryRow -MetricType $MetricType -Name $Name -Trial $Trial -Count ([string]$sorted.Count) -P50 ([string]$p50) -P95 ([string]$p95) -Mean ([string]($sum / $sorted.Count)) -Min ([string]$sorted[0]) -Max ([string]$sorted[-1]) -TickEnd '1'))
}

function New-FixturePackage {
    param(
        [Parameter(Mandatory)][string]$CaseName,
        [ValidateSet('none', 'package_hash', 'provenance', 'row_count', 'aggregate_recompute', 'inventory', 'receipt', 'encoding', 'timing', 'tick_identity')]
        [string]$Mutation = 'none'
    )
    $caseRoot = Join-Path $testRoot $CaseName
    $captureId = "g8a-test-$CaseName"
    $captureRoot = Join-Path $caseRoot $captureId
    foreach ($directory in @(
        'artifacts', 'commands/cargo-build', 'commands/benchmark', 'diff', 'executable',
        'hashes', 'metadata', 'source/snapshot'
    )) {
        [IO.Directory]::CreateDirectory((Join-Path $captureRoot $directory)) | Out-Null
    }

    $aggregateRows = [Collections.Generic.List[object]]::new()
    $aggregateRows.Add((New-SummaryRow 'throughput_trial' 'elapsed_wall' '1' '4' -TickEnd '1'))
    $aggregateRows.Add((New-SummaryRow 'throughput_trial' 'wall_per_tick' '1' '2' -TickEnd '1'))
    $aggregateRows.Add((New-SummaryRow 'throughput_trial' 'sustained_tps' '1' '500' -TickEnd '1'))
    Add-StatsSummaryRow $aggregateRows 'throughput_summary' 'wall_per_tick' 'all' @([double]2)
    Add-StatsSummaryRow $aggregateRows 'throughput_summary' 'sustained_tps' 'all' @([double]500)
    Add-StatsSummaryRow $aggregateRows 'pass' 'a' '1' @([double]1, [double]2)
    Add-StatsSummaryRow $aggregateRows 'grouped_subsystem' 'g' '1' @([double]1, [double]2)
    Add-StatsSummaryRow $aggregateRows 'grouped_envelope_ratio' 'g' '1' @([double]100, [double]100)
    Add-StatsSummaryRow $aggregateRows 'envelope' 'gpu_tick_envelope' '1' @([double]1, [double]2)
    Add-StatsSummaryRow $aggregateRows 'envelope' 'gpu_pass_sum' '1' @([double]1, [double]2)
    Add-StatsSummaryRow $aggregateRows 'envelope' 'diagnostic_residual' '1' @([double]0, [double]0)
    $census = [ordered]@{
        total_cells = 4
        any_active_cells = if ($Mutation -eq 'aggregate_recompute') { 2 } else { 3 }
        matter_active_cells = 2
        thermal_active_cells = 2
        pressure_active_cells = 0
        reaction_active_cells = 0
        total_chunks = 1
        active_chunks = 1
        runnable_chunks = 1
        sleeping_chunks = 0
    }
    foreach ($entry in $census.GetEnumerator()) {
        $aggregateRows.Add((New-SummaryRow 'activity_census' $entry.Key 'n/a' ([string]$entry.Value) -TickStart '7' -TickEnd '7'))
    }
    foreach ($entry in ([ordered]@{
        batched_unprofiled_elapsed = 10
        synchronized_unprofiled_elapsed = 15
        synchronized_profiled_elapsed = 20
        synchronization_overhead = 50
        profiling_increment = 33.333333333
        observed_profiled_path_overhead = 100
    }).GetEnumerator()) {
        $aggregateRows.Add((New-SummaryRow 'profiling_overhead' $entry.Key 'n/a' ([string]$entry.Value) -TickEnd '1'))
    }
    if ($Mutation -eq 'provenance') { $aggregateRows[0].run_id = 'wrong-run' }
    Write-Utf8NoBom (Join-Path $captureRoot 'artifacts/aggregate.csv') ((@($aggregateRows) | ConvertTo-Csv -NoTypeInformation -UseQuotes AsNeeded) -join "`n" + "`n")

    $rawTicks = @(
        [ordered]@{
            schema_version = 'powdergame-g8a-v5'; run_id = 'run-fixture'; commit_sha = '1111111111111111111111111111111111111111'; git_state = 'clean'
            width = '2'; height = '2'; chunk_size = '2'
            timestamp_period_ns = '1000000'; trial = '1'; sample_id = '0'; tick_index = '0'; a_start_tick = '0'; a_end_tick = $(if ($Mutation -eq 'timing') { '0' } else { '1' })
            pass_a_ms = '1'; group_g_ms = '1'; gpu_pass_sum_ms = '1'; gpu_tick_envelope_ms = '1'; residual_ms = '0'
        },
        [ordered]@{
            schema_version = 'powdergame-g8a-v5'; run_id = 'run-fixture'; commit_sha = '1111111111111111111111111111111111111111'; git_state = 'clean'
            width = '2'; height = '2'; chunk_size = '2'
            timestamp_period_ns = '1000000'; trial = '1'; sample_id = '1'; tick_index = $(if ($Mutation -eq 'tick_identity') { '0' } else { '1' }); a_start_tick = '4'; a_end_tick = '6'
            pass_a_ms = '2'; group_g_ms = '2'; gpu_pass_sum_ms = '2'; gpu_tick_envelope_ms = '2'; residual_ms = '0'
        }
    )
    Write-Utf8NoBom (Join-Path $captureRoot 'artifacts/aggregate_raw_ticks.csv') (($rawTicks | ConvertTo-Csv -NoTypeInformation -UseQuotes AsNeeded) -join "`n" + "`n")

    $cellLines = [Collections.Generic.List[string]]::new()
    $cellLines.Add('schema_version,run_id,commit_sha,git_state,census_tick,index,activity_mask')
    $cellMasks = if ($Mutation -eq 'row_count') { @(0, 1, 2) } else { @(0, 1, 2, 3) }
    for ($index = 0; $index -lt $cellMasks.Count; $index++) {
        $cellLines.Add("powdergame-g8a-v5,run-fixture,1111111111111111111111111111111111111111,clean,7,$index,$($cellMasks[$index])")
    }
    Write-Utf8NoBom (Join-Path $captureRoot 'artifacts/aggregate_raw_cells.csv') (($cellLines -join "`n") + "`n")
    Write-Utf8NoBom (Join-Path $captureRoot 'artifacts/aggregate_raw_chunks.csv') "schema_version,run_id,commit_sha,git_state,census_tick,index,activity_mask,chunk_state`npowdergame-g8a-v5,run-fixture,1111111111111111111111111111111111111111,clean,7,0,3,0`n"

    $snapshotPath = Join-Path $captureRoot 'source/snapshot/Cargo.toml'
    Write-Utf8NoBom $snapshotPath "[workspace]`n"
    $snapshotSha = Get-Sha256Hex $snapshotPath
    $snapshotSize = (Get-Item $snapshotPath).Length
    $inventoryText = "exists`tsize_bytes`tsha256`trepository_relative_path`tsnapshot_relative_path`ntrue`t$snapshotSize`t$snapshotSha`tCargo.toml`tsource/snapshot/Cargo.toml`n"
    $sourceManifests = @(
        'source/SOURCE_INPUTS_BEFORE_BUILD.tsv', 'source/SOURCE_INPUTS_AFTER_BUILD.tsv',
        'source/SOURCE_INPUTS_BEFORE_RUN.tsv', 'source/SOURCE_INPUTS_AFTER_RUN.tsv'
    )
    foreach ($relative in $sourceManifests) { Write-Utf8NoBom (Join-Path $captureRoot $relative) $inventoryText }
    $statusText = "# branch.oid 1111111111111111111111111111111111111111`n# branch.head fix/g8a-evidence-remediation-v5`n"
    Write-Utf8NoBom (Join-Path $captureRoot 'source/GIT_STATUS_BEFORE.txt') $statusText
    Write-Utf8NoBom (Join-Path $captureRoot 'source/GIT_STATUS_AFTER.txt') $statusText
    Write-Utf8NoBom (Join-Path $captureRoot 'diff/full_dirty.diff') ''
    Write-Utf8NoBom (Join-Path $captureRoot 'diff/full_dirty_after.diff') ''
    Write-Utf8NoBom (Join-Path $captureRoot 'executable/powdergame-benchmark.exe') 'fixture executable'

    foreach ($label in @('cargo-build', 'benchmark')) {
        $commandDirectory = Join-Path $captureRoot "commands/$label"
        $command = [ordered]@{
            executable = if ($label -eq 'benchmark') { 'C:\fixture\powdergame-benchmark.exe' } else { 'C:\fixture\cargo.exe' }
            argv = if ($label -eq 'benchmark') { @('--csv', 'C:\capture\artifacts\aggregate.csv') } else { @('build', '--locked', '--release', '-p', 'powdergame-benchmark') }
            cwd = 'C:\fixture\repo'
            environment_overrides = [ordered]@{}
        }
        Write-Utf8NoBom (Join-Path $commandDirectory 'command.json') (($command | ConvertTo-Json -Depth 5) + "`n")
        Write-Utf8NoBom (Join-Path $commandDirectory 'stdout.bin') $(if ($label -eq 'benchmark') { "Run ID: run-fixture`n" } else { '' })
        Write-Utf8NoBom (Join-Path $commandDirectory 'stderr.bin') ''
        Write-Utf8NoBom (Join-Path $commandDirectory 'exit_code.txt') "0`n"
    }

    $sourceSha = '1111111111111111111111111111111111111111'
    $branch = 'fix/g8a-evidence-remediation-v5'
    $originUrl = 'https://example.invalid/powdergame.git'
    $upstreamRef = 'origin/fix/g8a-evidence-remediation-v5'
    $commandRecords = [ordered]@{}
    $metadataCommands = [Collections.Generic.List[object]]::new()
    foreach ($label in @('cargo-build', 'benchmark')) {
        $commandDirectory = Join-Path $captureRoot "commands/$label"
        $commandJson = Get-Content -Raw -LiteralPath (Join-Path $commandDirectory 'command.json') | ConvertFrom-Json
        $record = [ordered]@{
            label = $label
            executable = [string]$commandJson.executable
            argv = @($commandJson.argv)
            cwd = [string]$commandJson.cwd
            environment_overrides = $commandJson.environment_overrides
            command_json = "commands/$label/command.json"
            stdout = "commands/$label/stdout.bin"
            stderr = "commands/$label/stderr.bin"
            exit_code_path = "commands/$label/exit_code.txt"
            exit_code = 0
        }
        $commandRecords[$label] = $record
        $metadataCommands.Add($record)
    }
    $artifactRecords = [ordered]@{}
    foreach ($entry in ([ordered]@{
        raw_cells = [ordered]@{ path = 'artifacts/aggregate_raw_cells.csv'; rows = $cellMasks.Count }
        raw_chunks = [ordered]@{ path = 'artifacts/aggregate_raw_chunks.csv'; rows = 1 }
        raw_ticks = [ordered]@{ path = 'artifacts/aggregate_raw_ticks.csv'; rows = $rawTicks.Count }
        aggregate = [ordered]@{ path = 'artifacts/aggregate.csv'; rows = $aggregateRows.Count }
    }).GetEnumerator()) {
        $record = New-FileRecord $captureRoot (Join-Path $captureRoot $entry.Value.path)
        $record['data_row_count'] = $entry.Value.rows
        $record['value_row_count'] = $entry.Value.rows
        $artifactRecords[$entry.Key] = $record
    }
    $manifestRecordsForMetadata = [ordered]@{
        before_build = New-FileRecord $captureRoot (Join-Path $captureRoot $sourceManifests[0])
        after_build = New-FileRecord $captureRoot (Join-Path $captureRoot $sourceManifests[1])
        before_run = New-FileRecord $captureRoot (Join-Path $captureRoot $sourceManifests[2])
        after_run = New-FileRecord $captureRoot (Join-Path $captureRoot $sourceManifests[3])
    }
    $dirtyBeforeRecord = New-FileRecord $captureRoot (Join-Path $captureRoot 'diff/full_dirty.diff')
    $dirtyAfterRecord = New-FileRecord $captureRoot (Join-Path $captureRoot 'diff/full_dirty_after.diff')
    $exePath = Join-Path $captureRoot 'executable/powdergame-benchmark.exe'
    $exeSha = Get-Sha256Hex $exePath
    $encoding = [ordered]@{
        cell_activity_bits = [ordered]@{
            matter = $(if ($Mutation -eq 'encoding') { 2 } else { 1 })
            thermal = 2
            pressure = 4
            reaction = 8
        }
        chunk_state_values = [ordered]@{ runnable = 0; sleeping = 1 }
    }
    $publicationOrder = @(
        'artifacts/aggregate_raw_cells.csv', 'artifacts/aggregate_raw_chunks.csv',
        'artifacts/aggregate_raw_ticks.csv', 'artifacts/aggregate.csv',
        'metadata/CAPTURE_METADATA.json', 'hashes/SHA256SUMS.txt', 'CAPTURE_RECEIPT.json'
    )
    $metadata = [ordered]@{
        metadata_schema = 'powdergame-g8a-capture-metadata-v2'
        official_mode = $true
        capture_id = $captureId
        run_id = 'run-fixture'
        started_utc = '2026-08-17T00:00:00.0000000Z'
        metadata_created_utc = '2026-08-17T00:01:00.0000000Z'
        repository = [ordered]@{
            root = 'C:\fixture\repo'; origin_url = $originUrl; branch = $branch; source_sha = $sourceSha
            upstream_ref = $upstreamRef; upstream_sha = $sourceSha; git_state = 'clean'; clean_before = $true; clean_after = $true
        }
        source = [ordered]@{
            input_count = 1; manifests = $manifestRecordsForMetadata; manifests_identical = $true; snapshot_root = 'source/snapshot'
            dirty_diff_before = $dirtyBeforeRecord; dirty_diff_after = $dirtyAfterRecord
        }
        executable = [ordered]@{
            path = 'executable/powdergame-benchmark.exe'; build_output_sha256 = $exeSha
            captured_sha256_before_run = $exeSha; captured_sha256_after_run = $exeSha; unchanged = $true
        }
        csv = [ordered]@{
            schema_version = 'powdergame-g8a-v5'; run_id = 'run-fixture'; stdout_run_id = 'run-fixture'; staged_records = $artifactRecords
        }
        census_encoding = $encoding
        toolchain = [ordered]@{ powershell = '7'; os_description = 'fixture'; process_architecture = 'X64'; git = 'fixture'; cargo = 'fixture'; rustc = 'fixture' }
        commands = $metadataCommands
        intended_publication_order = $publicationOrder
    }
    $metadataPath = Join-Path $captureRoot 'metadata/CAPTURE_METADATA.json'
    Write-Utf8NoBom $metadataPath (($metadata | ConvertTo-Json -Depth 16) + "`n")

    $manifestLines = [Collections.Generic.List[string]]::new()
    foreach ($file in @(Get-ChildItem -LiteralPath $captureRoot -Recurse -File | Where-Object {
        (Get-RelativeForwardPath $captureRoot $_.FullName) -notin @('CAPTURE_RECEIPT.json', 'hashes/SHA256SUMS.txt')
    } | Sort-Object FullName)) {
        $relative = Get-RelativeForwardPath $captureRoot $file.FullName
        $manifestLines.Add(('{0}  {1}' -f (Get-Sha256Hex $file.FullName), $relative))
    }
    $hashManifestPath = Join-Path $captureRoot 'hashes/SHA256SUMS.txt'
    Write-Utf8NoBom $hashManifestPath (($manifestLines -join "`n") + "`n")

    $receipt = [ordered]@{
        receipt_schema = 'powdergame-g8a-capture-receipt-v2'
        official_mode = $true
        complete = $Mutation -ne 'receipt'
        capture_id = $captureId
        run_id = 'run-fixture'
        started_utc = '2026-08-17T00:00:00.0000000Z'
        completed_utc = '2026-08-17T00:02:00.0000000Z'
        repository = [ordered]@{
            origin_url = $originUrl; branch = $branch; source_sha = $sourceSha; upstream_ref = $upstreamRef; upstream_sha = $sourceSha
            git_state = 'clean'; clean = $true
        }
        source = [ordered]@{
            input_count = if ($Mutation -eq 'inventory') { 2 } else { 1 }
            manifest_sha256 = Get-Sha256Hex (Join-Path $captureRoot $sourceManifests[0])
            manifest_paths = $sourceManifests
            manifests_identical = $true; source_unchanged = $true
            dirty_diff_before = $dirtyBeforeRecord; dirty_diff_after = $dirtyAfterRecord
            status_before_path = 'source/GIT_STATUS_BEFORE.txt'; status_after_path = 'source/GIT_STATUS_AFTER.txt'
        }
        executable = [ordered]@{
            path = 'executable/powdergame-benchmark.exe'; sha256 = $exeSha; unchanged = $true
        }
        schema_version = 'powdergame-g8a-v5'
        run_id_links_complete = $true
        commands = $commandRecords
        census_encoding = $encoding
        artifacts = $artifactRecords
        metadata = New-FileRecord $captureRoot $metadataPath
        publication_order = $publicationOrder
        hash_manifest = [ordered]@{
            path = 'hashes/SHA256SUMS.txt'; sha256 = Get-Sha256Hex $hashManifestPath; file_count = $manifestLines.Count; excludes_self_and_receipt = $true
        }
        package = [ordered]@{
            created_after_receipt = $true; zip_path_outside_capture = "$captureId.zip"; zip_sha256_path_outside_zip = 'PACKAGE_SHA256.txt'
        }
    }
    Write-Utf8NoBom (Join-Path $captureRoot 'CAPTURE_RECEIPT.json') (($receipt | ConvertTo-Json -Depth 12) + "`n")

    $packagePath = Join-Path $caseRoot "$captureId.zip"
    $archive = [IO.Compression.ZipFile]::Open($packagePath, [IO.Compression.ZipArchiveMode]::Create)
    try {
        foreach ($file in @(Get-ChildItem -LiteralPath $captureRoot -Recurse -File | Sort-Object FullName)) {
            $relative = Get-RelativeForwardPath $captureRoot $file.FullName
            [void][IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive, $file.FullName, "$captureId/$relative", [IO.Compression.CompressionLevel]::Optimal)
        }
    }
    finally {
        $archive.Dispose()
    }
    $packageHash = if ($Mutation -eq 'package_hash') { '0' * 64 } else { Get-Sha256Hex $packagePath }
    $packageHashPath = Join-Path $caseRoot 'PACKAGE_SHA256.txt'
    Write-Utf8NoBom $packageHashPath "$packageHash  $captureId.zip`n"
    return [ordered]@{
        package = $packagePath
        package_hash = $packageHashPath
        report = Join-Path $caseRoot 'INDEPENDENT_VERIFICATION.json'
    }
}

function Invoke-VerifierCase {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Mutation,
        [Parameter(Mandatory)][bool]$ExpectSuccess
    )
    $fixture = New-FixturePackage -CaseName $Name -Mutation $Mutation
    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = (Get-Command pwsh.exe -ErrorAction Stop).Source
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    foreach ($argument in @('-NoProfile', '-File', $verifierPath, '-PackagePath', $fixture.package, '-PackageHashPath', $fixture.package_hash, '-OutputPath', $fixture.report)) {
        $startInfo.ArgumentList.Add($argument)
    }
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    try {
        if (-not $process.Start()) { throw "Could not start verifier case: $Name" }
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        $process.WaitForExit()
        $stdout = $stdoutTask.GetAwaiter().GetResult()
        $stderr = $stderrTask.GetAwaiter().GetResult()
        $exitCode = $process.ExitCode
    }
    finally {
        $process.Dispose()
    }
    if (-not (Test-Path -LiteralPath $fixture.report -PathType Leaf)) { throw "Verifier case did not write JSON: $Name`n$stdout`n$stderr" }
    $report = Get-Content -Raw -LiteralPath $fixture.report | ConvertFrom-Json
    $actualSuccess = $exitCode -eq 0 -and [bool]$report.success
    if ($actualSuccess -ne $ExpectSuccess) {
        throw "Verifier case $Name expected success=$ExpectSuccess but exit=$exitCode report.success=$($report.success).`nstdout:`n$stdout`nstderr:`n$stderr"
    }
    if (-not $ExpectSuccess -and $exitCode -eq 0) { throw "Negative verifier case unexpectedly exited 0: $Name" }
    if ($Name -ceq 'valid' -and [uint64]$report.derived.raw_chunk_rows -ne 1) {
        throw 'Divisible 2x2 world with chunk_size=2 must recompute to exactly one raw chunk row.'
    }
    Write-Output "case=$Name mutation=$Mutation expected_success=$ExpectSuccess exit_code=$exitCode"
}

try {
    Invoke-VerifierCase 'valid' 'none' $true
    Invoke-VerifierCase 'package-hash' 'package_hash' $false
    Invoke-VerifierCase 'provenance' 'provenance' $false
    Invoke-VerifierCase 'row-count' 'row_count' $false
    Invoke-VerifierCase 'aggregate-recompute' 'aggregate_recompute' $false
    Invoke-VerifierCase 'inventory' 'inventory' $false
    Invoke-VerifierCase 'receipt' 'receipt' $false
    Invoke-VerifierCase 'encoding' 'encoding' $false
    Invoke-VerifierCase 'timing' 'timing' $false
    Invoke-VerifierCase 'tick-identity' 'tick_identity' $false
    Write-Output 'independent verifier fixture tests complete: 10 cases'
}
finally {
    if (Test-Path -LiteralPath $testRoot -PathType Container) {
        Remove-Item -LiteralPath $testRoot -Recurse -Force
    }
}
