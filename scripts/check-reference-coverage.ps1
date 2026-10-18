[CmdletBinding()]
param(
    [string]$AuditPath = (Join-Path $PSScriptRoot '../docs/reference-audit.json'),
    [switch]$RequireFullMatch
)
$ErrorActionPreference = 'Stop'
$workspaceRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$audit = Get-Content -LiteralPath $AuditPath -Raw | ConvertFrom-Json
if ($audit.schema -ne 2) { throw 'Unsupported reference-audit schema' }
if ($audit.all_features_match -isnot [bool] -or $audit.runtime_evidence.cadence_qualified -isnot [bool] -or
    $audit.runtime_evidence.hardware_qualified -isnot [bool] -or $audit.reference.wire_contract_qualified -isnot [bool]) { throw 'Acceptance and qualification values must be booleans' }

function Assert-Catalogue($items, [int]$expected, [string]$key, [string]$label) {
    $rows = @($items)
    if ($rows.Count -ne $expected) { throw "Incomplete $label catalogue: expected $expected rows" }
    $values = @($rows | ForEach-Object { $_.$key })
    if (@($values | Sort-Object -Unique).Count -ne $expected) { throw "Duplicate $label identity" }
    foreach ($row in $rows) {
        if ($row.status -notin @('not_started','stub','experimental','tested','production-ready')) { throw "Invalid $label status" }
        if ($row.conformance -notin @('missing','partial','matched','different_by_design')) { throw "Invalid $label conformance" }
        if ([string]::IsNullOrWhiteSpace($row.scope)) { throw "Missing $label scope" }
    }
}
Assert-Catalogue $audit.areas 30 'id' 'feature area'
Assert-Catalogue $audit.input_types 40 'name' 'input type'
Assert-Catalogue $audit.transitions 20 'name' 'transition'
Assert-Catalogue $audit.overlay_channels 16 'channel' 'overlay channel'
Assert-Catalogue $audit.audio_destinations 9 'name' 'audio destination'
Assert-Catalogue $audit.ui_workflows 12 'id' 'UI workflow'
Assert-Catalogue @($audit.shortcut_function_inventory) 1 'scope' 'shortcut inventory'
if (@(Compare-Object @(1..30) @($audit.areas.id)).Count -ne 0) { throw 'Feature area IDs must cover 1..30' }
if (@(Compare-Object @(1..16) @($audit.overlay_channels.channel)).Count -ne 0) { throw 'Overlay channels must cover 1..16' }
if ($audit.reference.observed_shortcut_functions -ne 777 -or $audit.reference.parsed_unique_shortcut_function_names -ne 777) {
    throw 'Observed shortcut-function catalogue is incomplete'
}
$groups = @($audit.shortcut_function_inventory.groups)
if ($groups.Count -ne 29 -or @($groups.name | Sort-Object -Unique).Count -ne 29) { throw 'Incomplete shortcut-function groups' }
foreach ($group in $groups) {
    if (@($group.functions).Count -ne $group.declared_count -or $group.declared_count -le 0) { throw 'Invalid shortcut-function group size' }
}
[string[]]$functionNames = @($groups | ForEach-Object { $_.functions })
if ($functionNames.Count -ne 777 -or @($functionNames | Sort-Object -Unique).Count -ne 777 -or
    @($functionNames | Where-Object { $_ -cnotmatch '^[A-Za-z][A-Za-z0-9_]*$' }).Count -ne 0) { throw 'Invalid/duplicate shortcut-function names' }
[Array]::Sort($functionNames, [StringComparer]::Ordinal)
$canonicalNames = [string]::Join("`n", $functionNames) + "`n"
$namesHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($canonicalNames))).ToLowerInvariant()
if ($namesHash -ne '804ae58059d823e96dffef86c9c2053c6fad5f746ccf5228ecc89643af4a99da' -or
    $audit.reference.function_names_sha256 -ne $namesHash) { throw 'Public-function names differ from the pinned behavioral catalogue' }
if ($audit.reference.protocol_compatible_functions -lt 0) { throw 'Invalid compatible-function count' }
if ($audit.reference.catalogue_kind -notlike '*not the accepted HTTP*' -or $audit.wire_observations.shortcut_vocabulary_equals_http -ne $false) { throw 'Shortcut and HTTP evidence must remain distinct' }
foreach ($area in $audit.areas) {
    foreach ($relativePath in @($area.source_evidence) + @($area.test_files)) {
        if ([string]::IsNullOrWhiteSpace($relativePath) -or [IO.Path]::IsPathRooted($relativePath)) { throw 'Evidence paths must be workspace-relative' }
        $evidence = [IO.Path]::GetFullPath((Join-Path $workspaceRoot $relativePath))
        if (-not $evidence.StartsWith($workspaceRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase) -or
            -not (Test-Path -LiteralPath $evidence -PathType Leaf)) { throw "Missing or out-of-workspace evidence: $relativePath" }
    }
}
$core = Get-Content -LiteralPath (Join-Path $workspaceRoot 'crates/engine/src/lib.rs') -Raw
$effectBlock = [regex]::Match($core, '(?s)pub enum TransitionKind \{(.*?)\n\}').Groups[1].Value
$engineEffects = @([regex]::Matches($effectBlock, '(?m)^\s*(\w+),\s*$') | ForEach-Object { $_.Groups[1].Value })
$ledgerEffects = @($audit.transitions | Where-Object { $null -ne $_.engine_variant } | ForEach-Object { $_.engine_variant })
if ($engineEffects.Count -eq 0 -or @(Compare-Object $engineEffects $ledgerEffects).Count -ne 0) { throw 'Transition inventory does not match the actual engine enum' }
$overlayCount = [int][regex]::Match($core, 'pub overlays: \[OverlayState; (\d+)\]').Groups[1].Value
if ($overlayCount -ne @($audit.overlay_channels | Where-Object { $_.status -ne 'not_started' }).Count) { throw 'Overlay scope exceeds the actual engine bank' }
$busCount = [int][regex]::Match($core, 'pub buses: \[AudioChannel; (\d+)\]').Groups[1].Value
if ($busCount -ne @($audit.audio_destinations | Where-Object { $_.status -ne 'not_started' }).Count) { throw 'Audio scope exceeds the actual engine bank' }
$allRows = @($audit.areas) + @($audit.input_types) + @($audit.transitions) + @($audit.overlay_channels) + @($audit.audio_destinations) + @($audit.ui_workflows) + @($audit.shortcut_function_inventory)
$unmatched = @($allRows | Where-Object { $_.conformance -ne 'matched' }).Count
$fullMatch = ($unmatched -eq 0 -and $audit.reference.wire_contract_qualified -eq $true -and
    $audit.runtime_evidence.cadence_qualified -eq $true -and $audit.runtime_evidence.hardware_qualified -eq $true)
if ($audit.all_features_match -ne $fullMatch) { throw 'The all-features-match claim is inconsistent with its evidence and gaps' }
Write-Output "Reference inventory verified: 30 areas, 40 input types, 20 effects, 16 overlay/stinger channels, 9 audio destinations, 12 UI workflows, 777 unique shortcut function names in 29 groups."
Write-Output "Actual engine scope: $($engineEffects.Count) effects, $overlayCount title panels, $busCount simulated audio buses. Full reference match: $fullMatch."
if ($RequireFullMatch -and -not $fullMatch) { throw 'Full reference acceptance FAILED: missing/partial features, protocol, hardware and cadence qualification remain.' }
