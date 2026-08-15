# Powdergame Testing

이 문서는 Powdergame Simulation의 테스트 철학을 정의한다.

---

## 1. Testing Goal

Powdergame은 bit-perfect replay를 correctness의 유일한 기준으로 사용하지 않는다.

테스트 목표는 다음이다.

- 세계 invariant가 깨지지 않음
- local rule이 의미대로 동작
- GPU 병렬화가 state corruption을 만들지 않음
- 최적화 전후 gameplay 의미가 유지됨
- performance regression을 발견할 수 있음
- 사용자가 실제로 느끼는 Product Gate까지 연결

---

## 2. Test Layers

### Unit / Rule Tests

작은 world에서:

- Material property
- transition
- First-Match
- density comparison
- ignition condition
- rupture threshold

등의 local logic을 확인한다.

### Invariant Tests

반드시 자동화한다.

- One Cell = Max One Matter
- valid material id
- no illegal duplicate ownership
- no out-of-bounds writes
- no NaN/Infinity runaway
- Claim/Resolve 결과가 single winner
- finite boundary 처리

### Semantic Simulation Tests

exact pixel checksum보다 의미를 검사한다.

예:

```text
Sand placed over EMPTY
→ eventually moves downward
```

```text
heavy movable Matter over lighter movable Matter
→ relative ordering tends toward heavy below
```

```text
Ice heated past transition condition
→ Water
```

```text
sealed Steam expansion
→ Pressure increases
```

### Scenario Tests

반복 가능한 M0 scenario:

- Sand Fall
- Water Flow
- Fire / Heat
- Pressure Burst
- Heavy Mixed World

### Performance Tests

`PERFORMANCE.md`의 metric을 기록한다.

### Product Validation

자동화할 수 없는 항목:

- Powder movement가 보기 좋은가
- Liquid/Gas가 너무 인위적으로 느껴지지 않는가
- 작은 Rule chain이 재미있는가
- stable bulk sleep이 world를 죽인 것처럼 느껴지지 않는가
- 다음 실험 욕구가 생기는가

최종 Milestone `ACHIEVED`는 사용자 승인 필요.

---

## 3. CPU Reference Tests

CPU Reference는 GPU의 exact oracle이 아니다.

CPU/GPU 비교 시:

- same invariant
- same qualitative behavior
- same transition direction
- expected range

를 확인한다.

예:

GPU pile shape와 CPU pile shape가 pixel-perfect하게 같지 않아도 된다.

---

## 4. GPU-specific Tests

GPU path에서는 다음을 특히 검증한다.

### Ownership contention

여러 source가 같은 target을 원할 때:

- winner exactly one
- loser remains valid
- no duplicate
- no lost unrelated Matter

### Chunk boundary

64×64 chunk 경계에서:

- movement
- Temperature
- Pressure
- wake propagation

이 끊기지 않아야 한다.

### World boundary

outer BLOCK 제거 후 Matter가 domain 밖으로 빠질 때:

- no memory access beyond domain
- Matter correctly disappears to Void

### Sleep/Wake

- stable chunk eventually sleeps
- incoming influence wakes required system
- slowly burning Wood does not sleep incorrectly
- stable Water/Steam bulk can sleep

---

## 5. Thermal Tests

M0 baseline:

- f32 finite values
- hot-to-cold direction is intuitive
- EMPTY does not conduct by itself
- 4-neighbor propagation works across chunk boundaries
- Ice ↔ Water ↔ Steam transitions
- no unbounded numeric runaway

Exact global heat conservation is not required.

If thermal deadband is introduced, test that:

- small irrelevant differences settle/sleep
- meaningful threshold reactions are not lost

---

## 6. Density Tests

Density Rank is gameplay ordering, not SI density.

Test:

- EMPTY fast path
- STATIC no normal swap
- heavier movable vs lighter movable ordering
- equal rank no density swap
- Sand/Water
- Oil/Water
- Gas rank example
- large stable bulk does not keep unnecessary work alive

---

## 7. Reaction Tests

- Material only scans its own compiled rule range
- ordered First-Match
- category priority applied at load/compile time
- self-write path has no neighbor mutation race
- ownership-changing effect goes through Claim/Resolve
- combustion common grammar works for at least Wood and Oil

---

## 8. Pressure Tests

Representative causal test:

```text
Water
→ heating
→ Steam expansion
→ insufficient room
→ Pressure
→ weak wall rupture
→ vent
```

Verify that the chain is composed from generic systems rather than a dedicated scripted explosion.

---

## 9. Regression Policy

A Milestone can enter `REGRESSION` after previously being achieved.

Performance optimization is not accepted if it silently breaks:

- world invariant
- transition semantics
- wake behavior
- user-visible interaction quality

Optimization benchmark and behavior regression should be evaluated together.

---

## 10. Evidence Recording

Important validation runs should record:

- commit SHA
- build/config
- hardware/driver
- world config
- test/benchmark scenario
- pass/fail
- relevant metrics
- artifact/screenshot/trace reference where useful

`STATUS.md` may later contain a machine-generated facts block from this evidence.

---

## 11. Interaction Lab relationship

Future Interaction Lab can become a high-volume exploratory/regression testing layer, but M0 does not depend on it.

Current tests should be able to directly construct small world fixtures and headless GPU scenarios without needing the full Lab.
