# First Playable World — 첫 5분 제품 계약

Status: **G9-A USER ACCEPTED WITH KNOWN FOLLOW-UP**

이 문서는 첫 플레이어가 Powdergame을 켠 뒤 약 5분 동안 무엇을 하고, 무엇을 이해하고, 어떤 다음 질문을 떠올려야 하는지 정의한다.

기술 fixture나 benchmark scenario의 계약이 아니다. `planning/MILESTONES.md`의 G9 Product Validation을 구체적인 플레이 경험으로 번역한다.

---

## 1. 한 문장 목표

> **플레이어가 작은 가설을 세계에 던지고, 예상보다 풍부한 결과를 보고, 즉시 다음 실험을 시작하게 한다.**

첫 5분이 끝났을 때 플레이어는 모든 시스템을 이해할 필요가 없다. 대신 다음 감각을 가져야 한다.

- “내가 놓은 것이 실제로 세계를 바꿨다.”
- “이 게임의 규칙은 배울 수 있다.”
- “아직 내가 모르는 반응이 많다.”
- “다음에는 이것과 저것을 같이 써보고 싶다.”

---

## 2. 첫 5분 시나리오

시간은 목표 리듬이며 강제 튜토리얼 타이머가 아니다.

### 0:00–0:30 — 세계와 도구를 이해한다

플레이어가 보는 것:

- 실제 simulation world
- 작은 M0 Material palette
- Draw / Erase
- Pause / Play / Single Step / Speed
- 현재 선택한 Material
- 마우스를 올리면 보이는 Material 이름

플레이어가 이해해야 하는 것:

- 화면은 단순한 동영상이 아니라 직접 편집 가능한 세계다.
- 한 Cell에는 한 Matter가 존재한다.
- 정답 순서를 따를 필요가 없다.

### 0:30–1:15 — Sand와 Water로 즉각적인 반응을 본다

플레이어 행동 예:

- Stone으로 작은 그릇이나 경사면을 만든다.
- Sand를 붓는다.
- Water를 놓아 Sand와 공간적으로 만나게 한다.

기대 경험:

- Sand는 쌓이고 흐른다.
- Water는 local movement로 퍼진다.
- 장애물의 작은 변경이 전체 형태를 바꾼다.
- Reset으로 처음부터 다시 시도할 수 있다.

### 1:15–2:15 — Heat가 새로운 상태를 만든다

플레이어 행동 예:

- Water 근처에 Heat를 적용한다.
- Pause/Step으로 변화의 순서를 본다.
- Hover Inspector로 Water / Steam과 Temperature를 확인한다.

기대 경험:

```text
Heat
→ Water temperature 상승
→ Steam
→ 공간과 주변 구조에 새로운 영향
```

플레이어는 exact threshold를 외울 필요가 없다. 원인과 결과가 보이고, 상세 정보가 필요할 때만 Inspector로 확인할 수 있어야 한다.

### 2:15–3:15 — 구조와 Pressure의 인과를 발견한다

플레이어 행동 예:

- Stone과 Wood로 밀폐에 가까운 구조를 만든다.
- Water를 넣고 가열한다.
- 구조가 열리거나 Steam이 배출되는 과정을 본다.

기대 경험:

- Pressure는 장식 숫자가 아니라 세계의 다음 행동을 만든다.
- 약한 구조와 강한 구조의 차이를 실험으로 배운다.
- 파열은 전용 폭발 script가 아니라 공통 규칙의 연쇄로 보인다.

이 장면은 반드시 성공하도록 강제된 튜토리얼 퍼즐일 필요가 없다. 실패한 구조도 다음 실험의 정보가 된다.

### 3:15–4:15 — Inspector로 세계를 읽는다

플레이어 행동 예:

- Water, Steam, Wood, Smoke 위에 마우스를 올린다.
- `I`로 상세 Inspector를 켠다.
- Temperature / Pressure / activity / state를 확인한다.

기대 경험:

- 색만 보고 추측해야 하는 혼란을 줄인다.
- 세계가 왜 그렇게 움직였는지 이해할 단서를 얻는다.
- Inspector는 정답표가 아니라 관찰 도구다.

### 4:15–5:00 — 자기 실험을 시작한다

플레이어 행동 예:

- Oil과 Fire를 추가한다.
- 다른 구조를 만든다.
- 발견한 현상을 다시 재현하거나 일부러 실패시킨다.

성공 신호:

> 플레이어가 안내가 끝나기 전에 스스로 다른 조합을 시도한다.

---

## 3. M0 First Playable의 최소 기능

### 반드시 필요

- Material palette: Boundary Block, Stone, Sand, Ice, Water, Steam, Smoke, Wood, Oil
- Draw / Erase
- Brush size 변경
- Pause / Play
- Single Step
- Speed 변경
- Reset
- Pan / Zoom
- Heat / Cool 또는 최소한의 thermal edit 도구
- Hover Material name
- Toggleable Cell Inspector
- 현재 선택 도구와 조작법이 보이는 최소 HUD
- 실패가 world corruption이나 crash로 이어지지 않는 안전한 편집 경로

### 강하게 권장

- Temperature / Pressure / Activity overlay
- 최근 실험을 되돌리는 간단한 Rewind 진입점
- 발견한 현상을 기록하는 초기 Research Note
- 작은 preset 또는 실험 시작점

