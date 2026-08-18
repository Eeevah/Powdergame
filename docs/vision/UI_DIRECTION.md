# UI Direction — 세계를 먼저 보고, 디테일은 필요할 때 연다

Status: **PRODUCT DIRECTION / G9 INPUT / CELL INSPECTOR V0 CONTRACT**

Powdergame UI의 목적은 많은 숫자를 보여주는 것이 아니다.

> **플레이어가 세계를 직접 만지고, 결과를 이해하고, 다음 가설을 세우게 한다.**

이 문서는 Benchmark Gallery와 Observatory의 진단 UI를 실제 제품 UI로 오해하지 않도록 경계를 정하고, Cell Inspector와 향후 overlay의 제품 방향을 정의한다.

---

## 1. UI 원칙

### 1.1 World first

화면의 주인공은 HUD가 아니라 world다.

- 기본 상태에서 world를 최대한 넓게 본다.
- 영구 패널은 최소화한다.
- 상세 수치는 hover, toggle, selection처럼 의도가 있을 때만 연다.
- debug provenance와 benchmark 수치는 제품 기본 화면에 상시 노출하지 않는다.

### 1.2 Details on demand

정보는 세 단계로 공개한다.

```text
보이는 현상
→ 간단한 이름과 상태
→ 사용자가 요청한 상세 수치
```

처음부터 모든 coefficient, threshold, flag, chunk counter를 보여주지 않는다.

### 1.3 관찰 가능하되 정답표가 되지 않는다

현재 Cell의 Temperature와 Pressure를 보는 것은 허용한다. 아직 발견하지 않은 반응의 정확한 공식과 모든 숨은 조건을 미리 공개하는 것은 별개다.

- 현재 관찰값: 공개 가능
- 현재 Material의 기본 성격: 공개 가능
- 이미 관찰한 현상: Research Note에 기록 가능
- 숨은 exact recipe와 남은 발견 수: 기본 비공개

### 1.4 Sample freshness를 정직하게 표시한다

GPU diagnostic readback은 화면의 현재 frame보다 늦을 수 있다.

Inspector가 표시하는 값에는 반드시 다음 중 하나가 있어야 한다.

```text
Sample: sim tick 7412
```

또는:

```text
Latest diagnostic · 8 ticks old
```

stale sample을 현재 authoritative value처럼 가장하지 않는다.

### 1.5 Simulation을 UI 때문에 바꾸지 않는다

- Inspector는 read-only다.
- Hover 때문에 매 frame 전체 GPU readback을 추가하지 않는다.
- 공식 benchmark timed loop에 Inspector 비용을 포함하지 않는다.
- presentation 편의를 위해 production physics를 special-case하지 않는다.

---

## 2. Cell Inspector v0

Cell Inspector는 Heavy Mixed와 G9에서 “색은 보이지만 무엇인지 모르겠다”는 문제를 해결하는 첫 comprehension 기능이다.

### 2.1 Compact Hover — 기본 ON

마우스가 world Cell 위에 있을 때 커서 근처에 짧게 표시한다.

```text
Water
```

의미 있는 즉시 상태가 있으면 한 줄까지 확장할 수 있다.

```text
Wood · Combusting
Steam · Hot
Stone
Empty
Boundary Block
```

규칙:

- world 밖에서는 표시하지 않는다.
- HUD 위에서는 world tooltip을 띄우지 않는다.
- 커서를 가리지 않도록 작은 offset을 둔다.
- Material 이름은 registry의 canonical display name을 사용한다.
- 색상만으로 식별하게 만들지 않는다.

### 2.2 Detailed Inspector — `I` Toggle

`I`로 상세 패널을 켜고 끈다.

기본 예:

```text
Water
Cell        143, 207
Material    Water
Temperature 72.4
Pressure    53.5
Activity    Matter · Thermal · Pressure
Chunk       2, 3 · Runnable
Sample      sim 7412 · diagnostic 928
```

연소 가능한 Matter 예:

```text
Wood
State       Combusting
Fuel        438 / 900
Flame       Active this tick
Temperature 164.2
Pressure    83.3
```

Sleep 예:

```text
Steam
Chunk       Sleeping
Activity    None
Sample      sim 1292 · diagnostic 321
```

### 2.3 v0 필드

항상 가능한 경우:

- Cell coordinate
- Material의 canonical name과 ID. v0 detailed Inspector는 명시적 계약에 따라 `Name (ID)`로 함께 표시하고, compact hover는 이름을 우선한다.
- Temperature
- Pressure
- raw flags의 사용자 의미 변환
- Matter / Thermal / Pressure / Reaction activity
- Chunk coordinate
- Runnable / Sleeping state
- diagnostic sample sequence
- sample simulation tick

