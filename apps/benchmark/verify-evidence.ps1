[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$PackagePath,
    [string]$PackageHashPath = '',
    [Parameter(Mandatory)][string]$OutputPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($PSVersionTable.PSVersion.Major -lt 7) {
    throw 'verify-evidence.ps1 requires PowerShell 7 or newer (pwsh.exe).'
}

Add-Type -AssemblyName System.IO.Compression.FileSystem

$expectedCsvSchema = 'powdergame-g8a-v5'
$expectedReceiptSchema = 'powdergame-g8a-capture-receipt-v2'
$invariantCulture = [Globalization.CultureInfo]::InvariantCulture
$numberStyles = [Globalization.NumberStyles]::Float
$integerStyles = [Globalization.NumberStyles]::Integer
$startedUtc = [DateTime]::UtcNow.ToString('o')
$checks = [Collections.Generic.List[object]]::new()
$findings = [Collections.Generic.List[string]]::new()
$derived = [ordered]@{}
$captureId = ''
$packageSha256 = ''
$temporaryDirectory = ''
$hadFailure = $false

function Add-VerificationCheck {
    param(
        [Parameter(Mandatory)][string]$Id,
        [Parameter(Mandatory)][bool]$Passed,
        [Parameter(Mandatory)][AllowEmptyString()][string]$Detail
    )
    $script:checks.Add([ordered]@{
        id = $Id
        passed = $Passed
        detail = $Detail
    })
    if (-not $Passed) {
        $script:hadFailure = $true
        $script:findings.Add("${Id}: $Detail")
    }
}

function Get-Sha256Hex {
    param([Parameter(Mandatory)][string]$Path)
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Cannot hash missing file: $Path"
    }
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-OptionalProperty {
    param(
        [AllowNull()][object]$Object,
        [Parameter(Mandatory)][string]$Name
    )
    if ($null -eq $Object) {
        return $null
    }
    $property = $Object.PSObject.Properties[$Name]
    if ($null -eq $property) {
        return $null
    }
    return $property.Value
}

function Test-SafeRelativePath {
    param([AllowNull()][string]$Path)
    if ([string]::IsNullOrWhiteSpace($Path)) {
        return $false
    }
    $normalized = $Path.Replace([char]92, [char]47)
    if ($normalized.StartsWith('/') -or [IO.Path]::IsPathRooted($normalized)) {
        return $false
    }
    if ($normalized.IndexOfAny([char[]]@([char]0, "`r", "`n", "`t")) -ge 0) {
        return $false
    }
    $segments = @($normalized.Split('/', [StringSplitOptions]::None))
    if ($segments.Count -eq 0) {
        return $false
    }
    foreach ($segment in $segments) {
        if ([string]::IsNullOrEmpty($segment) -or $segment -eq '.' -or $segment -eq '..') {
            return $false
        }
    }
    return $true
}

function Resolve-CapturePath {
    param(
        [Parameter(Mandatory)][string]$CaptureRoot,
        [Parameter(Mandatory)][string]$RelativePath
    )
    $normalized = $RelativePath.Replace([char]92, [char]47)
    if (-not (Test-SafeRelativePath -Path $normalized)) {
        throw "Unsafe capture-relative path: $RelativePath"
    }
    $rootFull = [IO.Path]::GetFullPath($CaptureRoot)
    $candidate = [IO.Path]::GetFullPath((Join-Path $rootFull $normalized.Replace([char]47, [char]92)))
    $rootPrefix = $rootFull.TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
    if (-not $candidate.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Capture-relative path escapes root: $RelativePath"
    }
    return $candidate
}

function Get-RelativeForwardPath {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$Path
    )
    return [IO.Path]::GetRelativePath($Root, $Path).Replace([char]92, [char]47)
}

function Read-FirstUtf8Line {
    param([Parameter(Mandatory)][string]$Path)
    $reader = [IO.StreamReader]::new($Path, [Text.UTF8Encoding]::new($false, $true), $true, 4096)
    try {
        return $reader.ReadLine()
    }
    finally {
        $reader.Dispose()
    }
}

function ConvertTo-StrictDouble {
    param(
        [AllowNull()][string]$Text,
        [Parameter(Mandatory)][string]$Label
    )
    $value = 0.0
    if ([string]::IsNullOrWhiteSpace($Text) -or
        -not [double]::TryParse($Text, $script:numberStyles, $script:invariantCulture, [ref]$value) -or
        [double]::IsNaN($value) -or [double]::IsInfinity($value)) {
        throw "Invalid finite number for ${Label}: '$Text'"
    }
    return $value
}

function ConvertTo-StrictUInt64 {
    param(
        [AllowNull()][string]$Text,
        [Parameter(Mandatory)][string]$Label
    )
    $value = [uint64]0
    if ([string]::IsNullOrWhiteSpace($Text) -or
        -not [uint64]::TryParse($Text, $script:integerStyles, $script:invariantCulture, [ref]$value)) {
        throw "Invalid unsigned integer for ${Label}: '$Text'"
    }
    return $value
}

function Test-Near {
    param(
        [double]$Actual,
        [double]$Expected,
        [double]$AbsoluteTolerance = 0.000001,
        [double]$RelativeTolerance = 0.000000001
    )
    $limit = [Math]::Max($AbsoluteTolerance, [Math]::Abs($Expected) * $RelativeTolerance)
    return [Math]::Abs($Actual - $Expected) -le $limit
}

function Get-Stats {
    param([Parameter(Mandatory)][double[]]$Values)
    if ($Values.Count -eq 0) {
        throw 'Cannot calculate statistics for an empty value set.'
    }
    $sorted = @($Values | Sort-Object)
    $p50Index = [int][Math]::Floor((0.50 * ($sorted.Count - 1)) + 0.5)
    $p95Index = [int][Math]::Floor((0.95 * ($sorted.Count - 1)) + 0.5)
    $sum = 0.0
    foreach ($value in $sorted) {
        $sum += [double]$value
    }
    return [ordered]@{
        count = $sorted.Count
        p50 = [double]$sorted[$p50Index]
        p95 = [double]$sorted[$p95Index]
        mean = $sum / $sorted.Count
        min = [double]$sorted[0]
        max = [double]$sorted[$sorted.Count - 1]
    }
}

function Assert-StatsRow {
    param(
        [Parameter(Mandatory)][object]$Row,
        [Parameter(Mandatory)][double[]]$Values,
        [Parameter(Mandatory)][string]$Label
    )
    $expected = Get-Stats -Values $Values
    $actualCount = ConvertTo-StrictUInt64 -Text ([string]$Row.count) -Label "$Label count"
    if ($actualCount -ne [uint64]$expected.count) {
        throw "$Label count mismatch: expected $($expected.count), got $actualCount"
    }
    foreach ($name in @('p50', 'p95', 'mean', 'min', 'max')) {
        $actual = ConvertTo-StrictDouble -Text ([string]$Row.$name) -Label "$Label $name"
        $expectedValue = [double]$expected[$name]
        if (-not (Test-Near -Actual $actual -Expected $expectedValue)) {
            throw "$Label $name mismatch: expected $expectedValue, got $actual"
        }
    }
}

function Get-SingleStringValue {
    param(
        [Parameter(Mandatory)][object[]]$Rows,
        [Parameter(Mandatory)][string]$Property,
        [Parameter(Mandatory)][string]$Label
    )
    $values = @($Rows | ForEach-Object { [string](Get-OptionalProperty -Object $_ -Name $Property) } | Sort-Object -Unique)
    if ($values.Count -ne 1 -or [string]::IsNullOrWhiteSpace($values[0])) {
        throw "$Label must be single-valued and nonempty."
    }
    return [string]$values[0]
}

function Get-OneRow {
    param(
        [Parameter(Mandatory)][object[]]$Rows,
        [Parameter(Mandatory)][scriptblock]$Predicate,
        [Parameter(Mandatory)][string]$Label
    )
    $matches = @($Rows | Where-Object $Predicate)
    if ($matches.Count -ne 1) {
        throw "$Label must have exactly one row; found $($matches.Count)."
    }
    return $matches[0]
}

function Assert-CaptureFileRecord {
    param(
        [Parameter(Mandatory)][string]$CaptureRoot,
        [Parameter(Mandatory)][object]$Record,
        [string]$ExpectedRelativePath = '',
        [Parameter(Mandatory)][string]$Label
    )
    $relative = [string](Get-OptionalProperty $Record 'path')
    if (-not [string]::IsNullOrWhiteSpace($ExpectedRelativePath) -and $relative -cne $ExpectedRelativePath) {
        throw "$Label path mismatch: expected $ExpectedRelativePath, got $relative"
    }
    $path = Resolve-CapturePath -CaptureRoot $CaptureRoot -RelativePath $relative
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "$Label file is missing: $relative" }
    $size = [uint64](Get-Item -LiteralPath $path).Length
    $sha = Get-Sha256Hex $path
    if ($size -ne [uint64](Get-OptionalProperty $Record 'size_bytes') -or $sha -cne [string](Get-OptionalProperty $Record 'sha256')) {
        throw "$Label size/SHA-256 record mismatch: $relative"
    }
    return $path
}

