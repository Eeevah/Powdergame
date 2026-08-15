# Foundation Design Session — 2026-08-15

> 이 문서는 **현재 설계가 어떻게 만들어졌는지**를 보존한다.
>
> 단순 결정 목록이 아니다. Assistant가 어떤 질문/선택지를 제시했고, 사용자가 무엇을 선택했으며, 어떤 추가 코멘트가 설계를 바꾸었는지까지 남긴다.
>
> 구현 시 현재 계약은 `vision/`, `specs/`, `architecture/`, `development/` 문서를 따른다. 이 문서는 그 계약의 **provenance와 맥락**이다.

---

## 0. Provenance / 기록 강도

이 세션은 길어서 일부 앞부분은 대화 압축 상태에서 복원되었다.

표기:

- **DIRECT** — 현재 세션의 실제 Q&A/사용자 코멘트가 명확하게 남아 있음
- **RECONSTRUCTED** — 세션 상태 요약과 승인 기록을 기반으로 복원. 의미/결정은 확정됐지만 원문 문장은 축약될 수 있음
- **SUPERSEDED** — 당시 잠정 선택이 이후 사용자 코멘트로 변경됨
- **DEFERRED** — 아이디어는 보존하지만 현재 구현 범위에서 제외

중요한 원칙은 결정 번호보다 **사용자 의도와 최종 의미**다. Interaction Lab 논의 후 질문 번호가 일부 재사용되었기 때문에 번호만으로 결정을 식별하지 않는다.

---

# Part A — Foundation before the late Q&A block

## A1. Documentation architecture — RECONSTRUCTED / APPROVED

### Question context

성공적인 기존 프로젝트의 구조를 참고하되 Powdergame의 문서를 어떻게 유지할 것인가.

### User direction

문서가 마일스톤/구조/도식화까지 잘 잡혀 있어야 하고, AI/Codex가 이어받더라도 프로젝트 방향을 잃지 않아야 한다.

### Final decision

Domain-separated documentation:

```text
docs/
├─ vision/
├─ planning/
├─ architecture/
├─ specs/
├─ development/
├─ design-history/
└─ HANDOFF.md
```

`ROADMAP`은 방향, `MILESTONES`는 Evidence Gate, `STATUS`는 실제 현재 상태로 분리한다.

Milestone 최종 `ACHIEVED`는 사용자 승인 없이는 선언하지 않는다.

---

## A2. Platform / Production path — RECONSTRUCTED / APPROVED

### Considered

- Browser 기반 실험을 제품 경로로 계속 유지
- Mac/Windows 범용 공용 경로
- Windows + RTX 5090에 집중

### User decision

현재 게임은 우선 사용자가 직접 쓰는 게임이며 다른 GPU/플랫폼 호환 때문에 구조를 복잡하게 만들 필요가 없다.

### Final decision

```text
Windows
Rust
winit
wgpu
DX12
RTX 5090 primary performance target
```

Browser/macOS는 현재 제품 경로가 아니다.

---

## A3. GPU Production authority — RECONSTRUCTED / APPROVED

### Question context

CPU reference와 GPU production 중 무엇이 실제 세계의 기준인가.

### Final decision

- GPU Production Simulation이 authoritative.
- CPU Reference는 작은 읽기 쉬운 비교/디버그 구현.
- CPU/GPU bit-exact 일치는 요구하지 않음.
- CPU↔GPU로 전체 world를 매 Tick 복사하지 않음.

---

## A4. Determinism philosophy — RECONSTRUCTED / APPROVED

### User direction

완벽한 재현성보다 성능이 더 중요하다. 일부 GPU float/order 차이는 문제없다.

### Final principle

> **No intentional randomness; no performance sacrifice for bit-perfect replay.**
>
> **Non-exact but stable.**

허용: 미세한 float/order 차이, valid local winner 차이.  
금지: state corruption, duplicate occupancy, NaN runaway, memory error.

---

## A5. Powdergame Cell identity — RECONSTRUCTED / APPROVED

### User correction

ONI 같은 same-space multi-material/mass 모델은 Powder Game 정체성과 다르다.

### User principle

한 공간에 하나가 있는 단순성이 제약이면서 게임의 정체성이다. Minecraft에서 한 위치에 한 Block이 있는 것과 비슷한 강한 규칙이다.

### Final decision

> **One Cell = Max One Matter.**

- per-cell mixture 없음
- per-cell `0.3 Water` 없음
- Matter가 있으면 one unit
- Temperature/Pressure/state는 두 번째 Matter가 아님

---

## A6. Long-term world layers — RECONSTRUCTED / APPROVED

장기 개념:

1. Matter
2. Field
3. Agent
4. Concept
5. Meta

하지만 M0는 Matter + Field만 구현한다. 미래 계층의 빈 abstraction을 미리 만들지 않는다.

---

## A7. Finite world / chunk — RECONSTRUCTED / APPROVED

### User correction

Infinite world는 목표가 아니다.