Material별 의미가 있을 때:

- Combusting
- Flame event
- Fuel progress
- phase-related identity
- rupture threshold와 adjacent Pressure는 **developer detail mode**에서만 고려

### 2.4 공개 수준

Inspector는 두 표시 수준을 가질 수 있다.

```text
Normal detail
Developer detail
```

Normal detail:

- 플레이어가 현상을 이해하는 데 필요한 현재 상태
- 인간이 읽을 수 있는 이름
- exact coefficient는 숨김

Developer detail:

- Material ID
- flags bit interpretation
- chunk activity mask
- state hash 또는 sample provenance
- threshold 비교

v0에서는 `I` 하나로 시작하고, developer detail 분리는 필요가 실제로 생길 때 추가한다.

---

## 3. 좌표 변환과 데이터 경계

### 3.1 Mouse → Cell

Renderer의 letterbox와 world aspect ratio를 그대로 사용한다.

```text
window cursor
→ rendered world rectangle 확인
→ world UV
→ integer Cell x/y
```

- black/empty letterbox 영역은 world 밖이다.
- resize와 DPI scale에서 같은 Cell을 가리켜야 한다.
- y-axis 방향을 명확히 테스트한다.

### 3.2 데이터 소스

권장 v0:

```text
기존 out-of-band diagnostic snapshot
→ CPU-side latest sample cache
→ hovered Cell lookup
→ text renderer
```

금지:

```text
mouse move마다 map_async
mouse move마다 full-world copy
hover를 위해 production tick 동기화
```

Gallery에 full cell snapshot이 없다면, bounded inspector sample을 별도 주기로 수집하되 official timing과 분리하고 비용을 계측한다.

### 3.3 업데이트 리듬

- cursor position: 매 frame 업데이트 가능
- Inspector value: 최신 완료 diagnostic sample 사용
- 정상적인 sample pending은 visually silent하다. Compact tooltip, detailed panel, placeholder를 모두 그리지 않는다.
- 새 hover Cell의 fresh sample이 도착하기 전까지 이전 Cell의 Material이나 field 값을 재사용하지 않는다.
- `I` toggle 상태는 pending 동안 유지하지만, matching sample이 `Ready`가 된 frame부터 detailed panel을 다시 표시한다.
- reset/scenario switch/world epoch 변경/staging recovery 동안 이전 world의 값을 숨기고, 새 world의 fresh sample 뒤에만 UI를 복원한다.
- 실제 readback/map/channel 실패는 정상 pending과 분리한다. Compact hover에는 오류를 표시하지 않고, detail mode가 ON일 때만 작은 고정 `Inspector unavailable` 상태를 표시한다. 상세 오류는 structured log에 남긴다.

---

## 4. Field Overlay 방향

Inspector 이후 선택적 overlay를 추가할 수 있다.

권장 순환:

```text
V
→ Material
→ Temperature
→ Pressure
→ Activity
→ Material
```

또는 작은 UI toggle로 분리한다.

### Temperature

- 연속 false-color ramp
- Material identity를 완전히 지우지 않는 선택지
- reference/low/high 범례
- 최종 art가 아니라 comprehension view임을 명확히 함

### Pressure

- signed/unsigned 의미에 맞는 범례
- local peak 하나가 전체 화면을 포화시키지 않도록 robust range 사용
- absolute Pressure와 Pressure activity를 혼동하지 않음

### Activity

- Matter / Thermal / Pressure / Reaction을 구분
- Chunk Runnable/Sleeping overlay 가능
- “존재하는 Matter”와 “현재 work가 필요한 Matter”를 구분

Overlay는 v0 Inspector를 막지 않는다. 먼저 이름과 Cell detail을 해결한다.

---

## 5. 기본 제품 HUD

First Playable의 기본 HUD는 다음만 상시 유지하는 것을 목표로 한다.

- 현재 선택한 Material/도구
- brush size
- Pause/Play 상태
- speed
- 최소 조작 힌트
- 필요 시 작은 world status

다음은 기본 화면에서 숨기거나 developer mode로 이동한다.

- source SHA
- binary hash
- exact diagnostic counters 전체
- benchmark identity
- artifact status
- full predicate table

이 값들은 Gallery/Harness에는 중요하지만 제품 기본 HUD의 중심이 아니다.

---

## 6. Palette와 도구 방향

### Material palette

- Material 이름과 식별 가능한 swatch
- behavior family 또는 한 줄 성격
- 아직 발견하지 않은 hidden interaction은 미리 설명하지 않음
- Foundation M0 set으로 먼저 검증

### World tools

M0 First Playable 최소:

- Draw Matter
- Erase
- brush size
- Heat
- Cool
- Pan
- Zoom
- Reset
- Pause / Step / Speed