function Read-RawCells {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][uint64]$ExpectedCount,
        [Parameter(Mandatory)][string]$ExpectedRunId,
        [Parameter(Mandatory)][string]$ExpectedCommit,
        [Parameter(Mandatory)][uint64]$MatterBit,
        [Parameter(Mandatory)][uint64]$ThermalBit,
        [Parameter(Mandatory)][uint64]$PressureBit,
        [Parameter(Mandatory)][uint64]$ReactionBit
    )
    $reader = [IO.StreamReader]::new($Path, [Text.UTF8Encoding]::new($false, $true), $true, 1048576)
    try {
        $headerLine = $reader.ReadLine()
        if ($null -eq $headerLine) {
            throw 'Raw cell CSV is empty.'
        }
        $header = @($headerLine.Split(',', [StringSplitOptions]::None))
        $expectedHeader = @('schema_version', 'run_id', 'commit_sha', 'git_state', 'census_tick', 'index', 'activity_mask')
        if (($header -join ',') -cne ($expectedHeader -join ',')) {
            throw "Raw cell header mismatch: $($header -join ',')"
        }
        $count = [uint64]0
        $any = [uint64]0
        $matter = [uint64]0
        $thermal = [uint64]0
        $pressure = [uint64]0
        $reaction = [uint64]0
        $censusTick = $null
        $knownMask = $MatterBit -bor $ThermalBit -bor $PressureBit -bor $ReactionBit
        while (($line = $reader.ReadLine()) -ne $null) {
            if ($line.Length -eq 0) {
                throw "Raw cell CSV contains a blank line at data row $count."
            }
            $fields = @($line.Split(',', [StringSplitOptions]::None))
            if ($fields.Count -ne $expectedHeader.Count) {
                throw "Raw cell row $count has $($fields.Count) columns."
            }
            if ($fields[0] -cne $script:expectedCsvSchema -or $fields[1] -cne $ExpectedRunId -or
                $fields[2] -cne $ExpectedCommit -or $fields[3] -cne 'clean') {
                throw "Raw cell provenance mismatch at row $count."
            }
            $rowTick = ConvertTo-StrictUInt64 -Text $fields[4] -Label "raw cell census_tick row $count"
            if ($null -eq $censusTick) { $censusTick = $rowTick }
            elseif ($rowTick -ne $censusTick) { throw "Raw cell census_tick changes at row $count." }
            $rowIndex = ConvertTo-StrictUInt64 -Text $fields[5] -Label "raw cell index row $count"
            if ($rowIndex -ne $count) {
                throw "Raw cell index discontinuity: expected $count, got $rowIndex."
            }
            $mask = ConvertTo-StrictUInt64 -Text $fields[6] -Label "raw cell activity_mask row $count"
            if (($mask -band ([uint64]::MaxValue -bxor $knownMask)) -ne 0) {
                throw "Raw cell activity mask has unknown bits at index ${count}: $mask"
            }
            if ($mask -ne 0) { $any++ }
            if (($mask -band $MatterBit) -ne 0) { $matter++ }
            if (($mask -band $ThermalBit) -ne 0) { $thermal++ }
            if (($mask -band $PressureBit) -ne 0) { $pressure++ }
            if (($mask -band $ReactionBit) -ne 0) { $reaction++ }
            $count++
        }
        if ($count -ne $ExpectedCount) {
            throw "Raw cell row count mismatch: expected $ExpectedCount, got $count."
        }
        return [ordered]@{
            count = $count
            census_tick = $censusTick
            any_active_cells = $any
            matter_active_cells = $matter
            thermal_active_cells = $thermal
            pressure_active_cells = $pressure
            reaction_active_cells = $reaction
        }
    }
    finally {
        $reader.Dispose()
    }
}

function Read-RawChunks {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][uint64]$ExpectedCount,
        [Parameter(Mandatory)][string]$ExpectedRunId,
        [Parameter(Mandatory)][string]$ExpectedCommit,
        [Parameter(Mandatory)][uint64]$KnownActivityMask,
        [Parameter(Mandatory)][uint64]$RunnableState,
        [Parameter(Mandatory)][uint64]$SleepingState
    )
    $reader = [IO.StreamReader]::new($Path, [Text.UTF8Encoding]::new($false, $true), $true, 262144)
    try {
        $headerLine = $reader.ReadLine()
        if ($null -eq $headerLine) {
            throw 'Raw chunk CSV is empty.'
        }
        $header = @($headerLine.Split(',', [StringSplitOptions]::None))
        $expectedHeader = @('schema_version', 'run_id', 'commit_sha', 'git_state', 'census_tick', 'index', 'activity_mask', 'chunk_state')
        if (($header -join ',') -cne ($expectedHeader -join ',')) {
            throw "Raw chunk header mismatch: $($header -join ',')"
        }
        $count = [uint64]0
        $active = [uint64]0
        $runnable = [uint64]0
        $sleeping = [uint64]0
        $censusTick = $null
        while (($line = $reader.ReadLine()) -ne $null) {
            if ($line.Length -eq 0) {
                throw "Raw chunk CSV contains a blank line at data row $count."
            }
            $fields = @($line.Split(',', [StringSplitOptions]::None))
            if ($fields.Count -ne $expectedHeader.Count) {
                throw "Raw chunk row $count has $($fields.Count) columns."
            }
            if ($fields[0] -cne $script:expectedCsvSchema -or $fields[1] -cne $ExpectedRunId -or
                $fields[2] -cne $ExpectedCommit -or $fields[3] -cne 'clean') {
                throw "Raw chunk provenance mismatch at row $count."
            }
            $rowTick = ConvertTo-StrictUInt64 -Text $fields[4] -Label "raw chunk census_tick row $count"
            if ($null -eq $censusTick) { $censusTick = $rowTick }
            elseif ($rowTick -ne $censusTick) { throw "Raw chunk census_tick changes at row $count." }
            $rowIndex = ConvertTo-StrictUInt64 -Text $fields[5] -Label "raw chunk index row $count"
            if ($rowIndex -ne $count) {
                throw "Raw chunk index discontinuity: expected $count, got $rowIndex."
            }
            $mask = ConvertTo-StrictUInt64 -Text $fields[6] -Label "raw chunk activity_mask row $count"
            if (($mask -band ([uint64]::MaxValue -bxor $KnownActivityMask)) -ne 0) {
                throw "Raw chunk activity mask has unknown bits at index ${count}: $mask"
            }
            $state = ConvertTo-StrictUInt64 -Text $fields[7] -Label "raw chunk state row $count"
            if ($state -ne $RunnableState -and $state -ne $SleepingState) {
                throw "Raw chunk state is not a receipt-defined runnable/sleeping value at index ${count}: $state"
            }
            if ($mask -ne 0) { $active++ }
            if ($state -eq $RunnableState) { $runnable++ } else { $sleeping++ }
            $count++
        }
        if ($count -ne $ExpectedCount) {
            throw "Raw chunk row count mismatch: expected $ExpectedCount, got $count."
        }
        return [ordered]@{
            count = $count
            census_tick = $censusTick
            active_chunks = $active
            runnable_chunks = $runnable
            sleeping_chunks = $sleeping
        }
    }
    finally {
        $reader.Dispose()
    }
}