### Final decision

- finite world
- reference 2048×2048 = 4,194,304 cells
- `WorldConfig`
- initial chunk 64×64
- 64는 invariant가 아니라 benchmark baseline

---

## A8. DAN-BALL boundary — RECONSTRUCTED / APPROVED

DAN-BALL의 editable outer BLOCK 감각을 따른다.

- 외곽 BLOCK을 erase 가능
- world는 finite 유지
- invisible wall을 추가하지 않음

밖으로 나간 Matter 처리 결정은 뒤 Q73에서 Void로 확정.

---

## A9. Fire / Combustion — RECONSTRUCTED / APPROVED

### User direction

Fire는 중요한 핵심 요소다. Temperature 없는 M0는 의미가 부족하다.

현실의 Oxygen이 필요하다는 이유만으로 Oxygen을 넣을 필요는 없다. Oxygen을 제거해 불을 끄는 식의 gameplay가 재미있을 때 추가하면 된다.

### Final decision

Fire는 기본적으로 permanent orange Matter가 아니라:

```text
Fuel + thermal ignition condition
→ combustion
→ Heat + Smoke
→ flame presentation
```

이라는 phenomenon/state로 시작한다.

---

## A10. Phase transition yield / Pressure — RECONSTRUCTED / APPROVED

### User preference

현실적인 1:1 conservation보다 공간적으로 재미있는 변화가 중요하다.

### Final direction

- phase transition yield는 data-driven
- Water → Steam이 여러 Cell을 요구할 수 있음
- 공간 부족 시 unresolved expansion이 Pressure로 연결 가능
- Pressure가 Matter를 밀고 약한 구조를 파열시키고 opening에서 vent 가능

---

## A11. Simulation / Presentation boundary — RECONSTRUCTED / APPROVED

### User idea

물리 계산은 단순화하되 post-processing/FX로 훨씬 풍부하게 보일 수 있다.

### Final principle

> **결과는 정직하게, 감각은 과장한다.**

Simulation Truth와 Presentation Effect를 분리한다.

---

## A12. Rewind — RECONSTRUCTED / APPROVED

### User direction

frame history가 아니라 몇 초 전으로 돌아가는 실험 도구면 충분하다.

### Final direction

- recent 10 seconds
- 1-second granularity
- up to 10 snapshots
- 과거 snapshot에서 다시 simulation 계속 가능
- deterministic command replay가 아니라 actual state snapshot

---

# Part B — DAN-BALL research / discovery philosophy

## Q59 — DAN-BALL을 어떻게 참고할 것인가 — RECONSTRUCTED / APPROVED

### Options

1. Powder Game에 실제 구현된 기능만 참고
2. **DAN-BALL 전체 작품군을 아이디어 광산으로 사용**
3. 인터랙션 감각만 참고

### User selection

**2번.**

### Context

사용자는 과거 DAN-BALL 작품에서 당시 하드웨어/구현 제약 때문에 별개 게임으로 나뉘거나 작게 구현된 아이디어를 현대 GPU 환경에서 다시 검토할 가치가 있다고 봄.

### Final decision

Powder Game 1/2뿐 아니라 Earth Editor, Elemental Box 등 전체 작품군을 아이디어 후보의 원천으로 본다. 과거 기술적 제약에 대한 역사적 사실은 별도 연구로 검증하고 추정과 구분한다.

---

## Q60 — Reference idea 관리 — RECONSTRUCTED / APPROVED

### Options

1. 발견 즉시 ROADMAP에 넣음
2. **별도 IDEA_CANDIDATES 후보 풀**
3. 문서화하지 않고 필요할 때 참고

### User selection

**2번.**

사용자는 이 영역을 별도 심층 연구 과제로도 진행하기로 함.

### Final direction

Research/Promising/Rejected/Adopted 같은 candidate 상태를 둘 수 있으나 실제 구조는 연구 결과가 들어온 뒤 확정한다.

---

# Part C — World fantasy / discovery

## C1. Temperature-driven universal transformation — DIRECT / APPROVED

### Assistant framing

모든 Matter가 온도 등에 따라 공통 Transition 시스템에 참여하는 방향을 제안.

### User commentary

> “온도에 따라 반응하잖아. ... ONI 게임에서는 모든물질이 고체 액체 기체가 있어. 온도에 따라 변해. 불이 붙는다는것도 기름도 붙고 다른것도 가능한게 있을거야. 이런식으로 현실과 100% 같다기보단 우리 Doodle god 게임 아이디어 차용하기로 했잖아? 창의력있게 게임내에서 정합성있게 돌아가면 돼. 예를들어서 유리엔 불이 안붙더라도 철에는 불이 붙을수도 있어 특정 조건에서는.”

### Final interpretation

현실의 phase/combustion intuition은 참고하되 모든 Matter의 변화 graph는 Powdergame 세계 규칙으로 정의한다.

---

## C2. Fictional Matter is first-class — DIRECT / APPROVED

### User principle

