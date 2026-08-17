# Powdergame Worktree, Artifact, and Executable Policy

이 문서는 개발 폴더·실행파일·launcher가 무한히 늘어나는 것을 막는 운영 계약이다. 기계 판독 정본은 `config/development-policy.json`, audit는 `tools/dev.ps1 audit`이다.

---

## 1. Worktree WIP limit

동시에 유지하는 기본 상한:

```text
canonical worktree        1
active feature worktree   1
temporary review worktree 1
```

새 Scenario나 문서 작업마다 worktree를 만들지 않는다. 기존 active feature worktree를 계속 사용한다.

새 worktree가 필요한 경우 구현 전에 기록한다.

- 기존 worktree로 안전하게 할 수 없는 이유
- 소유 branch
- 예상 수명
- 제거 조건

Retire 순서:

```text
clean 확인
→ upstream/push 확인
→ 필요한 artifact 보존
→ cargo clean
→ git worktree remove
```

active worktree의 target cache는 유지한다.

---

## 2. Single app executable

사용자가 찾고 실행하는 정식 앱 바이너리는 하나다.

```text
<active-worktree>\target\release\powdergame-windows.exe
```

새 Scenario, Observatory, Gallery는 같은 EXE의 mode/CLI/menu로 추가한다.

예외:

```text
powdergame-benchmark.exe
```

이는 headless 성능 측정이라는 독립 developer-only 역할이 있는 동안만 유지한다. 사용자 앱으로 게시하거나 다른 폴더에 복사하지 않는다.

Cargo가 `target/**/deps`에 만드는 test EXE는 build cache이며 사용자용 실행파일이 아니다.

---

## 3. Launchers

정식 root entrypoint:

```text
run_powdergame.bat
run_experiment.bat
```

- `run_powdergame.bat`: 일반 앱과 mode 인자 전달
- `run_experiment.bat`: Python coordinator가 필요한 자동 experiment

Gate별 `run_g*.bat`은 새로 만들지 않는다.

현재 legacy wrapper:

- `run_g5_demo.bat`
- `run_g7_activity_demo.bat`
- `run_g8_benchmark_gallery.bat`

이들은 `config/development-policy.json`에 명시된 migration debt이며 G8-B closure까지 canonical launcher로 교체한 뒤 제거한다. 새로운 legacy 항목을 추가하면 audit 실패다.

---

## 4. Binary copies

기본 금지:

- checkpoint에 EXE 복사
- archive에 EXE 복사
- evidence folder에 사용자용 EXE 복사
- 다른 worktree의 EXE를 canonical처럼 실행
- 이름만 바꾼 EXE 보존

증거 계약이 실제 binary 포함을 요구하면 create-new frozen copy를 해당 run/audit bundle 안에 만들 수 있다. 이 복사본은 사용자용 canonical EXE가 아니며 다음을 기록한다.

- source SHA
- binary SHA-256
- run ID
- 보존 이유
- 제거 또는 영구 보존 조건

---

## 5. Artifact root

Generated artifact는 Git 밖에 둔다.

```text
C:\Users\mdkap\source\Powdergame-artifacts
```

보존 등급:

| 등급 | 정책 |
|---|---|
| scratch | Scenario당 최근 3개 또는 7일, 자동 정리 가능 |
| candidate | immutable, no overwrite |
| accepted | immutable, 자동 삭제 금지 |
| rejected | 감사 이력, 자동 삭제 금지 |
| development session | 원시 timing; Gate/세션 집계 후 retention 적용 |

Run ID와 output path를 재사용하지 않는다.

---

## 6. Audit

```powershell
pwsh -NoProfile -File tools/dev.ps1 audit
```

검사:

- root launcher allowlist
- 새 `run_g*` 변형
- Git에 commit된 EXE
- required policy docs
- worktree count
- canonical launcher
- legacy migration debt
- artifact root 위치

CI에서는 repository 구조를 검사하고, local-only worktree/artifact 크기는 보고하지 않는다.

---

## 7. Exception contract

새 executable, launcher, worktree, artifact namespace가 필요하면 먼저 문서화한다.

```text
Why existing path cannot serve the role
Why simultaneous retention is required
Owner
Removal date/gate
Audit allowlist entry
```

기한 없는 예외는 허용하지 않는다.
