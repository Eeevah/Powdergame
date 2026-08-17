[CmdletBinding()]
param(
    [string]$DestinationRoot = '',
    [string[]]$BenchmarkArguments = @(),
    [switch]$Official,
    [string]$CaptureId = '',
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($PSVersionTable.PSVersion.Major -lt 7) {
    throw 'capture-evidence.ps1 requires PowerShell 7 or newer (pwsh.exe).'
}

$script:RepositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
$script:CaptureRoot = ''
$script:RecordedCommands = [Collections.Generic.List[object]]::new()
$script:FailureAfterPublication = ''

function Write-BytesSyncedCreateNew {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][byte[]]$Bytes
    )

    $stream = [IO.File]::Open($Path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
    try {
        $stream.Write($Bytes, 0, $Bytes.Length)
        $stream.Flush($true)
    }
    finally {
        $stream.Dispose()
    }
}

function Write-Utf8NoBomSyncedCreateNew {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][AllowEmptyString()][string]$Text
    )

    Write-BytesSyncedCreateNew -Path $Path -Bytes ([Text.UTF8Encoding]::new($false).GetBytes($Text))
}

function Get-Sha256Hex {
    param([Parameter(Mandatory)][string]$Path)

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Cannot hash missing file: $Path"
    }
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-RelativeForwardPath {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$Path
    )

    return [IO.Path]::GetRelativePath($Root, $Path).Replace([char]92, [char]47)
}

function Test-PathWithin {
    param(
        [Parameter(Mandatory)][string]$Candidate,
        [Parameter(Mandatory)][string]$Parent
    )

    $candidateFull = [IO.Path]::GetFullPath($Candidate).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    $parentFull = [IO.Path]::GetFullPath($Parent).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    if ($candidateFull.Equals($parentFull, [StringComparison]::OrdinalIgnoreCase)) {
        return $true
    }
    return $candidateFull.StartsWith($parentFull + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)
}

function Assert-PathAbsent {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )

    if (Test-Path -LiteralPath $Path) {
        throw "Refusing to overwrite existing $Label`: $Path"
    }
}

function New-ExclusiveDirectory {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )

    Assert-PathAbsent -Path $Path -Label $Label
    try {
        New-Item -Path $Path -ItemType Directory -ErrorAction Stop | Out-Null
    }
    catch {
        throw "Failed to create new $Label without reuse: $Path`n$($_.Exception.Message)"
    }
}

function Initialize-OfficialDestination {
    param([Parameter(Mandatory)][string]$Path)

    if ([string]::IsNullOrWhiteSpace($Path)) {
        throw 'Official capture requires -DestinationRoot naming a new empty directory outside the repository.'
    }

    $full = [IO.Path]::GetFullPath($Path)
    if (Test-PathWithin -Candidate $full -Parent $script:RepositoryRoot) {
        throw "Official destination must be outside the repository: $full"
    }
    if (Test-Path -LiteralPath $full -PathType Leaf) {
        throw "Official destination is a file, not a directory: $full"
    }
    if (-not (Test-Path -LiteralPath $full)) {
        [IO.Directory]::CreateDirectory($full) | Out-Null
    }
    $existing = @(Get-ChildItem -LiteralPath $full -Force)
    if ($existing.Count -ne 0) {
        throw "Official destination must be initially empty and is single-use: $full"
    }
    return $full
}

function Assert-CaptureId {
    param([Parameter(Mandatory)][string]$Value)

    if ($Value -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]{0,95}$' -or $Value -in @('.', '..')) {
        throw "Invalid CaptureId. Use 1-96 ASCII letters, digits, dot, underscore, or hyphen: $Value"
    }
}

function New-ProcessStartInfo {
    param(
        [Parameter(Mandatory)][string]$Executable,
        [Parameter(Mandatory)][string[]]$Arguments,
        [Parameter(Mandatory)][string]$WorkingDirectory,
        [Collections.IDictionary]$EnvironmentOverrides = @{}
    )

    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $Executable
    $startInfo.WorkingDirectory = $WorkingDirectory
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    foreach ($argument in $Arguments) {
        $startInfo.ArgumentList.Add([string]$argument)
    }
    foreach ($key in $EnvironmentOverrides.Keys) {
        $startInfo.Environment[[string]$key] = [string]$EnvironmentOverrides[$key]
    }
    return $startInfo
}

