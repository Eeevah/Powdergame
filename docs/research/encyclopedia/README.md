# Powdergame Master Encyclopedia

## Status

- Type: `DERIVED` research encyclopedia
- Authority: **non-authoritative** — this does not register or implement Materials.
- Goal: merge all material/world-primitive ideas gathered so far into a searchable, readable Doodle-God-like encyclopedia.
- Corpus snapshot: **530 source-index rows**. This includes preserved duplicates, aliases, skipped ideas and IP references; it is **not** a claim that the final game should contain 530 Materials.

> 플레이어에게 물질을 주는 게임이 아니라, 우주를 발명할 수 있는 문법을 준다.

이 사전은 숫자표가 아니라 **상상력과 상호작용을 위한 언어집**이다. 각 항목은 먼저 “이건 어떤 존재인가?”를 읽히게 하고, 그 다음 Powdergame에서 어떤 국소 규칙으로 표현할 수 있는지를 적는다.

## Entry contract

각 항목은 가능한 한 다음 두 층을 가진다.

### Player-readable encyclopedia layer

- **도감** — 두들갓처럼 짧고 기억에 남는 정체성 문장
- **핵심 행동** — 플레이어가 3초 안에 예상할 수 있는 동사
- **발견 가치** — 무엇과 붙였을 때 새 현상을 상상하게 하는가

### Development layer

- **Movement / Layer** — STATIC / POWDER / LIQUID / GAS 또는 Matter가 아닌 Field / Agent / Concept / Meta
- **상태** — M0 validated / catalog direction / candidate / future / reference-only
- **Simulation notes** — Temperature / Pressure / Density / Combustion / Transition 등 어떤 세계 문법으로 표현할지
- **출처 / provenance** — 어느 조사·현실·창작 reference에서 왔는지
- **주의** — IP 직접명칭, 중복, 전용 시스템 비용, 절대효과 위험, counter/failure 필요성

설명이 약한 항목은 다음 순서로 보강한다.

```text
existing Powdergame rule inference
→ real science / myth / media mechanism research
→ original imagination
```

현실 정보는 직관의 앵커이지 게임값의 정답이 아니다.

## Coverage policy

사전은 구현보다 넓어야 한다.

- **현실 물질:** 플레이어가 의미 있게 구분할 수 있는 현실 물질/재료 아키타입을 폭넓게 수용한다. 모든 화합물·합금 규격을 literal하게 복제하지는 않지만, 익숙한 물질을 검색했을 때 사전에 둘 자리가 있는 것을 목표로 한다.
- **Doodle-God-like elements:** Matter뿐 아니라 생명, 구조물, 도구, 사회, 마법, 추상 개념까지 웬만하면 검토 대상으로 보존한다.
- **런타임:** 사전에 있다고 전부 Material ID가 되는 것은 아니다. Matter / Phenomenon / Field / Agent / Structure / Concept / Meta 중 올바른 층으로 보낸다.

```text
encyclopedia inclusion is generous
        ↓
primitive classification is strict
        ↓
implementation remains evidence-gated
```

## Discovery-first rule

게임 안에서 이 사전을 처음부터 전부 보여주면 안 된다. 실제 제품 Dictionary는 플레이어가 관찰한 현상을 기록하는 **발견 도감**이어야 한다.

```text
Developer Master Encyclopedia
        ↓ select / normalize
Material / World Primitive definitions
        ↓ runtime
Player observes phenomena
        ↓
In-game Dictionary reveals discovered knowledge
```

따라서 이 디렉터리는 개발·콘텐츠 제작용 master corpus이며 플레이어용 답안지가 아니다.

## Volumes

### 01 — Foundation / Reality / Space

- `01A_FOUNDATION_CATALOG.md` — M0 validated 9종과 이미 알려진 초기 catalog 방향
- `01B_REAL_MATERIAL_LIBRARY.md` — 철·구리·세라믹·연료·고분자·방사성 소재 등 현실 reference
- `01C_GENERAL_SPACE_CANDIDATES.md` — Dirt/Mud/Clay부터 Dry Ice/Clathrate/Regolith까지 일반·우주 후보

### 02 — Original Matter 001–100

- `02A_ORIGINAL_MATTER_OM001_050.md`
- `02B_ORIGINAL_MATTER_OM051_100.md`

### 03 — Expanded Original Matter / Derivatives

- `03A_ORIGINAL_MATTER_OM101_130.md`
- `03B_ORIGINAL_MATTER_OM131_160.md`
- `03C_ORIGINAL_MATTER_OM161_180.md`
- `03D_VX_DERIVATIVES.md`

### 04 — Reference Quarry