> “실제로 있지도 않은 가상의 물질을 막 만들어내도 상관없어. 그게 게이머에게 이해가 되면 돼. 핵심은 물질이 상호작용을 하고 뭔가를 바꿔내고 영향을 주고 그런 본능적 재미야. 현실을 구현하는게 아니라 가상의 재밌는 놀이터야. 이게 진짜 핵심이다. 나만의 세계 창조. 현실을 고증하는게 아니라, 게임 내에서는 말이 되면 상관없는거지. 예를들어 비브라늄은 현실에 존재하지 않지만 이 게임엔 있어도 돼.”

### Impact

이 코멘트는 이후 `Game-Consistent Minimum Physics`의 최상위 제품 원칙이 됐다.

---

## C3. Discovery information level — DIRECT / APPROVED

### User concern

너무 많은 정답을 보여주면 창의력을 제한하지만 사전/업적이 도전 욕구를 자극하는 것도 사실.

### User direction

A와 B가 서로 반응했다는 것, 그리고 온도가 오르거나 내리는 등 **현상 수준**은 발견 후 알 수 있어도 된다.

### Hidden count decision

사용자:

> “그냥 2번으로 아직 발견하지 못한게 있다만 알려줘 몇갠지도 주지마.”

### Final decision

- Discovery는 현상 단위
- exact hidden condition/threshold는 기본 비공개
- 아직 발견하지 못한 것이 있다는 힌트는 가능
- 남은 정확한 개수는 비공개

---

# Part D — Interaction Lab developer tool

## D1. Initial automation idea — DIRECT

사용자는 material/rule이 추가될 때 자동화 도구로 관계를 분석하고 싶다고 제안.

처음에는 AI가 candidate를 만들거나 콘텐츠를 생성하는 Content Forge 쪽으로 논의가 넓어짐.

---

## D2. Critical user correction: the tool does NOT create Matter — DIRECT / APPROVED

### User correction

> “더 자세하게 말하면은 이 개발 도구는 물질을 만드는 도구가 아니야. 어떤 걸 다 정한 다음에 그걸 주면 이게 다른 물체랑 어떻게 상호작용하는지를 게임 내 엔진 같은 걸로 검증을 하는 거지.”

### Final role

Interaction Lab:

```text
already-designed Material/Rules
→ real Simulation Core
→ existing Materials + environment sweeps
→ observe actual outcomes
→ report unexpected/regression/emergent chains
```

AI는 결과 분석 보조자일 수 있지만 simulation truth는 실제 엔진 결과다.

---

## D3. Why Interaction Lab matters — DIRECT

### User rationale

> “새로운 물질이 들어가면 그 물질은 모든 거랑 상호작용을 할 테니까. ... 알지 못하는 것까지 알아내야 아 어디를 어떻게 조정해야겠다 이 정도 알 수 있어.”

### Meaning

Interaction Lab의 핵심은 기대한 정답 확인보다 **예상하지 못한 상호작용 발견**이다.

---

## Q69 (Lab) — Basic automatic scope — DIRECT / APPROVED

### Options

1. A↔B pair only
2. **A↔B + representative environment**
3. 처음부터 광범위 multi-material combinations

### User selection

**2번.**

대표 환경 예: normal/cold/hot/extreme heat, pressure differences, open/confined.

---

## Q70 (Lab) — Which simulator is truth? — DIRECT / APPROVED

### Options

1. CPU Reference
2. **actual GPU Production Simulation headless**
3. CPU+GPU always

### User selection

**2번.**

---

## D4. Lab became too large — DIRECT / DEFERRED

Assistant가 chunk별 isolated chamber, batch experiment, recording, adaptive exploration까지 제안하자 사용자가 우선순위를 재조정.

### User decision

> “너무 거창해진다면은 일단은 다음에 추가하는 걸로 하자. 이건 게임보다 더 중요한 건 아니야.”

### Final decision

- Interaction Lab idea는 보존
- Developer Tooling
- 현재 M0에서 구현하지 않음
- 본 게임 architecture를 Lab 때문에 복잡하게 만들지 않음
- headless GPU simulation hook 정도만 자연스럽게 유지

---

# Part E — M0 Thermal / world integrity

## Q71 (M0; numbering reused) — M0 phase-change breadth — DIRECT / APPROVED

### Options

1. Water ↔ Steam only
2. **Water ↔ Steam + solid/liquid/gas transition example**
3. many transformation graphs

### User selection

**2번.**

---

## Q72 — Representative transition — DIRECT / APPROVED

### Options

1. **Ice ↔ Water ↔ Steam**
2. Sand → Molten Sand → Glass
3. Metal ↔ Molten Metal

### User selection

**1번.**

### Reason

공통 Temperature/Transition 시스템이 고체·액체·기체 모두에서 동작한다는 것을 가장 직관적으로 검증.

---

## Q73 — Open boundary outside behavior — DIRECT / APPROVED

### Options

