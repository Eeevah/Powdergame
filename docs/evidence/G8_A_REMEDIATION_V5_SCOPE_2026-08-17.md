# G8-A Evidence Remediation v5 Scope Freeze

Date: 2026-08-17
Base commit: `a67abaf959aba0423627f35b79fce7c82d8ec9b5`
Working branch: `fix/g8a-evidence-remediation-v5`

Pre-branch preservation record:

- external directory:
  `C:\Users\mdkap\source\Powdergame-remediation-backups\20260817T022543470Z-a67abaf959ab`;
- full tracked/untracked binary patch: `FULL_DIRTY_BINARY.patch`;
- patch SHA-256:
  `9993574e552f2cf9523aa2e1c3ee8b0f7ebe6dae422eebf5a8ecbe2737c0cd5b`;
- recorded untracked paths: 7;
- the branch was attached without reset, stash, rebase, pull, or source-file
  rewriting.

## Purpose

This branch packages the detached dirty G8-A correction work into a reproducible
source candidate and defines the evidence publication boundary. It does not
close G8-A by assertion. A current candidate exists only after the committed,
pushed, clean source SHA completes the official capture and independent
verification workflow.

## Frozen implementation scope

Allowed changes are limited to:

- the G8-A benchmark CLI, fixed calibration fixture, statistics, and evidence
  serialization;
- timestamp validation and profiler reporting used by G8-A;
- persistent tracked-buffer inventory reporting;
- preservation and serialization of raw cell/chunk activity census data;
- clean-source capture, immutable/no-overwrite publication, receipt, package
  hashing, and independent verification tooling;
- tests and documentation required to specify or verify those contracts.

The following are explicitly outside this branch:

- G8-B benchmark fixtures;
- G8-C matrix work;
- G9 gameplay implementation;
- new materials or reactions;
- performance optimization;
- merge or integration into `main`;
- Canonical Recovery.

## Publication contract

1. Official capture accepts only a clean, attached source branch.
2. The capture ID and every final path are create-once and never overwritten.
3. Raw cell data is published before raw chunk data. Remaining raw timing and
   aggregate/metadata artifacts follow.
4. `CAPTURE_RECEIPT.json` is written last inside the capture directory. Its
   absence means the capture is incomplete regardless of other files.
5. A failed capture is retained under its original capture ID and is not
   repaired or rerun under that ID.
6. The ZIP is created only after the receipt. Its SHA-256 is written outside the
   ZIP in `PACKAGE_SHA256.txt`.
7. Independent verification uses a separate script and code path from capture.

## Historical boundary

The existing file set commonly called v4 is retained as historical data. It is
not rebound to later source bytes and is not rewritten. Only a new v5 capture
from the final clean source SHA may be recorded as the current evidence
candidate.

## Source and generated-data boundary

Only source, tests, scripts, and documentation belong in the branch commit.
Benchmark CSVs, capture receipts, executables, logs, snapshots, ZIPs, and
package hashes remain outside Git and are generated only after the source SHA is
fixed and pushed.
