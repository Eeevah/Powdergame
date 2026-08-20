# Checkpoint — TE-3D architecture accepted with locked amendments — 2026-08-21 01:11 KST

## Repository coordinate

- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Session start / D-018 design baseline: `b05b44207ecba1442b67dd1e80b1025590c08d60`
- TE-2 closure commit: `fd97e8b89f277e1205c8b5bcd970002bfd87e7c4`
- Production TE-2 source: `fb7e568e21012b6067269f4e1b82c36c865023d0`
- Review-remediation source: `097728128343cf89383920c968a010b3dcf8e8c0`
- Closure coordinate: the docs/memory commit containing this checkpoint on the
  same branch; runtime source remains byte-unchanged from the baseline

## The story so far

G9-A and TE-2 remain **USER ACCEPTED WITH KNOWN FOLLOW-UP**. D-018 accepts
Hybrid A+C with locked amendments: one Water-equivalent family Cell, 1:1
transitions, two phase-energy halves, local H after TE-2 Q, constants
80/480/80/10/70, 32 MiB at 2048², no-sink metastability and atomic same-source
TE-5 activation. The two non-blocking TE-2 follow-ups remain
`LONG_HORIZON_SEALED_AIR_DRIFT_BUDGET` and
`TE2_CANDIDATE_HUD_LABEL_POLISH`.

ADR-0006 is **ACCEPTED FOR FUTURE ATOMIC IMPLEMENTATION**. The locked design
adds a real TE-2 energy-removal sink, value-derived vaporization-ready Water,
radius-2 seed/veto with a 30-tick bound, generic phase-target hygiene and exact
internal hash provenance. It does not implement or activate runtime.

The amended pure reference proof passed its only run. The fresh-context v2
review then closed every required High attack and finished **INDEPENDENT V2
DESIGN REVIEW PASS — UNRESOLVED CRITICAL 0 / HIGH 0**. TE-3D therefore stops
at **ARCHITECTURE ACCEPTED WITH LOCKED AMENDMENTS**. TE-3 runtime remains
**NOT STARTED** and the TE-5 bridge is **DESIGN REQUIRED / NOT STARTED**.

## Valid evidence

- V2 reference script/result SHA-256:
  `c3624e467638a62ef2b62f96c8b12954ceef70609feeac47da70eca69f84db23` /
  `f727101543f4eaa7582def01940e2567dd3b79bc6e585cfad4051160de1d90ea`.
- Fixed seed `0x54453344`, 50,000 enthalpy trials, 4,096 generated regions,
  maximum absolute H error `1.52587890625e-05`, radius-2 maximum 209 new
  initiations in a sampled 30-tick window and 100 closed cycles ending with
  one Water quantity at 20°C/E=0.
- Independent v2 review SHA-256:
  `c6d63fd84d8057e6cbe201696df0a4914e1a396eaeaf2bc189a5ebcd24a9a31d`;
  unresolved Critical/High `0/0`, three Medium and two Low future obligations.
- The proof/review do not establish WGSL, bindings, movement, sleep,
  performance, appearance, TE-5 or runtime acceptance. No
  Cargo/test/check/clippy/GPU/FULL/build/launch/candidate/G8/G8-C run occurred.
- Local personal-infra Wiki remained user-dirty; verified remote
  `origin/main` `b8c22c1dc477f7d08f35b54e11ca95c6ad10d4c3` was used read-only.

## Decided

- D-018 is the latest user-confirmed TE-3 architecture decision; D-017's G9-A
  and TE-2 acceptance plus two follow-ups remain active.
- ADR-0006 is **ACCEPTED FOR FUTURE ATOMIC IMPLEMENTATION** and TE-3D is
  **ARCHITECTURE ACCEPTED WITH LOCKED AMENDMENTS**.
- Locked constants are `Lf=80`, `Lv=480`, surface maximum 80°C, minimum delta
  10°C, free-air maximum 70°C and `NUCLEATION_RADIUS=2`.
- Water yield 1 cannot become production/user-testable before a separately
  authorized same-source TE-5 replacement preserves the frozen G5 causal
  chain atomically; that replacement is not designed or authorized here.
- TE-3 runtime, Air-pressure force, TE-4 and G9-B/C/D/E remain **NOT STARTED**.

## Waiting on user

Separate user authorization is required before designing the TE-5
pressure-volume bridge or beginning any TE-3/TE-5 runtime implementation.
Future runtime, device, G5 causal and product/user evidence remains unrun.

## Next first action

Obtain explicit user authorization for the TE-5 pressure-volume bridge design
before changing runtime or beginning any TE-3/TE-5 implementation work.

## Tried

- Audited the internal `edge_priority` implementations and reused their exact
  arithmetic/constants with only a newly documented coordinate mapping.
- Predeclared the radius-2 hard properties and 30-tick bound, then ran the new
  reference proof exactly once and preserved the v1 receipt separately.
- Used a fresh-context reviewer; all Critical/High attacks closed, while five
  lower-severity future-evidence obligations remain visible.
- Preserved all runtime/G5/TE-1/TE-2 evidence at its original source and
  changed no Rust, WGSL, Cargo, build, launch or Wiki file.