1. **domain 밖으로 나가면 Void 소멸**
2. invisible wall
3. 외부 별도 공간 보관

### User selection

**1번.**

---

## Q74 — EMPTY identity — DIRECT / APPROVED

### Options

1. **EMPTY는 Matter가 아님**
2. Vacuum Material
3. Air가 모든 빈 공간을 채움

### User selection

**1번.**

---

## Q75 — EMPTY의 Temperature/Pressure 의미 — DIRECT / APPROVED

### Options

1. **Matter가 없으면 Field도 물리적 매질로 비활성**
2. Temperature 비활성 / Pressure 존재
3. 둘 다 EMPTY에서 존재

### User selection

**1번.**

### Meaning

Dense array slot은 있을 수 있으나 EMPTY가 숨은 Air/Heat/Pressure medium으로 행동하지 않는다.

---

# Part F — Rule conflicts / phase ordering

## Q76 — Multiple matching rules — DIRECT / APPROVED AS 2A

처음 선택지:

1. 먼저 발견/실행된 Rule 하나
2. priority + one primary transition per Tick
3. same-Tick recursive chain

사용자는 성능이 매우 중요하므로 1과 2의 실제 비용 차이를 질문.

### User concern

모든 조건이 동시에 만족하는지 확인하고 다시 비교하는 구조라면 delay/pass/barrier가 늘어나는지 우려.

### Refined option 2A

> **Ordered First-Match + only necessary spatial Resolve**

```text
Material별 preordered rules
→ 높은 순서부터 condition check
→ first match
→ stop
```

모든 candidate를 모아서 다시 sort/resolve하지 않음.

Movement/spawn처럼 state ownership이 충돌하는 경우만 Resolve.

### User selection

**2A.**

---

## Q77 — Rule priority source — DIRECT / APPROVED

### Options

1. 모든 Rule에 숫자 직접 지정
2. **기본 category order + 필요한 Rule만 override**
3. file order 그대로

### User selection

**2번.**

### Final direction

runtime sorting 없음. load/compile 시 정렬.

---

## Q78 — Reaction ownership — DIRECT / USER CORRECTION / APPROVED

Assistant 제안:

1. 각 Material 내부에 reaction 작성
2. self transition은 Material, pair interaction은 global Reaction Catalog
3. 모든 변화를 global catalog

### User decision / correction

> “이거 2번, 3번은 현실적으로 구현 불가능할 수가 있어. 1번으로 하고 ... 충돌이 생겼을 때는 그냥 뭐 먼저 만난 순서대로 하든지 ... 약간의 랜덤성 부도 큰 상관없을 것 같은데 ... 글로벌 단에서는 ... 온도가 먼저 적용한다, 이런 식으로.”

### Final decision

- **1번: Material owns interaction rules**
- 완벽한 pair normalization을 강제하지 않음
- 세계 전체에는 몇 개의 coarse phase/category order만 둠
- 같은 phase 경쟁은 cheap first-match/order로 해결 가능
- intentional heavy RNG system은 만들지 않음
- One Cell = One Matter invariant만 확실히 보장

---

## Q79 — Tick causal philosophy — DIRECT / APPROVED

### Options

1. same-Tick strong causality with barriers
2. **loose causality; next Tick reflection allowed**
3. near-fully asynchronous

### User selection

**2번.**

### Principle

> **물리적 인과는 조금 늦어도 된다. 상태 무결성은 늦으면 안 된다.**

60 TPS에서 한 Tick 늦는 것은 자연스럽다면 허용하고 불필요한 full-world barrier를 피한다.

---

# Part G — Field cost / Active work

## G1. User raises full-field update overhead — DIRECT

### User question

매 Tick 모든 Cell의 Temperature/Pressure 등을 소수점 단위로 정확하게 읽고 쓰고 기록하면 overhead가 너무 크지 않은가. 최적화가 필요한가.

### Derived decisions

- State를 보유하는 것과 변화 history를 매 Tick 기록하는 것을 구분
- f32 baseline을 바로 포기하지 않음
- **Precision보다 Work를 먼저 줄임**
- Active Chunk
- Field-specific Active Set
- meaningful Event만 기록
- active compaction/shared memory/f16 등은 benchmark 후보

### Key principles

> **Dense State, Sparse Work.**

> **Precision은 희생하지 않고 Work를 줄인다.**

---

## G2. Minecraft-like local changes and GPU parallelism — DIRECT / MAJOR USER PRINCIPLE

### User analogy

Minecraft에는 구리 산화, 젖은 흙 등 block neighbor 변화가 있고 CPU/순서/random tick 구조가 대량 처리에서 제약이 있다고 이해하고 있음.

사용자는 Powdergame에서는 각 Cell이 바로 옆 Cell과만 상호작용하는 점을 GPU 병렬화에 적극 이용해야 한다고 강조.

### User principle

각 Cell이 서로 완전한 전역 영향을 주는 것이 아니라 local neighborhood만 보므로 **동시에 많이 돌려 실시간 세계를 만든다.**

