# Canonical Recovery Evidence (2026-08-17)

## Declaration

- **Result**: PASS — verified implementation/evidence and Foundation research lines are integrated and validated locally.
- **Branch**: `integration/canonical-recovery`
- **Publication**: local only; no recovery-branch push, recovery pull request, `main` promotion, release, or deployment was performed.
- **Product status**: G8 remains `IN_PROGRESS`; G8-A is a verified v5 evidence candidate with same-SHA user visual validation pending; G8-B/G8-C and G9 remain pending; M0 `ACHIEVED` remains `NO`.
- **Decision boundary**: this recovery does not select or start the next Gate.

## Synchronized Starting State

- `personal-infra-wiki`: clean `main` at `98148d16a65965c9ceadf24e1fb5ed94f9bef37b`, equal to `origin/main` after fetch and fast-forward check.
- Powdergame `origin/main`: `1304b71a15df140a994737becb5f47f421758801`.
- verified implementation/evidence source: local and remote `fix/g8a-evidence-remediation-v5` at `9abec9ee632b9abe429b13cf0cfb2e3ae7eacefe`.
- Foundation source: clean local `feature/foundation-material-wiki` at `ccd0d7b00fb99128e8750ef09e5c4cce068bce09`; `origin/main` is its direct parent.
- Repository object integrity passed; no Git lock, merge, rebase, cherry-pick, or revert state was present. The recovery path and branch did not exist before creation.

The verified implementation source was selected instead of its ancestor `a67abaf` because it contains the complete product-first documentation plus the later G8-A v5 evidence-integrity corrections. Draft PR #1 remains open and draft; its head `a5efc1b` was not merged or cherry-picked because its product-first content is already subsumed by `9abec9e` and taking it separately would regress the later corrections.

## Official G8-A v5 Evidence

- capture ID: `g8a-v5-9abec9e-20260817T032827206Z`
- run ID: `g8a-1786937354490`
- source/upstream: clean and unchanged at `9abec9ee632b9abe429b13cf0cfb2e3ae7eacefe`
- receipt: official and complete
- package SHA-256: `4b9f44f66c18235f80d33738d15f3418c65c98e68e254aa01d13fc3a66eb6ec8`
- receipt SHA-256: `084012de7549eb8742f8974e40f21407c7d70f5d8bee346c60384a878a0ccbf3`
- independent-verifier record SHA-256: `143b628e4bcd59a77df94c33aa085d4d4144addb403275e89eff1c3097fd260b`
- independent verification: success, 11/11 checks passed, zero findings
- raw shape: 4,194,304 cell rows, 1,024 chunk rows, 768 tick rows, 129 aggregate rows
- reference calibration: 865.304 TPS P50 / 1.155663 ms per tick P50; profiled envelope 1.018080 ms P50

These figures establish the G8-A technical evidence candidate only. They do not establish G8-B/G8-C, product validation, or same-SHA user visual approval.

## Protected Original Checkout

The dirty checkout `C:\Users\mdkap\source\repos\Powdergame` remained on `feature/m0-g5-pressure-field` at `c8fcb5e`. Its only observed dirty paths were preserved without reset, clean, stash, checkout, merge, or overwrite:

| Path | State | SHA-256 | Bytes |
|---|---|---|---:|
| `Cargo.lock` | modified | `58bff3b453136bb3a5ea97357d91a9b7567321d8a950e34ec4a5c4fb5d7a82ef` | 68,075 |
| `docs/planning/MATERIAL_CANDIDATES.md` | untracked | `792c3797fdea62333533ee8b21ce35b08ffc4a5018786b2b35d94cd008b94649` | 33,259 |
| `run.bat` | untracked | `d0a9522f9a68e132bcaab965a2b392ae7113e6ae1bffa9e49bda92a082dac73f` | 153 |

## Integration

The isolated worktree `C:\Users\mdkap\source\repos\Powdergame-canonical-recovery` was created from `9abec9e`. A no-fast-forward merge of `ccd0d7b` produced:

- merge commit: `e5871bdc53093700c44562826860c4d482f31ba5`
- message: `merge: integrate foundation material wiki with verified runtime line`
- parents: `9abec9e` and `ccd0d7b`
- conflict: only `docs/README.md`, with two expected hunks
- resolution: preserved implementation-line planning, development, evidence, and adversarial-review content while adding the Foundation research hierarchy and authority ordering

At the merge commit, every non-document path, including Cargo, apps, engine, WGSL, and tests, is byte-identical to `9abec9e`; `docs/research` is byte-identical to `ccd0d7b`. Reconciliation then changed documentation only, including normalization of two P1 frontmatter movement values from descriptive aliases to the canonical `LIQUID` and `STATIC` enums.

## PRE/POST Validation

The same required set was run before and after integration and reconciliation:

| Check | PRE | POST |
|---|---|---|
| formatting check | PASS | PASS |
| locked workspace tests, single-threaded | PASS — 385 passed, 3 expected ignored, 0 failed | PASS — 385 passed, 3 expected ignored, 0 failed |
| locked workspace clippy, all targets, warnings denied | PASS | PASS |
| evidence capture self-test | PASS — 12 invariants | PASS — 12 invariants |
| independent-verifier fixtures | PASS — 10 cases | PASS — 10 cases |
| Windows release smoke, 60 frames | PASS — RTX 5090 / DX12, clean exit | PASS — RTX 5090 / DX12, clean exit |
| whitespace/error-marker check | PASS | PASS |

The workspace test run exercised WGSL, headless GPU, world integrity, movement, density, thermal, phase, combustion, pressure, rupture, sleep/wake, profiler, and G8 benchmark coverage. The three ignored tests are the expected two manual performance cases and one long-run case. No broad demo matrix, ignored performance benchmark, or new official capture was run.

## Documentation Validation

- Foundation pages: 16/16 present and structurally valid.
- P1 pages: 9/9 present and structurally valid.
- Material Wiki index membership, reciprocal links, 507 relative targets, and 8 local fragments resolve. Its 166 frontmatter source references split into 152 resolving local paths and 14 external HTTP URLs.
- frontmatter status, implementation-state, movement, and ID checks pass; canonical IDs are unique.
- the nine runtime-registered identities remain distinct from candidate/not-registered research pages.
- no Fire or EMPTY Material page was introduced.
- `git diff --check` passes for the reconciled working tree.

## Completion Boundary

The merge remains separate from this documentation reconciliation, which is intended for the commit subject `docs: reconcile canonical documentation after branch recovery`. The source branches, protected dirty checkout, personal Wiki, and Draft PR #1 remain unchanged. Publishing this branch, opening a recovery pull request, promoting `main`, performing same-SHA user visual validation, or choosing G8-B/G8-C, G9, or post-M0 P1 work requires a separate user decision.
