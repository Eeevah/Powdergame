# Powdergame Determinism Specification

이 문서는 Powdergame이 어떤 수준의 재현성을 요구하고 어떤 비결정성을 허용하는지 정의한다.

---

## 1. Core Principle

Powdergame은 bit-perfect deterministic simulation을 목표로 하지 않는다.

> **No intentional randomness; no performance sacrifice for bit-perfect replay.**

의도적으로 난수를 추가해 세계를 흔드는 것이 기본 방향은 아니다.

다만 GPU 병렬 실행, floating-point approximation, local arbitration 등에서 미세한 차이가 발생할 수 있으며, 그 차이를 제거하기 위해 큰 성능 비용을 지불하지 않는다.

목표는:

> **Non-exact but stable.**

이다.

---

## 2. Acceptable Non-exactness

허용되는 것:

- floating-point 근사 차이
- GPU execution order 차이
- 같은 coarse rule 안에서 여러 유효 후보가 경쟁할 때 서로 다른 valid winner
- local heat/pressure 전달의 미세한 수치 차이
- parallel movement에서 pile/flow 모양이 조금 달라지는 것
- 같은 초기 상황에서 microscopic detail이 완전히 동일하지 않은 것

이런 차이를 없애기 위해 simulation throughput, GPU parallelism, memory locality를 크게 희생하지 않는다.

---

## 3. Unacceptable Behavior

다음은 비결정성으로 정당화할 수 없다.

- 한 Cell에 Matter 두 개
- race 때문에 설명 없이 Matter 복제
- logic/race bug 때문에 설명 없이 Matter 소멸
- invalid state
- NaN/Infinity runaway
- out-of-bounds memory access
- GPU race 때문에 algorithm 자체가 깨짐
- save/load가 유효한 world state를 복구하지 못함

> **정확한 replay보다 상태 무결성이 우선한다.**

---

## 4. Stateless Local Arbitration

같은 destination을 여러 Matter가 원할 때 고정 방향 bias를 줄이기 위해:

```text
cheap_hash(target_position, tick)
```

형태의 stateless arbitration을 사용할 수 있다.

이것은 gameplay RNG system이 아니다.

조건:

- per-cell RNG state 없음
- global RNG synchronization 없음
- 추가 random-state memory 없음
- 실제 ownership collision에서만 사용 가능
- 결과는 One Cell = One Matter invariant를 만족

Fixed Direction baseline과 실제 RTX 5090 benchmark를 비교할 수 있다. cheap hash가 예상외로 비싸다면 더 단순한 arbitration으로 바꿀 수 있다.

---

## 5. Same-phase Rule Competition

같은 Cell의 같은 coarse phase에서 여러 reaction이 동시에 유효할 수 있다.

기본 정책:

- precompiled material rule order
- ordered first-match
- 저비용 neighbor ordering

모든 candidate를 모은 뒤 global deterministic priority resolution을 하지 않는다.

세계 Rule상 여러 결과가 모두 유효하다면 그중 하나가 저비용 정책으로 선택되어도 된다.

---

## 6. CPU Reference Relationship

CPU Reference는 GPU Production의 bit-exact oracle이 아니다.

비교는 다음을 중심으로 한다.

- invariant
- semantic outcome
- expected behavior range
- stable transition
- no corruption

예:

```text
CPU와 GPU 모두
Sand가 중력 방향으로 침강한다.
Water와 Oil이 density ordering에 따라 층을 만든다.
Ice/Water/Steam transition이 유효하다.
```

이면 의미 있는 reference 비교가 된다.

모래더미의 정확한 pixel checksum이 같을 필요는 없다.

---

## 7. Testing Philosophy

exact checksum을 correctness의 유일한 기준으로 사용하지 않는다.

### Invariant Test

- single Matter occupancy
- valid material id
- finite field values
- valid claim/commit
- no illegal duplicate ownership

### Semantic Test

- Powder가 아래로 침강
- Liquid가 local하게 흐름
- 더 무거운 movable Matter가 상대적으로 아래를 선호
- hot/cold 상태가 올바른 방향으로 완화
- pressure가 높은 곳의 영향이 낮은 곳/약한 구조로 전달

### Range / Statistical Test

미세하게 다른 valid 결과가 허용되는 경우 exact pixel output보다 허용 범위/특징을 검사할 수 있다.

---

## 8. Replay / Rewind

GPU Simulation이 bit-perfect deterministic하지 않으므로 Rewind를 command replay에만 의존하지 않는다.

현재 Rewind 방향:

- 실제 world state snapshot 보존
- 최근 약 10초
- 1초 granularity
- 과거 snapshot으로 복귀 후 새 실험 가능

즉 Rewind는 과거를 정확히 재계산하는 시스템이 아니라 실제 과거 state를 보존하는 시스템이다.

---

## 9. Randomness Policy

기본 원칙은 의도적인 RNG를 사용하지 않는 것이다.

다만 다음은 구분한다.

### Gameplay Randomness

게임 디자인상 확률이 재미를 만드는 미래 Rule.

→ 별도 명시적 시스템으로 추가해야 한다.

### Arbitration Variation

좌표+Tick hash 등 ownership collision bias를 없애기 위한 저비용 tie-breaker.

→ gameplay randomness가 아니다.

### GPU Non-exactness

parallel ordering/float approximation에서 자연스럽게 발생하는 미세 차이.

→ 성능을 희생해서 제거하지 않는다.

---

## 10. Final Principle

Powdergame은 같은 입력에서 매번 한 픽셀까지 똑같은 세계를 만드는 것이 목표가 아니다.

목표는:

> **같은 세계 법칙이 안정적으로 적용되고, 결과가 유효하며, 상태가 깨지지 않고, 플레이어가 인과관계를 이해할 수 있는 것.**

정확한 재현성은 그 목표를 방해하지 않는 범위에서만 추구한다.
