# Powdergame — User Vision

> 이 파일은 기존 경로 호환을 위해 유지한다. **현재 최상위 비전 문서는 [`vision/USER_VISION.md`](vision/USER_VISION.md)다.**
>
> 2026-08-15 Foundation Design Session에서 사용자가 직접 선택하고 보완한 내용이 초기 연구/플랫폼 가설보다 우선한다.

## 핵심 제품 문장

> **플레이어에게 물질을 주는 게임이 아니라, 우주를 발명할 수 있는 문법을 주는 게임.**

## 가장 중요한 사용자 원칙

> **현실을 구현하는 것이 아니라 가상의 재미있는 놀이터를 만든다. 핵심은 나만의 세계 창조다. 현실 고증보다 게임 안에서 이해 가능한 논리와 상호작용이 중요하다.**

Powdergame은 Doodle God의 조합·발견·창조 감각과 DAN-BALL Powder Game의 공간적 상호작용·창발성을 결합한다.

현실의 불, 열, 압력, 전기, 부력, 산화, 빛 같은 현상은 강력한 참고자료지만 구현의 절대 규칙이 아니다.

- 현실에 없는 Matter를 만들어도 된다.
- 현실에 없는 상변화/반응을 만들어도 된다.
- 현실과 다른 연소/전기/압력/광학 규칙도 가능하다.
- 중요한 것은 플레이어가 원인과 결과를 이해하고 다시 이용할 수 있는가이다.
- 복잡성은 셀 하나에 많은 상태를 넣는 것이 아니라 수많은 단순한 셀이 서로 영향을 주며 만들어야 한다.

## Powder Game 정체성

### One Cell = Max One Matter

한 Cell에는 Matter가 최대 하나만 존재한다.

셀 내부에 여러 물질의 비율을 저장하는 혼합 모델을 기본 구조로 하지 않는다.

### Unit Cell Quantity

Matter가 있으면 한 단위다. `0.3 Water` 같은 양을 셀에 두지 않는다.

Temperature/Pressure/state는 두 번째 Matter가 아니다.

## 핵심 재미

Matter와 Field가:

- 이동
- 부력/침강
- 열 전달
- 상변화
- 연소
- 압력
- 파열
- 생성/소멸
- 다른 반응의 원인

이 되어 긴 연쇄작용을 만든다.

예:

```text
Water 가열
→ Steam
→ 팽창
→ 밀폐 공간에서 Pressure
→ 약한 구조 파열
→ Steam 분출
→ 주변 Matter에 Heat/Movement
→ 새로운 반응
```

이 연쇄를 하나의 `boiler_explosion` 전용 코드로 만들기보다 작은 공통 Rule의 결합으로 나오게 하는 것이 목표다.

## 발견

사전은 정답표가 아니라 **플레이어가 발견한 세계의 연구 노트**다.

- 기본 성격은 일부 보여줄 수 있다.
- 숨은 상호작용의 정확한 조건/수치는 미리 공개하지 않는다.
- A와 B 사이에서 처음 관찰한 Temperature 상승, Pressure 생성, 변환 같은 **현상 단위**를 기록한다.
- 아직 발견하지 못한 것이 있다는 정도는 알려줄 수 있다.
- 남은 정확한 개수는 보여주지 않는다.

## 현실보다 게임 내 정합성

Vibranium, Cryosteel 같은 가상의 물질도 완전히 유효하다.

예:

```text
Vibranium
- 매우 단단함
- 압력 저항이 극단적으로 높음
- 열에 거의 반응하지 않음
- 대부분의 Matter에 inert
```

플레이어가 몇 번 실험하고 성격을 이해할 수 있다면 좋은 Matter다.

## 성능은 제품 비전의 일부

> **셀 하나를 극도로 싸게 만들고, 수백만 Cell을 GPU에서 병렬 실행해 복잡한 세계를 만든다.**

현실 공식을 정밀하게 푸는 대신 원하는 현상을 만드는 최소 상태와 최소 local operation을 사용한다.

```text
부력      → Density Rank 비교 + local displacement
열        → 의미 있는 ΔT + cheap transfer
압력      → local ΔP + push/rupture
전기(향후) → conductive + strength/loss
방사선    → intensity + attenuation
빛        → transmit / absorb / reflect
```

> **부력을 계산하지 않는다. 정렬한다.**

> **물질의 양이 아니라 변화 가능한 영역이 계산량을 결정하게 한다.**

## 현재 공식 구현 방향

```text
Platform: Windows
Language: Rust
Window/Input: winit
GPU API: wgpu
Backend: DX12
Primary Performance Target: RTX 5090
World: finite chunked dense grid
Production Simulation: GPU authoritative
```

현재는 Browser/macOS/범용 GPU 호환을 위해 성능과 단순성을 희생하지 않는다.

GPU Production이 실제 게임의 기준이며 CPU Reference는 작은 테스트/디버그/의미 비교용이다.

## 장기 세계 계층

1. Matter
2. Field
3. Agent
4. Concept
5. Meta

M0에서는 Matter와 Field만 구현한다. 미래 계층의 빈 abstraction을 미리 만들지 않는다.

## 현재 개발 목표 — M0 First World

M0는 콘텐츠량이 아니라 다음 세계 문법의 성립을 증명한다.

- 2048×2048 reference world
- 64×64 initial chunk
- Static/Powder/Liquid/Gas
- Density Rank 기반 침강/부력/층분리
- Ice ↔ Water ↔ Steam
- Temperature
- Wood/Oil combustion
- Steam expansion → Pressure → push/rupture/vent
- Active/Sleep
- Read Neighbors / Write Self
- 필요한 공간 경쟁만 Claim/Resolve
- subsystem별 GPU performance measurement
- 실제 플레이에서 창발적 재미가 있는지 사용자 승인

자세한 최신 비전은 [`vision/USER_VISION.md`](vision/USER_VISION.md), 구현 계약은 `specs/`, 결정 맥락은 `design-history/`를 따른다.
