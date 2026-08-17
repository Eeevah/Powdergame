# Powdergame Validation Policy

이 문서는 **변경 영향에 맞는 최소 충분 검증**을 정의한다.

핵심 원칙:

> Gate 이름 때문에 FULL을 실행하지 않는다. 실제 변경 경계가 넓은 회귀 위험을 만들 때만 FULL을 실행한다.

> 같은 source SHA에서 이미 성공한 동일 검증은 다시 실행하지 않고 provenance가 있는 결과로 재사용한다.

`docs/development/TESTING.md`는 무엇을 검증하는지 설명하고, 이 문서는 **언제 어느 검증 층을 실행하는지** 설명한다. 기계 판독 규칙은 `config/development-policy.json`, 개발자 진입점은 `tools/dev.ps1 validation-plan`이다.

---

## 1. Validation roles

각 층은 서로 다른 질문에 답한다.

| 층 | 담당하는 증거 | 대체하지 못하는 것 |
|---|---|---|
| Targeted test | 변경한 정상·오류 경로가 계약을 지키는가 | 앱 startup, 사용자 관찰 |
| FULL workspace test | Engine/Core/WGSL/Layout/shared runtime 회귀가 없는가 | 사용자 승인, 공식 성능 |
| Minimal app smoke | 정식 EXE가 시작되고 renderer/mode가 로드되고 정상 종료하는가 | 장기 lifecycle correctness |
| Candidate | 실제 production simulation과 telemetry가 승인 계약을 만족하는가 | map 실패·CLI 오류 등 비정상 경로 |
| Official evidence capture | source/binary/run/artifact가 forensic provenance로 봉인됐는가 | 제품 재미와 사용자 승인 |
| User validation | 결과가 이해 가능하고 다음 실험 욕구를 만드는가 | 자동 invariant 증명 |

Candidate가 강하더라도 targeted error-path test를 생략하지 않는다. FULL이 통과해도 candidate나 사용자 승인을 대신하지 않는다.

---

## 2. Change-impact matrix

### A. Docs-only

예:

- `docs/**`
- 상태·handoff·evidence 설명
- source SHA를 바꾸지 않는 closure 문서

필수:

```text
문서 링크·stale phrase 확인
tools/dev.ps1 audit
git diff --check
```

금지:

```text
Rust/GPU FULL
app smoke
candidate 재실행
```

기록:

```text
tested_source_sha = 실제 검증된 부모 source
docs_closure_sha  = 문서 commit
```

두 SHA를 동일한 의미로 쓰지 않는다.

### B. Harness / coordinator / CLI-only

예:

- `apps/windows/src/experiment/**`
- `tools/experiment/**`
- experiment argument parser
- report/contact-sheet/receipt coordinator

필수:

```text
fmt/check
직접 관련 Rust/Python targeted tests
필요한 오류 경로 test
minimal bounded app smoke
candidate가 작업 산출물일 때 candidate 정확히 1회
```

기본값:

```text
workspace FULL = NOT REQUIRED
```

다만 변경이 app-wide readback, source/binary provenance, publication state machine으로 번지면 C 등급으로 올린다.

### C. Renderer / app / observatory / readback / provenance

예:

- `apps/windows/src/main.rs`
- `renderer.rs`
- `observatory.rs`
- `gallery.rs`
- `apps/windows/build.rs`
- official capture/verifier 경계

필수:

```text
영향받는 앱 모드 targeted tests
map/poll/reset/CLI 실패 경로 tests
minimal bounded app smoke
clippy + diff-check
```

FULL은 **자동 필수값이 아니라 권장값**이다. 다음 중 하나면 FULL로 올린다.

- 여러 app mode가 공유하는 runtime state machine 변경
- 기존 GPU buffer 재사용·reset ordering 변경
- source/binary sealing이 candidate 전체에 영향을 줌
- 영향 범위를 targeted tests로 닫을 수 없다고 판단

### D. Fixture-only

예:

- geometry
- initial Matter/Field 배치
- scenario-specific observation mask
- production physics를 건드리지 않는 candidate fixture

필수:

```text
fixture pin
shared reset/staging targeted test
bounded scenario GPU test
candidate 정확히 1회
```

기본값:

```text
workspace FULL = NOT REQUIRED
```

shared reset/staging infrastructure 자체를 수정하면 E 등급으로 올린다.

