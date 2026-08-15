# 2026-08-16 Expanded Fictional Matter Research Intake

## Status

- Type: research intake / non-authoritative
- Adoption: `REFERENCE + CANDIDATE`
- Authority: existing Vision / ADR / SPEC / MILESTONES remain authoritative

추가 연구자료 `Powdergame 확장 가상물질 대사전: 강화형·신규 물질·제작·생태·경제·진행도 통합 설계`를 추적한다.

## Source manifest

- Original container: ChatGPT research widget JSON
- Original size: 462,126 bytes
- SHA-256: `ac6fe9c35a25b422d6a30503f9fa09de8f3825430ae150b89704cc6f5733cebe`
- Extracted report size: 51,909 bytes
- Extracted report SHA-256: `c0c918af677899e053c662f4c0738f0f71b47fe25cb9ceb61f69081c9eff521b`

## Scope

이 자료는 기존 OM-001~OM-100을 전제로 다음을 확장한다.

- 강화 파생형 VX-001~VX-010
- 신규 Original Matter OM-101~OM-180
- 열/압력/피로, 유체/표면, 반응/정제, 생태, 전기/자기, 광학/복사, 정보/확률, 건설/재활용 후보
- `capacity8`, `fatigue8`, `purity8`, `state_flags` 후보
- 생산·정제·재활용·바이옴·경제·진행도 연결 아이디어

## High-value design findings

### Capacity / fatigue / byproduct

강한 Matter에 무한 효과를 주기보다 다음 구조를 쓰는 아이디어가 특히 가치가 높다.

```text
input
→ capacity/fatigue accumulation
→ threshold
→ reduced performance or failure
→ byproduct
→ recovery/recycling
```

`capacity8`나 `fatigue8`를 모든 Cell에 바로 추가한다는 뜻은 아니다. 현재 Minimum Sufficient Representation 원칙에 따라 기존 Field/packed state로 가능한지와 GPU memory/bandwidth 비용을 먼저 검증한다.

### Strong Matter needs counters

후보 Material은 가능하면 다음 질문에 답해야 한다.

```text
무엇을 입력받는가?
무엇이 누적되는가?
어떤 임계값을 넘는가?
무엇을 출력하는가?
무엇이 멈추게 하는가?
실패 뒤 무엇이 남는가?
그 부산물은 다시 어디에 쓰이는가?
```

### New Field must open a family, not one Matter

Electricity, Radiation, Light, Biology, Information, Space-Time 같은 미래 Field는 한두 개 Material 전용으로 추가하지 않는다. 저장체·전달체·센서·액추에이터·차폐/절연·실패 모드 등 여러 Material이 공유할 때만 구현 가치를 검토한다.

## Candidate ranges

- VX-001~VX-010: 기존 시그니처 강화형
- OM-101~110: Thermal / Pressure / Fatigue
- OM-111~120: Fluid / Wetting / Density
- OM-121~130: Reaction / Refining
- OM-131~140: Ecology / Environment
- OM-141~150: Electricity / Ion / Magnetism
- OM-151~160: Optics / Radiation
- OM-161~170: Information / Probability / Meta
- OM-171~180: Construction / Recycling / Economy

이 번호는 research candidate namespace이며 Material Registry의 예약 ID가 아니다.

## Source-recommended near-term pool

원문은 기존 열·압력·밀도·이동·상전이·태그 검사로 비교적 표현하기 쉬운 후보로 다음 24종을 우선 제안한다.

`OM-101, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 115, 117, 119, 120, 124, 126, 127, 129, 171, 175, 176, 177`

이는 연구자의 우선순위 제안일 뿐 현재 M0 backlog 또는 milestone 추가가 아니다.

## Do not adopt directly

다음은 모두 후보이며 구현 계약이 아니다.

- global per-cell `capacity8` / `fatigue8`
- exact density / hardness / `k*` / conductivity values
- radius, tick counts, cycle life, recycling percentage
- crafting tiers / rarity / economy
- Value-Crystal / Entropy-Slag progression
- Electricity / Radiation / Light / Biology / Information / Space-Time systems

현재 Evidence Gate를 확장하지 않는다.

## Recommended role

기존 연구와 역할을 다음처럼 본다.

```text
Reality reference
→ broad material pool
→ mechanic taxonomy + OM-001~100
→ this source: counters, lifetime, byproducts + OM-101~180
→ Powdergame-specific derived candidates
→ user approval
→ ADR/SPEC/content registration
```

이번 자료의 가장 좋은 활용법은 물질 수를 단순히 늘리는 것이 아니라, **기존 후보들의 강한 효과에 비용·카운터·수명·부산물을 붙이는 설계 검수표**로 사용하는 것이다.
