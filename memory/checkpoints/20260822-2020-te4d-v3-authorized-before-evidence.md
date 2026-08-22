# Checkpoint — TE-4D v3 transaction/oracle closure authorized — 2026-08-22

## Repository coordinate
- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Start SHA: `85745533c1cb9a9505826f5aaba5dd1ba085b627`
- D-030 authorization SHA: `aae47f8b0afab0c0521d5bf476229446bb5bd3ce`
- Wiki authority: `origin/main` `9ed82115cf75b73aee034107008ea8cf83ed23af`; dirty local checkout preserved read-only
- Runtime: unchanged / not started

## The story so far
TE-4D v1/v2 remain immutable blocked history. D-030 authorizes a distinct v3
identity that fixes combustion-stage snapshot semantics, mutation-derived
transaction receipts and a frozen independent complete F07/F08 oracle.

## Valid evidence
- V1 script/failure SHA-256: `886fe5b7...f82` / `6342bad5...c1bb`; attempts/completions `1/0`; model work `0/0/0`.
- V2 manifest/script/result-file SHA-256: `9b763c1c...53ba` / `c01e2869...a769` / `24ebd797...f151`.
- V2 scoped result payload SHA-256: `717f4ef7...132c`; attempts/completions `1/1`; 100,000 sequences; 10,000 grids.
- Frozen process reported 13 reference PASS and four expected production `NOT_ESTABLISHED`; it was not rerun.

## Decided
- `COMBUSTION_STAGE_SNAPSHOT` is the exact Air precondition lifetime.
- Same-stage self-Smoke does not roll back authorized work; the next snapshot extinguishes before emission.
- Required path counts must be audited from before/after state mutations.
- The F07/F08 oracle must be independently generated and frozen before evidence.
- Exact coefficients are selected identities; optimality is not claimed.

## Waiting on the user
The user must review the candidate only after the one completed v3 execution
and fresh independent review. Runtime remains separately unauthorized.

## Current authorization
Create/freeze v3 docs/reference artifacts, execute v3 exactly once, obtain a
fresh-context review, validate docs/reference/memory only, commit and push the
named feature branch. No runtime, Cargo, build, launch, PR or main merge.

## Blocker
- None yet for v3. Any failed campaign, oracle mismatch, zero audited required
  path, or unresolved Critical/High review finding blocks the design.

## Next first action
Generate and inspect the independent oracle before freezing the v3 manifest
and evidence script.

## Tried
- Wiki and Powdergame coordinates match the requested SHAs; dirty Wiki is untouched.
- Immutable v1/v2 hashes match and neither identity was executed.
- Live pass/binding audit supports 42/84/1,344 and post-Smoke Air visibility.
- No Cargo, GPU, FULL, build or application launch was run.
