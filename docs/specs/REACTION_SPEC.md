# Powdergame Reaction Specification

이 문서는 Matter의 상호작용과 Rule 실행 모델을 정의한다.

---

## 1. 목표

Reaction 시스템은 다음을 동시에 만족해야 한다.

- Material 수가 늘어도 runtime hot path가 커지지 않을 것
- Rule authoring은 사람이 읽을 수 있을 것
- AI가 content authoring을 도울 수 있을 것
- 일반 reaction은 GPU에서 local하게 병렬 실행될 것
- 완벽한 전역 순서보다 낮은 비용을 우선할 것
- 한 Cell = 한 Matter invariant는 절대 깨지지 않을 것

---

## 2. Material-owned Interaction Rules

현재 기본 방향은 각 Material이 자기 interaction 목록을 소유하는 것이다.

예:

```text
Oil
- hot neighbor → combustion candidate
- Acid neighbor → special response

Metal
- Acid neighbor → corrosion transition
- extreme heat → molten transition
```

A+B 관계를 반드시 하나의 거대한 global pair database에 정규화하지 않는다.

그 이유:

- 콘텐츠 작성/확장이 단순함
- 해당 셀은 자기 Material의 작은 Rule 목록만 보면 됨
- GPU에서 전체 Reaction Catalog를 검색하지 않아도 됨
- 중복이 생겨도 저비용 ordering으로 해결 가능

중복 제거를 위해 runtime 구조를 비싸게 만들지 않는다.

---

## 3. Transitions vs Neighbor Interactions

작성 관점에서 다음을 구분할 수 있다.

### Self Transition

자기 상태/Field만으로 결정 가능.

```text
Water temperature > threshold
→ Steam
```

### Neighbor Interaction

주변 Matter/Field 조건이 필요.

```text
Metal sees Acid neighbor
→ Corroded Metal
```

두 종류 모두 최종적으로 Material별 compiled rule range 안에서 실행될 수 있다.

---

## 4. Data-driven Direction

초기 구현은 다음 방향을 갖는다.

> **Material Registry + engine-defined physics, later evolvable toward Rule DSL.**

처음부터 복잡한 Rule DSL editor를 만들지 않는다.

필수 기반:

- Material ID가 엔진 곳곳에 하드코딩되지 않음
- condition/effect 개념 존재
- content schema version
- property/tag 기반 authoring 가능
- runtime에는 Material별 작은 rule set으로 compile 가능

장기적으로:

```text
Material Data
→ Condition / Effect / Rule API
→ Simulation Core
```

에서:

```text
Material Data / Rule DSL
→ same Rule API
→ Simulation Core
```

로 발전할 수 있어야 한다.

---

## 5. No Material-name Branch Explosion

피해야 할 기본 패턴:

```text
if material == WOOD ...
if material == OIL ...
if material == COPPER ...
```

가능하면 property/tag/compiled rule을 사용한다.

예:

```text
flammable
ignition_threshold
pressure_resistance
conductive
```

다만 tag를 runtime에서 무거운 dynamic query로 검색하지 않는다. content compile/load 단계에서 실제 Material rule table로 평탄화할 수 있다.

---

## 6. Ordered First-Match

한 Cell에서 여러 핵심 변화가 동시에 가능할 수 있다.

예:

```text
X
- extreme heat → Molten X
- Acid contact → Corroded X
- ignition condition → Combusting X
```

모든 matching rule을 별도 list로 만든 뒤 다시 priority resolve하지 않는다.

기본은:

```text
pre-ordered rules
→ test rule 1
→ match? yes → primary transition selected → stop
→ no → rule 2
→ ...
```

즉 **First-Match**.

불필요한 후속 rule은 읽지 않는다.

---

## 7. Coarse Global Phase Order

개별 Material pair마다 거대한 숫자 priority를 설정하지 않는다.

세계 전체에는 소수의 coarse category order만 둔다.

현재 개념 예:

```text
Critical / Destroy
Phase Transition
Special Reaction
Combustion
State Change
```

정확한 최종 phase/order는 구현/benchmark에서 조정 가능하지만 원칙은 다음과 같다.

- category는 소수
- compile/load 때 미리 정렬
- runtime sort 없음
- 정말 필요한 Rule만 override 가능
- 숫자 priority jungle을 만들지 않음

Loose Causal Phase 철학 때문에 subsystem 전체의 강한 순차 barrier와는 구분한다.

---

## 8. Same-phase Competition

같은 coarse phase에서 여러 유효 reaction이 경쟁할 경우 완벽한 전역 결정성을 위해 큰 비용을 쓰지 않는다.