function Invoke-RecordedProcess {
    param(
        [Parameter(Mandatory)][string]$Label,
        [Parameter(Mandatory)][string]$Executable,
        [Parameter(Mandatory)][string[]]$Arguments,
        [Parameter(Mandatory)][string]$WorkingDirectory,
        [Collections.IDictionary]$EnvironmentOverrides = @{},
        [string]$StdoutOverride = ''
    )

    if ([string]::IsNullOrWhiteSpace($script:CaptureRoot)) {
        throw 'Capture root has not been initialized.'
    }
    $commandDirectory = Join-Path $script:CaptureRoot ('commands\' + $Label)
    New-ExclusiveDirectory -Path $commandDirectory -Label "command label '$Label'"
    $commandJsonPath = Join-Path $commandDirectory 'command.json'
    $environmentJson = [ordered]@{}
    foreach ($key in $EnvironmentOverrides.Keys) {
        $environmentJson[[string]$key] = [string]$EnvironmentOverrides[$key]
    }
    $commandJson = [ordered]@{
        executable = $Executable
        argv = @($Arguments)
        cwd = $WorkingDirectory
        environment_overrides = $environmentJson
    } | ConvertTo-Json -Depth 8
    Write-Utf8NoBomSyncedCreateNew -Path $commandJsonPath -Text ($commandJson + "`n")

    $stdoutPath = if ([string]::IsNullOrEmpty($StdoutOverride)) {
        Join-Path $commandDirectory 'stdout.bin'
    }
    else {
        $StdoutOverride
    }
    $stderrPath = Join-Path $commandDirectory 'stderr.bin'
    Assert-PathAbsent -Path $stdoutPath -Label "stdout for '$Label'"
    Assert-PathAbsent -Path $stderrPath -Label "stderr for '$Label'"

    $process = [Diagnostics.Process]::new()
    $process.StartInfo = New-ProcessStartInfo -Executable $Executable -Arguments $Arguments -WorkingDirectory $WorkingDirectory -EnvironmentOverrides $EnvironmentOverrides
    $stdoutStream = [IO.File]::Open($stdoutPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
    $stderrStream = [IO.File]::Open($stderrPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
    $exitCode = $null
    $processFailure = $null
    try {
        if (-not $process.Start()) {
            throw "Failed to start recorded process: $Executable"
        }
        $stdoutTask = $process.StandardOutput.BaseStream.CopyToAsync($stdoutStream)
        $stderrTask = $process.StandardError.BaseStream.CopyToAsync($stderrStream)
        $process.WaitForExit()
        [void]$stdoutTask.GetAwaiter().GetResult()
        [void]$stderrTask.GetAwaiter().GetResult()
        $exitCode = $process.ExitCode
    }
    catch {
        $processFailure = $_
    }
    finally {
        try { $stdoutStream.Flush($true) } catch { if ($null -eq $processFailure) { $processFailure = $_ } }
        try { $stderrStream.Flush($true) } catch { if ($null -eq $processFailure) { $processFailure = $_ } }
        $stdoutStream.Dispose()
        $stderrStream.Dispose()
        $process.Dispose()
    }

    $exitCodePath = Join-Path $commandDirectory 'exit_code.txt'
    $exitText = if ($null -eq $exitCode) { 'PROCESS_NOT_COMPLETED' } else { [string]$exitCode }
    Write-Utf8NoBomSyncedCreateNew -Path $exitCodePath -Text ($exitText + "`n")
    if ($null -ne $processFailure) {
        throw "Recorded process '$Label' did not complete; raw files retained at $commandDirectory`n$($processFailure.Exception.Message)"
    }

    $record = [pscustomobject]@{
        label = $Label
        executable = $Executable
        argv = @($Arguments)
        cwd = $WorkingDirectory
        environment_overrides = $environmentJson
        command_json = $commandJsonPath
        stdout = $stdoutPath
        stderr = $stderrPath
        exit_code_path = $exitCodePath
        exit_code = [int]$exitCode
    }
    $script:RecordedCommands.Add($record)
    return $record
}

function Read-Utf8Text {
    param([Parameter(Mandatory)][string]$Path)

    return [Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes($Path))
}

function Invoke-RecordedTextCommand {
    param(
        [Parameter(Mandatory)][string]$Label,
        [Parameter(Mandatory)][string]$Executable,
        [Parameter(Mandatory)][string[]]$Arguments,
        [Parameter(Mandatory)][string]$WorkingDirectory,
        [Collections.IDictionary]$EnvironmentOverrides = @{}
    )

    $record = Invoke-RecordedProcess -Label $Label -Executable $Executable -Arguments $Arguments -WorkingDirectory $WorkingDirectory -EnvironmentOverrides $EnvironmentOverrides
    $text = (Read-Utf8Text -Path $record.stdout).TrimEnd("`r", "`n")
    return [pscustomobject]@{ record = $record; text = $text }
}

function Assert-CommandSucceeded {
    param([Parameter(Mandatory)]$Command)

    if ($Command.exit_code -ne 0) {
        throw "Command '$($Command.label)' failed with exit $($Command.exit_code); see $($Command.command_json)"
    }
}

function Assert-OfficialCleanStatus {
    param(
        [Parameter(Mandatory)][AllowEmptyString()][string]$StatusText,
        [Parameter(Mandatory)][string]$Phase
    )

    $dirtyRecords = @($StatusText -split "`r?`n" | Where-Object {
            -not [string]::IsNullOrWhiteSpace($_) -and -not $_.StartsWith('# ')
        })
    if ($dirtyRecords.Count -ne 0) {
        throw "Official capture rejects dirty tracked or untracked source at $Phase. This CaptureId is consumed and has no receipt."
    }
}

function Assert-SafeInventoryPaths {
    param([Parameter(Mandatory)][string[]]$Paths)

    if ($Paths.Count -eq 0) {
        throw 'Source input inventory is empty.'
    }
    foreach ($relativePath in $Paths) {
        if ([string]::IsNullOrEmpty($relativePath)) {
            throw 'Source input inventory contains an empty path.'
        }
        if ($relativePath.IndexOfAny([char[]]@("`t", "`r", "`n")) -ge 0) {
            throw "Source input path cannot be represented safely in TSV: $relativePath"
        }
        if ([IO.Path]::IsPathRooted($relativePath) -or $relativePath.Replace([char]92, [char]47).Split('/') -contains '..') {
            throw "Unsafe source input path: $relativePath"
        }
    }
}

function Assert-InventoryShape {
    param(
        [Parameter(Mandatory)][string[]]$Paths,
        [Parameter(Mandatory)][int]$RowCount
    )

    Assert-SafeInventoryPaths -Paths $Paths
    if ($Paths.Count -ne $RowCount) {
        throw "Source inventory row mismatch: paths=$($Paths.Count), rows=$RowCount"
    }
}

function Get-TrackedPaths {
    param(
        [Parameter(Mandatory)][string]$Label,
        [Parameter(Mandatory)][string]$GitExecutable
    )

    $listing = Invoke-RecordedProcess -Label "git-ls-files-$Label" -Executable $GitExecutable -Arguments @('-C', $script:RepositoryRoot, '-c', 'core.quotepath=false', 'ls-files', '-z', '--cached', '--') -WorkingDirectory $script:RepositoryRoot
    Assert-CommandSucceeded -Command $listing
    $bytes = [IO.File]::ReadAllBytes($listing.stdout)
    if ($bytes.Length -eq 0 -or $bytes[$bytes.Length - 1] -ne 0) {
        throw "Tracked source inventory '$Label' is empty or lacks its terminating NUL."
    }
    $text = [Text.Encoding]::UTF8.GetString($bytes)
    $paths = @($text.Split([char]0, [StringSplitOptions]::RemoveEmptyEntries) | Sort-Object -Unique)
    Assert-SafeInventoryPaths -Paths $paths
    return $paths
}

function Write-SourceState {
    param(
        [Parameter(Mandatory)][string]$Label,
        [Parameter(Mandatory)][string]$GitExecutable,
        [string]$SnapshotRoot = ''
    )

    $manifestPath = Join-Path $script:CaptureRoot ("source\SOURCE_INPUTS_{0}.tsv" -f $Label.ToUpperInvariant())
    $rows = [Collections.Generic.List[string]]::new()
    $rows.Add("exists`tsize_bytes`tsha256`trepository_relative_path`tsnapshot_relative_path")
    $paths = @(Get-TrackedPaths -Label $Label -GitExecutable $GitExecutable)
    foreach ($relativePath in $paths) {
        $nativeRelative = $relativePath.Replace([char]47, [IO.Path]::DirectorySeparatorChar)
        $sourcePath = Join-Path $script:RepositoryRoot $nativeRelative
        if (-not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) {
            throw "Tracked source input is not a readable file: $relativePath"
        }
        $sourceInfo = Get-Item -LiteralPath $sourcePath
        $size = $sourceInfo.Length.ToString([Globalization.CultureInfo]::InvariantCulture)
        $sha = Get-Sha256Hex -Path $sourcePath
        $snapshotRelative = 'source/snapshot/' + $relativePath.Replace([char]92, [char]47)
        if (-not [string]::IsNullOrEmpty($SnapshotRoot)) {
            $snapshotPath = Join-Path $SnapshotRoot $nativeRelative
            [IO.Directory]::CreateDirectory((Split-Path -Parent $snapshotPath)) | Out-Null
            [IO.File]::Copy($sourcePath, $snapshotPath, $false)
            if ((Get-Sha256Hex -Path $snapshotPath) -ne $sha) {
                throw "Source changed while snapshotting: $relativePath"
            }
            if ((Get-RelativeForwardPath -Root $script:CaptureRoot -Path $snapshotPath) -ne $snapshotRelative) {
                throw "Unexpected snapshot path for source input: $relativePath"
            }
        }
        $rows.Add(('true`t{0}`t{1}`t{2}`t{3}' -f $size, $sha, $relativePath, $snapshotRelative))
    }
    Assert-InventoryShape -Paths $paths -RowCount ($rows.Count - 1)
    Write-Utf8NoBomSyncedCreateNew -Path $manifestPath -Text (($rows -join "`n") + "`n")
    return [pscustomobject]@{
        label = $Label
        manifest_path = $manifestPath
        manifest_sha256 = Get-Sha256Hex -Path $manifestPath
        path_count = $paths.Count
    }
}

function Write-FullDirtyDiff {
    param(
        [Parameter(Mandatory)][string]$Label,
        [Parameter(Mandatory)][string]$GitExecutable,
        [Parameter(Mandatory)][string]$OutputPath
    )

    $temporaryIndex = Join-Path $script:CaptureRoot ("work\git-index-$Label")
    $indexEnvironment = [ordered]@{ GIT_INDEX_FILE = $temporaryIndex }
    $readTree = Invoke-RecordedProcess -Label "git-read-tree-$Label" -Executable $GitExecutable -Arguments @('-C', $script:RepositoryRoot, 'read-tree', 'HEAD') -WorkingDirectory $script:RepositoryRoot -EnvironmentOverrides $indexEnvironment
    Assert-CommandSucceeded -Command $readTree
    $gitAdd = Invoke-RecordedProcess -Label "git-add-full-$Label" -Executable $GitExecutable -Arguments @('-C', $script:RepositoryRoot, 'add', '-A', '--') -WorkingDirectory $script:RepositoryRoot -EnvironmentOverrides $indexEnvironment
    Assert-CommandSucceeded -Command $gitAdd
    $gitDiff = Invoke-RecordedProcess -Label "git-diff-full-$Label" -Executable $GitExecutable -Arguments @('-C', $script:RepositoryRoot, 'diff', '--cached', '--binary', '--full-index', '--no-ext-diff', '--no-textconv', 'HEAD', '--') -WorkingDirectory $script:RepositoryRoot -EnvironmentOverrides $indexEnvironment -StdoutOverride $OutputPath
    Assert-CommandSucceeded -Command $gitDiff
    return [pscustomobject]@{
        path = $OutputPath
        size_bytes = (Get-Item -LiteralPath $OutputPath).Length
        sha256 = Get-Sha256Hex -Path $OutputPath
    }
}

function Get-CsvIdentityAndCount {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Missing required $Label CSV: $Path"
    }
    $parser = [Microsoft.VisualBasic.FileIO.TextFieldParser]::new($Path, [Text.Encoding]::UTF8)
    $parser.TextFieldType = [Microsoft.VisualBasic.FileIO.FieldType]::Delimited
    $parser.SetDelimiters(',')
    $parser.HasFieldsEnclosedInQuotes = $true
    $schemas = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $runIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $rowCount = 0L
    $valueRowCount = 0L
    try {
        if ($parser.EndOfData) {
            throw "$Label CSV is empty: $Path"
        }
        $header = @($parser.ReadFields())
        $headerLookup = @{}
        for ($index = 0; $index -lt $header.Count; $index++) {
            if ($headerLookup.ContainsKey($header[$index])) {
                throw "$Label CSV has a duplicate header: $($header[$index])"
            }
            $headerLookup[$header[$index]] = $index
        }
        $regularIdentity = $headerLookup.ContainsKey('schema_version') -and $headerLookup.ContainsKey('run_id')
        $metadataIdentity = $headerLookup.ContainsKey('record_type') -and $headerLookup.ContainsKey('name') -and $headerLookup.ContainsKey('text_value')
        if (-not $regularIdentity -and -not $metadataIdentity) {
            throw "$Label CSV lacks schema/run identity columns."
        }

        while (-not $parser.EndOfData) {
            $fields = @($parser.ReadFields())
            if ($fields.Count -ne $header.Count) {
                throw "$Label CSV row $($rowCount + 2) has $($fields.Count) fields; expected $($header.Count)."
            }
            $rowCount++
            if ($regularIdentity) {
                $schemaValue = [string]$fields[$headerLookup['schema_version']]
                $runIdValue = [string]$fields[$headerLookup['run_id']]
                if ([string]::IsNullOrWhiteSpace($schemaValue) -or [string]::IsNullOrWhiteSpace($runIdValue)) {
                    throw "$Label CSV row $($rowCount + 1) has blank schema/run identity."
                }
                $schemas.Add($schemaValue) | Out-Null
                $runIds.Add($runIdValue) | Out-Null
            }
            else {
                $recordType = [string]$fields[$headerLookup['record_type']]
                $name = [string]$fields[$headerLookup['name']]
                $textValue = [string]$fields[$headerLookup['text_value']]
                if ($recordType -eq 'metadata' -and $name -eq 'schema_version') {
                    $schemas.Add($textValue) | Out-Null
                }
                if ($recordType -eq 'metadata' -and $name -eq 'run_id') {
                    $runIds.Add($textValue) | Out-Null
                }
                if ($recordType -eq 'value') {
                    $valueRowCount++
                }
            }
        }
    }
    finally {
        $parser.Dispose()
    }
    if ($rowCount -eq 0 -or $schemas.Count -ne 1 -or $runIds.Count -ne 1) {
        throw "$Label CSV identity is not exactly one nonempty schema and run ID (rows=$rowCount, schemas=$($schemas.Count), run_ids=$($runIds.Count))."
    }
    if ($valueRowCount -eq 0) {
        $valueRowCount = $rowCount
    }
    return [pscustomobject]@{
        path = $Path
        schema_version = @($schemas)[0]
        run_id = @($runIds)[0]
        data_row_count = $rowCount
        value_row_count = $valueRowCount
        size_bytes = (Get-Item -LiteralPath $Path).Length
        sha256 = Get-Sha256Hex -Path $Path
    }
}

function Publish-FileNoOverwrite {
    param(
        [Parameter(Mandatory)][string]$Source,
        [Parameter(Mandatory)][string]$Destination,
        [Parameter(Mandatory)][string]$Label,
        [AllowEmptyCollection()][Collections.Generic.List[string]]$PublicationJournal
    )

    if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) {
        throw "Cannot publish missing $Label`: $Source"
    }
    Assert-PathAbsent -Path $Destination -Label $Label
    [IO.File]::Move($Source, $Destination, $false)
    if ($null -ne $PublicationJournal) {
        $PublicationJournal.Add($Destination)
    }
}

function Publish-FinalArtifactSet {
    param(
        [Parameter(Mandatory)][Collections.IDictionary]$StagedPaths,
        [Parameter(Mandatory)][Collections.IDictionary]$FinalPaths,
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[string]]$PublicationJournal
    )

    foreach ($name in @('raw_cells', 'raw_chunks', 'raw_ticks', 'aggregate', 'metadata')) {
        Publish-FileNoOverwrite -Source $StagedPaths[$name] -Destination $FinalPaths[$name] -Label $name -PublicationJournal $PublicationJournal
        if ($script:FailureAfterPublication -eq $name) {
            throw "Injected self-test failure after publishing $name"
        }
    }
}

