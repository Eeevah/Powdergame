# Presentation Roadmap — 결과는 정직하게, 감각은 과장한다

Status: **PRODUCT ROADMAP / NOT A CURRENT G8 IMPLEMENTATION GATE**

이 문서는 Simulation Truth 위에 Powdergame의 최종 시각·음향 감각을 쌓는 순서를 정의한다.

Presentation은 장식이 아니다. 플레이어가 원인과 결과를 이해하고, 거대한 세계가 살아 있다는 감각을 얻고, 다시 실험하고 싶어지게 하는 제품 계층이다.

동시에 Presentation 때문에 production physics를 바꾸거나, 고정 Gallery를 실제 게임으로 착각해서도 안 된다.

---

## 1. North Star

> **Cell은 세계를 계산하는 단위이지 최종 화면의 픽셀 스타일을 강제하는 단위가 아니다.**

Simulation이 말하는 결과는 정직하게 보존한다.

- Matter identity
- 위치와 이동
- Temperature
- Pressure
- combustion
- phase transition
- rupture
- gameplay Field/State

Presentation은 그 결과를 더 읽기 쉽고 감각적으로 표현한다.

- flame shape
- Smoke volume/softness
- heat haze
- glow/bloom
- pressure wave/distortion
- sparks/trails
- lighting
- camera response
- audio

Presentation effect는 authoritative Matter나 Field를 몰래 만들지 않는다.

---

## 2. 우선순위 원칙

1. **읽을 수 있어야 아름답게 만들 수 있다.**  
   Material identity와 인과가 불명확한 상태에서 FX를 늘리지 않는다.

2. **제품 surface를 먼저 정의한다.**  
   Observatory/Gallery polish가 First Playable World를 대신하지 않는다.

3. **한 현상씩 vertical slice로 완성한다.**  
   모든 shader를 한꺼번에 추가하지 않고 Fire, Heat, Pressure처럼 사용자 가치가 큰 체인을 먼저 만든다.

4. **비용을 측정한다.**  
   FX는 simulation budget과 별도로 GPU cost를 기록하고 단계적으로 켜고 끌 수 있어야 한다.

5. **Simulation Truth를 가리지 않는다.**  
   화려함 때문에 Material 위치, opening, 흐름, 위험 상태를 잘못 읽게 만들지 않는다.

---

# P0 — Material Visual Identity

Goal: 색을 외우지 않아도 주요 Matter를 구분한다.

## Scope

- Foundation M0 Material의 안정적인 base identity
- Empty / Boundary Block / Stone 구분
- Sand / Water / Oil의 형태·색 차이
- Steam / Smoke 구분
- Ice / Wood 구분
- selected Material swatch
- color-only 의존을 줄이는 shape/texture/value 차이

## Acceptance

- 정지 screenshot에서 주요 Material을 대체로 구분 가능
- 움직이는 장면에서도 Water와 Oil, Steam과 Smoke가 혼동되지 않음
- debug overlay OFF에서도 기본 identity 유지

## Non-goal

- 최종 fire/lighting pipeline
- cinematic post-processing

---

# P1 — Player Comprehension

Goal: “무엇이고 왜 움직이는가”를 플레이어가 스스로 조사할 수 있다.

## Scope

- compact hover Material name
- `I` Cell Inspector detail toggle
- sample freshness 표시
- Material/Temperature/Pressure/activity/chunk state
- 최소 HUD의 정보 우선순위 정리
- Gallery/Debug UI와 Product UI 분리

## Acceptance

- 색을 모르더라도 Cell identity 확인 가능
- stale diagnostic 값을 현재 값으로 오해하지 않음
- Inspector가 simulation을 block하지 않음
- Heavy Mixed와 First Playable에서 실제 이해 비용이 감소

Detailed contract: `../vision/UI_DIRECTION.md`

---

# P2 — Field Visualization

Goal: 보이지 않는 Field가 세계의 행동과 연결되어 보인다.

## Scope

### Temperature

- optional false-color overlay
- hot/cold 범례
- Material identity 유지 옵션
- thermal front 강조

### Pressure

- optional field overlay
- mean/gradient/local peak를 구분할 수 있는 표현
- pressure activity와 absolute pressure를 혼동하지 않는 범례
- opening과 vent 방향을 읽을 수 있는 visual cue

### Activity/Sleep

- developer/product-appropriate overlay 분리
- Runnable / Sleeping chunk 확인
- Field별 activity 구분

## Acceptance

- overlay가 world geometry와 Material을 지우지 않음
- OFF 상태 비용과 시각이 정상
- exact threshold를 몰라도 인과를 이해할 수 있음

---

# P3 — Fire / Heat Vertical Slice

Goal: Fire가 permanent orange pixel이 아니라 energy phenomenon으로 느껴진다.

## Scope