### Final architecture

`Cell-Owned Local Interaction`:

```text
read self + neighbors
→ compute my outcome
→ write self next
```

neighbor를 직접 수정하지 않는다.

---

## Q80 — Matter interaction neighborhood — DIRECT / APPROVED

### Options

1. 4-neighbor
2. **8-neighbor**
3. Material별 arbitrary distance

### User selection

**2번.**

장거리 효과는 별도 Field/System으로 빼는 방향.

---

## Q81 — Stencil usage — DIRECT / APPROVED

### Options

1. 모든 system이 8칸 동일 사용
2. **8칸은 최대 범위, system별 고정 stencil**
3. Material별 arbitrary stencil

### User selection

**2번.**

예:

- Reaction: 8-neighbor
- Powder: down + diagonals
- Liquid: down/diagonal/lateral
- Field: own stencil

---

## Q82 — Slow changes scheduling — DIRECT / APPROVED

### Options

1. every active Cell every Tick
2. random subset
3. **fixed distributed schedule**

### User selection

**3번.**

### User emphasis

> “이런건 진짜 최적화해서 최소비용으로 돌아야한다.”

### Final direction

- FAST/MEDIUM/SLOW/VERY_SLOW tier 가능
- 좌표 기반 fixed distribution 가능
- 매 Tick 모든 셀 launch 후 대부분 early exit하는 가짜 최적화는 피함
- management cost가 실제 saving보다 비싸면 단순 방식을 우선

---

## Q83 — Slow rule progress — DIRECT / SUPERSEDED

### Initial options

1. scheduled check에서 즉시 변화
2. **small progress accumulation**
3. probabilistic immediate conversion

### Initial user selection

**2번.**

### Follow-up user correction

사용자가 곧바로 cost를 재검토:

> “이거 너무 커지는 거 아니야? ... 그냥 바뀌어도 사실 상관없거든. 왜냐면 마인크래프트도 그렇게 되는데. ... 비용 면을 가장 크게 생각해봐야 돼.”

### Superseded final decision

**Universal progress field를 기본으로 두지 않는다.**

```text
Copper
→ Weathered Copper
→ Oxidized Copper
```

처럼 Material stage transition으로 표현 가능.

정말 연속량이 gameplay에 필요할 때만 특수 state를 나중에 설계.

### Why this matters

사용자의 최상위 성능 원칙이 명확해짐:

> **하나하나가 아주 저렴하게 작동해야 큰 세계 전체가 돌아갈 수 있다.**

---

## User Performance Principle — DIRECT / TOP-LEVEL

사용자:

> “우리의 목표는 정말 최소한의 로직, 아주 저비용의 이런 방법을 이용해서 요점인 큰 세계를 만드는 거야. 그러니까 이 모든 게 다 돌아가려면 하나하나는 아주 저렴하게 작동해야 돼. 그리고 병렬화 가능한 게 핵심이야. 그래야 이 CPU 하나가 혼자서 열심히 순서 계산하고 있을 시간이 아니라, 동시에 여러 개를 돌려가지고 실시간으로 돌아가는 것처럼 할 수 있다.”

### Impact

이 발언이 `Minimum Sufficient Physics`와 GPU local architecture의 핵심 근거가 됨.

---

## Q84 — Chunk sleep timing — DIRECT / APPROVED

### Options

1. 변화 없는 즉시 Sleep
2. **짧은 안정 기간 후 Sleep**
3. 오래 Active 유지

### User selection

**2번.**

### User example / correction

사용자는 천천히 타는 Wood/도화선 같은 경우를 제시.

Wood가 조금씩 계속 타고 있다면 변화가 작더라도 Active Chunk여야 한다.

### Final decision

Sleep은 “느리다”를 뜻하지 않는다.

- combustion/cooling/movement/pressure가 실제 변화 중이면 관련 subsystem Active
- 일정 안정 기간 동안 의미 있는 변화가 없을 때만 Sleep
- exact sleep Tick count는 benchmark

> **Sleep은 물리를 생략하는 편법이 아니라 계산해도 결과가 없는 work만 제거하는 최적화다.**

---

# Part H — Read/Write and movement

## Q85 — Direct neighbor writes — DIRECT / APPROVED

### Options

1. neighbor 직접 수정 자유롭게 허용
2. **기본은 자기 Cell만 write**
3. 상황마다 자유 혼용

### User selection

**2번.**

### Final rule

> **Read Neighbors, Write Self.**

movement/swap/spawn처럼 ownership 자체가 바뀌는 경우만 예외.

---

## Q86 — Multiple Matter competing for same EMPTY — DIRECT / APPROVED

### Options

1. fixed direction priority
2. **cheap coordinate/tick stateless hash arbitration**
3. expensive fair competition

사용자는 1과 2의 성능 차이를 먼저 질문.

Assistant analysis:

