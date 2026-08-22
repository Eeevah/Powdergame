# Powdergame Development Learning Loop

이 문서는 개발 중 발생한 시행착오를 **다음 작업의 자동 이득**으로 바꾸는 절차를 정의한다.

새 방법론을 별도로 설치하는 것이 목적이 아니다. Powdergame의 기존 Gate·evidence·SHA·사용자 승인 체계 위에 학습 루프를 추가한다.

```text
Observe
→ Classify
→ Fix one
→ Verify
→ Promote
→ Guard
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
- named fixture가 실제로 실행한 경로
- proof attempt/completion과 중단 위치
- 화면의 주장과 실제 staged state의 차이

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
| substitute illusion | 비슷한 증거를 진짜 계약 대신 사용 | fixture 이름이나 reduced model PASS를 production path 증거로 확대 |
| scope interruption | final validation 도중 요구가 바뀜 | FULL 중 새 blocker 도착 |
| cost blindness | 안전을 이유로 모든 변경을 최고 등급 검증 | harness-only 변경에 전체 GPU FULL |
| review-surface drift | 사용자 화면이 자신이 주장하는 장면을 실제로 staging하지 않음 | TE-2/TE-3 candidate label과 실제 state 불일치 |
| temporal-ontology mismatch | Tick을 가로질러 소비되는 자원을 한 Tick의 claim으로 표현 | TE-5B volume-relief token |
| local-completeness illusion | 국소 heuristic을 전역 feasible solution의 증명처럼 사용 | TE-5C proportional share, TE-5D fixed-depth matching |
| hidden-truth gap | authoritative state가 UI에 없어 올바른 물리가 고장처럼 보임 | 100°C latent plateau에서 phase energy 비표시 |
| delivery-path gap | artifact는 있으나 사용자가 실제로 여는 진입점이 없음 | candidate EXE는 있으나 CLI argument 필요 |

분류는 비난이 아니라 다음 guard 위치를 정하기 위한 것이다.

---

## 3. Root-cause record

승격 후보는 최소한 다음을 분리한다.

```text
Observation:
실제로 무엇이 일어났는가

User / engineering loss:
잘못된 판단, 시간 손실, 재검증, 제품 혼란, 안전 위험

Proximate cause:
직접적인 기술 원인

Systemic cause:
설계·프롬프트·검증·문서 구조가 왜 미리 막지 못했는가

Failed guard:
있었지만 불충분했던 테스트·정책·리뷰

Adopted rule:
다음 작업에서 적용할 짧고 검증 가능한 규칙

Machine guard:
test / audit / schema / prompt contract / NOT_ESTABLISHED state

Scope:
Powdergame-only / GPU simulation / evidence workflow / all projects

Evidence:
source SHA / receipt / screenshot / counterexample / review