### M0에서 필수 아님

- 수십·수백 Material
- 완성된 Discovery encyclopedia
- 최종 modern FX 전체
- 정교한 Save/Load 호환 체계
- Rule DSL / Modding
- AI가 Matter를 생성하는 기능
- 모든 조합을 자동 탐색하는 Interaction Lab

---

## 4. Player Comprehension Loop

Powdergame의 기본 조작 루프는 다음과 같아야 한다.

```text
놓는다
→ 본다
→ 궁금해한다
→ Inspector로 확인한다
→ 조건을 바꾼다
→ 다시 본다
→ 자기 규칙으로 기억한다
```

UI는 이 루프를 돕되, 숨은 공식을 모두 공개해 탐험을 없애면 안 된다.

### 기본 공개

- Material 이름
- 현재 관찰 가능한 상태
- Temperature / Pressure 같은 현재 값
- Combusting, Sleeping, Runnable 같은 의미 있는 상태
- sample 시점

### 기본 비공개

- 아직 발견하지 않은 정확한 반응식
- 모든 threshold와 coefficient의 전체 표
- 남은 발견의 정확한 개수
- 미래 결과를 미리 알려주는 정답 힌트

> **게임은 현상을 보여주고, 공식은 필요 이상으로 설명하지 않는다.**

---

## 5. Debug surface와 제품 surface의 경계

First Playable World는 Benchmark Gallery를 그대로 편집 가능하게 만든 화면이 아니다.

### Gallery가 잘하는 것

- 고정 fixture 비교
- subsystem 진단
- telemetry와 provenance 표시
- acceptance evidence 생성

### First Playable이 해야 하는 것

- 세계가 화면의 중심
- 플레이어의 선택과 조작이 중심
- 필요한 정보만 단계적으로 공개
- debug terminology 없이도 기본 행동을 이해 가능
- 다음 실험으로 빠르게 이어짐

개발 중에는 같은 renderer, staging, Inspector infrastructure를 재사용할 수 있지만, 정보 구조와 조작 흐름은 별도로 검증한다.

### G9-A implementation candidate

Source `a00e39b2e00bfbd9ac28214c44cd22cc97542bb4` is the continuity v2 Sandbox product surface inside the canonical Windows EXE. The canonical no-argument BAT/EXE launch opens this surface, while the frozen Gallery remains explicit. Existing EMPTY-only Draw, stable Ice/Steam placement, grouped palette and Heat/Cool feedback remain. Inspector now separates requested hover from the presented sample: during a bounded hold it keeps the original Cell, Material, simulation tick and freshness under a visible `Previous sample` label, never moves that identity to the new cursor, and shows compact cursor copy only after a fresh current-Cell sample arrives. The detailed panel keeps one geometry across Ready, Held and Sampling.

Starter Lab intentionally avoids five benchmark panels: a Stone foundation supports one open left basin with Water and Sand, a central Wood bridge, and one open right cup with Oil and Ice, while more than 75% of the world remains empty for player-built experiments. It has no authored heat, pressure, flags, scripted outcome or automatic progression.

The previous revision was directly re-reviewed and still required Inspector continuity work. Continuity v2 passed focused automated integrity and one bounded launch check, but has not passed user re-review. Thermal Transport & Ignition Causality is planned/design-required/not-started before G9-B. G9-B/C/D/E remain **NOT STARTED**.

---

## 6. G9 사용자 검증 질문

G9 후보를 직접 플레이한 뒤 사용자가 다음 질문에 답한다.

1. 첫 30초 안에 무엇을 할 수 있는지 이해했는가?
2. Matter를 놓은 결과가 즉각적이고 납득 가능했는가?
3. Inspector가 세계를 이해하는 데 도움을 주었는가, 아니면 방해했는가?
4. 적어도 한 번은 안내 없이 자기 실험을 시작했는가?
5. 예상하지 못했지만 다시 이용해보고 싶은 결과가 있었는가?
6. 세계 크기와 동시 상호작용이 게임의 판타지에 충분했는가?
7. debug 화면이 아니라 실제 게임을 하고 있다는 느낌이 들었는가?
8. 다시 켜서 다른 것을 시도하고 싶은가?

자동 테스트와 Harness는 이 질문에 대신 답할 수 없다.

---

## 7. 실패 조건

다음 중 하나가 강하면 First Playable은 아직 통과하지 않는다.

- 색을 외우지 않으면 Material을 구분할 수 없음
- HUD가 world보다 더 큰 비중을 차지함
- 무엇을 클릭해야 하는지 알 수 없음
- 매 실험이 preset 재생으로 끝남
- Inspector가 stale sample을 현재 값처럼 표시함
- Heat/Pressure가 보이지만 플레이어 행동과 연결되지 않음
- 결과가 전용 scripted event처럼 보임
- 사용자가 다음 질문을 만들지 못하고 종료함

---

## 8. 관련 문서

- 최상위 제품 비전: `USER_VISION.md`
- UI 정보 공개와 Inspector: `UI_DIRECTION.md`
- 시각적 발전 순서: `../planning/PRESENTATION_ROADMAP.md`
- 완료 Gate: `../planning/MILESTONES.md`
- 현재 구현 상태: `../planning/STATUS.md`
