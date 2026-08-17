# Powdergame Lessons Ledger

이 문서는 **승격된 교훈만** 보존하는 append-only 원장이다. 기존 항목의 의미를 조용히 바꾸지 않는다. 결론이 바뀌면 새 ID를 추가하고 `Supersedes`를 기록한다.

| ID | Date | Status | Observation / loss | Adopted rule | Machine guard / durable location | Evidence |
|---|---|---|---|---|---|---|
| PG-L001 | 2026-08-17 | adopted | Warm Rust build는 약 0.3초였지만 직렬 workspace test는 약 310.84초였다. “빌드가 느리다”는 표현이 실제 병목을 가렸다. | 변경 영향에 따라 FULL을 조건부로 실행하고 최종 위험 경계에서 한 번만 수행한다. | `VALIDATION_POLICY.md`, `config/development-policy.json`, `tools/dev.ps1 validation-plan` | `build-audit-20260817T1115497421092Z-5bb4e6fb`, source `5af031f1...` |
| PG-L002 | 2026-08-17 | adopted | Gate/worktree마다 Cargo `target`이 2–8GB씩 중복되어 약 45GB까지 증가했다. | active worktree cache는 유지하고 retired worktree만 `cargo clean → remove`한다. | `WORKTREE_ARTIFACT_EXECUTABLE_POLICY.md`, `tools/dev.ps1 audit` | 사용자 디스크 inventory, Build-Time Audit target 5.60GiB |
| PG-L003 | 2026-08-17 | adopted | 동일 앱을 여는 Gate별 BAT와 오래된 EXE 위치가 사용자를 혼란스럽게 했다. | 사용자용 app binary 1개, 일반 launcher 1개, experiment launcher 1개만 유지한다. | `config/development-policy.json`, repository audit; legacy wrappers는 G8-B closure까지 명시적 allowlist | 사용자 결정 |
| PG-L004 | 2026-08-17 | adopted | FULL 진행 중 새 요구가 들어와 이미 진행한 장시간 검증을 중단하고 새 SHA에서 다시 시작했다. | requirement freeze 뒤 source seal/FULL을 시작하고, 비차단 follow-up은 현재 checkpoint 뒤로 미룬다. | `VALIDATION_POLICY.md` source-seal contract | Fire/Heat 작업 로그 |
| PG-L005 | 2026-08-17 | adopted | docs-only closure 후에도 FULL을 다시 실행하려는 습관이 반복됐다. | tested source SHA와 docs closure SHA를 분리하고 docs-only 변경에는 Rust/GPU FULL을 실행하지 않는다. | validation class `docs-only` | Sand/Water docs closure workflow |
| PG-L006 | 2026-08-17 | adopted | Candidate, smoke, FULL이 서로 다른 증거 역할인데 같은 correctness 검사를 반복했다. | targeted error paths, minimal smoke, candidate, FULL의 역할을 분리하고 서로 대체하지 않는다. | `VALIDATION_POLICY.md` role table | G8-B Sand/Water/Fire review |
| PG-L007 | 2026-08-17 | adopted | 사용자가 매번 장기 실행을 기다리고 수동 screenshot/HUD transcription을 해야 했다. | Harness가 keyframe·telemetry·report를 만들고 사람은 `NEEDS_HUMAN_REVIEW` 장면만 추가 관찰한다. | Experiment Harness, Contact Sheet, Review Packet | Sand pilot / Water candidate |
| PG-L008 | 2026-08-17 | adopted | 자동 verdict나 Review Packet 하나를 더 넓은 Gate 승인 또는 forensic proof로 오해할 위험이 있었다. | 자동 판정, 사용자 승인, forensic audit bundle의 범위를 분리한다. | evidence contract와 review prompt boundary | Sand Review Prompt and adversarial review |

## Entry template

새 항목은 아래 형식으로 추가한다.

```markdown
| PG-L### | YYYY-MM-DD | proposed/adopted/superseded |
Observation and measured loss/risk |
Adopted rule |
Machine guard or durable document |
Evidence IDs / SHAs |
```