Supersedes:
대체하는 lesson ID가 있으면 명시
```

“코덱스가 실수했다”, “복잡해서 놓쳤다”는 root cause가 아니다. 왜 현재 시스템이 그 실수를 허용했는지까지 적는다.

---

## 4. Fix one

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

## 5. Verify

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
- named fixture가 실제로 실행한 branch/path count
- proof `attempts`, `completions`, `NOT_ESTABLISHED`
- candidate label과 staged state/metric/checkpoint 일치

속도만 빨라지고 검증 누락이나 재작업이 늘면 개선이 아니다.

---

## 6. Lesson Promotion Gate

모든 substantial task는 최종 보고 전에 다음 중 하나로 분류한다.

```text
LESSON_PROMOTION:
REQUIRED
PROJECT_ONLY
NONE — <reason>
```

### REQUIRED

다음 중 하나면 필수다.

- 사용자가 직접 defect·혼란·missing entrypoint·hidden state를 발견
- architecture/proof가 반례로 차단
- named evidence가 실제 claim을 실행하지 않은 것으로 판명
- one-shot evidence가 중단 또는 무효화
- 같은 유형의 rework가 반복
- 약 15분 이상 avoidable loss
- source/provenance/artifact/user-trust 위험
- 사용자가 durable rule을 명시적으로 요구

### PROJECT_ONLY

Powdergame 구현 특유의 교훈이지만 다른 프로젝트에 일반화할 근거가 아직 없을 때 사용한다. `LESSONS_LEDGER.md`에는 승격하되 Wiki에는 요약하지 않는다.

### NONE

trivial edit, 일회성 typo, 아직 검증되지 않은 가설처럼 standing rule로 만들 가치가 없을 때 사용한다. 이유를 반드시 적는다.

---

## 7. Promote

승격 경로:

```text
observation
→ root-cause record
→ LESSONS_LEDGER entry
→ policy / validation contract
→ machine guard
→ personal-infra-wiki summary
→ 필요 시 Ballast managed agent rule / reusable skill
```

승격 기준:

- 같은 문제가 2회 이상 반복
- 한 번에 15분 이상 손실
- source/provenance/artifact 안전 위험
- 사용자 신뢰를 해칠 수 있는 잘못된 표시
- 사용자 장기 규칙
- architecture blocker가 다른 모델/프로젝트에도 재사용 가능한 반례를 제공

모든 관찰을 규칙으로 만들지 않는다. 반복 가능하고 검증된 교훈만 승격한다.

### Project ledger와 Wiki 역할

`docs/development/LESSONS_LEDGER.md`:

- Powdergame source/evidence에 직접 연결
- append-only
- 구체적인 guard와 source identity 보존

`personal-infra-wiki`:

- 여러 세션·프로젝트에 재사용 가능한 root cause와 workflow
- 왜 채택했는지
- 다른 프로젝트에 적용할 조건
- 검증된 troubleshooting
- canonical project ledger/source 링크

원시 session log, feature branch의 순간 상태, 아직 검증되지 않은 설계안은 Wiki에 복사하지 않는다.

### Feature branch에서의 Wiki 승격

제품 상태나 runtime acceptance는 canonical main 승격 전까지 Wiki에서 “현재 제품 truth”로 단정하지 않는다.

하지만 다음은 verified feature evidence에서 승격할 수 있다.

- 반복된 workflow failure
- immutable blocked-design counterexample
- evidence integrity rule
- user-confirmed operating policy

이때 Wiki는 source SHA와 상태를 명시하고, product state와 workflow lesson을 분리한다.

---

## 8. Dirty Wiki safe promotion

사용자 변경이 있는 local Wiki는 reset·stash·overwrite하지 않는다.

그러나 dirty checkout은 lesson promotion을 취소하는 이유가 아니다.

안전한 경로:

```text
verified origin/main
→ clean temporary worktree 또는 GitHub remote branch
→ relevant files only
→ Wiki validation
→ draft PR
→ 사용자 local changes와 독립적으로 review
```

원칙:

- local dirty 파일은 변경하지 않는다.
- 가능하면 사용자가 수정 중인 동일 파일을 피한다.
- unavoidable conflict는 PR에 명시한다.
- branch/PR이 source of truth가 되는 것은 merge 뒤다.
- final report에 Wiki base SHA, branch, PR, untouched local paths를 기록한다.

상세 절차는 personal-infra-wiki의 `wiki/troubleshooting/wiki-dirty-worktree-preservation.md`를 따른다.

---

## 9. Guard

교훈은 가능한 한 실행 가능한 guard로 끝난다.

예:

- fixture coverage manifest와 required path counter
- `NOT_ESTABLISHED` status
- evidence attempt/completion 분리
- exact branch fetch guard
- source-bound receipt
- candidate `label → staged state → visible metric → checkpoint` test
- mode-aware Inspector profile
- one-click entrypoint audit
- validation-plan rule
- CI/audit check

Guard를 만들 수 없다면 그 이유와 human review obligation을 남긴다.

---

## 10. Sweep

정본이 바뀌면 관련 표현을 검색한다.

- superseded SHA
- `pending`, `planned`, `not accepted`
- 삭제 예정 launcher
- 오래된 worktree 경로
- 이전 canonical EXE
- obsolete validation chain
- rejected proof를 PASS처럼 인용한 문구
- hidden state를 observable state처럼 설명한 문구

과거 증거는 삭제하지 않고 `superseded`, `historical`, `blocked`, `NOT_ESTABLISHED`로 남긴다. 현재 정본과 혼동되는 문구만 제거한다.

---

## 11. Retire

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

## 12. Append-only lessons ledger

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

## 13. Session timing

```powershell
pwsh -NoProfile -File tools/dev.ps1 session-start -Task fire-heat
pwsh -NoProfile -File tools/dev.ps1 measure -SessionId <id> -Category FAST cargo check --workspace --all-targets
pwsh -NoProfile -File tools/dev.ps1 session-span -SessionId <id> -Name implementation -DurationSeconds 600
pwsh -NoProfile -File tools/dev.ps1 session-end -SessionId <id>
```

`session-start` 시각은 agent가 첫 읽기·명령 전에 실행했을 때만 prompt 시작의 observed proxy다. 알 수 없는 시간을 추정해서 채우지 않는다.

---

## 14. Final report contract

Substantial task의 final report에는 다음을 포함한다.

```text
LESSON_PROMOTION:
<REQUIRED / PROJECT_ONLY / NONE>

Promoted:
<ledger IDs / Wiki pages / guard>

Not promoted:
<reason>

Wiki:
<base SHA / branch or PR / local dirty paths untouched>
```

docs-only promotion은 runtime evidence를 무효화하지 않으며 그 자체로 FULL·build·launch를 요구하지 않는다.