- fixed order가 최저 비용
- cheap hash는 추가 random state 없이 정수 연산 몇 번 가능
- memory access를 늘리지 않는 stateless hash라면 전체 cost가 매우 작을 가능성
- collision에서만 계산 가능
- fixed bias를 줄일 수 있음

### User selection

**2번.**

### Caveat

실제 5090 benchmark에서 예상외로 비싸면 fixed/tick-alternating 방식으로 변경 가능.

---

## Q87 — Movement neighbor search — DIRECT / APPROVED

### Options

1. 가능한 방향 전부 검사
2. **우선순위대로 검사하고 first valid에서 종료**
3. Material별 arbitrary exploration

### User selection

**2번.**

Principle:

> **알 필요 없는 것은 읽지 않는다.**

---

## Q88 — Liquid lateral distance — DIRECT / APPROVED

### Options

1. immediate 1-cell simplistic
2. **한 Tick에서는 local 8-neighbor, 여러 Tick에 걸쳐 퍼짐**
3. 한 Tick에 4~8셀 장거리 scan

### User selection

**2번.**

큰 흐름은 수많은 local movement의 동시 병렬 누적으로 만든다.

---

# Part I — Gas/Liquid bulk and Density

## Q89 — Gas movement model — DIRECT / APPROVED WITH EXPANSION

초기 선택지:

1. Liquid movement를 반대로
2. **cheap Gas movement + material buoyancy tendency**
3. Pressure/Temperature를 항상 정교하게 읽는 gas solver

### User selection

**2번**, 단 중요한 추가 요청:

> 액체/고체가 흐르는 것도 부력 차이가 있으니 자세히 정해야 한다.

### Derived design

Gas만의 `upward_bias`에 머물지 않고 **공통 Density Rank + local displacement**로 확장.

- Powder ↔ Liquid 침강
- Liquid ↔ Liquid 층분리
- Gas ↔ Gas 상대적 부력
- 가상 Matter도 같은 규칙 사용
- STATIC은 일반 displacement 제외

---

## I1. User catches “Gas keeps chunk active forever” problem — DIRECT / MAJOR OPTIMIZATION

### User question

기체는 모든 방향으로 계속 움직이므로 Gas가 하나라도 있으면 Chunk가 영원히 Active가 되는 것 아닌가. Liquid/Gas가 아주 많을 때 더 덜 정교하지만 싼 계산이 가능한가.

### Key realization

`Steam ↔ Steam`처럼 같은 Matter끼리 위치를 바꾸는 것은 world state 관점에서 아무 변화가 없다.

### Final principle

> **Gas/Liquid는 존재하기 때문에 Active가 아니라 의미 있는 변화가 가능하기 때문에 Active다.**

안정된 bulk 내부는 Sleep 가능.

실제 Active:

- EMPTY interface
- different-Matter interface
- density inversion
- Temperature gradient
- Pressure gradient
- reaction frontier

### User response

> “좋아 좋아. 이게 오히려 더 맞는 방향이야.”

### Derived phrase

> **물질의 양이 아니라 변화량/변화 가능한 경계가 계산량을 결정하게 만든다.**

---

## Q90 — Density representation — DIRECT / APPROVED

### Options

1. f32 physical density
2. **small integer Density Rank**
3. only a few categories

### User selection

**2번.**

### User optimization challenge

사용자는 `A > B`, `A == B` 정도만 필요하면 실제 물리 계산 자체가 필요 없다고 지적.

### Final design

- Density = small integer rank
- per-cell 저장 안 함
- Material compact descriptor에서 가져옴
- EMPTY/non-movable early exit
- 실제 movable collision에서만 rank compare
- giant pair LUT는 단순 compare보다 memory read가 비쌀 수 있어 기본안 아님

### Key phrase

> **부력을 계산하지 않는다. 정렬한다.**

---

# Part J — Minimum Sufficient Physics

## J1. User asks whether Density trick generalizes — DIRECT

### User question

Density 방식이 다른 계산과 충돌하지 않는가. Temperature, electricity, radiation, visible light 같은 다른 물리에도 비슷한 저비용 아이디어를 적용할 수 있는가.

### Derived generalization

각 물리를 현실 공식 그대로 풀지 않고 **그 gameplay 현상에 필요한 최소 정보**만 계산.

Examples:

```text
Density
→ rank compare

Temperature
→ ΔT direction + cheap transfer + material thermal properties

Electricity
→ conductive? + strength/loss frontier

Radiation
→ intensity + attenuation/blocking

Gameplay Light
→ transmit/absorb/reflect + intensity

Pressure
→ local ΔP + resistance/rupture
```

### User response

> “정말 좋은 발견이야.”

### Named principle

**Minimum Sufficient Physics** / **Game-Consistent Minimum Physics**.

---

## Q91 — Property representation policy — DIRECT / APPROVED

### Options

1. 하나의 numeric 체계로 통일
2. **각 system이 필요한 가장 싼 representation 사용**
3. 처음 전부 f32 후 최적화

### User selection

**2번.**

### User addition

