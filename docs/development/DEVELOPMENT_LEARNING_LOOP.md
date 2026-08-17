# Powdergame Development Learning Loop

이 문서는 개발 중 발생한 시행착오를 **다음 작업의 자동 이득**으로 바꾸는 절차를 정의한다.

새 방법론을 별도로 설치하는 것이 목적이 아니다. Powdergame의 기존 Gate·evidence·SHA·사용자 승인 체계 위에 학습 루프를 추가한다.

```text
Observe
→ Classify
→ Fix one
→ Verify
→ Promote
→ Sweep
→ Retire
```

---

## 1. Observe

원시 사실을 먼저 남긴다.

- prompt 접수부터 verified candidate까지 wall time
- phase별 시간
- command별 시간과 exit code
- failed command
- rework loop
- FULL/candidate 횟수
- worktree/target/artifact 증감
- 사용자 거절 또는 추가 관찰
- provenance 누락
- 새 launcher/EXE/folder 생성

원시 로그는 Git에 넣지 않는다.

```text
C:\Users\mdkap\source\Powdergame-artifacts\
  development-sessions\
    <session-id>\
```

---

## 2. Classify

반복 실수는 다음 중 하나로 분류한다.

| 유형 | 뜻 | Powdergame 예 |
|---|---|---|
| default regression | 예전 습관이 다시 기본값이 됨 | 새 Gate마다 `run_g*.bat` 추가 |
| propagation miss | 한 정본만 바뀌고 관련 문서가 낡음 | STATUS는 승인, HANDOFF는 pending |
| delegation leak | 하위 작업에 핵심 계약이 전달되지 않음 | subtask가 별도 worktree/EXE 생성 |
| variant evasion | 다른 확장자·경로로 같은 위반이 돌아옴 | `.bat` 대신 `.cmd` launcher 증식 |
| compression loss | 요약 과정에서 중요한 제한이 빠짐 | docs-only closure에 FULL 금지 누락 |
| substitute illusion | 비슷한 증거를 진짜 계약 대신 사용 | candidate PASS를 error-path test로 대체 |
| scope interruption | final validation 도중 요구가 바뀜 | FULL 중 새 blocker 도착 |
| cost blindness | 안전을 이유로 모든 변경을 최고 등급 검증 | harness-only 변경에 전체 GPU FULL |

분류는 비난이 아니라 다음 guard 위치를 정하기 위한 것이다.

---

## 3. Fix one

한 번에 병목 하나만 바꾼다.

좋은 변경:

```text
targeted validation 적용
→ 동일 조건에서 시간 감소 확인
```

나쁜 변경:

```text
nextest + linker + sccache + test 통합 + profile 변경
→ 무엇이 효과였는지 알 수 없음
```

---

## 4. Verify

개선은 느낌으로 채택하지 않는다.

최소 비교 항목:

- wall time
- test inventory
- pass/fail/ignored
- flake
- first-pass user acceptance
- rework loops
- provenance completeness
- policy violation count

속도만 빨라지고 검증 누락이나 재작업이 늘면 개선이 아니다.

---

## 5. Promote

다음 중 하나면 세션 메모를 standing rule로 승격한다.

- 같은 문제가 2회 이상 반복
- 한 번에 15분 이상 손실
- source/provenance/artifact 안전 위험
- 사용자가 장기 규칙으로 명시

승격 단계:

```text
observation
→ LESSONS_LEDGER entry
→ policy 문서
→ config/development-policy.json
→ tools/dev.ps1 audit/validation-plan
→ 필요 시 CI guard
```

모든 관찰을 규칙으로 만들지 않는다. 반복 가능하고 검증된 교훈만 승격한다.

---

## 6. Sweep

정본이 바뀌면 관련 표현을 검색한다.

- superseded SHA
- `pending`, `planned`, `not accepted`
- 삭제 예정 launcher
- 오래된 worktree 경로
- 이전 canonical EXE
- obsolete validation chain

과거 증거는 삭제하지 않고 `superseded` 또는 `historical`로 남긴다. 현재 정본과 혼동되는 문구만 제거한다.

---

## 7. Retire

임시 도구는 완료 조건과 함께 만든다.

- worktree
- branch
- launcher wrapper
- scratch artifact
- compatibility alias
- rejected candidate unpacked copy
- temporary validation exception

종료 조건을 만족하면 실제로 제거한다. “나중에 정리”는 완료 상태가 아니다.

---

## 8. Append-only lessons ledger

`docs/development/LESSONS_LEDGER.md`는 승격된 교훈만 기록한다.

각 항목:

```text
ID
Date
Observation
Loss/Risk
Adopted rule
Machine guard
Evidence
Status
Supersedes / Superseded by
```

과거 결론이 바뀌면 행을 수정해 지우지 않고 새 항목을 추가하여 supersede 관계를 남긴다.

---

## 9. Session timing

```powershell
pwsh -NoProfile -File tools/dev.ps1 session-start -Task fire-heat
pwsh -NoProfile -File tools/dev.ps1 measure -SessionId <id> -Category FAST cargo check --workspace --all-targets
pwsh -NoProfile -File tools/dev.ps1 session-span -SessionId <id> -Name implementation -DurationSeconds 600
pwsh -NoProfile -File tools/dev.ps1 session-end -SessionId <id>
```

`session-start` 시각은 agent가 첫 읽기·명령 전에 실행했을 때만 prompt 시작의 observed proxy다. 알 수 없는 시간을 추정해서 채우지 않는다.

---

## 10. Wiki promotion

Powdergame repo가 세부 정책의 source of truth다.

`personal-infra-wiki`에는 canonical main 승격 뒤 다음만 요약한다.

- 왜 채택했는지
- 다른 프로젝트에도 재사용 가능한 workflow
- 검증된 troubleshooting
- 정본 문서와 source SHA

원시 session log, 임시 experiment, feature branch의 순간 상태는 Wiki에 복사하지 않는다.
