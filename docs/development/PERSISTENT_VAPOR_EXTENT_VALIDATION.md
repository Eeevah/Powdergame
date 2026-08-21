# Persistent Vapor Extent Validation

- **Status:** predeclared docs/reference validation contract
- **Runtime evidence:** none; TE-3/TE-5 runtime not started
- **Historical receipts:** TE-5B and TE-5C do not transfer

## 1. One-shot reference proof contract

The external proof script and result live outside the repository. Before its
only execution, freeze seed `0x54453544`, 50,000 randomized matching graphs,
10,000 bounded multi-tick grids, `MAX_REASSIGNMENT_DEPTH=6`,
`MATCH_SETTLE_TICKS=6`, relaxation `0.10`, diffusion `0.025`, Wood threshold
80, source equilibrium 100 and horizons recorded in the script.

The proof must model reciprocal links, reservation Air displacement,
zero-Air refill exclusion, the vacancy-walk column, owner movement into its
extent and another EMPTY, density swap, condensation, Void release, matching,
phase pressure and generic-pressure separation. It must emit deterministic
JSON and a smallest counterexample on failure.

Graph coverage has two distinct claims:

1. direct exhaustive enumeration performed by the script;
2. theorem-backed or structured families which are not to be mislabeled as
   direct enumeration.

The requested exhaustive all-labeled 6×6 bipartite space contains `2^36`
graphs. If the one-shot process does not literally visit that space or provide
a mechanically checked symbolic equivalent, the receipt must say the minimum
coverage obligation is unmet. A sampled or family proof is not renamed
"exhaustive".

## 2. Fixtures

| ID | Required observation |
|---|---|
| TE5D-F01 | one reciprocal Steam/extent pair, lossless Air displacement, pressure relaxes |
| TE5D-F02 | fully confined Steam becomes compressed and reaches threshold non-instantly |
| TE5D-F03 | original vacancy-walk cannot mint capacity; second Steam is compressed |
| TE5D-F04 | asymmetric A→E1/E2, B→E1 obtains both extents below threshold |
| TE5D-F05 | no-matching graph leaves exactly maximum-matching deficit compressed |
| TE5D-F06 | owner→own extent flips the reciprocal pair and preserves Air |
| TE5D-F07 | owner→other EMPTY closes the three-Cell Air transaction |
| TE5D-F08 | density swap keeps extent and updates backlink |
| TE5D-F09 | condensation releases Vacuum and phase pressure declines |
| TE5D-F10 | receiver failure is byte-identical and source compresses |
| TE5D-F11 | open beaker has enough persistent extents and no false rupture |
| TE5D-F12 | finite boiler reaches phase pressure, rupture, opening and decline |
| TE5D-F13 | multiple condensation releases exact count; no orphan |
| TE5D-F14 | Draw blocks target; owner erase, reset and staging are canonical |
| TE5D-F15 | chunk seam, sleep equivalence, wake halo and scratch lifetimes hold |
| TE5D-F16 | generic and phase pressure remain separate; rupture sums once |

All fixture geometry, event order, horizon and tolerance are constants before
execution. Modeled PASS does not establish WGSL bindings, GPU races, sleep,
actual movement visuals, performance or user acceptance.

## 3. Coefficient checks

The predeclared comparison set is `(relaxation,diffusion)` = `(0.05,0.0125)`,
`(0.10,0.025)`, `(0.15,0.0375)`. The middle pair is normative regardless of
observed output. For every pair check non-negativity, no extrema overshoot and
`r+4d<=1`. The normative isolated compressed source must first reach 80 in a
bounded non-instant interval; a six-tick matching delay must remain below 80;
after relief its later peak must decline.

## 4. Future structural and runtime evidence

Before implementation acceptance, pin all Current/Next writers, reciprocal
repair behavior, exact scratch reuse, <=8 storage bindings, 62-pass projection
or its approved replacement, 124 query identities, allocation bytes,
reset/editor/staging coverage and activity equivalence. Device tests must
cover every fixture and a source-bound atomic G5 chain. None is run here.

## 5. Stop rule

Any failed invariant, complete-matching false-pressure counterexample,
unverified 6×6 exhaustive obligation or unresolved Critical/High review
finding makes TE-5D DESIGN BLOCKED. The receipt must classify the repair as
another persistent field, full-world scratch, wider matching scope, different
volume representation or relaxation of 1:1 quantity. The frozen algorithm or
coefficients are not changed after the one-shot run.

## 6. One-shot receipt — 2026-08-21

Exactly one proof process executed. An earlier call through the stale default
`python` launcher failed before creating a process or result file. The actual
command was:

```powershell
& 'C:\Users\mdkap\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' `
  'C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te5d_persistent_extent_proof.py' `
  'C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te5d_persistent_extent_proof_result.json'
```

- seed: `0x54453544`
- script SHA-256: `06d0cea8500fcc3a2ffa4010d0dab70770a3fb2fd8a94f0bf47846cd980dedb9`
- result SHA-256: `853379af86ee536166cb752bffbf45cefe5eec93bc10038a023679d507d7a29a`
- randomized matching graphs: 50,000
- bounded multi-tick grids: 10,000
- direct labeled graph enumeration: 682 graphs across every 1×1 through 3×3
  shape
- structured 6×6 cycle families: 12
- required all-labeled 6×6 space: `2^36 = 68,719,476,736`; directly visited 0
- deterministic in-process replay digest:
  `b4b608c08d597e45be83283fbfb516feeeef7cd68bd2272676de04463a75b914`
- JSON parse and post-run script/result hashes: verified

The result reported `DESIGN_BLOCKED` because the literal/symbolic all-labeled
6×6 obligation was not met. Its modeled fresh-start graph checks reported no
matching failure, but that is not evidence for arbitrary persistent initial
links. Static post-receipt analysis found the eight-source alternating-chain
witness described in ADR-0009: a complete matching exists, while the frozen
depth-six atomic retry cannot reach the free endpoint. The source can
therefore cross Wood threshold at the modeled isolated-source tick 16.

Disposition: unresolved High matching completeness blocker. Required repair:
**wider matching scope**; a future fixed-budget GPU design may additionally
need a full-world search scratch. The proof was not patched or rerun, and no
coefficient or algorithm was substituted after observing the result.