각 요소에 어떤 representation이 좋은지 생각하되 실제 test에서 병목이 확인되면 그에 따라 결정을 바꾸자.

### Final rule

```text
continuous value needed → f32 or proper numeric
ordering only           → integer rank
boolean only            → bit
few states              → small enum
```

그리고 benchmark-driven optimization.

---

## J2. Heat cannot be rank-only — DIRECT

### User question

열은 방향 비교만이 아니라 전도율, 어느 물체가 더 많은 열을 품을 수 있는지 같은 차이가 필요한데 단순 비교 철학으로 가능한가. 전기/압력/폭발도 전달되다 약해지거나 보존되는 차이가 있다.

### Clarification

Minimum Sufficient Physics는 “모든 걸 `A>B`로 만든다”는 뜻이 아님.

**방향은 비교로, 양이 gameplay에 중요하면 싼 numeric transfer로.**

Temperature에 필요한 최소 후보:

- Temperature
- conductivity
- heat capacity
- meaningful ΔT/deadband

Electricity:

- conductive
- electrical strength
- loss/resistance
- source가 있으면 replenishment

Pressure:

- local ΔP
- pressure does not simply decay to zero in sealed space
- equilibrium/vent/structure work로 해소

Explosion:

- 별도 complex solver보다 Heat + Pressure source를 inject하고 기존 systems가 결과를 만듦

---

## User Physics Principle — DIRECT / TOP-LEVEL

사용자:

> “실제 우리 자연에서 발생하는 현상들이 그대로 반영되면 좋지만, 꼭 그럴 필요가 없다는 거야. 그냥 그거에 참고를 해가지고 우리 게임 나름대로 어떤 논리를 통해서 상호작용이 일어나면 된다는 거지. 그리고 그건 정말 최소화된 비용이어야 돼.”

### Final interpretation

현실은 intuitive seed.  
Powdergame 내부 consistency + fun + low cost가 최종 기준.

---

## Q93 — Conservation — DIRECT / APPROVED

### Options

1. 실제 보존법칙 최대한 엄격히
2. **gameplay에 중요한 범위에서 approximate/local conservation**
3. conservation 거의 무시

### User selection

**2번.**

### Final phrase

> **로컬에서는 납득 가능하게, 글로벌에서는 회계하지 않는다.**

Magic Crystal이 Heat를 만들거나 Void가 energy를 없애는 세계 Rule도 허용.

---

## Q94 — Field transfer write pattern — DIRECT / APPROVED

### Options

1. pairwise exact A−10/B+10 transaction
2. **각 Cell이 neighbors를 보고 자기 Next field만 계산**
3. hybrid from M0

### User selection

**2번.**

### User addition

> “이 전체 시스템 내에서 정말 완벽한 정합성 가질 필요가 없어. 그냥 상식적으로 이상하지 않으면 돼.”

### Final impact

Temperature/Pressure도 `Read Neighbors, Write Self` 기본 적용. exact conservation보다 parallelism과 stable intuitive behavior 우선.

---

## Q95 — Thermal/default field stencil — DIRECT / APPROVED

Assistant가 Thermal propagation을 4-neighbor baseline으로 제안.

### User response

> “그럼 일단 일반으로 하자.”

이후 사용자:

> “그래, 나머지도 기본적으로는 내 방향으로 하고.”

### Final interpretation

기본 Field propagation은 **4-neighbor**에서 시작.

- Temperature → 4
- Pressure → 4
- future electricity/diffusive radiation → 4 baseline
- Matter direct Reaction → 최대 8
- Movement → behavior-specific
- Light/beam → dedicated direction

부족함이 실제로 보일 때만 더 비싼 stencil benchmark.

---

## Q96 — Close M0 physics design — DIRECT / APPROVED

Assistant가 M0 physics를 여기서 닫고 Evidence Gate/implementation으로 넘어갈 것을 제안.

### User approval

이 정도면 테스트를 할 수 있을 것 같다고 판단.

### M0 direction

- 더 많은 미래 물리를 추가 설계하지 않음
- 현재 최소 세계를 실제 GPU에서 만들고 측정
- baseline을 보고 다음 결정

---

# Part K — M0 Evidence Gate confirmation

Assistant가 `M0 — First World`를 다음 G0~G9로 제안:

- G0 Runtime
- G1 World Integrity
- G2 Local Movement
- G3 Density
- G4 Thermal / Reaction
- G5 Pressure
- G6 Parallel Integrity
- G7 Sleeping
- G8 Performance Evidence
- G9 Product Validation

### User approval

> “좋아, 이거 해보자고?”

이후 M0를 이 구조로 확정.

중요: 숫자 performance target은 baseline 전에 억지로 박지 않는다.

---

# Part L — Documentation philosophy

## L1. User rejects lossy summary — DIRECT / APPROVED

### User request

> “문서가 아주 중요해. 이거 내용 빼먹지 말고. 우리 질문과 답변 요약해서 적지도 말고, 그냥 전부 다 적는 건 어때? 좀 말은 좀 다듬을 수 있어도.”