function Test-CsvAndRecomputation {
    param(
        [Parameter(Mandatory)][string]$CaptureRoot,
        [Parameter(Mandatory)][object]$Receipt
    )
    $aggregatePath = Resolve-CapturePath -CaptureRoot $CaptureRoot -RelativePath 'artifacts/aggregate.csv'
    $rawTicksPath = Resolve-CapturePath -CaptureRoot $CaptureRoot -RelativePath 'artifacts/aggregate_raw_ticks.csv'
    $rawCellsPath = Resolve-CapturePath -CaptureRoot $CaptureRoot -RelativePath 'artifacts/aggregate_raw_cells.csv'
    $rawChunksPath = Resolve-CapturePath -CaptureRoot $CaptureRoot -RelativePath 'artifacts/aggregate_raw_chunks.csv'
    foreach ($required in @($aggregatePath, $rawTicksPath, $rawCellsPath, $rawChunksPath)) {
        if (-not (Test-Path -LiteralPath $required -PathType Leaf)) {
            throw "Required artifact is missing: $required"
        }
    }

    $aggregateHeaderLine = Read-FirstUtf8Line $aggregatePath
    $rawTickHeaderLine = Read-FirstUtf8Line $rawTicksPath
    if ($null -eq $aggregateHeaderLine -or $null -eq $rawTickHeaderLine) { throw 'Aggregate or raw tick CSV is empty.' }
    $aggregateHeader = @($aggregateHeaderLine.Split(',', [StringSplitOptions]::None))
    $rawTickHeader = @($rawTickHeaderLine.Split(',', [StringSplitOptions]::None))
    foreach ($headerSpec in @(
        [ordered]@{ label = 'aggregate'; columns = $aggregateHeader; required = @('schema_version', 'run_id', 'commit_sha', 'git_state', 'width', 'height', 'chunk_size', 'trial', 'tick_start', 'tick_end', 'metric_type', 'name', 'value', 'count', 'p50', 'p95', 'mean', 'min', 'max') },
        [ordered]@{ label = 'raw tick'; columns = $rawTickHeader; required = @('schema_version', 'run_id', 'commit_sha', 'git_state', 'width', 'height', 'chunk_size', 'timestamp_period_ns', 'trial', 'sample_id', 'tick_index', 'gpu_pass_sum_ms', 'gpu_tick_envelope_ms', 'residual_ms') }
    )) {
        if ($headerSpec.columns.Count -eq 0 -or @($headerSpec.columns | Sort-Object -Unique).Count -ne $headerSpec.columns.Count) {
            throw "$($headerSpec.label) CSV header is empty or contains duplicate columns."
        }
        foreach ($requiredColumn in $headerSpec.required) {
            if ($requiredColumn -cnotin $headerSpec.columns) { throw "$($headerSpec.label) CSV is missing required column: $requiredColumn" }
        }
    }

    $aggregateRows = @(Import-Csv -LiteralPath $aggregatePath)
    $rawTickRows = @(Import-Csv -LiteralPath $rawTicksPath)
    if ($aggregateRows.Count -eq 0 -or $rawTickRows.Count -eq 0) {
        throw 'Aggregate and raw tick CSVs must both contain data rows.'
    }
    $aggregateSchema = Get-SingleStringValue -Rows $aggregateRows -Property 'schema_version' -Label 'aggregate schema_version'
    $aggregateRunId = Get-SingleStringValue -Rows $aggregateRows -Property 'run_id' -Label 'aggregate run_id'
    $aggregateCommit = Get-SingleStringValue -Rows $aggregateRows -Property 'commit_sha' -Label 'aggregate commit_sha'
    $aggregateGitState = Get-SingleStringValue -Rows $aggregateRows -Property 'git_state' -Label 'aggregate git_state'
    $rawSchema = Get-SingleStringValue -Rows $rawTickRows -Property 'schema_version' -Label 'raw tick schema_version'
    $rawRunId = Get-SingleStringValue -Rows $rawTickRows -Property 'run_id' -Label 'raw tick run_id'
    $rawCommit = Get-SingleStringValue -Rows $rawTickRows -Property 'commit_sha' -Label 'raw tick commit_sha'
    $rawGitState = Get-SingleStringValue -Rows $rawTickRows -Property 'git_state' -Label 'raw tick git_state'
    $receiptRunId = [string](Get-OptionalProperty -Object $Receipt -Name 'run_id')
    $receiptHead = [string](Get-OptionalProperty -Object (Get-OptionalProperty -Object $Receipt -Name 'repository') -Name 'source_sha')
    if ($aggregateSchema -cne $script:expectedCsvSchema -or $rawSchema -cne $script:expectedCsvSchema) {
        throw "CSV schema mismatch: aggregate=$aggregateSchema raw_ticks=$rawSchema"
    }
    if ($aggregateRunId -cne $receiptRunId -or $rawRunId -cne $receiptRunId -or
        $aggregateCommit -cne $receiptHead -or $rawCommit -cne $receiptHead -or
        $aggregateGitState -cne 'clean' -or $rawGitState -cne 'clean') {
        throw 'Aggregate/raw tick/receipt provenance does not match.'
    }

    $width = ConvertTo-StrictUInt64 -Text (Get-SingleStringValue -Rows $aggregateRows -Property 'width' -Label 'aggregate width') -Label 'width'
    $height = ConvertTo-StrictUInt64 -Text (Get-SingleStringValue -Rows $aggregateRows -Property 'height' -Label 'aggregate height') -Label 'height'
    $chunkSize = ConvertTo-StrictUInt64 -Text (Get-SingleStringValue -Rows $aggregateRows -Property 'chunk_size' -Label 'aggregate chunk_size') -Label 'chunk_size'
    if ($width -eq 0 -or $height -eq 0 -or $chunkSize -eq 0) {
        throw 'World dimensions and chunk_size must be nonzero.'
    }
    foreach ($dimension in @('width', 'height', 'chunk_size')) {
        $rawDimension = Get-SingleStringValue -Rows $rawTickRows -Property $dimension -Label "raw tick $dimension"
        $aggregateDimension = Get-SingleStringValue -Rows $aggregateRows -Property $dimension -Label "aggregate $dimension"
        if ($rawDimension -cne $aggregateDimension) { throw "Raw tick/aggregate world dimension mismatch: $dimension" }
    }
    $expectedCellCount = [uint64]($width * $height)
    # PowerShell numeric casts round fractional values. Floor the integer-ceiling
    # numerator explicitly so divisible dimensions do not gain a phantom chunk.
    $chunksX = [uint64][Math]::Floor([double]($width + $chunkSize - 1) / [double]$chunkSize)
    $chunksY = [uint64][Math]::Floor([double]($height + $chunkSize - 1) / [double]$chunkSize)
    $expectedChunkCount = [uint64]($chunksX * $chunksY)
    $encoding = Get-OptionalProperty $Receipt 'census_encoding'
    $cellBits = Get-OptionalProperty $encoding 'cell_activity_bits'
    $chunkStates = Get-OptionalProperty $encoding 'chunk_state_values'
    $matterBit = [uint64](Get-OptionalProperty $cellBits 'matter')
    $thermalBit = [uint64](Get-OptionalProperty $cellBits 'thermal')
    $pressureBit = [uint64](Get-OptionalProperty $cellBits 'pressure')
    $reactionBit = [uint64](Get-OptionalProperty $cellBits 'reaction')
    $runnableState = [uint64](Get-OptionalProperty $chunkStates 'runnable')
    $sleepingState = [uint64](Get-OptionalProperty $chunkStates 'sleeping')
    if ($matterBit -ne 1 -or $thermalBit -ne 2 -or $pressureBit -ne 4 -or $reactionBit -ne 8 -or
        $runnableState -ne 0 -or $sleepingState -ne 1) {
        throw 'Receipt census_encoding does not match the v5 bit/state contract.'
    }
    $knownActivityMask = $matterBit -bor $thermalBit -bor $pressureBit -bor $reactionBit
    $cells = Read-RawCells -Path $rawCellsPath -ExpectedCount $expectedCellCount -ExpectedRunId $receiptRunId -ExpectedCommit $receiptHead -MatterBit $matterBit -ThermalBit $thermalBit -PressureBit $pressureBit -ReactionBit $reactionBit
    $chunks = Read-RawChunks -Path $rawChunksPath -ExpectedCount $expectedChunkCount -ExpectedRunId $receiptRunId -ExpectedCommit $receiptHead -KnownActivityMask $knownActivityMask -RunnableState $runnableState -SleepingState $sleepingState
    if ($cells.census_tick -ne $chunks.census_tick) {
        throw "Raw cell/chunk census ticks differ: $($cells.census_tick) vs $($chunks.census_tick)."
    }

    $censusExpected = [ordered]@{
        total_cells = $expectedCellCount
        any_active_cells = $cells.any_active_cells
        matter_active_cells = $cells.matter_active_cells
        thermal_active_cells = $cells.thermal_active_cells
        pressure_active_cells = $cells.pressure_active_cells
        reaction_active_cells = $cells.reaction_active_cells
        total_chunks = $expectedChunkCount
        active_chunks = $chunks.active_chunks
        runnable_chunks = $chunks.runnable_chunks
        sleeping_chunks = $chunks.sleeping_chunks
    }
    if (@($aggregateRows | Where-Object { $_.metric_type -ceq 'activity_census' }).Count -ne $censusExpected.Count) {
        throw 'Aggregate activity_census row set is incomplete or contains extras.'
    }
    foreach ($entry in $censusExpected.GetEnumerator()) {
        $row = Get-OneRow -Rows $aggregateRows -Predicate {
            $_.metric_type -ceq 'activity_census' -and $_.name -ceq $entry.Key
        } -Label "aggregate census $($entry.Key)"
        $actual = ConvertTo-StrictDouble -Text ([string]$row.value) -Label "aggregate census $($entry.Key)"
        if ($actual -ne [double]$entry.Value) {
            throw "Aggregate census $($entry.Key) mismatch: expected $($entry.Value), got $actual."
        }
        $tickStart = ConvertTo-StrictUInt64 -Text ([string]$row.tick_start) -Label "aggregate census $($entry.Key) tick_start"
        $tickEnd = ConvertTo-StrictUInt64 -Text ([string]$row.tick_end) -Label "aggregate census $($entry.Key) tick_end"
        if ($tickStart -ne $cells.census_tick -or $tickEnd -ne $cells.census_tick) {
            throw "Aggregate census $($entry.Key) tick does not match raw census tick."
        }
    }

    $passColumns = @($rawTickRows[0].PSObject.Properties.Name | Where-Object { $_ -cmatch '^pass_.+_ms$' })
    $groupColumns = @($rawTickRows[0].PSObject.Properties.Name | Where-Object { $_ -cmatch '^group_.+_ms$' })
    if ($passColumns.Count -eq 0 -or $groupColumns.Count -eq 0) {
        throw 'Raw tick CSV has no pass or grouped measurement columns.'
    }
    $tupleKeys = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($row in $rawTickRows) {
        $trialId = ConvertTo-StrictUInt64 -Text ([string]$row.trial) -Label 'raw tick trial'
        $sampleId = ConvertTo-StrictUInt64 -Text ([string]$row.sample_id) -Label 'raw tick sample_id'
        $tickIndex = ConvertTo-StrictUInt64 -Text ([string]$row.tick_index) -Label 'raw tick tick_index'
        $tupleKey = "$trialId/$sampleId/$tickIndex"
        if (-not $tupleKeys.Add($tupleKey)) { throw "Duplicate raw tick identity tuple: $tupleKey" }
        $passSum = 0.0
        $firstPassStart = $null
        $previousPassEnd = $null
        $lastPassEnd = $null
        foreach ($column in $passColumns) {
            $passSum += ConvertTo-StrictDouble -Text ([string]$row.$column) -Label "raw tick $column"
        }
        $groupSum = 0.0
        foreach ($column in $groupColumns) {
            $groupSum += ConvertTo-StrictDouble -Text ([string]$row.$column) -Label "raw tick $column"
        }
        $recordedPassSum = ConvertTo-StrictDouble -Text ([string]$row.gpu_pass_sum_ms) -Label 'raw tick gpu_pass_sum_ms'
        $envelope = ConvertTo-StrictDouble -Text ([string]$row.gpu_tick_envelope_ms) -Label 'raw tick gpu_tick_envelope_ms'
        $residual = ConvertTo-StrictDouble -Text ([string]$row.residual_ms) -Label 'raw tick residual_ms'
        if (-not (Test-Near $passSum $recordedPassSum) -or -not (Test-Near $groupSum $recordedPassSum) -or
            -not (Test-Near ($envelope - $recordedPassSum) $residual)) {
            throw "Raw tick arithmetic mismatch at trial=$($row.trial), sample_id=$($row.sample_id)."
        }
        $timestampPeriod = ConvertTo-StrictDouble -Text ([string]$row.timestamp_period_ns) -Label 'timestamp_period_ns'
        foreach ($column in $passColumns) {
            $passName = $column.Substring(5, $column.Length - 8)
            $startName = "${passName}_start_tick"
            $endName = "${passName}_end_tick"
            if ($null -eq $row.PSObject.Properties[$startName] -or $null -eq $row.PSObject.Properties[$endName]) {
                throw "Raw tick timestamp columns missing for pass $passName."
            }
            $startTick = ConvertTo-StrictUInt64 -Text ([string]$row.$startName) -Label $startName
            $endTick = ConvertTo-StrictUInt64 -Text ([string]$row.$endName) -Label $endName
            if ($endTick -le $startTick) { throw "Raw timestamp pair is empty or reversed for pass $passName." }
            if ($null -ne $previousPassEnd -and $startTick -lt $previousPassEnd) {
                throw "Raw pass timestamps overlap or run out of order before pass $passName."
            }
            if ($null -eq $firstPassStart) { $firstPassStart = $startTick }
            $previousPassEnd = $endTick
            $lastPassEnd = $endTick
            $duration = ([double]($endTick - $startTick) * $timestampPeriod) / 1000000.0
            $recordedDuration = ConvertTo-StrictDouble -Text ([string]$row.$column) -Label $column
            if (-not (Test-Near $duration $recordedDuration)) {
                throw "Raw timestamp duration mismatch for pass $passName."
            }
        }
        $recomputedEnvelope = ([double]($lastPassEnd - $firstPassStart) * $timestampPeriod) / 1000000.0
        if (-not (Test-Near $recomputedEnvelope $envelope)) {
            throw "Raw GPU envelope does not equal first-pass start through last-pass end at trial=$trialId sample_id=$sampleId."
        }
    }

    $trialValues = @{}
    $trials = @($rawTickRows | ForEach-Object { [string]$_.trial } | Sort-Object -Unique)
    $expectedTimingKeys = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($column in $passColumns) {
        [void]$expectedTimingKeys.Add('pass/' + $column.Substring(5, $column.Length - 8))
    }
    foreach ($column in $groupColumns) {
        $groupName = $column.Substring(6, $column.Length - 9)
        [void]$expectedTimingKeys.Add("grouped_subsystem/$groupName")
        [void]$expectedTimingKeys.Add("grouped_envelope_ratio/$groupName")
    }
    foreach ($name in @('gpu_tick_envelope', 'gpu_pass_sum', 'diagnostic_residual')) {
        [void]$expectedTimingKeys.Add("envelope/$name")
    }
    foreach ($trial in $trials) {
        $trialRows = @($rawTickRows | Where-Object { [string]$_.trial -ceq $trial } | Sort-Object { [uint64]$_.sample_id })
        $sampleIds = @($trialRows | ForEach-Object { ConvertTo-StrictUInt64 -Text ([string]$_.sample_id) -Label "trial $trial sample_id" } | Sort-Object)
        for ($index = 0; $index -lt $sampleIds.Count; $index++) {
            if ($sampleIds[$index] -ne [uint64]$index) {
                throw "Raw sample_id discontinuity in trial $trial at position $index."
            }
            $rowSampleId = ConvertTo-StrictUInt64 -Text ([string]$trialRows[$index].sample_id) -Label "trial $trial sample_id"
            $rowTickIndex = ConvertTo-StrictUInt64 -Text ([string]$trialRows[$index].tick_index) -Label "trial $trial tick_index"
            if ($rowSampleId -ne [uint64]$index -or $rowTickIndex -ne [uint64]$index) {
                throw "Raw sample_id/tick_index is not continuous from zero in trial $trial at position $index."
            }
        }
        $aggregateTimingRows = @($aggregateRows | Where-Object {
            [string]$_.trial -ceq $trial -and $_.metric_type -in @('pass', 'grouped_subsystem', 'grouped_envelope_ratio', 'envelope')
        })
        $actualTimingKeys = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        foreach ($aggregateRow in $aggregateTimingRows) {
            $key = "$($aggregateRow.metric_type)/$($aggregateRow.name)"
            if (-not $expectedTimingKeys.Contains($key) -or -not $actualTimingKeys.Add($key)) {
                throw "Aggregate timing row is unexpected or duplicated for trial ${trial}: $key"
            }
            $tickStart = ConvertTo-StrictUInt64 -Text ([string]$aggregateRow.tick_start) -Label "timing trial $trial tick_start"
            $tickEnd = ConvertTo-StrictUInt64 -Text ([string]$aggregateRow.tick_end) -Label "timing trial $trial tick_end"
            if ($tickStart -ne 0 -or $tickEnd -ne [uint64]($trialRows.Count - 1)) {
                throw "Aggregate timing tick range does not match raw rows for trial $trial."
            }
            $values = [Collections.Generic.List[double]]::new()
            switch ([string]$aggregateRow.metric_type) {
                'pass' {
                    $column = "pass_$($aggregateRow.name)_ms"
                    foreach ($rawRow in $trialRows) { $values.Add((ConvertTo-StrictDouble ([string]$rawRow.$column) $column)) }
                }
                'grouped_subsystem' {
                    $column = "group_$($aggregateRow.name)_ms"
                    foreach ($rawRow in $trialRows) { $values.Add((ConvertTo-StrictDouble ([string]$rawRow.$column) $column)) }
                }
                'grouped_envelope_ratio' {
                    $column = "group_$($aggregateRow.name)_ms"
                    foreach ($rawRow in $trialRows) {
                        $group = ConvertTo-StrictDouble ([string]$rawRow.$column) $column
                        $envelope = ConvertTo-StrictDouble ([string]$rawRow.gpu_tick_envelope_ms) 'gpu_tick_envelope_ms'
                        if ($envelope -eq 0.0) { throw 'Cannot recompute group/envelope ratio with zero envelope.' }
                        $values.Add(($group / $envelope) * 100.0)
                    }
                }
                'envelope' {
                    $column = switch ([string]$aggregateRow.name) {
                        'gpu_tick_envelope' { 'gpu_tick_envelope_ms' }
                        'gpu_pass_sum' { 'gpu_pass_sum_ms' }
                        'diagnostic_residual' { 'residual_ms' }
                        default { throw "Unknown aggregate envelope metric: $($aggregateRow.name)" }
                    }
                    foreach ($rawRow in $trialRows) { $values.Add((ConvertTo-StrictDouble ([string]$rawRow.$column) $column)) }
                }
            }
            Assert-StatsRow -Row $aggregateRow -Values $values.ToArray() -Label "aggregate $($aggregateRow.metric_type)/$($aggregateRow.name)/trial=$trial"
        }
        if ($actualTimingKeys.Count -ne $expectedTimingKeys.Count) {
            throw "Aggregate timing row set is incomplete for trial ${trial}: expected $($expectedTimingKeys.Count), got $($actualTimingKeys.Count)."
        }
    }

    foreach ($trial in @($aggregateRows | Where-Object { $_.metric_type -ceq 'throughput_trial' } | ForEach-Object { [string]$_.trial } | Sort-Object -Unique)) {
        $elapsedRow = Get-OneRow $aggregateRows { $_.metric_type -ceq 'throughput_trial' -and [string]$_.trial -ceq $trial -and $_.name -ceq 'elapsed_wall' } "throughput elapsed trial $trial"
        $wallRow = Get-OneRow $aggregateRows { $_.metric_type -ceq 'throughput_trial' -and [string]$_.trial -ceq $trial -and $_.name -ceq 'wall_per_tick' } "throughput wall trial $trial"
        $tpsRow = Get-OneRow $aggregateRows { $_.metric_type -ceq 'throughput_trial' -and [string]$_.trial -ceq $trial -and $_.name -ceq 'sustained_tps' } "throughput tps trial $trial"
        $elapsed = ConvertTo-StrictDouble ([string]$elapsedRow.value) "throughput elapsed trial $trial"
        $wall = ConvertTo-StrictDouble ([string]$wallRow.value) "throughput wall trial $trial"
        $tps = ConvertTo-StrictDouble ([string]$tpsRow.value) "throughput tps trial $trial"
        $tickStart = ConvertTo-StrictUInt64 ([string]$elapsedRow.tick_start) 'throughput tick_start'
        $tickEnd = ConvertTo-StrictUInt64 ([string]$elapsedRow.tick_end) 'throughput tick_end'
        if ($tickEnd -lt $tickStart) { throw "Invalid throughput tick range for trial $trial." }
        $tickCount = [double]($tickEnd - $tickStart + 1)
        if ($elapsed -le 0.0 -or -not (Test-Near $wall ($elapsed / $tickCount)) -or -not (Test-Near $tps (($tickCount * 1000.0) / $elapsed))) {
            throw "Throughput arithmetic mismatch for trial $trial."
        }
        $trialValues[$trial] = [ordered]@{ wall_per_tick = $wall; sustained_tps = $tps }
    }
    foreach ($metric in @('wall_per_tick', 'sustained_tps')) {
        $summaryRow = Get-OneRow $aggregateRows { $_.metric_type -ceq 'throughput_summary' -and $_.name -ceq $metric } "throughput summary $metric"
        $values = @($trialValues.Keys | Sort-Object | ForEach-Object { [double]$trialValues[$_][$metric] })
        Assert-StatsRow -Row $summaryRow -Values $values -Label "throughput summary $metric"
    }

    $overheadRows = @($aggregateRows | Where-Object { $_.metric_type -ceq 'profiling_overhead' })
    if ($overheadRows.Count -ne 6) { throw "Aggregate profiling_overhead row count must be 6; got $($overheadRows.Count)." }
    $batched = ConvertTo-StrictDouble ([string](Get-OneRow $overheadRows { $_.name -ceq 'batched_unprofiled_elapsed' } 'overhead batched').value) 'overhead batched'
    $synchronized = ConvertTo-StrictDouble ([string](Get-OneRow $overheadRows { $_.name -ceq 'synchronized_unprofiled_elapsed' } 'overhead synchronized').value) 'overhead synchronized'
    $profiled = ConvertTo-StrictDouble ([string](Get-OneRow $overheadRows { $_.name -ceq 'synchronized_profiled_elapsed' } 'overhead profiled').value) 'overhead profiled'
    if ($batched -eq 0.0 -or $synchronized -eq 0.0) { throw 'Overhead baselines cannot be zero.' }
    $overheadExpected = [ordered]@{
        synchronization_overhead = (($synchronized - $batched) / $batched) * 100.0
        profiling_increment = (($profiled - $synchronized) / $synchronized) * 100.0
        observed_profiled_path_overhead = (($profiled - $batched) / $batched) * 100.0
    }
    foreach ($entry in $overheadExpected.GetEnumerator()) {
        $row = Get-OneRow $overheadRows { $_.name -ceq $entry.Key } "overhead $($entry.Key)"
        $actual = ConvertTo-StrictDouble ([string]$row.value) "overhead $($entry.Key)"
        if (-not (Test-Near $actual ([double]$entry.Value))) {
            throw "Overhead recomputation mismatch for $($entry.Key): expected $($entry.Value), got $actual."
        }
    }

    $script:derived.raw_cell_rows = $cells.count
    $script:derived.raw_chunk_rows = $chunks.count
    $script:derived.raw_tick_rows = $rawTickRows.Count
    $script:derived.aggregate_rows = $aggregateRows.Count
    $script:derived.world = [ordered]@{ width = $width; height = $height; chunk_size = $chunkSize }
    $script:derived.recomputed_census = $censusExpected
    $script:derived.aggregate_stat_rows_recomputed = @($aggregateRows | Where-Object { $_.metric_type -in @('pass', 'grouped_subsystem', 'grouped_envelope_ratio', 'envelope', 'throughput_summary') }).Count
    return [ordered]@{ run_id = $receiptRunId; commit_sha = $receiptHead }
}