허용 가능한 저비용 정책:

- precompiled rule order
- first matching neighbor order
- stateless local arbitration where actual spatial ownership competes

결과가 세계 Rule상 둘 다 유효하다면 실행마다 미세하게 다른 승자가 나올 수 있다.

의도적인 random gameplay를 추가하려는 것이 아니다.

---

## 9. Read Neighbors, Write Self

일반 reaction은 다른 Cell을 직접 수정하지 않는다.

예:

```text
Wood
→ neighbor hot/combusting Matter 관찰
→ 자기 combustion state/transition 결정
```

```text
Metal
→ neighbor Acid 관찰
→ 자기 Corroded Metal transition
```

이 구조는 다수의 GPU thread가 같은 target Cell에 쓰는 race를 줄인다.

---

## 10. Effects

Rule은 필요한 최소 effect vocabulary를 가질 수 있다.

예:

```text
TransformSelf
Set/ClearState
Add/RemoveHeat
Add/ReducePressure
RequestSpawn
RequestMove
RequestSwap
EmitSimulationEvent
```

정확한 API는 구현 시 확정한다.

중요한 구분:

### Self Effect

자기 Next state만 변경.

→ 가능한 한 직접 처리.

### Spatial Ownership Effect

다른 Cell의 ownership을 바꾸거나 새 Matter를 요청.

→ Claim/Resolve 필요.

---

## 11. Fire / Combustion

M0에서 Fire는 단순 Matter pair recipe가 아니다.

예:

```text
Wood/Oil
+ sufficient thermal condition
→ combustion state/transition
→ Heat
→ Smoke request
→ flame presentation event
```

Wood와 Oil은 서로 다른 movement/property를 가져도 같은 combustion grammar를 이용할 수 있다.

현실 Oxygen은 자동 요구사항이 아니다.

---

## 12. Reaction Discovery Metadata

플레이어 Discovery는 조합별 문장을 엔진 코드에 하드코딩하지 않아도 된다.

Rule/effect가 semantic event를 내보내면 Discovery 시스템은 실제로 관찰된 현상을 기록할 수 있다.

예:

```text
A ↔ B
- TemperatureIncrease
- PressureGenerated
- TransformationObserved
```

플레이어에게 정확한 threshold/계수는 기본적으로 공개하지 않는다.

발견의 단위는 **현상**이다.

### Hidden knowledge

사전은 아직 발견하지 못한 상호작용이 있다는 정도는 알려줄 수 있지만 남은 정확한 개수/조건은 보여주지 않는다.

---

## 13. Authoring vs Runtime

사람/AI는 개발 단계에서 Material과 Rule을 작성할 수 있다.

AI가 runtime에서 “A와 B가 만나면 어떻게 할까?”를 추론하는 구조는 현재 목표가 아니다.

```text
Human / AI authoring
→ content definition
→ validation/compile
→ game runtime executes cheap rules
```

AI는 콘텐츠 후보 제작/검토를 도울 수 있지만 runtime simulation truth는 정해진 Rule과 Simulation Core가 만든다.

---

## 14. Deferred Interaction Lab

향후 Interaction Lab은 완성된 Material/Rule을 실제 GPU Production Simulation에 넣어 기존 Matter/대표 환경과 자동 실험하는 개발자 도구다.

중요:

- Material을 자동 생성하는 도구가 아님
- runtime 게임 시스템이 아님
- 실제 Simulation Core를 사용
- pair + representative environment를 기본 탐색 범위로 고려
- 예상 밖 chain/regression을 찾는 것이 목적
- 상세 구현은 현재 보류

---

## 15. Rule Cost Principle

새 Rule이 많아져도 모든 Cell이 모든 Rule을 검색해서는 안 된다.

목표:

```text
cell material_id
→ that Material's small compiled rule range
→ required local neighbor only
→ first match
```

Rule correctness를 위해 별도의 full-world GPU pass를 추가하는 것은 기본적으로 피한다. 정말 필요하다면 benchmark로 비용을 정당화한다.

---

## 16. World Consistency over Scientific Correctness

Reaction의 최종 기준:

- 현실 과학과 같을 필요 없음
- 플레이어가 원인/결과를 학습할 수 있어야 함
- 같은 조건에서 상식적으로 납득할 수 있어야 함
- 다른 Rule과 연결될 수 있어야 함
- 계산 비용이 충분히 낮아야 함

현실에 없는 Material과 Reaction은 허용하며 오히려 세계 창조의 중요한 부분이다.
