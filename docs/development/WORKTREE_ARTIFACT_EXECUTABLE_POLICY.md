# Powdergame Worktree, Artifact, and Executable Policy

Date: 2026-08-17

이 문서는 Powdergame의 worktree, launcher, executable, artifact 운영 정본이다.
기계 판독 정본은 `config/development-policy.json`, audit 진입점은
`tools/dev.ps1 audit`이다. 이 정책은 Gate 결과를 선언하지 않는다.

---

## 1. Worktree WIP limit

동시에 유지하는 기본 상한:

```text
canonical worktree        1
active feature worktree   1
temporary review worktree 1
```

새 Scenario나 문서 작업마다 worktree를 만들지 않는다. 기존 active feature
worktree를 계속 사용한다. 새 worktree가 필요하면 구현 전에 다음을 기록한다.

- 기존 worktree로 안전하게 처리할 수 없는 이유
- 소유 branch
- 예상 수명
- 제거 조건

종료된 worktree는 다음 순서로 retire한다.

```text
clean 확인
→ push/upstream 확인
→ 필요한 artifact 보존
→ cargo clean
→ git worktree remove
→ 오래된 release EXE 부재 확인
```

Active worktree의 target cache는 유지한다. Active worktree에는 이 retire 절차를
적용하지 않으며, task가 `cargo clean` 또는 worktree 제거를 금지하면 그 지시가
우선한다.

---

## 2. Single app executable

사용자가 찾고 실행하는 정식 앱 바이너리는 하나다. 현재 active feature
worktree의 canonical path는 다음과 같다.

```text
C:\Users\mdkap\source\repos\Powdergame-g8b\target\release\powdergame-windows.exe
```

일반형 계약은 `<active-worktree>\target\release\powdergame-windows.exe`다. 새
Scenario, Gate, Observatory, Gallery는 같은 EXE의 menu, CLI argument, 또는
mode로 추가한다. 별도 사용자용 EXE를 만들지 않는다.

Developer-only 예외:

```text
target\release\powdergame-benchmark.exe
```

이는 headless G8 성능 측정이라는 독립 역할이 실제로 필요한 동안만 유지한다.
다른 폴더로 복사하거나 사용자 앱처럼 게시하지 않는다. Cargo가
`target/**/deps`에 만드는 test executable은 내부 build cache이며 사용자용
실행파일이 아니다.

새 executable이 필요하면 구현 전에 다음을 문서화한다.

- 기존 EXE로 처리할 수 없는 이유
- 두 실행파일을 동시에 유지해야 하는 이유
- 소유 기능과 owner
- 보존 기한
- 제거 조건
- audit allowlist 항목

---

## 3. Launchers

정식 root entrypoint는 두 개다.

```text
run_powdergame.bat
run_experiment.bat
```

- `run_powdergame.bat`: 일반 앱, 기존 편의 mode 이름, 또는 raw app CLI 인자
- `run_experiment.bat`: Python coordinator가 필요한 자동 experiment

예:

```bat
run_powdergame.bat
run_powdergame.bat pressure
run_powdergame.bat gallery
run_powdergame.bat --benchmark-gallery --smoke-frames 3
```

Gate별 `run_g*.bat`은 새로 만들지 않는다. `run_g5_demo.bat`은
`run_powdergame.bat pressure`로 교체되어 제거되었다. 현재 migration debt는
다음 두 legacy wrapper뿐이다.

- `run_g7_activity_demo.bat` → `run_powdergame.bat --activity-demo`
- `run_g8_benchmark_gallery.bat` → `run_powdergame.bat --benchmark-gallery`

이들은 `config/development-policy.json`에 명시하며 G8-B closure까지 canonical
launcher로 교체한 뒤 제거한다. 새로운 legacy 항목을 추가하면 audit 실패다.

---

## 4. Binary copies and evidence exception

기본적으로 checkpoint, evidence, archive 폴더에 EXE 복사본을 보존하지 않는다.
대신 source SHA와 binary SHA-256을 기록한다. 다음은 금지한다.

- 사용자용 EXE를 checkpoint/archive/evidence 폴더에 복사
- 다른 worktree의 EXE를 canonical app처럼 실행
- 이름만 바꾼 EXE 보존

증거 계약이 binary 포함을 명시적으로 요구하면 create-new frozen copy를 해당
Run 또는 Audit Bundle 안에 만들 수 있다. 이 복사본은 사용자 설치나 canonical
app이 아니다. 반드시 source SHA, binary SHA-256, Run ID, 보존 이유와 제거
조건을 기록한다.

Post-remediation Experiment Harness seal은 이 예외를 사용한다. Scratch를 포함한
각 새 Harness Run은 unique Run 내부의 create-new frozen EXE를 실행한다.
Candidate mode는 같은 bytes를 sibling Audit Bundle에도 포함하지만 scratch는
Audit Bundle을 만들지 않는다. Frozen copy는 immutable Run과 함께 보존하고,
명시적인 artifact-retention 결정으로 전체 Run을 retire할 때만 제거한다. 사용자
앱처럼 게시하거나 별도 복사하지 않는다.

---

## 5. Artifact root and retention

Generated artifact는 Git 밖에 둔다.

```text
C:\Users\mdkap\source\Powdergame-artifacts
```

| 등급 | 정책 |
|---|---|
| scratch | Scenario당 최근 3개 또는 7일; 명시된 retention 절차로 정리 가능 |
| candidate | immutable, no overwrite |
| accepted | immutable, 자동 삭제 금지 |
| rejected | 감사 이력, 자동 삭제 금지 |
| development session | raw timing; Gate/세션 집계 후 retention 적용 |

Run ID와 output path를 재사용하지 않는다.

---

## 6. Audit

```powershell
pwsh -NoProfile -File tools/dev.ps1 audit
```

Audit은 다음을 검사한다.

- root launcher allowlist와 새 `run_g*` 변형
- Git에 commit된 EXE
- required policy files
- worktree count
- canonical launcher
- legacy migration debt
- artifact root가 repository 밖인지 여부

CI에서는 repository 구조를 검사하고 local-only worktree/artifact 크기는 보고하지
않는다. Declared migration debt는 warning이며 새 위반은 failure다.

---

## 7. Exception and completion reporting

새 executable, launcher, worktree, artifact namespace 예외는 기존 경로로 처리할
수 없는 이유, 동시 보존 필요성, owner, 제거 date/Gate, audit allowlist를 먼저
기록한다. 기한 없는 예외는 허용하지 않는다.

작업 종료 보고에는 다음을 포함한다.

- canonical app binary path
- 사용자용 app binary 복사본 수
- 유지 중인 developer-only executable
- 생성하거나 제거한 launcher
- executable-copy 예외의 이유와 제거/보존 조건
- worktree와 branch/upstream 상태