function Get-FileRecord {
    param(
        [Parameter(Mandatory)][string]$CaptureRoot,
        [Parameter(Mandatory)][string]$Path
    )

    return [ordered]@{
        path = Get-RelativeForwardPath -Root $CaptureRoot -Path $Path
        size_bytes = (Get-Item -LiteralPath $Path).Length
        sha256 = Get-Sha256Hex -Path $Path
    }
}

function New-HashManifest {
    param(
        [Parameter(Mandatory)][string]$DestinationRoot,
        [Parameter(Mandatory)][string]$CaptureRoot,
        [Parameter(Mandatory)][string]$ManifestPath,
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[string]]$PublicationJournal
    )

    Assert-PathAbsent -Path $ManifestPath -Label 'capture hash manifest'
    $receiptPath = Join-Path $CaptureRoot 'CAPTURE_RECEIPT.json'
    $files = @(Get-ChildItem -LiteralPath $CaptureRoot -Recurse -Force -File | Where-Object {
            $_.FullName -ne $ManifestPath -and $_.FullName -ne $receiptPath
        } | Sort-Object { Get-RelativeForwardPath -Root $CaptureRoot -Path $_.FullName })
    $records = [Collections.Generic.List[object]]::new()
    $lines = [Collections.Generic.List[string]]::new()
    foreach ($file in $files) {
        $record = Get-FileRecord -CaptureRoot $CaptureRoot -Path $file.FullName
        $records.Add($record)
        $lines.Add(('{0}  {1}' -f $record.sha256, $record.path))
    }
    $stagingPath = Join-Path $DestinationRoot ('.hash-manifest-{0}.tmp' -f [Guid]::NewGuid().ToString('N'))
    try {
        Write-Utf8NoBomSyncedCreateNew -Path $stagingPath -Text (($lines -join "`n") + "`n")
        Publish-FileNoOverwrite -Source $stagingPath -Destination $ManifestPath -Label 'capture hash manifest' -PublicationJournal $PublicationJournal
    }
    finally {
        if (Test-Path -LiteralPath $stagingPath -PathType Leaf) {
            [IO.File]::Delete($stagingPath)
        }
    }
    return [pscustomobject]@{
        path = $ManifestPath
        sha256 = Get-Sha256Hex -Path $ManifestPath
        file_count = $records.Count
        records = @($records)
    }
}

function Publish-CaptureReceipt {
    param(
        [Parameter(Mandatory)][string]$DestinationRoot,
        [Parameter(Mandatory)][string]$CaptureRoot,
        [Parameter(Mandatory)][Collections.IDictionary]$Receipt,
        [Parameter(Mandatory)][string[]]$RequiredPreReceiptPaths,
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[string]]$PublicationJournal
    )

    $receiptPath = Join-Path $CaptureRoot 'CAPTURE_RECEIPT.json'
    Assert-PathAbsent -Path $receiptPath -Label 'capture receipt'
    foreach ($requiredPath in $RequiredPreReceiptPaths) {
        if (-not (Test-Path -LiteralPath $requiredPath -PathType Leaf)) {
            throw "Receipt suppressed because a required pre-receipt file is missing: $requiredPath"
        }
    }
    $workPath = Join-Path $CaptureRoot 'work'
    if (Test-Path -LiteralPath $workPath) {
        throw "Receipt suppressed because transient work remains: $workPath"
    }
    if (-not $Receipt.official_mode -or -not $Receipt.complete) {
        throw 'Receipt suppressed because official_mode and complete must both be true.'
    }

    $stagingPath = Join-Path $DestinationRoot ('.capture-receipt-{0}.tmp' -f [Guid]::NewGuid().ToString('N'))
    try {
        Write-Utf8NoBomSyncedCreateNew -Path $stagingPath -Text (($Receipt | ConvertTo-Json -Depth 24) + "`n")
        Publish-FileNoOverwrite -Source $stagingPath -Destination $receiptPath -Label 'capture receipt' -PublicationJournal $PublicationJournal
    }
    finally {
        if (Test-Path -LiteralPath $stagingPath -PathType Leaf) {
            [IO.File]::Delete($stagingPath)
        }
    }
    return $receiptPath
}