원자료의 익숙한 이름과 lore를 버리지 않고 mechanics를 채굴하기 위한 보존층이다. 유명 작품 고유명사는 모두 `REFERENCE_ONLY`이며 최종 콘텐츠로 직접 승격하지 않는다.

- `04A_REFERENCE_QUARRY_GENERAL.md`
- `04B_REFERENCE_QUARRY_SPACE.md`
- `04C_REFERENCE_QUARRY_MEDIA_A.md`
- `04D_REFERENCE_QUARRY_MEDIA_B1.md`
- `04D_REFERENCE_QUARRY_MEDIA_B2.md`
- `04D_REFERENCE_QUARRY_MEDIA_B3.md`
- `04E_REFERENCE_QUARRY_FANTASY_ALCHEMY.md`
- `04F_BROAD_REFERENCE_ADDITIONS.md`

### 05 — Doodle-God World Primitives

- `05_DOODLEGOD_WORLD_PRIMITIVES.md` — 기억 젤, 시간 먼지, 확률 결정, 언어 포자, 신화 잉크, 인과 실 등 Matter를 넘어서는 Field/Agent/Concept/Meta 후보와 발견 레시피

### 06 — Web Reality Anchors

- `06_WEB_REALITY_ANCHORS.md` — Mars perchlorate, methane hydrate, Titan hydrocarbon lakes, aerogel, shape-memory alloy, piezoelectricity, superionic ice 등 설명 보강용 현실 앵커

### 07 — Real-world coverage policy

- `07_REAL_WORLD_COVERAGE_POLICY.md` — 현실 물질을 사전에는 폭넓게 넣되, behavior family/alias/variant를 통해 runtime을 불필요하게 폭발시키지 않는 기준

### 08 — Doodle God coverage review

- `08_DOODLE_GOD_COVERAGE_REVIEW.md` — classic/Blitz 계열의 물질·생명·기술·사회·마법·추상 요소를 거의 전수 검토하는 층 분류와 Powdergame 변환 원칙

### Search index

- `MASTER_INDEX.md` — landing page
- `MASTER_INDEX_A.md` — index part A
- `MASTER_INDEX_B.md` — index part B

Research/index ID는 실제 Material Registry ID가 아니다.

## Canonicalization rule

사전에 이름이 있다고 실제 게임 Material이 되는 것은 아니다.

```text
source / lore name
→ mechanic extraction
→ behavior family
→ duplicate / alias merge
→ Powdergame-original representative
→ counter / failure / byproduct design
→ user review
→ ADR / SPEC / content registration
```

예를 들어 `Methane / Vespene / Tibanna`가 모두 “가연성 가스”라는 같은 핵심 동사에 수렴한다면 세 이름을 그대로 만들 이유가 없다. 반대로 같은 출발점이라도 interaction chain이 완전히 다르면 별도 Matter가 될 수 있다.

현실 물질은 이 규칙을 조금 다르게 적용한다. 익숙한 실재 이름 자체가 discovery value를 갖기 때문에 encyclopedia entry는 유지할 수 있지만, runtime에서는 동일 behavior family의 variant/alias가 될 수 있다.

## IP / provenance rule

유명 작품의 이름·설정은 `REFERENCE_ONLY`다. 이름을 그대로 쌓아 올리는 게임이 아니라:

```text
익숙한 그림
→ 왜 재미있는지 행동으로 분해
→ 현재 Powdergame 세계 법칙에 맞게 재구성
→ 독자 이름과 실패 모드까지 설계
```

하는 것을 기본으로 한다.

Doodle God 역시 동일하다. 개별 조합식을 복사하는 것이 아니라 **‘물질→생명→도구→사회→개념으로 발견 그래프가 확장되는 감각’**을 Powdergame의 causal simulation으로 번역한다.

## Current next step

이 corpus의 다음 작업은 두 축을 병행한다.

### Broad coverage

- 현실 물질 family의 빈 영역 채우기
- Doodle God classic/Blitz element sweep에서 누락된 자연물·생명·제품·기술·사회·마법 개념 추가
- 각 항목을 Matter / Field / Agent / Structure / Concept / Meta로 분류

### Canonicalization

- exact duplicate / alias 병합
- palette/lore-only 항목 제거 또는 reference-only 유지
- 24 behavior family에 연결
- IP reference → original representative 변환
- final player-facing flavor 작성
- interaction / counter / byproduct 연결
- `FOUNDATION-COMPATIBLE / NEAR-TERM / FUTURE-FAMILY / META-DEFER` 재판정

그 뒤 사용자 검토를 통과한 항목만 실제 Material/content 설계로 승격한다.
