# Conservative Phase Packets Validation

- **Evidence identity:** `TE3Q-PHASE-PACKETS-REFERENCE-V1`
- **Runtime validation:** NOT AUTHORIZED / NOT RUN
- **Reference execution limit:** exactly one frozen model execution

## 1. Evidence files

The external standard-library-only script and result are:

```text
C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te3q_phase_packets_reference_v1.py
C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te3q_phase_packets_reference_v1_result.json
```

Frozen SHA-256:

```text
script c938c6e3ce7074abc6d5144c708f85a17be349bb84f962238e568c17d55ed03c
result a0181d4ca0ed63eb92cac5cd04098ff438546903c8dc6853e8b0b5d5ab208ed7
```

Before freezing, only syntax compilation, standard-library import inspection
and `--list-fixtures` are allowed. The list command performs no model
evaluation. After the script hash is frozen, the model command may run once;
the result path must not pre-exist and the script refuses overwrite.

## 2. Frozen parameters

```text
seed                         = 0x54453351
algebra_trials               = 100000
multi_tick_grids             = 10000
PHASE_UNIT_SCALE             = 2
Lf / Lv                      = 80 / 480
H absolute / relative tol    = 1e-3 / 2e-6
merge neighbourhood          = orthogonal only
merge arbitration            = deterministic parity/hash then one claim winner
phase pressure relaxation    = 0.20
phase pressure diffusion     = 0.05
phase pressure epsilon       = 0.01
phase pressure maximum       = 100
Wood observation threshold   = 80
isolated crossing window     = updates 8..8
grid sizes                   = 5x5 through 16x16
grid horizon                 = 8 through 48 ticks
chunk partitions             = 4 and 8 Cell widths
```

No coefficient, formula, merge order, geometry, tolerance or pass projection
may change after execution based on output.

## 2.1 Execution receipt

The frozen model was executed exactly once with:

```powershell
& 'C:\Users\mdkap\AppData\Local\Programs\Python\Python313\python.exe' `
  'C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te3q_phase_packets_reference_v1.py' `
  --run `
  --output 'C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te3q_phase_packets_reference_v1_result.json'
```

The process exited `0`. JSON parsing succeeded, the embedded script hash
matched the frozen file, and the receipt reported:

| Field | Result |
|---|---:|
| process execution | 1 |
| fixture functions | 14 / 14 PASS |
| split/merge algebra | 100,000 PASS |
| bounded multi-tick grids | 10,000 PASS |
| blocked / split / merge operations | 11,650 / 38,077 / 54 |
| movement / density-swap operations | 184,140 / 4,067 |
| algebra digest | `56276518534e401c0af66bccd04fbc89a406d4fdfabea85f6a52d88fb929ad60` |
| grid digest | `4568dc5936d53b9480acd62ef88d96715ee26abbf76bdc2bcdbd2563005511c8` |
| deterministic replay digest | `259c83a2526385036313cf9346d2b8c7fbcbb2937b0375381afef5610fd50e60` |
| smallest counterexample | none |

Automatic disposition is **MATHEMATICAL_REFERENCE_PASS**. This does not
advance ADR-0011 or runtime; independent review and user architecture review
remain mandatory.

Fresh review then found that several named fixtures do not execute their named
paths: F08 avoids contention/cold-lid/checkerboard work, F09 returns a literal
trace without a beaker/ticks, F11 returns constant chunk/sleep claims, F13 has
no Current/Next editor/reset path, and F14 returns constants. Random campaigns
also omit Atmosphere receivers, Void, chunk/sleep and production claim
semantics. Thus the receipt remains a valid PASS for its reduced mathematical
model but does not satisfy this validation contract. Final candidate
disposition is **DESIGN BLOCKED**, Critical `0` / High `8` / Medium `1`.

## 3. Executable fixture matrix

| ID | Required modeled assertion |
|---|---|
| PQ-F01 | Water/2 with receiver splits to two Steam/1; units/H exact; source target zero phase pressure |
| PQ-F02 | blocked Water/2 becomes Steam/2 and crosses pressure 80 on update 8 |
| PQ-F03 | staggered one-column vacancy witness consumes the only EMPTY as Matter; later boil compresses |
| PQ-F04 | a real opening lets Steam/2 split and its pressure declines |
| PQ-F05 | two endpoint-ready Steam/1 merge to Water/2 + Vacuum EMPTY with exact units/H |
| PQ-F06 | Steam/2 condenses in place to Water/2 and removes its pressure source |
| PQ-F07 | lone Steam/1 remains finite; second orthogonal packet enables merge |
| PQ-F08 | cold-lid packets merge locally without global one-tick conversion/checkerboard residue |
| PQ-F09 | open beaker heat→split→rise→cool→pair merge→fall trace conserves units |
| PQ-F10 | finite boiler consumes headspace, compresses, crosses Wood, opens, splits and declines |
| PQ-F11 | movement, density swap, chunk partition and sleep-on/off preserve packet ownership |
| PQ-F12 | receiver failure leaves target Air and source Steam/2 exact |
| PQ-F13 | Draw/Erase/reset canonicalize both halves and reject invalid state |
| PQ-F14 | generic G5/TE-2 evidence remains unbound; new source reports new obligations |

Every fixture is a callable model path. A label-only `MODELED_PASS` value is
forbidden. Random coverage includes thin columns, open beakers, sealed boilers,
cold lids, density swaps, diagonal openings that do not count as merge pairs,
chunk seams and mixed latent progress.

## 4. Automatic acceptance criteria

The result is mathematical PASS only if all fixtures pass and:

- all 100,000 algebra trials conserve integer units exactly and H within the
  frozen scaled tolerance;
- all 10,000 grids conserve units except logged Void/destructive events;
- no Water/1, Ice/1, zero-unit phase Matter or out-of-range E appears;
- the vacancy-walk control cannot keep every later completion expanded;
- phase-pressure sources exist only for Steam/2 and all values are finite in
  `[0,100]`;
- split/condensation removes the source and produces a lower later pressure;
- open controls stay below Wood threshold while sealed boilers reach it;
- split and merge Air accounting is exact;
- an in-process deterministic replay produces the same canonical digest.

Failure emits the smallest recorded counterexample and final disposition
`DESIGN_BLOCKED`. The result distinguishes mathematical, GPU, visual and
product status.

## 5. Evidence limitations

The reference cannot establish WGSL compilation, bindings, races, f32 device
rounding, actual GAS motion, Environment receiver implementation, sleep,
Inspector/editor paths, profiler integration, memory allocation, performance,
visual readability or user acceptance. Cargo, GPU/Naga/device, workspace FULL,
build, launch and TE-3/TE-5 runtime counts must remain zero.