function Remove-SuccessWorkTree {
    param([Parameter(Mandatory)][string]$CaptureRoot)

    $workPath = [IO.Path]::GetFullPath((Join-Path $CaptureRoot 'work'))
    if (-not (Test-PathWithin -Candidate $workPath -Parent $CaptureRoot) -or $workPath.Equals([IO.Path]::GetFullPath($CaptureRoot), [StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing unsafe success-work cleanup: $workPath"
    }
    if (Test-Path -LiteralPath $workPath) {
        [IO.Directory]::Delete($workPath, $true)
    }
}

function New-Package {
    param(
        [Parameter(Mandatory)][string]$DestinationRoot,
        [Parameter(Mandatory)][string]$CaptureRoot,
        [Parameter(Mandatory)][string]$CaptureId
    )

    $zipPath = Join-Path $DestinationRoot ($CaptureId + '.zip')
    $packageHashPath = Join-Path $DestinationRoot 'PACKAGE_SHA256.txt'
    Assert-PathAbsent -Path $zipPath -Label 'capture ZIP'
    Assert-PathAbsent -Path $packageHashPath -Label 'ZIP-external package hash'
    $stagingZip = Join-Path $DestinationRoot ('.package-{0}.zip.tmp' -f [Guid]::NewGuid().ToString('N'))
    try {
        [IO.Compression.ZipFile]::CreateFromDirectory($CaptureRoot, $stagingZip, [IO.Compression.CompressionLevel]::Optimal, $true)
        Publish-FileNoOverwrite -Source $stagingZip -Destination $zipPath -Label 'capture ZIP'
    }
    finally {
        if (Test-Path -LiteralPath $stagingZip -PathType Leaf) {
            [IO.File]::Delete($stagingZip)
        }
    }

    $zipSha = Get-Sha256Hex -Path $zipPath
    $stagingHash = Join-Path $DestinationRoot ('.package-sha256-{0}.tmp' -f [Guid]::NewGuid().ToString('N'))
    try {
        Write-Utf8NoBomSyncedCreateNew -Path $stagingHash -Text ("$zipSha  $([IO.Path]::GetFileName($zipPath))`n")
        Publish-FileNoOverwrite -Source $stagingHash -Destination $packageHashPath -Label 'ZIP-external package hash'
    }
    finally {
        if (Test-Path -LiteralPath $stagingHash -PathType Leaf) {
            [IO.File]::Delete($stagingHash)
        }
    }
    return [pscustomobject]@{
        zip_path = $zipPath
        zip_sha256 = $zipSha
        package_sha256_path = $packageHashPath
    }
}

function Convert-CommandRecordForMetadata {
    param(
        [Parameter(Mandatory)]$Command,
        [Parameter(Mandatory)][string]$CaptureRoot
    )

    return [ordered]@{
        label = $Command.label
        executable = $Command.executable
        argv = @($Command.argv)
        cwd = $Command.cwd
        environment_overrides = $Command.environment_overrides
        command_json = Get-RelativeForwardPath -Root $CaptureRoot -Path $Command.command_json
        stdout = Get-RelativeForwardPath -Root $CaptureRoot -Path $Command.stdout
        stderr = Get-RelativeForwardPath -Root $CaptureRoot -Path $Command.stderr
        exit_code_path = Get-RelativeForwardPath -Root $CaptureRoot -Path $Command.exit_code_path
        exit_code = $Command.exit_code
    }
}

function Invoke-CaptureSelfTest {
    $tempParent = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    $testRoot = Join-Path $tempParent ('powdergame-capture-selftest-' + [Guid]::NewGuid().ToString('N'))
    [IO.Directory]::CreateDirectory($testRoot) | Out-Null
    $passed = [Collections.Generic.List[string]]::new()

    function Assert-Throws {
        param(
            [Parameter(Mandatory)][scriptblock]$Action,
            [Parameter(Mandatory)][string]$Name,
            [string]$ExpectedMessage = ''
        )
        $threw = $false
        $caughtMessage = ''
        try { & $Action } catch { $threw = $true; $caughtMessage = $_.Exception.Message }
        if (-not $threw) { throw "Self-test expected nonzero failure: $Name" }
        if (-not [string]::IsNullOrEmpty($ExpectedMessage) -and $caughtMessage -notlike "*$ExpectedMessage*") {
            throw "Self-test '$Name' caught the wrong failure: $caughtMessage"
        }
        $passed.Add($Name)
    }

    try {
        $dirtyCase = Join-Path $testRoot 'dirty-rejection'
        [IO.Directory]::CreateDirectory($dirtyCase) | Out-Null
        Assert-Throws -Name 'dirty official source suppresses receipt' -Action {
            Assert-OfficialCleanStatus -StatusText "# branch.head test`n? untracked.txt" -Phase 'self-test'
        }
        if (Test-Path -LiteralPath (Join-Path $dirtyCase 'CAPTURE_RECEIPT.json')) {
            throw 'Self-test dirty rejection unexpectedly created a receipt.'
        }

        $duplicate = Join-Path $testRoot 'duplicate-id'
        New-ExclusiveDirectory -Path $duplicate -Label 'self-test capture ID'
        Assert-Throws -Name 'duplicate CaptureId is rejected' -Action {
            New-ExclusiveDirectory -Path $duplicate -Label 'self-test capture ID'
        }

        $overwriteCase = Join-Path $testRoot 'no-overwrite'
        [IO.Directory]::CreateDirectory($overwriteCase) | Out-Null
        $overwriteSource = Join-Path $overwriteCase 'source.tmp'
        $overwriteFinal = Join-Path $overwriteCase 'final.txt'
        Write-Utf8NoBomSyncedCreateNew -Path $overwriteSource -Text 'new'
        Write-Utf8NoBomSyncedCreateNew -Path $overwriteFinal -Text 'old'
        Assert-Throws -Name 'pre-existing final file is not overwritten' -Action {
            Publish-FileNoOverwrite -Source $overwriteSource -Destination $overwriteFinal -Label 'self-test final'
        }
        if ((Read-Utf8Text -Path $overwriteFinal) -ne 'old') {
            throw 'Self-test overwrite guard changed the existing final.'
        }

        $failureCase = Join-Path $testRoot 'failure-injection'
        [IO.Directory]::CreateDirectory((Join-Path $failureCase 'stage')) | Out-Null
        [IO.Directory]::CreateDirectory((Join-Path $failureCase 'artifacts')) | Out-Null
        [IO.Directory]::CreateDirectory((Join-Path $failureCase 'metadata')) | Out-Null
        $failureStaged = [ordered]@{}
        $failureFinal = [ordered]@{}
        foreach ($name in @('raw_cells', 'raw_chunks', 'raw_ticks', 'aggregate', 'metadata')) {
            $failureStaged[$name] = Join-Path $failureCase ("stage\$name.tmp")
            $failureFinal[$name] = if ($name -eq 'metadata') { Join-Path $failureCase 'metadata\CAPTURE_METADATA.json' } else { Join-Path $failureCase ("artifacts\$name.csv") }
            Write-Utf8NoBomSyncedCreateNew -Path $failureStaged[$name] -Text $name
        }
        $failureJournal = [Collections.Generic.List[string]]::new()
        $script:FailureAfterPublication = 'raw_cells'
        Assert-Throws -Name 'failure injection preserves partial capture without receipt' -ExpectedMessage 'Injected self-test failure' -Action {
            Publish-FinalArtifactSet -StagedPaths $failureStaged -FinalPaths $failureFinal -PublicationJournal $failureJournal
        }
        $script:FailureAfterPublication = ''
        if (-not (Test-Path -LiteralPath $failureFinal['raw_cells'] -PathType Leaf) -or
            (Test-Path -LiteralPath $failureFinal['raw_chunks']) -or
            (Test-Path -LiteralPath (Join-Path $failureCase 'CAPTURE_RECEIPT.json'))) {
            throw 'Self-test failure injection publication boundary is incorrect.'
        }

        $completeCase = Join-Path $testRoot 'receipt-last'
        foreach ($directory in @('stage', 'artifacts', 'metadata', 'hashes')) {
            [IO.Directory]::CreateDirectory((Join-Path $completeCase $directory)) | Out-Null
        }
        $completeStaged = [ordered]@{}
        $completeFinal = [ordered]@{}
        foreach ($name in @('raw_cells', 'raw_chunks', 'raw_ticks', 'aggregate', 'metadata')) {
            $completeStaged[$name] = Join-Path $completeCase ("stage\$name.tmp")
            $completeFinal[$name] = if ($name -eq 'metadata') { Join-Path $completeCase 'metadata\CAPTURE_METADATA.json' } else { Join-Path $completeCase ("artifacts\$name.csv") }
            Write-Utf8NoBomSyncedCreateNew -Path $completeStaged[$name] -Text $name
        }
        $completeJournal = [Collections.Generic.List[string]]::new()
        Publish-FinalArtifactSet -StagedPaths $completeStaged -FinalPaths $completeFinal -PublicationJournal $completeJournal
        [IO.Directory]::Delete((Join-Path $completeCase 'stage'), $true)
        $hashManifest = Join-Path $completeCase 'hashes\SHA256SUMS.txt'
        Write-Utf8NoBomSyncedCreateNew -Path $hashManifest -Text "self-test`n"
        $completeJournal.Add($hashManifest)
        $required = @($completeFinal.Values) + @($hashManifest)
        $receipt = [ordered]@{ official_mode = $true; complete = $true; capture_id = 'self-test' }
        $missingRequired = Join-Path $completeCase 'metadata\MISSING.json'
        Assert-Throws -Name 'incomplete pre-receipt set suppresses receipt' -ExpectedMessage 'required pre-receipt file is missing' -Action {
            Publish-CaptureReceipt -DestinationRoot $testRoot -CaptureRoot $completeCase -Receipt $receipt -RequiredPreReceiptPaths (@($required) + @($missingRequired)) -PublicationJournal $completeJournal
        }
        if (Test-Path -LiteralPath (Join-Path $completeCase 'CAPTURE_RECEIPT.json')) {
            throw 'Self-test incomplete pre-receipt set unexpectedly created a receipt.'
        }
        $receiptPath = Publish-CaptureReceipt -DestinationRoot $testRoot -CaptureRoot $completeCase -Receipt $receipt -RequiredPreReceiptPaths $required -PublicationJournal $completeJournal
        if ($completeJournal[$completeJournal.Count - 1] -ne $receiptPath -or -not (Test-Path -LiteralPath $receiptPath -PathType Leaf)) {
            throw 'Self-test receipt was not the final publication marker.'
        }
        $passed.Add('receipt is last and requires complete pre-receipt set')

        Assert-Throws -Name 'inventory path/count mismatch is rejected' -Action {
            Assert-InventoryShape -Paths @('a', 'b') -RowCount 1
        }
        Assert-Throws -Name 'empty inventory path is rejected' -Action {
            Assert-InventoryShape -Paths @('a', '') -RowCount 2
        }

        $csvCase = Join-Path $testRoot 'csv-identity'
        [IO.Directory]::CreateDirectory($csvCase) | Out-Null
        $validCsv = Join-Path $csvCase 'raw_cells.csv'
        Write-Utf8NoBomSyncedCreateNew -Path $validCsv -Text "schema_version,run_id,index,activity_mask`npowdergame-g8a-v5,run-selftest,0,1`npowdergame-g8a-v5,run-selftest,1,0`n"
        $validIdentity = Get-CsvIdentityAndCount -Path $validCsv -Label 'self-test raw cells'
        if ($validIdentity.schema_version -ne 'powdergame-g8a-v5' -or $validIdentity.run_id -ne 'run-selftest' -or
            $validIdentity.data_row_count -ne 2 -or $validIdentity.value_row_count -ne 2) {
            throw 'Self-test rectangular raw CSV identity/row count was not preserved.'
        }
        $passed.Add('rectangular raw CSV identity and row count are preserved')
        $mismatchedCsv = Join-Path $csvCase 'mismatched.csv'
        Write-Utf8NoBomSyncedCreateNew -Path $mismatchedCsv -Text "schema_version,run_id,index`npowdergame-g8a-v5,run-a,0`npowdergame-g8a-v5,run-b,1`n"
        Assert-Throws -Name 'multi-valued CSV run identity is rejected' -ExpectedMessage 'identity is not exactly one' -Action {
            Get-CsvIdentityAndCount -Path $mismatchedCsv -Label 'self-test mismatched'
        }

        $recordedCommandCase = Join-Path $testRoot 'recorded-text-command'
        [IO.Directory]::CreateDirectory((Join-Path $recordedCommandCase 'commands')) | Out-Null
        $savedCaptureRoot = $script:CaptureRoot
        $savedRecordedCommands = $script:RecordedCommands
        try {
            $script:CaptureRoot = $recordedCommandCase
            $script:RecordedCommands = [Collections.Generic.List[object]]::new()
            $pwshExecutable = (Get-Command pwsh.exe -ErrorAction Stop).Source
            $textResults = @(Invoke-RecordedTextCommand -Label 'real-text-probe' -Executable $pwshExecutable -Arguments @('-NoProfile', '-Command', '[Console]::Out.Write("recorded-text-probe")') -WorkingDirectory $script:RepositoryRoot)
            if ($textResults.Count -ne 1 -or $textResults[0].text -ne 'recorded-text-probe' -or
                $textResults[0].record.stdout -ne (Join-Path $recordedCommandCase 'commands\real-text-probe\stdout.bin') -or
                $script:RecordedCommands.Count -ne 1) {
                throw 'Real recorded text command returned a contaminated or incomplete output shape.'
            }
            $passed.Add('real recorded text command returns one uncontaminated result')
        }
        finally {
            $script:CaptureRoot = $savedCaptureRoot
            $script:RecordedCommands = $savedRecordedCommands
        }

        foreach ($name in $passed) {
            Write-Output "SELFTEST OK: $name"
        }
        Write-Output "SELFTEST OK: $($passed.Count) capture invariants exercised without build or GPU execution"
    }
    finally {
        $script:FailureAfterPublication = ''
        $testRootFull = [IO.Path]::GetFullPath($testRoot)
        if (-not (Test-PathWithin -Candidate $testRootFull -Parent $tempParent) -or $testRootFull.Equals($tempParent, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing unsafe self-test cleanup: $testRootFull"
        }
        if (Test-Path -LiteralPath $testRootFull) {
            [IO.Directory]::Delete($testRootFull, $true)
        }
    }
}

if ($SelfTest) {
    if ($Official -or -not [string]::IsNullOrWhiteSpace($DestinationRoot) -or -not [string]::IsNullOrWhiteSpace($CaptureId) -or $BenchmarkArguments.Count -ne 0) {
        throw '-SelfTest cannot be combined with official capture parameters.'
    }
    Invoke-CaptureSelfTest
    exit 0
}

if (-not $Official) {
    throw 'This runner only creates canonical candidates. Invoke it with -Official and a new empty -DestinationRoot.'
}
if ([string]::IsNullOrWhiteSpace($CaptureId)) {
    $CaptureId = 'g8a-v5-{0}-{1}' -f ([DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffZ')), ([Guid]::NewGuid().ToString('N').Substring(0, 8))
}
Assert-CaptureId -Value $CaptureId
$destinationRootFull = Initialize-OfficialDestination -Path $DestinationRoot
$captureRoot = Join-Path $destinationRootFull $CaptureId
$zipPath = Join-Path $destinationRootFull ($CaptureId + '.zip')
$packageHashPath = Join-Path $destinationRootFull 'PACKAGE_SHA256.txt'
Assert-PathAbsent -Path $captureRoot -Label 'CaptureId directory'
Assert-PathAbsent -Path $zipPath -Label 'CaptureId ZIP'
Assert-PathAbsent -Path $packageHashPath -Label 'ZIP-external package hash'
New-ExclusiveDirectory -Path $captureRoot -Label 'CaptureId directory'
$script:CaptureRoot = $captureRoot

try {
    foreach ($directory in @(
            'artifacts',
            'commands',
            'diff',
            'executable',
            'hashes',
            'metadata',
            'source',
            'source\snapshot',
            'work',
            'work\benchmark-output',
            'work\cargo-target',
            'work\staging'
        )) {
        [IO.Directory]::CreateDirectory((Join-Path $captureRoot $directory)) | Out-Null
    }

    $startedUtc = [DateTime]::UtcNow.ToString('o')
    $gitExecutable = (Get-Command git.exe -ErrorAction Stop).Source
    $cargoExecutable = (Get-Command cargo.exe -ErrorAction Stop).Source
    $rustcExecutable = (Get-Command rustc.exe -ErrorAction Stop).Source

    $repoTop = Invoke-RecordedTextCommand -Label 'git-repository-root' -Executable $gitExecutable -Arguments @('-C', $script:RepositoryRoot, 'rev-parse', '--show-toplevel') -WorkingDirectory $script:RepositoryRoot
    Assert-CommandSucceeded -Command $repoTop.record
    if (-not ([IO.Path]::GetFullPath($repoTop.text)).Equals($script:RepositoryRoot, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Script repository root mismatch: expected $script:RepositoryRoot, git reported $($repoTop.text)"
    }
    $headBefore = Invoke-RecordedTextCommand -Label 'git-head-before' -Executable $gitExecutable -Arguments @('-C', $script:RepositoryRoot, 'rev-parse', 'HEAD') -WorkingDirectory $script:RepositoryRoot
    Assert-CommandSucceeded -Command $headBefore.record
    $branchBefore = Invoke-RecordedTextCommand -Label 'git-branch-before' -Executable $gitExecutable -Arguments @('-C', $script:RepositoryRoot, 'branch', '--show-current') -WorkingDirectory $script:RepositoryRoot
    Assert-CommandSucceeded -Command $branchBefore.record
    if ([string]::IsNullOrWhiteSpace($branchBefore.text)) {
        throw 'Official capture requires a named branch; detached HEAD is rejected.'
    }
    $upstreamRef = Invoke-RecordedTextCommand -Label 'git-upstream-ref' -Executable $gitExecutable -Arguments @('-C', $script:RepositoryRoot, 'rev-parse', '--abbrev-ref', '--symbolic-full-name', '@{upstream}') -WorkingDirectory $script:RepositoryRoot
    Assert-CommandSucceeded -Command $upstreamRef.record
    $upstreamSha = Invoke-RecordedTextCommand -Label 'git-upstream-sha' -Executable $gitExecutable -Arguments @('-C', $script:RepositoryRoot, 'rev-parse', '@{upstream}') -WorkingDirectory $script:RepositoryRoot
    Assert-CommandSucceeded -Command $upstreamSha.record
    if ($headBefore.text -ne $upstreamSha.text) {
        throw "Official capture requires HEAD to equal its pushed upstream: HEAD=$($headBefore.text), upstream=$($upstreamSha.text)"
    }
    $originUrl = Invoke-RecordedTextCommand -Label 'git-origin-url' -Executable $gitExecutable -Arguments @('-C', $script:RepositoryRoot, 'remote', 'get-url', 'origin') -WorkingDirectory $script:RepositoryRoot
    Assert-CommandSucceeded -Command $originUrl.record

    $statusBeforePath = Join-Path $captureRoot 'source\GIT_STATUS_BEFORE.txt'
    $statusBefore = Invoke-RecordedProcess -Label 'git-status-before' -Executable $gitExecutable -Arguments @('-C', $script:RepositoryRoot, 'status', '--porcelain=v2', '--branch', '--untracked-files=all') -WorkingDirectory $script:RepositoryRoot -StdoutOverride $statusBeforePath
    Assert-CommandSucceeded -Command $statusBefore
    $dirtyBefore = Write-FullDirtyDiff -Label 'before' -GitExecutable $gitExecutable -OutputPath (Join-Path $captureRoot 'diff\full_dirty.diff')
    Assert-OfficialCleanStatus -StatusText (Read-Utf8Text -Path $statusBeforePath) -Phase 'capture start'
    if ($dirtyBefore.size_bytes -ne 0) {
        throw "Official capture rejects a nonempty full binary diff at capture start: $($dirtyBefore.path)"
    }

    $toolchainGit = Invoke-RecordedTextCommand -Label 'toolchain-git-version' -Executable $gitExecutable -Arguments @('--version') -WorkingDirectory $script:RepositoryRoot
    Assert-CommandSucceeded -Command $toolchainGit.record
    $toolchainCargo = Invoke-RecordedTextCommand -Label 'toolchain-cargo-version' -Executable $cargoExecutable -Arguments @('--version', '--verbose') -WorkingDirectory $script:RepositoryRoot
    Assert-CommandSucceeded -Command $toolchainCargo.record
    $toolchainRustc = Invoke-RecordedTextCommand -Label 'toolchain-rustc-version' -Executable $rustcExecutable -Arguments @('-Vv') -WorkingDirectory $script:RepositoryRoot
    Assert-CommandSucceeded -Command $toolchainRustc.record

    $sourceBeforeBuild = Write-SourceState -Label 'before_build' -GitExecutable $gitExecutable -SnapshotRoot (Join-Path $captureRoot 'source\snapshot')
    $isolatedTarget = Join-Path $captureRoot 'work\cargo-target'
    $build = Invoke-RecordedProcess -Label 'cargo-build' -Executable $cargoExecutable -Arguments @('build', '--locked', '--release', '-p', 'powdergame-benchmark', '--target-dir', $isolatedTarget) -WorkingDirectory $script:RepositoryRoot
    Assert-CommandSucceeded -Command $build
    $sourceAfterBuild = Write-SourceState -Label 'after_build' -GitExecutable $gitExecutable

    $builtExecutable = Join-Path $isolatedTarget 'release\powdergame-benchmark.exe'
    if (-not (Test-Path -LiteralPath $builtExecutable -PathType Leaf)) {
        throw "Built executable not found: $builtExecutable"
    }
    $capturedExecutable = Join-Path $captureRoot 'executable\powdergame-benchmark.exe'
    [IO.File]::Copy($builtExecutable, $capturedExecutable, $false)
    $builtExecutableSha = Get-Sha256Hex -Path $builtExecutable
    $executableShaBefore = Get-Sha256Hex -Path $capturedExecutable
    if ($builtExecutableSha -ne $executableShaBefore) {
        throw 'Copied executable SHA-256 does not match the isolated build output.'
    }
    $sourceBeforeRun = Write-SourceState -Label 'before_run' -GitExecutable $gitExecutable

    if (@($BenchmarkArguments | Where-Object { $_ -ieq '--csv' -or $_ -ilike '--csv=*' }).Count -ne 0) {
        throw 'Do not pass --csv in BenchmarkArguments; the official runner owns all artifact paths.'
    }
    $stagedAggregate = Join-Path $captureRoot 'work\benchmark-output\aggregate.csv'
    $stagedRawTicks = Join-Path $captureRoot 'work\benchmark-output\aggregate_raw_ticks.csv'
    $stagedRawCells = Join-Path $captureRoot 'work\benchmark-output\aggregate_raw_cells.csv'
    $stagedRawChunks = Join-Path $captureRoot 'work\benchmark-output\aggregate_raw_chunks.csv'
    foreach ($path in @($stagedAggregate, $stagedRawTicks, $stagedRawCells, $stagedRawChunks)) {
        Assert-PathAbsent -Path $path -Label 'staged benchmark artifact'
    }
    $effectiveBenchmarkArguments = @($BenchmarkArguments) + @('--csv', $stagedAggregate)
    $benchmark = Invoke-RecordedProcess -Label 'benchmark' -Executable $capturedExecutable -Arguments $effectiveBenchmarkArguments -WorkingDirectory $script:RepositoryRoot

    $sourceAfterRun = Write-SourceState -Label 'after_run' -GitExecutable $gitExecutable
    $headAfter = Invoke-RecordedTextCommand -Label 'git-head-after' -Executable $gitExecutable -Arguments @('-C', $script:RepositoryRoot, 'rev-parse', 'HEAD') -WorkingDirectory $script:RepositoryRoot
    Assert-CommandSucceeded -Command $headAfter.record
    $statusAfterPath = Join-Path $captureRoot 'source\GIT_STATUS_AFTER.txt'
    $statusAfter = Invoke-RecordedProcess -Label 'git-status-after' -Executable $gitExecutable -Arguments @('-C', $script:RepositoryRoot, 'status', '--porcelain=v2', '--branch', '--untracked-files=all') -WorkingDirectory $script:RepositoryRoot -StdoutOverride $statusAfterPath
    Assert-CommandSucceeded -Command $statusAfter
    $dirtyAfter = Write-FullDirtyDiff -Label 'after' -GitExecutable $gitExecutable -OutputPath (Join-Path $captureRoot 'diff\full_dirty_after.diff')
    $executableShaAfter = Get-Sha256Hex -Path $capturedExecutable

    if ($benchmark.exit_code -ne 0) {
        throw "Benchmark failed with exit $($benchmark.exit_code); failed capture retained without receipt at $captureRoot"
    }
    Assert-OfficialCleanStatus -StatusText (Read-Utf8Text -Path $statusAfterPath) -Phase 'capture end'
    if ($dirtyAfter.size_bytes -ne 0) {
        throw "Official capture rejects a nonempty full binary diff at capture end: $($dirtyAfter.path)"
    }
    if ($headBefore.text -ne $headAfter.text) {
        throw "HEAD changed during official capture: before=$($headBefore.text), after=$($headAfter.text)"
    }
    $sourceStates = @($sourceBeforeBuild, $sourceAfterBuild, $sourceBeforeRun, $sourceAfterRun)
    if (@($sourceStates.manifest_sha256 | Sort-Object -Unique).Count -ne 1 -or @($sourceStates.path_count | Sort-Object -Unique).Count -ne 1) {
        throw 'Source inventory path/count/hash changed during official capture.'
    }
    if ($executableShaBefore -ne $executableShaAfter) {
        throw 'Captured executable changed during benchmark execution.'
    }

    $aggregateIdentity = Get-CsvIdentityAndCount -Path $stagedAggregate -Label 'aggregate'
    $rawTicksIdentity = Get-CsvIdentityAndCount -Path $stagedRawTicks -Label 'raw ticks'
    $rawCellsIdentity = Get-CsvIdentityAndCount -Path $stagedRawCells -Label 'raw cells'
    $rawChunksIdentity = Get-CsvIdentityAndCount -Path $stagedRawChunks -Label 'raw chunks'
    $csvIdentities = @($aggregateIdentity, $rawTicksIdentity, $rawCellsIdentity, $rawChunksIdentity)
    if (@($csvIdentities.schema_version | Sort-Object -Unique).Count -ne 1 -or $aggregateIdentity.schema_version -ne 'powdergame-g8a-v5') {
        throw "CSV schema identity is not uniformly powdergame-g8a-v5: $($csvIdentities.schema_version -join ', ')"
    }
    if (@($csvIdentities.run_id | Sort-Object -Unique).Count -ne 1) {
        throw "CSV run IDs do not match: $($csvIdentities.run_id -join ', ')"
    }
    $stdoutText = Read-Utf8Text -Path $benchmark.stdout
    $stdoutRunId = ''
    if ($stdoutText -match '(?m)^Run ID:\s+(\S+)\s*$') {
        $stdoutRunId = $Matches[1]
    }
    if ([string]::IsNullOrWhiteSpace($stdoutRunId) -or $stdoutRunId -ne $aggregateIdentity.run_id) {
        throw "Benchmark stdout Run ID does not match CSV Run ID: stdout='$stdoutRunId', csv='$($aggregateIdentity.run_id)'"
    }

    $finalPaths = [ordered]@{
        raw_cells = Join-Path $captureRoot 'artifacts\aggregate_raw_cells.csv'
        raw_chunks = Join-Path $captureRoot 'artifacts\aggregate_raw_chunks.csv'
        raw_ticks = Join-Path $captureRoot 'artifacts\aggregate_raw_ticks.csv'
        aggregate = Join-Path $captureRoot 'artifacts\aggregate.csv'
        metadata = Join-Path $captureRoot 'metadata\CAPTURE_METADATA.json'
    }
    foreach ($path in $finalPaths.Values) {
        Assert-PathAbsent -Path $path -Label 'final capture artifact'
    }
    $stagedMetadata = Join-Path $captureRoot 'work\staging\CAPTURE_METADATA.json'
    $commandMetadata = @($script:RecordedCommands | ForEach-Object { Convert-CommandRecordForMetadata -Command $_ -CaptureRoot $captureRoot })
    $metadata = [ordered]@{
        metadata_schema = 'powdergame-g8a-capture-metadata-v2'
        official_mode = $true
        capture_id = $CaptureId
        run_id = $aggregateIdentity.run_id
        started_utc = $startedUtc
        metadata_created_utc = [DateTime]::UtcNow.ToString('o')
        repository = [ordered]@{
            root = $script:RepositoryRoot
            origin_url = $originUrl.text
            branch = $branchBefore.text
            source_sha = $headBefore.text
            upstream_ref = $upstreamRef.text
            upstream_sha = $upstreamSha.text
            git_state = 'clean'
            clean_before = $true
            clean_after = $true
        }
        source = [ordered]@{
            input_count = $sourceBeforeBuild.path_count
            manifests = [ordered]@{
                before_build = Get-FileRecord -CaptureRoot $captureRoot -Path $sourceBeforeBuild.manifest_path
                after_build = Get-FileRecord -CaptureRoot $captureRoot -Path $sourceAfterBuild.manifest_path
                before_run = Get-FileRecord -CaptureRoot $captureRoot -Path $sourceBeforeRun.manifest_path
                after_run = Get-FileRecord -CaptureRoot $captureRoot -Path $sourceAfterRun.manifest_path
            }
            manifests_identical = $true
            snapshot_root = 'source/snapshot'
            dirty_diff_before = Get-FileRecord -CaptureRoot $captureRoot -Path $dirtyBefore.path
            dirty_diff_after = Get-FileRecord -CaptureRoot $captureRoot -Path $dirtyAfter.path
        }
        executable = [ordered]@{
            path = 'executable/powdergame-benchmark.exe'
            build_output_sha256 = $builtExecutableSha
            captured_sha256_before_run = $executableShaBefore
            captured_sha256_after_run = $executableShaAfter
            unchanged = $true
        }
        csv = [ordered]@{
            schema_version = 'powdergame-g8a-v5'
            run_id = $aggregateIdentity.run_id
            stdout_run_id = $stdoutRunId
            staged_records = [ordered]@{
                raw_cells = [ordered]@{ path = 'artifacts/aggregate_raw_cells.csv'; data_row_count = $rawCellsIdentity.data_row_count; value_row_count = $rawCellsIdentity.value_row_count; size_bytes = $rawCellsIdentity.size_bytes; sha256 = $rawCellsIdentity.sha256 }
                raw_chunks = [ordered]@{ path = 'artifacts/aggregate_raw_chunks.csv'; data_row_count = $rawChunksIdentity.data_row_count; value_row_count = $rawChunksIdentity.value_row_count; size_bytes = $rawChunksIdentity.size_bytes; sha256 = $rawChunksIdentity.sha256 }
                raw_ticks = [ordered]@{ path = 'artifacts/aggregate_raw_ticks.csv'; data_row_count = $rawTicksIdentity.data_row_count; value_row_count = $rawTicksIdentity.value_row_count; size_bytes = $rawTicksIdentity.size_bytes; sha256 = $rawTicksIdentity.sha256 }
                aggregate = [ordered]@{ path = 'artifacts/aggregate.csv'; data_row_count = $aggregateIdentity.data_row_count; value_row_count = $aggregateIdentity.value_row_count; size_bytes = $aggregateIdentity.size_bytes; sha256 = $aggregateIdentity.sha256 }
            }
        }
        census_encoding = [ordered]@{
            cell_activity_bits = [ordered]@{ matter = 1; thermal = 2; pressure = 4; reaction = 8 }
            chunk_state_values = [ordered]@{ runnable = 0; sleeping = 1 }
        }
        toolchain = [ordered]@{
            powershell = $PSVersionTable.PSVersion.ToString()
            os_description = [Runtime.InteropServices.RuntimeInformation]::OSDescription
            process_architecture = [Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture.ToString()
            git = $toolchainGit.text
            cargo = $toolchainCargo.text
            rustc = $toolchainRustc.text
        }
        commands = $commandMetadata
        intended_publication_order = @('artifacts/aggregate_raw_cells.csv', 'artifacts/aggregate_raw_chunks.csv', 'artifacts/aggregate_raw_ticks.csv', 'artifacts/aggregate.csv', 'metadata/CAPTURE_METADATA.json', 'hashes/SHA256SUMS.txt', 'CAPTURE_RECEIPT.json')
    }
    Write-Utf8NoBomSyncedCreateNew -Path $stagedMetadata -Text (($metadata | ConvertTo-Json -Depth 24) + "`n")

    $stagedPaths = [ordered]@{
        raw_cells = $stagedRawCells
        raw_chunks = $stagedRawChunks
        raw_ticks = $stagedRawTicks
        aggregate = $stagedAggregate
        metadata = $stagedMetadata
    }
    $publicationJournal = [Collections.Generic.List[string]]::new()
    Publish-FinalArtifactSet -StagedPaths $stagedPaths -FinalPaths $finalPaths -PublicationJournal $publicationJournal
    $expectedPublishedOrder = @($finalPaths.Values)
    if (($publicationJournal -join "`n") -ne ($expectedPublishedOrder -join "`n")) {
        throw 'Final artifact publication order deviated from raw cells -> raw chunks -> raw ticks -> aggregate -> metadata.'
    }

    Remove-SuccessWorkTree -CaptureRoot $captureRoot
    $manifestPath = Join-Path $captureRoot 'hashes\SHA256SUMS.txt'
    $hashManifest = New-HashManifest -DestinationRoot $destinationRootFull -CaptureRoot $captureRoot -ManifestPath $manifestPath -PublicationJournal $publicationJournal
    $artifactRecords = [ordered]@{}
    foreach ($name in @('raw_cells', 'raw_chunks', 'raw_ticks', 'aggregate')) {
        $identity = switch ($name) {
            'raw_cells' { $rawCellsIdentity }
            'raw_chunks' { $rawChunksIdentity }
            'raw_ticks' { $rawTicksIdentity }
            'aggregate' { $aggregateIdentity }
        }
        $record = Get-FileRecord -CaptureRoot $captureRoot -Path $finalPaths[$name]
        $record['data_row_count'] = $identity.data_row_count
        $record['value_row_count'] = $identity.value_row_count
        $artifactRecords[$name] = $record
    }
    $metadataRecord = Get-FileRecord -CaptureRoot $captureRoot -Path $finalPaths.metadata
    $commandsByLabel = [ordered]@{}
    foreach ($command in $script:RecordedCommands) {
        $commandsByLabel[$command.label] = Convert-CommandRecordForMetadata -Command $command -CaptureRoot $captureRoot
    }
    $receipt = [ordered]@{
        receipt_schema = 'powdergame-g8a-capture-receipt-v2'
        official_mode = $true
        complete = $true
        capture_id = $CaptureId
        run_id = $aggregateIdentity.run_id
        started_utc = $startedUtc
        completed_utc = [DateTime]::UtcNow.ToString('o')
        repository = [ordered]@{
            origin_url = $originUrl.text
            branch = $branchBefore.text
            source_sha = $headBefore.text
            upstream_ref = $upstreamRef.text
            upstream_sha = $upstreamSha.text
            git_state = 'clean'
            clean = $true
        }
        source = [ordered]@{
            input_count = $sourceBeforeBuild.path_count
            manifest_sha256 = $sourceBeforeBuild.manifest_sha256
            manifest_paths = @(
                Get-RelativeForwardPath -Root $captureRoot -Path $sourceBeforeBuild.manifest_path
                Get-RelativeForwardPath -Root $captureRoot -Path $sourceAfterBuild.manifest_path
                Get-RelativeForwardPath -Root $captureRoot -Path $sourceBeforeRun.manifest_path
                Get-RelativeForwardPath -Root $captureRoot -Path $sourceAfterRun.manifest_path
            )
            manifests_identical = $true
            source_unchanged = $true
            dirty_diff_before = Get-FileRecord -CaptureRoot $captureRoot -Path $dirtyBefore.path
            dirty_diff_after = Get-FileRecord -CaptureRoot $captureRoot -Path $dirtyAfter.path
            status_before_path = Get-RelativeForwardPath -Root $captureRoot -Path $statusBeforePath
            status_after_path = Get-RelativeForwardPath -Root $captureRoot -Path $statusAfterPath
        }
        executable = [ordered]@{
            path = 'executable/powdergame-benchmark.exe'
            sha256 = $executableShaAfter
            unchanged = $true
        }
        schema_version = 'powdergame-g8a-v5'
        run_id_links_complete = $true
        census_encoding = [ordered]@{
            cell_activity_bits = [ordered]@{ matter = 1; thermal = 2; pressure = 4; reaction = 8 }
            chunk_state_values = [ordered]@{ runnable = 0; sleeping = 1 }
        }
        artifacts = $artifactRecords
        metadata = $metadataRecord
        commands = $commandsByLabel
        publication_order = @('artifacts/aggregate_raw_cells.csv', 'artifacts/aggregate_raw_chunks.csv', 'artifacts/aggregate_raw_ticks.csv', 'artifacts/aggregate.csv', 'metadata/CAPTURE_METADATA.json', 'hashes/SHA256SUMS.txt', 'CAPTURE_RECEIPT.json')
        hash_manifest = [ordered]@{
            path = Get-RelativeForwardPath -Root $captureRoot -Path $hashManifest.path
            sha256 = $hashManifest.sha256
            file_count = $hashManifest.file_count
            excludes_self_and_receipt = $true
        }
        package = [ordered]@{
            created_after_receipt = $true
            zip_path_outside_capture = $CaptureId + '.zip'
            zip_sha256_path_outside_zip = 'PACKAGE_SHA256.txt'
        }
    }
    $requiredPreReceipt = @($finalPaths.Values) + @(
        $manifestPath,
        $capturedExecutable,
        $sourceBeforeBuild.manifest_path,
        $sourceAfterBuild.manifest_path,
        $sourceBeforeRun.manifest_path,
        $sourceAfterRun.manifest_path,
        $dirtyBefore.path,
        $dirtyAfter.path,
        $statusBeforePath,
        $statusAfterPath
    )
    $receiptPath = Publish-CaptureReceipt -DestinationRoot $destinationRootFull -CaptureRoot $captureRoot -Receipt $receipt -RequiredPreReceiptPaths $requiredPreReceipt -PublicationJournal $publicationJournal
    if ($publicationJournal[$publicationJournal.Count - 1] -ne $receiptPath) {
        throw 'Receipt was not the final file publication inside the capture.'
    }

    # Do not create or alter anything inside the capture after this point.
    $package = New-Package -DestinationRoot $destinationRootFull -CaptureRoot $captureRoot -CaptureId $CaptureId
    Write-Output (ConvertTo-Json ([ordered]@{
                capture_id = $CaptureId
                source_sha = $headBefore.text
                capture_root = $captureRoot
                receipt_path = $receiptPath
                zip_path = $package.zip_path
                zip_sha256 = $package.zip_sha256
                package_sha256_path = $package.package_sha256_path
            }) -Depth 4)
}
catch {
    $receiptPathOnFailure = Join-Path $captureRoot 'CAPTURE_RECEIPT.json'
    if (Test-Path -LiteralPath $receiptPathOnFailure -PathType Leaf) {
        [Console]::Error.WriteLine("Official capture/package failed after receipt publication. Capture and receipt are preserved; this CaptureId must not be reused: $captureRoot")
    }
    else {
        [Console]::Error.WriteLine("Official capture failed incomplete. Files are preserved without CAPTURE_RECEIPT.json; this CaptureId must not be reused: $captureRoot")
    }
    throw
}