- burning Matter 위의 smooth flame presentation
- emissive glow
- heat haze/refraction
- sparks 또는 short-lived particles
- Smoke가 world를 완전히 가리지 않는 soft presentation
- ignition → growth → fuel consumption → extinction의 리듬
- optional camera/audio response

## Truth boundary

- flame은 Matter ID가 아님
- simulation의 combustion flags/events를 읽음
- Smoke gameplay cells와 high-resolution visual smoke를 구분
- visual particle이 gameplay collision을 만들지 않음

## Acceptance

- 불이 어디서 왜 시작됐는지 보임
- fuel이 끝나면 presentation도 종료됨
- Smoke와 Heat가 Material identity를 파괴하지 않음
- FX OFF에서 동일 simulation result

---

# P4 — Pressure / Rupture Vertical Slice

Goal: 보이지 않는 Pressure가 opening과 venting을 통해 감각적으로 이해된다.

## Scope

- rupture flash/debris-like non-authoritative particles
- vent plume shaping
- pressure-wave distortion
- short camera impulse
- local audio cue
- opening 이후 relief가 시각적으로 읽히는 transition

## Truth boundary

- rupture cell과 opening은 simulation truth
- shockwave visual은 gameplay impulse를 암시하되 새 physics를 만들지 않음
- plume은 actual Steam/Smoke movement와 방향을 맞춤

## Acceptance

- 전용 scripted explosion처럼 보이지 않음
- structure opening과 field relief의 순서를 이해 가능
- Pressure overlay 없이도 기본 causal chain이 읽힘

---

# P5 — World Feel and Scale

Goal: 수백만 Cell의 동시 상호작용이 하나의 살아 있는 세계로 느껴진다.

## Scope

- camera pan/zoom feel
- scale-dependent rendering detail
- background and boundary language
- ambient particles/light
- large-event response without constant screen shake
- scene-level lighting and tone mapping
- audio ambience + event hierarchy

## Constraints

- camera 밖 simulation fidelity를 줄이는 spatial LOD를 기본 가정하지 않음
- presentation detail LOD는 가능
- large world가 UI chrome에 눌리지 않음

---

# P6 — Discovery Presentation

Goal: 플레이어가 발견한 세계를 자기 연구 노트처럼 기억한다.

## Scope

- phenomenon-level discovery entry
- screenshot 또는 small replay reference
- Material pair/conditions의 관찰 기록
- “아직 모르는 성질이 있다” 수준의 미스터리
- exact hidden count 비공개
- Inspector에서 Research Note로 연결

## Non-goal

- 모든 recipe 공개
- exact threshold encyclopedia
- 정해진 빈칸 채우기 UX

---

## 3. 단계와 Gate 관계

### G8

- fixture와 performance evidence가 목적
- P1의 최소 Inspector는 Heavy Mixed comprehension을 위해 앞당길 수 있음
- P2+의 큰 FX 작업은 official timing을 오염시키지 않도록 분리

### G9 / M0 First Playable

최소 필요:

```text
P0 Material identity
+ P1 Inspector/comprehension
+ 필요한 P2 overlay 일부
+ 기본 조작 UI
```

P3/P4의 최소 vertical slice는 Product Validation의 감각을 높일 수 있지만, 모든 final FX가 M0 필수는 아니다.

### M1+

- P3/P4 확장
- P5 world feel
- P6 discovery presentation
- Material family가 늘어날 때 visual language 확장

---

## 4. 구현 전 체크

새 Presentation 기능은 다음을 먼저 답한다.

1. 플레이어가 무엇을 더 잘 이해하게 되는가?
2. 어떤 Simulation Truth를 읽는가?
3. authoritative state를 만들거나 바꾸지 않는가?
4. OFF 상태와 fallback이 있는가?
5. GPU 비용을 어떻게 측정하는가?
6. Gallery/First Playable/Final 중 어느 surface 소유인가?
7. 임시 debug effect의 제거 조건은 무엇인가?

---

## 5. 보류 항목

다음은 필요가 실제로 확인될 때 설계한다.

- volumetric fluid renderer
- full-screen global illumination
- physically based atmospheric simulation
- cinematic timeline system
- final audio middleware
- platform-wide graphics scalability matrix

“아름다운 세계”라는 목표가 모든 고비용 graphics system을 즉시 도입하라는 뜻은 아니다. Powdergame의 규칙·상호작용·스케일을 가장 잘 전달하는 효과부터 선택한다.

---

## 6. 관련 문서

- 제품 North Star: `../START_HERE.md`
- 최상위 Vision: `../vision/USER_VISION.md`
- First Playable: `../vision/FIRST_PLAYABLE_WORLD.md`
- UI/Inspector: `../vision/UI_DIRECTION.md`
- Simulation/Presentation boundary: `../architecture/ARCHITECTURE.md`
- Performance measurement: `../development/PERFORMANCE.md`
- 현재 Gate: `MILESTONES.md`, `STATUS.md`