### E. Engine / Core / WGSL / State layout / Cargo / shared reset

예:

- `engine/core/**`
- `engine/gpu/**`
- WGSL
- world buffer/layout
- production pass graph
- Material descriptor contract
- `Cargo.toml`, `Cargo.lock`
- shared reset/staging semantics

필수:

```text
변경 관련 targeted tests
workspace FULL 정확히 1회
minimal app smoke
candidate가 영향을 받으면 candidate 1회
clippy + diff-check
```

이 등급만이 기본적으로 FULL을 요구한다.

### F. Official evidence tooling

예:

- official capture
- receipt/hash/inventory
- independent verifier
- source/binary snapshot

필수:

```text
capture self-tests
failure-injection tests
verifier fixtures
source/binary/provenance checks
```

Engine FULL은 실제 Engine source가 바뀌었을 때만 실행한다.

---

## 3. Same-SHA validation reuse

성공 결과는 다음 키로 식별한다.

```text
source SHA
toolchain
build profile
command
relevant config
hardware/backend when applicable
```

키가 같으면 동일 검증을 반복하지 않는다.

다시 실행할 수 있는 조건:

- source bytes가 바뀜
- test/capture implementation이 바뀜
- 이전 결과가 incomplete/failed
- 환경 차이가 주장에 영향을 줌
- 사용자가 명시적으로 재실행 요청
- flake 조사에서 반복 횟수가 사전에 정해짐

문서 수정, 보고서 문장 수정, branch pointer 이동만으로 FULL이나 candidate를 다시 실행하지 않는다.

---

## 4. Source seal and requirement freeze

최종 검증 전에 요구사항을 동결한다.

```text
scope freeze
→ targeted tests
→ clippy/diff-check
→ clean source seal
→ 필요한 FULL 1회
→ minimal smoke 1회
→ candidate 1회
```

FULL이 시작된 뒤 새 요구가 도착하면:

- 비차단 follow-up은 pending 목록에 기록하고 현재 checkpoint를 끝낸다.
- 현재 결과를 무효화하는 correctness/security/evidence blocker만 실행 중 FULL을 interrupted로 기록한다.
- blocker를 모두 수정한 뒤 새 clean SHA에서 최종 FULL을 딱 한 번 실행한다.
- 중단된 FULL은 PASS로 계산하지 않는다.

---

## 5. Minimal smoke contract

Smoke는 다음만 확인한다.

```text
strict argument parsing
정식 binary startup
GPU/renderer initialization
요청 mode/fixture load
bounded 몇 frame 또는 몇 tick
정상 exit code
```

Smoke에서 장시간 lifecycle을 다시 실행하지 않는다. 잘못된 bounded 인자는 대화형 실행으로 fallback하지 않고 nonzero로 실패해야 한다.

---

## 6. Cost guardrails

다음은 최종 보고에 반드시 남긴다.

- FULL 실행 횟수
- candidate 실행 횟수
- 가장 긴 command 5개
- source seal 이후 source 변경 횟수
- interrupted validation
- 동일 SHA 재실행 여부와 이유
- prompt-to-verified-candidate wall time
- first-pass user acceptance와 rework 횟수

한 명령이 60초 이상이거나 한 단계가 전체 시간의 30% 이상이면 병목 후보로 기록한다.

2026-08-17 audit에서는 warm build가 아니라 `cargo test --workspace -- --test-threads=1` 실행이 약 310.84초로 확인됐다. 따라서 최우선 개선은 compiler 변경이 아니라 **불필요한 FULL을 제거하고 최종 위험 경계에서 한 번만 실행하는 것**이다.

---

## 7. Tooling

```powershell
pwsh -NoProfile -File tools/dev.ps1 validation-plan -BaseRef <base>
pwsh -NoProfile -File tools/dev.ps1 audit
```

`validation-plan`은 변경 경로에 따라 필요한 검증 층과 명령을 제안한다. 이는 최종 판단 보조 도구이며, unknown path나 여러 경계가 섞인 변경은 더 높은 위험 등급으로 올린다.

`cargo-nextest`, linker, sccache, profile 변경은 현재 정책의 일부가 아니다. 동일 clean SHA에서 기존 runner 대비 시간·flake·test inventory를 측정한 pilot이 성공한 뒤에만 채택한다.