구조 편집과 Material 배치는 동일한 safe edit/reset contract를 사용한다.

---

## 7. Discovery와 Inspector의 관계

Inspector는 현재 Cell을 읽는 도구다. Discovery는 플레이어가 관찰한 현상을 기억하는 시스템이다.

```text
Inspector
→ 지금 무엇이 있는가 / 어떤 상태인가

Research Note
→ 내가 무엇과 무엇을 만나게 했고 어떤 현상을 봤는가
```

Inspector가 Discovery를 자동 완성하면 안 된다. 의미 있는 현상 검출과 플레이어 기록은 별도 제품 결정이다.

---

## 8. Debug UI와 Product UI

| 항목 | Debug/Gallery | Product 기본 |
|---|---|---|
| Source SHA | 필요 | 숨김 |
| Binary hash | 필요할 수 있음 | 숨김 |
| Predicate state | 필요 | 숨김 또는 developer mode |
| Material name | 필요 | 필요 |
| Temperature/Pressure | 진단용 상세 | 요청 시 표시 |
| Sample tick | 필요 | Inspector 상세에 표시 |
| Scenario identity | 필요 | preset 사용 시만 |
| Brush/selected Matter | 부차적 | 핵심 |
| Discovery hint | 없음 | 제품 기능 |

공유 renderer와 text system을 사용해도 정보 우선순위는 달라야 한다.

---

## 9. Cell Inspector v0 Acceptance

### Implementation candidate

- Status: **IMPLEMENTATION CANDIDATE / USER ACCEPTANCE PENDING**.
- Tested source: `3c342d25099683df53e303d1920cebe1f6578b74`. 이 SHA 이후의 docs-only closure는 tested source provenance와 구분한다.
- Rendering과 physical-pixel cursor picking은 동일한 CPU-authoritative `WorldViewport`의 letterbox origin/scale을 공유한다.
- Hover sample은 Material, Temperature, Pressure, raw flags, Cell activity, Chunk state의 six 4-byte field를 하나의 24-byte batch로 수집한다. 동시 pending request는 하나로 제한하고 주기는 10 Hz 이하이며, mouse movement마다 full-world readback은 없다.
- Request identity는 Cell/Chunk, simulation tick, diagnostic sequence, selection generation, world epoch를 묶는다. Reset, scenario switch, staging failure, shutdown은 pending/sample identity를 무효화해 이전 world의 stale sample이 현재처럼 보이지 않게 한다.
- Presentation state는 `Hidden / Pending / Ready / Failed`다. `Hidden / Pending`은 완전 무표시, `Ready`는 matching sample만 표시, `Failed`는 detail ON일 때만 작은 고정 오류 panel을 표시한다.
- Silent-pending validation: fmt check, Inspector/UI targeted tests `16/16`, existing viewport tests `7/7`, affected Windows all-target check/clippy, strict development-policy audit, and exactly one 3-frame Gallery startup smoke passed. Workspace FULL과 experiment candidate는 실행하지 않았다.
- Startup smoke는 hover UX를 자동 승인하지 않는다. Silent pending의 최종 사용자 acceptance는 계속 pending이다.
- Scenario 5 Heavy Mixed World는 **PENDING — do not start**이고 G8-B는 **NOT CLOSED**다.

### Functional

- cursor가 가리키는 Cell 이름이 정확하다.
- resize/DPI/letterbox에서 좌표가 정확하다.
- `I` toggle이 안정적으로 동작한다.
- reset/scenario switch 후 stale Cell을 표시하지 않는다.
- Material, Temperature, Pressure, activity, chunk state가 같은 sample에 묶인다.
- `R` reset, `1–6` Gallery switch, `ESC`와 충돌하지 않는다.

### Performance

- hover만으로 simulation tick을 block하지 않는다.
- mouse movement에 full-world GPU copy를 추가하지 않는다.
- Inspector OFF 비용은 사실상 0에 가깝다.
- Inspector ON 비용은 official benchmark에서 제외되고 별도로 측정 가능하다.

### UX

- 사용자가 색을 외우지 않고 Matter를 구분한다.
- compact hover가 world를 가리지 않는다.
- 상세 정보는 필요할 때만 열린다.
- sample freshness가 이해 가능하다.
- 숨은 공식의 정답표처럼 느껴지지 않는다.

---

## 10. 관련 문서

- 제품 North Star: `../START_HERE.md`
- 최상위 비전: `USER_VISION.md`
- 첫 5분: `FIRST_PLAYABLE_WORLD.md`
- Presentation 순서: `../planning/PRESENTATION_ROADMAP.md`
- Simulation/Presentation 구조: `../architecture/ARCHITECTURE.md`
- Material 이름과 descriptor: `../specs/MATERIAL_SPEC.md`