### Decision

문서를 단순 Q&A transcript로 복사하지는 않지만 **정보를 버리는 요약을 하지 않는다.**

정식 SPEC에는 현재 계약을 구조화해 쓰고, Design History에는 질문/선택/코멘트/대안/수정 맥락을 보존한다.

> **요약하지 않는다. 정리한다.**

---

## L2. User asks to preserve explicit selection provenance — DIRECT / APPROVED

### User request

> “니가 물어본 부분, 그다음에 내가, 사용자인 내가 답변한 부분, 이렇게 해서 내가 추가로 코멘트 준 부분이나 내가 선택했다는 것도 명확하게 알 수 있도록 해주면 좋을 것 같아.”

### Decision

Design History는 가능한 경우:

- Assistant Question
- Options
- Assistant Recommendation
- User Selection
- User Commentary
- Discussion / correction
- Final Decision
- Status / Superseded
- Reflected Docs

를 남긴다.

---

## L3. Current Q&A outranks older pre-implementation docs — DIRECT / APPROVED

### User correction during documentation

> “지금 질문과 답변한 게 사실 더 강력한 증거거든. 요거에 따라서 기존 문서를 좀 변경해도 괜찮아. 아직은 실제로 개발한 게 아니니까.”

### Final documentation policy

아직 구현 전이므로 기존 문서를 역사 자료라며 그대로 방치하지 않는다.

현재 Q&A에서 명시적으로 합의된 내용이 더 강한 evidence이므로:

- README 최신화
- 기존 `00_USER_VISION.md` 최신화
- `01_MASTER_DESIGN_REPORT.md`의 충돌하는 초기 가설 수정
- 최신 SPEC/ADR/Performance/Milestone을 authority로 사용

Git history가 과거 문서를 보존하므로 repository의 현재 `main`은 가능한 한 **현재 정답을 말하게 만든다.**

---

# Part M — Deferred / Benchmark-driven decisions

다음은 의도적으로 숫자/구현을 아직 고정하지 않았다.

- exact Chunk sleep delay
- chunk size final choice (initial 64)
- f16 use 여부
- Material property exact bit packing
- shared-memory tile 여부
- indirect dispatch / active compaction
- subtile active mask
- exact thermal deadband
- exact conductivity/heat-capacity representation
- exact Pressure transfer coefficients
- phase transition yield values
- arbitration hash implementation
- Interaction Lab detailed architecture

공통 이유:

> **실제 RTX 5090 baseline과 gameplay test를 보기 전에 복잡한 최적화를 추측으로 영구 고정하지 않는다.**

---

# Part N — Consolidated User Principles

이 세션에서 사용자가 직접 반복해서 강조한 핵심을 제품/엔진 언어로 보존한다.

### 1. 나만의 세계 창조

현실 재현이 목표가 아니다. 가상 Matter/가상 물리도 세계 안에서 이해되면 된다.

### 2. 본능적 상호작용의 재미

Matter가 만나고, 영향을 주고, 바꾸고, 그 결과가 다시 다른 결과를 만드는 것이 핵심이다.

### 3. 큰 세계가 우선

모든 기능이 동시에 돌아가려면 각 Cell/Rule은 아주 싸야 한다.

### 4. GPU 병렬성이 핵심

CPU가 순서대로 Cell을 해석하는 구조보다 local independent work를 대량 병렬로 처리한다.

### 5. 완벽한 정합성은 필요 없음

상식적으로 이상하지 않고 상태가 깨지지 않는다면 exact physical conservation/replay를 요구하지 않는다.

### 6. 변화 없는 계산은 하지 않음

Gas/Liquid가 존재한다는 이유만으로 영원히 움직이지 않는다. stable bulk는 Sleep 가능하다.

### 7. 실제 병목을 보고 최적화

각 physics representation은 가장 싼 충분한 형태를 사용하고, 이후 benchmark에서 병목이 확인된 subsystem만 더 공격적으로 최적화한다.

---

# Part O — Resulting engine thesis

이 긴 Q&A에서 도출된 현재 핵심은 다음과 같다.

> **Game-Consistent Minimum Physics**
>
> 현실의 자연현상을 직관의 출발점으로 사용하되 현실 공식을 구현 목표로 삼지 않는다. 플레이어가 이해할 수 있고 서로 연쇄작용하며 재미있는 Powdergame 고유의 법칙을 만든다. 각 법칙은 가능한 최소 상태·최소 메모리 접근·최소 local operation으로 구현한다.

그리고 실행 구조는:

```text
finite dense world
        ↓
active/sleep work selection
        ↓
local neighborhood read
        ↓
cheap comparison / transfer / first-match
        ↓
write self next
        ↓
ownership change only → minimal Claim/Resolve
        ↓
GPU massive parallelism
        ↓
small rules combine into emergent world
```

이 설계가 실제 RTX 5090 M0에서 성립하는지를 다음 단계에서 검증한다.