function Write-JsonCreateNew {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][object]$Value
    )
    $parent = Split-Path -Parent ([IO.Path]::GetFullPath($Path))
    if (-not (Test-Path -LiteralPath $parent -PathType Container)) {
        throw "Output parent directory does not exist: $parent"
    }
    $json = ($Value | ConvertTo-Json -Depth 30) + "`n"
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes($json)
    $stream = [IO.File]::Open([IO.Path]::GetFullPath($Path), [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
    try {
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.Flush($true)
    }
    finally {
        $stream.Dispose()
    }
}

# The verifier report is a new artifact. Refuse a caller path inside the
# sibling immutable capture directory before entering the reporting catch path.
$preflightPackageFull = [IO.Path]::GetFullPath($PackagePath)
$preflightOutputFull = [IO.Path]::GetFullPath($OutputPath)
$preflightCaptureRoot = Join-Path (Split-Path -Parent $preflightPackageFull) ([IO.Path]::GetFileNameWithoutExtension($preflightPackageFull))
$preflightCapturePrefix = [IO.Path]::GetFullPath($preflightCaptureRoot).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
if ($preflightOutputFull -eq [IO.Path]::GetFullPath($preflightCaptureRoot) -or
    $preflightOutputFull.StartsWith($preflightCapturePrefix, [StringComparison]::OrdinalIgnoreCase)) {
    throw "Verifier output must be outside the immutable capture directory: $preflightCaptureRoot"
}

try {
    $packageFull = [IO.Path]::GetFullPath($PackagePath)
    $outputFull = [IO.Path]::GetFullPath($OutputPath)
    if ([string]::IsNullOrWhiteSpace($PackageHashPath)) {
        $PackageHashPath = Join-Path (Split-Path -Parent $packageFull) 'PACKAGE_SHA256.txt'
    }
    $packageHashFull = [IO.Path]::GetFullPath($PackageHashPath)
    if (-not (Test-Path -LiteralPath $packageFull -PathType Leaf)) { throw "Package ZIP does not exist: $packageFull" }
    if (-not (Test-Path -LiteralPath $packageHashFull -PathType Leaf)) { throw "External package hash does not exist: $packageHashFull" }
    if (Test-Path -LiteralPath $outputFull) { throw "Refusing to overwrite verifier output: $outputFull" }
    if ($outputFull -eq $packageFull -or $outputFull -eq $packageHashFull) { throw 'Verifier output must not replace an input.' }

    $packageSha256 = Get-Sha256Hex -Path $packageFull
    $hashRecordLines = @([IO.File]::ReadAllLines($packageHashFull) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    if ($hashRecordLines.Count -ne 1 -or $hashRecordLines[0] -notmatch '^([0-9A-Fa-f]{64})  ([^\r\n]+)$') {
        throw 'PACKAGE_SHA256.txt must contain exactly one "<sha256><two spaces><zip filename>" record.'
    }
    $recordedPackageHash = $Matches[1].ToLowerInvariant()
    $recordedPackageName = $Matches[2]
    $packageHashValid = $recordedPackageHash -ceq $packageSha256 -and $recordedPackageName -ceq [IO.Path]::GetFileName($packageFull)
    Add-VerificationCheck -Id 'package.external_sha256' -Passed $packageHashValid -Detail "recorded=$recordedPackageHash actual=$packageSha256 filename=$recordedPackageName"
    if (-not $packageHashValid) { throw 'External package SHA-256 record does not match the ZIP.' }
    $derived.package_hash_record_sha256 = Get-Sha256Hex $packageHashFull

    $temporaryDirectory = Join-Path ([IO.Path]::GetTempPath()) ('powdergame-verify-' + [Guid]::NewGuid().ToString('N'))
    [IO.Directory]::CreateDirectory($temporaryDirectory) | Out-Null
    $archive = [IO.Compression.ZipFile]::OpenRead($packageFull)
    try {
        if ($archive.Entries.Count -eq 0) { throw 'Package ZIP is empty.' }
        $entryNames = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
        $rootNames = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        foreach ($entry in $archive.Entries) {
            $name = $entry.FullName.Replace([char]92, [char]47).TrimEnd('/')
            if ([string]::IsNullOrWhiteSpace($name) -or -not (Test-SafeRelativePath $name)) {
                throw "ZIP contains an unsafe entry path: $($entry.FullName)"
            }
            if (-not $entryNames.Add($name)) { throw "ZIP contains a duplicate/case-colliding entry: $name" }
            $segments = @($name.Split('/'))
            [void]$rootNames.Add($segments[0])
            $unixType = (([uint32]$entry.ExternalAttributes -shr 16) -band 0xF000)
            if ($unixType -eq 0xA000) { throw "ZIP contains a symbolic link entry: $name" }
        }
        if ($rootNames.Count -ne 1) { throw "ZIP must contain exactly one capture root; found $($rootNames.Count)." }
        $captureId = [string]@($rootNames)[0]
    }
    finally {
        $archive.Dispose()
    }
    [IO.Compression.ZipFile]::ExtractToDirectory($packageFull, $temporaryDirectory, $false)
    $captureRoot = Join-Path $temporaryDirectory $captureId
    if (-not (Test-Path -LiteralPath $captureRoot -PathType Container)) { throw 'Extracted capture root is missing.' }
    Add-VerificationCheck -Id 'package.safe_single_root' -Passed $true -Detail "capture_root=$captureId"

    $receiptPath = Resolve-CapturePath $captureRoot 'CAPTURE_RECEIPT.json'
    if (-not (Test-Path -LiteralPath $receiptPath -PathType Leaf)) { throw 'CAPTURE_RECEIPT.json is absent; capture is incomplete.' }
    $receipt = Get-Content -Raw -LiteralPath $receiptPath | ConvertFrom-Json
    $receiptRepository = Get-OptionalProperty $receipt 'repository'
    $receiptSource = Get-OptionalProperty $receipt 'source'
    $receiptHead = [string](Get-OptionalProperty $receiptRepository 'source_sha')
    $receiptBranch = [string](Get-OptionalProperty $receiptRepository 'branch')
    $receiptValid = [string](Get-OptionalProperty $receipt 'receipt_schema') -ceq $expectedReceiptSchema -and
        [bool](Get-OptionalProperty $receipt 'official_mode') -eq $true -and
        [bool](Get-OptionalProperty $receipt 'complete') -eq $true -and
        [string](Get-OptionalProperty $receipt 'capture_id') -ceq $captureId -and
        [string](Get-OptionalProperty $receipt 'capture_id') -ceq [IO.Path]::GetFileNameWithoutExtension($packageFull) -and
        [string](Get-OptionalProperty $receipt 'schema_version') -ceq $expectedCsvSchema -and
        [bool](Get-OptionalProperty $receipt 'run_id_links_complete') -and
        [string](Get-OptionalProperty $receiptRepository 'git_state') -ceq 'clean' -and
        [bool](Get-OptionalProperty $receiptRepository 'clean') -and
        $receiptHead -cmatch '^[0-9a-f]{40}([0-9a-f]{24})?$' -and
        -not [string]::IsNullOrWhiteSpace($receiptBranch) -and
        -not [string]::IsNullOrWhiteSpace([string](Get-OptionalProperty $receipt 'run_id')) -and
        [uint64](Get-OptionalProperty $receiptSource 'input_count') -gt 0 -and
        [string](Get-OptionalProperty $receiptSource 'manifest_sha256') -cmatch '^[0-9a-f]{64}$' -and
        [bool](Get-OptionalProperty $receiptSource 'manifests_identical') -and
        [bool](Get-OptionalProperty $receiptSource 'source_unchanged')
    Add-VerificationCheck -Id 'receipt.complete_official_marker' -Passed $receiptValid -Detail "schema=$([string](Get-OptionalProperty $receipt 'receipt_schema')) official=$([string](Get-OptionalProperty $receipt 'official_mode')) complete=$([string](Get-OptionalProperty $receipt 'complete'))"
    if (-not $receiptValid) { throw 'Receipt completeness, official-mode, identity, or clean-source fields are invalid.' }

    $hashManifestRelative = [string](Get-OptionalProperty (Get-OptionalProperty $receipt 'hash_manifest') 'path')
    $hashManifestRecordedSha = [string](Get-OptionalProperty (Get-OptionalProperty $receipt 'hash_manifest') 'sha256')
    $hashManifestRecordedCount = [uint64](Get-OptionalProperty (Get-OptionalProperty $receipt 'hash_manifest') 'file_count')
    if ($hashManifestRelative -cne 'hashes/SHA256SUMS.txt' -or
        -not [bool](Get-OptionalProperty (Get-OptionalProperty $receipt 'hash_manifest') 'excludes_self_and_receipt')) {
        throw "Unexpected hash manifest contract: $hashManifestRelative"
    }
    $hashManifestPath = Resolve-CapturePath $captureRoot $hashManifestRelative
    if ((Get-Sha256Hex $hashManifestPath) -cne $hashManifestRecordedSha) { throw 'Receipt hash-manifest SHA-256 does not match.' }
    $manifestRecords = @{}
    foreach ($line in [IO.File]::ReadLines($hashManifestPath)) {
        if ($line -notmatch '^([0-9A-Fa-f]{64})  ([^\r\n]+)$') { throw "Malformed internal hash record: $line" }
        $sha = $Matches[1].ToLowerInvariant()
        $relative = $Matches[2].Replace([char]92, [char]47)
        if (-not (Test-SafeRelativePath $relative) -or $relative -ceq 'CAPTURE_RECEIPT.json' -or $relative -ceq $hashManifestRelative) {
            throw "Invalid path in internal hash manifest: $relative"
        }
        if ($manifestRecords.ContainsKey($relative.ToLowerInvariant())) { throw "Duplicate internal hash path: $relative" }
        $manifestRecords[$relative.ToLowerInvariant()] = [ordered]@{ path = $relative; sha256 = $sha }
    }
    if ([uint64]$manifestRecords.Count -ne $hashManifestRecordedCount) { throw 'Receipt hash-manifest file_count does not match manifest rows.' }
    $actualHashableFiles = @(Get-ChildItem -LiteralPath $captureRoot -Recurse -File | Where-Object {
        $relative = Get-RelativeForwardPath $captureRoot $_.FullName
        $relative -cne 'CAPTURE_RECEIPT.json' -and $relative -cne $hashManifestRelative
    })
    if ($actualHashableFiles.Count -ne $manifestRecords.Count) { throw "Internal hash coverage mismatch: files=$($actualHashableFiles.Count), records=$($manifestRecords.Count)." }
    $verifiedFileHashes = [ordered]@{}
    foreach ($file in $actualHashableFiles) {
        $relative = Get-RelativeForwardPath $captureRoot $file.FullName
        $key = $relative.ToLowerInvariant()
        if (-not $manifestRecords.ContainsKey($key)) { throw "Internal file lacks a hash record: $relative" }
        $actualSha = Get-Sha256Hex $file.FullName
        if ($actualSha -cne $manifestRecords[$key].sha256) { throw "Internal hash mismatch: $relative" }
        $verifiedFileHashes[$relative] = $actualSha
    }
    Add-VerificationCheck -Id 'hashes.internal_complete' -Passed $true -Detail "verified_files=$($manifestRecords.Count) receipt_sha256=$(Get-Sha256Hex $receiptPath)"
    $derived.receipt_sha256 = Get-Sha256Hex $receiptPath
    $derived.internal_file_sha256 = $verifiedFileHashes

    $publicationOrder = @((Get-OptionalProperty $receipt 'publication_order') | ForEach-Object { [string]$_ })
    $expectedPublicationOrder = @(
        'artifacts/aggregate_raw_cells.csv',
        'artifacts/aggregate_raw_chunks.csv',
        'artifacts/aggregate_raw_ticks.csv',
        'artifacts/aggregate.csv',
        'metadata/CAPTURE_METADATA.json',
        'hashes/SHA256SUMS.txt',
        'CAPTURE_RECEIPT.json'
    )
    $publicationValid = ($publicationOrder -join "`n") -ceq ($expectedPublicationOrder -join "`n") -and $publicationOrder[-1] -ceq 'CAPTURE_RECEIPT.json'
    Add-VerificationCheck -Id 'receipt.publication_order' -Passed $publicationValid -Detail ($publicationOrder -join ' -> ')
    if (-not $publicationValid) { throw 'Receipt publication order is not the required v5 receipt-last order.' }

    $packageContract = Get-OptionalProperty $receipt 'package'
    if (-not [bool](Get-OptionalProperty $packageContract 'created_after_receipt') -or
        [string](Get-OptionalProperty $packageContract 'zip_path_outside_capture') -cne [IO.Path]::GetFileName($packageFull) -or
        [string](Get-OptionalProperty $packageContract 'zip_sha256_path_outside_zip') -cne [IO.Path]::GetFileName($packageHashFull)) {
        throw 'Receipt package contract does not match the ZIP and external hash inputs.'
    }

    $metadataRecord = Get-OptionalProperty $receipt 'metadata'
    $metadataPath = Assert-CaptureFileRecord -CaptureRoot $captureRoot -Record $metadataRecord -ExpectedRelativePath 'metadata/CAPTURE_METADATA.json' -Label 'receipt metadata'
    $metadata = Get-Content -Raw -LiteralPath $metadataPath | ConvertFrom-Json
    $metadataRepository = Get-OptionalProperty $metadata 'repository'
    $metadataSource = Get-OptionalProperty $metadata 'source'
    if ([string](Get-OptionalProperty $metadata 'metadata_schema') -cne 'powdergame-g8a-capture-metadata-v2' -or
        -not [bool](Get-OptionalProperty $metadata 'official_mode') -or
        [string](Get-OptionalProperty $metadata 'capture_id') -cne $captureId -or
        [string](Get-OptionalProperty $metadata 'run_id') -cne [string]$receipt.run_id -or
        [string](Get-OptionalProperty $metadataRepository 'source_sha') -cne $receiptHead -or
        [string](Get-OptionalProperty $metadataRepository 'branch') -cne $receiptBranch -or
        [string](Get-OptionalProperty $metadataRepository 'git_state') -cne 'clean' -or
        -not [bool](Get-OptionalProperty $metadataRepository 'clean_before') -or
        -not [bool](Get-OptionalProperty $metadataRepository 'clean_after') -or
        [string](Get-OptionalProperty $metadataRepository 'origin_url') -cne [string](Get-OptionalProperty $receiptRepository 'origin_url') -or
        [string](Get-OptionalProperty $metadataRepository 'upstream_ref') -cne [string](Get-OptionalProperty $receiptRepository 'upstream_ref') -or
        [string](Get-OptionalProperty $metadataRepository 'upstream_sha') -cne [string](Get-OptionalProperty $receiptRepository 'upstream_sha')) {
        throw 'Capture metadata identity/clean-source fields do not match the receipt.'
    }
    if ([string](Get-OptionalProperty $metadata 'started_utc') -cne [string](Get-OptionalProperty $receipt 'started_utc') -or
        [string]::IsNullOrWhiteSpace([string](Get-OptionalProperty $receipt 'completed_utc')) -or
        [string]::IsNullOrWhiteSpace([string](Get-OptionalProperty $metadata 'metadata_created_utc'))) {
        throw 'Capture receipt/metadata timestamps are absent or not linked.'
    }
    $metadataToolchain = Get-OptionalProperty $metadata 'toolchain'
    foreach ($toolchainField in @('powershell', 'os_description', 'process_architecture', 'git', 'cargo', 'rustc')) {
        if ([string]::IsNullOrWhiteSpace([string](Get-OptionalProperty $metadataToolchain $toolchainField))) {
            throw "Capture metadata toolchain field is blank: $toolchainField"
        }
    }
    if ([string](Get-OptionalProperty $metadataSource 'snapshot_root') -cne 'source/snapshot') {
        throw 'Capture metadata snapshot_root is not source/snapshot.'
    }
    $metadataManifests = Get-OptionalProperty $metadataSource 'manifests'
    $manifestKeys = @('before_build', 'after_build', 'before_run', 'after_run')
    $sourceManifestPaths = @($manifestKeys | ForEach-Object {
        [string](Get-OptionalProperty (Get-OptionalProperty $metadataManifests $_) 'path')
    })
    $receiptManifestPaths = @((Get-OptionalProperty $receiptSource 'manifest_paths') | ForEach-Object { [string]$_ })
    $expectedSourceManifests = @(
        'source/SOURCE_INPUTS_BEFORE_BUILD.tsv',
        'source/SOURCE_INPUTS_AFTER_BUILD.tsv',
        'source/SOURCE_INPUTS_BEFORE_RUN.tsv',
        'source/SOURCE_INPUTS_AFTER_RUN.tsv'
    )
    if (($sourceManifestPaths -join "`n") -cne ($expectedSourceManifests -join "`n") -or
        ($receiptManifestPaths -join "`n") -cne ($expectedSourceManifests -join "`n")) {
        throw 'Receipt/metadata source manifest path list is incomplete or out of order.'
    }
    $sourceInputCount = [uint64](Get-OptionalProperty $receiptSource 'input_count')
    if ($sourceInputCount -eq 0 -or $sourceInputCount -ne [uint64](Get-OptionalProperty $metadataSource 'input_count')) { throw 'Receipt/metadata source input count is invalid or mismatched.' }
    $baselineManifestText = $null
    for ($manifestIndex = 0; $manifestIndex -lt $sourceManifestPaths.Count; $manifestIndex++) {
        $relative = $sourceManifestPaths[$manifestIndex]
        $manifestRecord = Get-OptionalProperty $metadataManifests $manifestKeys[$manifestIndex]
        $manifestPath = Assert-CaptureFileRecord -CaptureRoot $captureRoot -Record $manifestRecord -ExpectedRelativePath $relative -Label "source manifest $($manifestKeys[$manifestIndex])"
        $lines = @([IO.File]::ReadAllLines($manifestPath))
        if ($lines.Count -ne ($sourceInputCount + 1)) { throw "Inventory row count mismatch in ${relative}: expected $sourceInputCount, got $($lines.Count - 1)." }
        if ($lines[0] -cne "exists`tsize_bytes`tsha256`trepository_relative_path`tsnapshot_relative_path") { throw "Inventory header mismatch: $relative" }
        $seenPaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
        foreach ($line in $lines[1..($lines.Count - 1)]) {
            $columns = @($line.Split("`t", [StringSplitOptions]::None))
            if ($columns.Count -ne 5 -or $columns[0] -cne 'true') { throw "Invalid inventory row in $relative" }
            $size = ConvertTo-StrictUInt64 $columns[1] "inventory size $relative"
            if ($columns[2] -cnotmatch '^[0-9a-f]{64}$' -or -not (Test-SafeRelativePath $columns[3]) -or -not (Test-SafeRelativePath $columns[4])) { throw "Invalid inventory path/hash in $relative" }
            if (-not $seenPaths.Add($columns[3])) { throw "Duplicate inventory repository path in ${relative}: $($columns[3])" }
            $expectedSnapshot = 'source/snapshot/' + $columns[3].Replace([char]92, [char]47)
            if ($columns[4] -cne $expectedSnapshot) { throw "Inventory snapshot mapping mismatch in ${relative}: $($columns[3])" }
            $snapshotPath = Resolve-CapturePath $captureRoot $columns[4]
            if (-not (Test-Path -LiteralPath $snapshotPath -PathType Leaf)) { throw "Inventory snapshot is missing: $($columns[4])" }
            if ([uint64](Get-Item -LiteralPath $snapshotPath).Length -ne $size -or (Get-Sha256Hex $snapshotPath) -cne $columns[2]) { throw "Inventory snapshot size/hash mismatch: $($columns[4])" }
        }
        $text = [IO.File]::ReadAllText($manifestPath)
        if ($null -eq $baselineManifestText) { $baselineManifestText = $text }
        elseif ($text -cne $baselineManifestText) { throw "Source inventory changed during capture: $relative" }
    }
    $baselineManifestSha = Get-Sha256Hex (Resolve-CapturePath $captureRoot $sourceManifestPaths[0])
    if ($baselineManifestSha -cne [string](Get-OptionalProperty $receiptSource 'manifest_sha256') -or
        -not [bool](Get-OptionalProperty $metadataSource 'manifests_identical')) {
        throw 'Receipt/metadata source manifest binding is invalid.'
    }
    Add-VerificationCheck -Id 'source.inventory_stable_snapshot_bound' -Passed $true -Detail "paths=$sourceInputCount manifests=$($sourceManifestPaths.Count)"

    $statusRelatives = @(
        [string](Get-OptionalProperty $receiptSource 'status_before_path'),
        [string](Get-OptionalProperty $receiptSource 'status_after_path')
    )
    if (($statusRelatives -join "`n") -cne ("source/GIT_STATUS_BEFORE.txt`nsource/GIT_STATUS_AFTER.txt")) {
        throw 'Receipt status paths do not match the v2 capture layout.'
    }
    foreach ($statusRelative in $statusRelatives) {
        $statusText = [IO.File]::ReadAllText((Resolve-CapturePath $captureRoot $statusRelative))
        if (@($statusText -split "`r?`n" | Where-Object { $_ -cmatch '^[12u?] ' }).Count -ne 0) { throw "Official capture status is dirty: $statusRelative" }
        $oidLine = @($statusText -split "`r?`n" | Where-Object { $_ -cmatch '^# branch\.oid ' })
        $headLine = @($statusText -split "`r?`n" | Where-Object { $_ -cmatch '^# branch\.head ' })
        if ($oidLine.Count -ne 1 -or $oidLine[0].Substring(13) -cne $receiptHead -or
            $headLine.Count -ne 1 -or $headLine[0].Substring(14) -cne $receiptBranch) {
            throw "Git status provenance mismatch: $statusRelative"
        }
    }
    foreach ($diffName in @('dirty_diff_before', 'dirty_diff_after')) {
        $diffRecord = Get-OptionalProperty $metadataSource $diffName
        $receiptDiffRecord = Get-OptionalProperty $receiptSource $diffName
        if ((($diffRecord | ConvertTo-Json -Compress) -cne ($receiptDiffRecord | ConvertTo-Json -Compress))) {
            throw "Receipt/metadata diff file record mismatch: $diffName"
        }
        $dirtyDiffPath = Assert-CaptureFileRecord -CaptureRoot $captureRoot -Record $diffRecord -Label "metadata $diffName"
        if ((Get-Item -LiteralPath $dirtyDiffPath).Length -ne 0) { throw "Official clean capture contains a nonempty $diffName." }
    }
    Add-VerificationCheck -Id 'source.clean_git_state' -Passed $true -Detail "head=$receiptHead branch=$receiptBranch"

    $executableRecord = Get-OptionalProperty $receipt 'executable'
    $executableRelative = [string](Get-OptionalProperty $executableRecord 'path')
    $executablePath = Resolve-CapturePath $captureRoot $executableRelative
    $executableSha = Get-Sha256Hex $executablePath
    $metadataExecutable = Get-OptionalProperty $metadata 'executable'
    $executableValid = $executableRelative -ceq [string](Get-OptionalProperty $metadataExecutable 'path') -and
        $executableSha -ceq [string](Get-OptionalProperty $executableRecord 'sha256') -and
        $executableSha -ceq [string](Get-OptionalProperty $metadataExecutable 'build_output_sha256') -and
        $executableSha -ceq [string](Get-OptionalProperty $metadataExecutable 'captured_sha256_before_run') -and
        $executableSha -ceq [string](Get-OptionalProperty $metadataExecutable 'captured_sha256_after_run') -and
        [bool](Get-OptionalProperty $executableRecord 'unchanged') -and
        [bool](Get-OptionalProperty $metadataExecutable 'unchanged')
    Add-VerificationCheck -Id 'executable.hash_stable' -Passed $executableValid -Detail "path=$executableRelative sha256=$executableSha"
    if (-not $executableValid) { throw 'Captured executable hash/size/stability record mismatch.' }

    $receiptArtifacts = Get-OptionalProperty $receipt 'artifacts'
    $expectedArtifactPaths = [ordered]@{
        raw_cells = 'artifacts/aggregate_raw_cells.csv'
        raw_chunks = 'artifacts/aggregate_raw_chunks.csv'
        raw_ticks = 'artifacts/aggregate_raw_ticks.csv'
        aggregate = 'artifacts/aggregate.csv'
    }
    $artifactKeys = @($receiptArtifacts.PSObject.Properties.Name | Sort-Object)
    if (($artifactKeys -join "`n") -cne (@($expectedArtifactPaths.Keys | Sort-Object) -join "`n")) {
        throw 'Receipt artifact map does not exactly contain the four v5 CSV artifacts.'
    }
    foreach ($artifactName in $expectedArtifactPaths.Keys) {
        $artifactRecord = Get-OptionalProperty $receiptArtifacts $artifactName
        [void](Assert-CaptureFileRecord -CaptureRoot $captureRoot -Record $artifactRecord -ExpectedRelativePath $expectedArtifactPaths[$artifactName] -Label "receipt artifact $artifactName")
        if ([uint64](Get-OptionalProperty $artifactRecord 'data_row_count') -eq 0 -or
            [uint64](Get-OptionalProperty $artifactRecord 'value_row_count') -eq 0) {
            throw "Receipt artifact row counts are missing or zero: $artifactName"
        }
    }

    $commandRecords = Get-OptionalProperty $receipt 'commands'
    if ($null -eq $commandRecords) { throw 'Receipt lacks commands map.' }
    $commandLabels = @($commandRecords.PSObject.Properties.Name | Sort-Object)
    foreach ($requiredLabel in @('cargo-build', 'benchmark')) {
        if ($requiredLabel -cnotin $commandLabels) { throw "Receipt lacks required command: $requiredLabel" }
    }
    $metadataCommands = @((Get-OptionalProperty $metadata 'commands'))
    $metadataCommandLabels = @($metadataCommands | ForEach-Object { [string](Get-OptionalProperty $_ 'label') } | Sort-Object)
    if (($metadataCommandLabels -join "`n") -cne ($commandLabels -join "`n")) {
        throw 'Metadata and receipt command label sets differ.'
    }
    $verifiedCommands = [Collections.Generic.List[object]]::new()
    foreach ($label in $commandLabels) {
        $record = $commandRecords.PSObject.Properties[$label].Value
        $metadataCommandMatches = @($metadataCommands | Where-Object { [string](Get-OptionalProperty $_ 'label') -ceq $label })
        if ($metadataCommandMatches.Count -ne 1) { throw "Metadata command $label must have exactly one record." }
        $metadataCommand = $metadataCommandMatches[0]
        $commandPath = Resolve-CapturePath $captureRoot ([string](Get-OptionalProperty $record 'command_json'))
        $stdoutPath = Resolve-CapturePath $captureRoot ([string](Get-OptionalProperty $record 'stdout'))
        $stderrPath = Resolve-CapturePath $captureRoot ([string](Get-OptionalProperty $record 'stderr'))
        $exitPath = Resolve-CapturePath $captureRoot ([string](Get-OptionalProperty $record 'exit_code_path'))
        foreach ($path in @($commandPath, $stdoutPath, $stderrPath, $exitPath)) { if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Command record file missing for ${label}: $path" } }
        $command = Get-Content -Raw -LiteralPath $commandPath | ConvertFrom-Json
        if ([string]::IsNullOrWhiteSpace([string](Get-OptionalProperty $command 'executable')) -or $null -eq (Get-OptionalProperty $command 'argv') -or [string]::IsNullOrWhiteSpace([string](Get-OptionalProperty $command 'cwd'))) {
            throw "Command JSON is incomplete: $label"
        }
        $jsonArgv = @((Get-OptionalProperty $command 'argv') | ForEach-Object { [string]$_ })
        $receiptArgv = @((Get-OptionalProperty $record 'argv') | ForEach-Object { [string]$_ })
        $metadataArgv = @((Get-OptionalProperty $metadataCommand 'argv') | ForEach-Object { [string]$_ })
        if ([string](Get-OptionalProperty $record 'label') -cne $label -or
            [string](Get-OptionalProperty $record 'executable') -cne [string](Get-OptionalProperty $command 'executable') -or
            [string](Get-OptionalProperty $metadataCommand 'executable') -cne [string](Get-OptionalProperty $command 'executable') -or
            [string](Get-OptionalProperty $record 'cwd') -cne [string](Get-OptionalProperty $command 'cwd') -or
            [string](Get-OptionalProperty $metadataCommand 'cwd') -cne [string](Get-OptionalProperty $command 'cwd') -or
            ($receiptArgv -join [char]0) -cne ($jsonArgv -join [char]0) -or
            ($metadataArgv -join [char]0) -cne ($jsonArgv -join [char]0) -or
            ((Get-OptionalProperty $record 'environment_overrides') | ConvertTo-Json -Compress) -cne ((Get-OptionalProperty $command 'environment_overrides') | ConvertTo-Json -Compress) -or
            ((Get-OptionalProperty $metadataCommand 'environment_overrides') | ConvertTo-Json -Compress) -cne ((Get-OptionalProperty $command 'environment_overrides') | ConvertTo-Json -Compress)) {
            throw "Command JSON/receipt/metadata exact invocation mismatch: $label"
        }
        foreach ($recordField in @('label', 'command_json', 'stdout', 'stderr', 'exit_code_path', 'exit_code')) {
            if ([string](Get-OptionalProperty $metadataCommand $recordField) -cne [string](Get-OptionalProperty $record $recordField)) {
                throw "Command metadata/receipt record mismatch: $label/$recordField"
            }
        }
        $exitText = [IO.File]::ReadAllText($exitPath).TrimEnd("`r", "`n")
        $exitCode = 0L
        if (-not [long]::TryParse($exitText, [ref]$exitCode) -or $exitCode -ne [long](Get-OptionalProperty $record 'exit_code')) { throw "Command exit code mismatch: $label" }
        if ($exitCode -ne 0) { throw "Official capture command exited nonzero: $label=$exitCode" }
        $verifiedCommands.Add([ordered]@{
            label = $label
            executable = [string](Get-OptionalProperty $command 'executable')
            argv = $jsonArgv
            cwd = [string](Get-OptionalProperty $command 'cwd')
            exit_code = $exitCode
            command_json_sha256 = Get-Sha256Hex $commandPath
            stdout_sha256 = Get-Sha256Hex $stdoutPath
            stderr_sha256 = Get-Sha256Hex $stderrPath
        })
    }
    $allCommandDirectories = @(Get-ChildItem -LiteralPath (Resolve-CapturePath $captureRoot 'commands') -Directory | ForEach-Object { $_.Name } | Sort-Object)
    if (($allCommandDirectories -join "`n") -cne ($commandLabels -join "`n")) { throw 'Receipt command map does not exactly cover command directories.' }
    Add-VerificationCheck -Id 'commands.exact_records_and_exit_codes' -Passed $true -Detail "commands=$($commandLabels.Count); all exit_code=0"
    $derived.commands = $verifiedCommands

    if ((@((Get-OptionalProperty $metadata 'intended_publication_order') | ForEach-Object { [string]$_ }) -join "`n") -cne ($publicationOrder -join "`n") -or
        ((Get-OptionalProperty $metadata 'census_encoding') | ConvertTo-Json -Compress) -cne ((Get-OptionalProperty $receipt 'census_encoding') | ConvertTo-Json -Compress)) {
        throw 'Metadata/receipt publication order or census encoding differs.'
    }
    $metadataCsv = Get-OptionalProperty $metadata 'csv'
    if ([string](Get-OptionalProperty $metadataCsv 'schema_version') -cne $expectedCsvSchema -or
        [string](Get-OptionalProperty $metadataCsv 'run_id') -cne [string]$receipt.run_id -or
        [string](Get-OptionalProperty $metadataCsv 'stdout_run_id') -cne [string]$receipt.run_id) {
        throw 'Metadata CSV schema/run identity does not match the receipt.'
    }
    $metadataStagedRecords = Get-OptionalProperty $metadataCsv 'staged_records'
    foreach ($artifactName in $expectedArtifactPaths.Keys) {
        $receiptArtifact = Get-OptionalProperty $receiptArtifacts $artifactName
        $stagedRecord = Get-OptionalProperty $metadataStagedRecords $artifactName
        foreach ($field in @('path', 'data_row_count', 'value_row_count', 'size_bytes', 'sha256')) {
            if ([string](Get-OptionalProperty $stagedRecord $field) -cne [string](Get-OptionalProperty $receiptArtifact $field)) {
                throw "Metadata/receipt artifact record mismatch: $artifactName/$field"
            }
        }
    }

    $csvResult = Test-CsvAndRecomputation -CaptureRoot $captureRoot -Receipt $receipt
    $independentRowCounts = [ordered]@{
        raw_cells = [uint64]$derived.raw_cell_rows
        raw_chunks = [uint64]$derived.raw_chunk_rows
        raw_ticks = [uint64]$derived.raw_tick_rows
        aggregate = [uint64]$derived.aggregate_rows
    }
    foreach ($artifactName in $independentRowCounts.Keys) {
        $receiptCount = [uint64](Get-OptionalProperty (Get-OptionalProperty $receiptArtifacts $artifactName) 'data_row_count')
        if ($receiptCount -ne $independentRowCounts[$artifactName]) {
            throw "Receipt/independent CSV row count mismatch: $artifactName receipt=$receiptCount independent=$($independentRowCounts[$artifactName])"
        }
    }
    Add-VerificationCheck -Id 'csv.provenance_row_counts_recomputation' -Passed $true -Detail "run_id=$($csvResult.run_id) cells=$($derived.raw_cell_rows) chunks=$($derived.raw_chunk_rows) ticks=$($derived.raw_tick_rows)"

    $benchmarkRecord = $commandRecords.PSObject.Properties['benchmark'].Value
    $benchmarkStdout = [Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes((Resolve-CapturePath $captureRoot ([string]$benchmarkRecord.stdout))))
    $stdoutRunIds = @([regex]::Matches($benchmarkStdout, '(?m)^Run ID:\s+(\S+)\s*$') | ForEach-Object { $_.Groups[1].Value } | Sort-Object -Unique)
    if ($stdoutRunIds.Count -ne 1 -or $stdoutRunIds[0] -cne [string]$receipt.run_id) { throw 'Benchmark stdout run ID does not match receipt/CSV.' }
    Add-VerificationCheck -Id 'run_id.stdout_receipt_csv_equal' -Passed $true -Detail "run_id=$($receipt.run_id)"
}
catch {
    Add-VerificationCheck -Id 'verification.fatal' -Passed $false -Detail $_.Exception.Message
}
finally {
    if (-not [string]::IsNullOrWhiteSpace($temporaryDirectory) -and (Test-Path -LiteralPath $temporaryDirectory -PathType Container)) {
        try {
            Remove-Item -LiteralPath $temporaryDirectory -Recurse -Force
        }
        catch {
            Add-VerificationCheck -Id 'verification.temporary_cleanup' -Passed $false -Detail $_.Exception.Message
        }
    }
}

$finishedUtc = [DateTime]::UtcNow.ToString('o')
$result = [ordered]@{
    verification_schema = 'powdergame-independent-verification-v1'
    expected_csv_schema = $expectedCsvSchema
    started_utc = $startedUtc
    finished_utc = $finishedUtc
    package_path = if ([string]::IsNullOrWhiteSpace($PackagePath)) { '' } else { [IO.Path]::GetFullPath($PackagePath) }
    package_sha256 = $packageSha256
    capture_id = $captureId
    success = -not $hadFailure
    checks = $checks
    derived = $derived
    findings = $findings
}

try {
    Write-JsonCreateNew -Path $OutputPath -Value $result
}
catch {
    [Console]::Error.WriteLine("Independent verifier could not write output: $($_.Exception.Message)")
    exit 2
}

if ($hadFailure) {
    [Console]::Error.WriteLine("Independent verification FAILED; report: $([IO.Path]::GetFullPath($OutputPath))")
    foreach ($finding in $findings) { [Console]::Error.WriteLine("- $finding") }
    exit 1
}

Write-Output "Independent verification complete: capture_id=$captureId package_sha256=$packageSha256 cells=$($derived.raw_cell_rows) chunks=$($derived.raw_chunk_rows)"
Write-Output "Report: $([IO.Path]::GetFullPath($OutputPath))"
exit 0
