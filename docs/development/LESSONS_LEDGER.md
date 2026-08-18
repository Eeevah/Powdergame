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
| PG-L009 | 2026-08-17 | adopted | 첫 policy audit 구현에서 PowerShell 함수명이 `Git` 실행파일을 가렸고, 다음 수정에서는 자동 변수 `$Args`와 같은 이름을 써 Git argument가 비어 두 차례 CI가 실패했다. | 외부 명령 wrapper는 command 이름·PowerShell 자동 변수와 겹치지 않게 하고, 실행파일을 하나 선택한 뒤 named array parameter로 전달한다. 새 guard는 실제 CI에서 PASS하기 전 채택하지 않는다. | `tools/dev.ps1`의 `Invoke-RepoGit -GitArgs`, GitHub Actions `Development policy audit` | failed runs `32036149706`, `32036356485`, `32036736080`; passing run `32036953896` |
| PG-L010 | 2026-08-18 | adopted | Contact Sheet가 milestone 종류 순서로 생성되면 실제 simulation 시간축이 뒤섞이고 같은 tick의 상태가 중복 타일로 보일 수 있었다. | Frame은 `sim_tick` 오름차순으로 정렬하고, 같은 tick과 state hash의 milestone은 가능한 한 한 frame의 badge로 접으며, reset은 항상 마지막에 둔다. | Contact Sheet chronological ordering / same-tick badge folding / reset-last deterministic ordering test | Fire / Heat candidate review follow-up |
| PG-L011 | 2026-08-18 | adopted | 원본 Run `HASHES.sha256`와 Audit Bundle 내부 경로가 다르고 canonical Git archive의 EOL 표현이 실제 build-input bytes와 다를 수 있어 bundle 하나만으로 forensic verification 경계가 모호했다. | `SOURCE_INPUT_MANIFEST.json`이 해시한 exact build-input bytes와 canonical Git archive를 구분하고, Audit Bundle 자체 inclusion/path/omission contract를 별도로 기록한다. | Bundle-local manifest/path mapping/hash tests와 exact source-input-bytes archive byte-identity test | Fire / Heat Audit Bundle review follow-up |
| PG-L012 | 2026-08-18 | adopted | Pressure through opening milestone은 검출됐지만 같은 결과를 Wood combustion이 만들 수 있었고, complete opening tick이 900-active-tick burn duration과 맞물렸다. | Scenario acceptance는 결과 milestone만이 아니라 이름을 붙인 causal chain을 증명하고 같은 결과를 만드는 다른 subsystem을 배제해야 한다. | Subsystem-specific confound telemetry, causal predicates, and synthetic confound regressions | Pressure run `g8b-pressure-burst-v0-20260818T014452058676Z-353fb706` human review |
| PG-L013 | 2026-08-18 | adopted | UTC timestamp를 local/KST로 다시 해석해 task wall time이 약 9시간 부풀었다. | Duration은 RFC3339 UTC 또는 monotonic counter로만 계산하고, timezone은 사람용 표시에만 쓴다. | Fixed-clock, UTC/KST double-offset, cross-process, and UTC-versus-monotonic consistency tests in `tools/test-dev-timer.ps1` | Timer fix source `328757bd8cd5b07cd0ed4c66a592f973dcd66981` |
| PG-L014 | 2026-08-18 | adopted | 선행 task의 wall `6,473` 초 중 command 기록은 `126` 초에 불과해 command timing만으로 병목과 작업 비용을 설명할 수 없었다. | Command timing과 함께 phase union, wall 대비 비율, 그 둘로 설명되지 않는 unclassified time을 출판한다. | Session summary command/phase union, ratios, unclassified bucket, and overlap regressions | Timer fix source `328757bd8cd5b07cd0ed4c66a592f973dcd66981` |
| PG-L015 | 2026-08-18 | adopted | 명시적 `--benchmark-gallery` smoke는 통과했지만 사용자가 실제로 실행하는 no-argument canonical path는 G0 empty clear 화면을 열었다. Repository 밖 convenience launcher는 canonical audit 범위에서 빠질 수도 있었다. | User-facing launcher acceptance는 exact no-argument entrypoint를 실행해 검증하고 technical baseline은 explicit diagnostic flag로만 제공한다. 외부 convenience launcher는 최종 migration 전까지 내용을 inventory한다. | No-argument mode routing unit test, no-argument bounded launcher smoke with Gallery staging/log assertion, explicit runtime-baseline regression, and `tools/dev.ps1 audit` launcher contract | Pressure Burst acceptance launcher follow-up |

## Entry template

새 항목은 아래 형식으로 추가한다.

```markdown
| PG-L### | YYYY-MM-DD | proposed/adopted/superseded |
Observation and measured loss/risk |
Adopted rule |
Machine guard or durable document |
Evidence IDs / SHAs |
```
