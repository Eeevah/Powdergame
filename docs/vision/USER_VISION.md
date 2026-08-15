# Powdergame — Authoritative User Vision

> **플레이어에게 물질을 주는 게임이 아니라, 우주를 발명할 수 있는 문법을 주는 게임.**

이 문서는 Powdergame의 현재 최상위 제품 기준이다. 기술 설계, 최적화, 콘텐츠, 마일스톤이 이 비전과 충돌하면 먼저 비전을 확인한다.

---

## 1. Powdergame이 무엇인가

Powdergame은 DAN-BALL Powder Game 계열의 즉각적인 공간적 상호작용과 Doodle God의 조합·발견·세계 창조 감각을 결합한 **세계 창조 샌드박스**다.

두 게임의 특징을 메뉴 수준에서 단순 결합하지 않는다.

- Doodle God에서 중요한 것은 “이것과 이것을 만나게 하면 무엇이 생길까?”라는 호기심과 발견이다.
- Powder Game에서 중요한 것은 실제 공간에 물질을 놓고, 서로 닿고, 흐르고, 타고, 변하고, 무너지며 개발자가 직접 작성하지 않은 결과가 생기는 것이다.

Powdergame의 목표는 이 둘을 같은 세계 안에서 연결하는 것이다.

> **플레이어가 상상한 가설을 실제 공간에 던지고, 세계가 자기 규칙으로 대답하게 한다.**

---

## 2. 가장 중요한 사용자 원칙: 현실 재현이 아니라 나만의 세계 창조

### User Principle

> 현실을 구현하는 것이 아니라 가상의 재미있는 놀이터를 만든다. 핵심은 나만의 세계 창조다. 현실 고증보다 게임 안에서 이해 가능한 논리와 상호작용이 중요하다.

현실의 자연현상은 매우 좋은 참고자료다. 불, 물, 열, 압력, 전기, 부력, 산화, 빛 같은 현상은 플레이어가 이미 직관을 갖고 있기 때문에 강력한 출발점이다.

그러나 현실 과학은 이 게임의 법전이 아니다.

- 현실에 존재하지 않는 물질을 자유롭게 만들 수 있다.
- 현실에서 일어나지 않는 상변화나 반응도 가능하다.
- 철이 특정 조건에서 게임적으로 연소할 수도 있다.
- 현실의 유리가 하지 않는 반응을 Powdergame의 Glass가 할 수도 있다.
- Vibranium처럼 완전히 가상의 Matter가 존재해도 된다.

중요한 질문은 “현실에서 진짜인가?”가 아니라 다음이다.

1. 상호작용이 재미있는가?
2. 원인과 결과를 플레이어가 이해할 수 있는가?
3. 다른 시스템과 새로운 연쇄작용을 만드는가?
4. 같은 조건에서 게임 세계의 규칙이 상식적으로 일관되는가?
5. 큰 세계에서 충분히 싸게 계산할 수 있는가?
6. 현실성은 도움이 될 때만 참고한다.

가상의 물질 예시:

```text
Vibranium
- 매우 단단하다.
- 일반적인 Pressure에 거의 파괴되지 않는다.
- 대부분의 물질과 반응하지 않는다.
- 일반적인 Heat에도 거의 변화하지 않는다.
```

이 물질이 현실에 없더라도 플레이어가 몇 번 실험한 뒤 “이건 거의 반응하지 않는 절대적인 방벽 물질이구나”라고 이해할 수 있다면 좋은 게임 물질이다.

---

## 3. 핵심 재미는 상호작용이다

원소의 숫자 자체는 목표가 아니다.

Powdergame의 핵심은 Matter가 서로 만나:

- 이동하고
- 밀고
- 뜨고/가라앉고
- 온도를 주고받고
- 타고
- 냉각되고
- 상변화하고
- 압력을 만들고
- 부서지고
- 다른 Matter로 변하고
- 다른 Field를 만들고
- 그 결과가 다시 다음 반응의 원인이 되는 것

이다.

좋은 콘텐츠는 단일 레시피보다 **연쇄작용을 만드는 규칙**이다.

예:

```text
Water 가열
→ Steam
→ 공간 부족
→ Pressure 상승
→ Wood 파열
→ Steam 분출
→ 주변 Matter 가열/이동
→ 또 다른 반응
```

이 연쇄 전체를 `boiler_explosion()` 같은 전용 기능으로 만들지 않는 것이 이상적이다. 각각의 작은 세계 규칙이 독립적으로 작동한 결과여야 한다.

---

## 4. Doodle God식 발견의 방향

발견 시스템은 정답표가 아니다.

플레이어에게 처음부터 정확한 반응식, threshold, 수치, 남은 발견 개수를 보여주면 창의력을 제한한다.

### 기본 공개 원칙

물질의 아주 기본적인 성격은 어느 정도 보여줄 수 있다.

```text
Vibranium
- Solid
- 매우 단단해 보인다
```

하지만 숨은 상호작용의 상세 조건은 공개하지 않는다.

실제 실험을 통해 A와 B에서 의미 있는 현상을 처음 관찰하면 사전에 **현상 단위**로 기록한다.

예:

```text
A ↔ B
관찰됨:
- Temperature 상승
- Pressure 발생
- Matter 변화
```

정확히 몇 도에서, 어느 계수로, 몇 Tick 후 발생하는지는 기본적으로 숨긴다.

> **게임은 현상을 알려주고, 공식은 숨긴다.**

### 아직 발견하지 못한 것이 있다는 힌트

사전은 `4 / 17`처럼 남은 정확한 개수를 보여주지 않는다.

대신 필요하면:

> 아직 발견하지 못한 성질이 있다.

정도만 알려준다.

이로써 수집·도전 욕구는 남기되 플레이어를 “정해진 빈칸 채우기”에 가두지 않는다.

### 발견 사전의 역할

> **사전은 정답표가 아니라 플레이어가 발견한 세계의 연구 노트다.**

반응하지 않는다는 사실도 의미 있는 발견이 될 수 있다.

---

## 5. 세계는 현실 물질 DB가 아니라 변환 가능한 어휘다

모든 Matter는 공통적으로 변환 가능성을 가질 수 있다. 현재 전환이 없는 Matter는 transition 목록이 비어 있으면 된다.

모든 Matter를 현실적인 `Solid → Liquid → Gas` 3상태에 강제로 맞추지 않는다.

가능한 예:

```text
Ice ↔ Water ↔ Steam
```

```text
Sand + extreme heat → Glass 계열
```

```text
Metal + 특정 조건 → Burning Metal → 다른 Matter
```

```text
Dream Sand + Water → Crystal
Crystal + Extreme Pressure → Light
Light + Black Metal → Void Matter
```

현실성과 무관하게 게임 세계에서 배우고 다시 이용할 수 있는 규칙이면 된다.

---

## 6. Powder Game의 공간적 정체성은 유지한다

Powdergame은 복잡한 재료 혼합 시뮬레이터가 아니다.

### One Cell = Max One Matter

한 Tick의 한 Cell에는 Matter가 최대 하나만 존재한다.

```text
한 Cell = Water 30% + Oxygen 20% + Oil 50%
```

같은 내부 혼합 모델을 기본 구조로 만들지 않는다.

복잡성은 한 Cell 안에 많은 것을 넣어서 만드는 것이 아니라 **수많은 단순한 Cell이 공간적으로 서로 영향을 주면서** 만든다.

이는 제약이면서 동시에 게임의 정체성이다.

### Unit Cell Quantity

Matter가 존재하는 Cell은 한 단위의 Matter다.

- `0.2 Water` 같은 per-cell 물질량을 기본 모델로 두지 않는다.
- density, heat capacity 등은 Matter의 성질이지 셀 안의 양이 아니다.
- Temperature, Pressure, combustion state 등의 Field/State는 두 번째 Matter가 아니다.

---

## 7. 단순한 세계 법칙에서 창발성이 나와야 한다

Powdergame은 모든 재미있는 결과를 개발자가 직접 스크립트로 작성하는 게임이 되어서는 안 된다.

좋은 예:

```text
Laser
→ Metal이 빛을 흡수
→ Temperature 상승
→ Metal 상변화
→ Molten Metal 이동
→ Water 접촉
→ Steam 생성
→ Pressure 상승
→ Wall rupture
```

`Laser + Wall = Explosion`이라는 규칙을 직접 쓰지 않아도 이런 결과가 생기는 것이 목표다.

개발자가 예상하지 못했던 결과가 세계 법칙 안에서 말이 된다면 그것은 버그가 아니라 **창발적 콘텐츠 후보**다.

---

## 8. 성능은 제품 비전의 일부다

이 게임에서 성능은 단순 기술적 품질이 아니다.

큰 세계에서 수많은 상호작용이 동시에 살아 있어야 핵심 판타지가 성립한다.

따라서 목표는 “셀 하나를 현실적으로 정교하게 계산하는 것”이 아니다.

> **셀 하나를 극도로 싸게 만들고, 수백만 Cell을 GPU에서 병렬 실행해서 복잡한 세계를 만든다.**

성능 최적화로 절약한 계산 예산은 단순히 FPS 숫자를 높이는 데만 쓰지 않는다.

- 더 큰 세계
- 더 많은 Matter
- 더 많은 동시 반응
- Temperature / Pressure / 전기 / 빛 / 방사선 같은 더 많은 세계 법칙
- 더 풍부한 연쇄작용
- 빠른 Rewind와 실험
- 더 좋은 Presentation

에 다시 투자한다.

---

## 9. Game-Consistent Minimum Physics

현실의 물리식을 그대로 구현하기보다 플레이어가 현상을 이해하고 활용하는 데 필요한 **최소 상태와 최소 Local 연산**으로 표현한다.

예:

```text
부력
→ 실제 유체역학 대신 Density Rank 비교와 local displacement

열
→ 의미 있는 ΔT + 최소 전도/열용량 모델

압력
→ local ΔP와 저비용 전달/저항/파열

전기
→ conductive 여부 + 전달 strength/loss

방사선
→ intensity + attenuation/blocking

Gameplay Light
→ transmit / absorb / reflect + 남은 intensity
```

현실 물리를 닮았으면 좋지만 정확히 같을 필요는 없다.

상식적으로 이상하지 않고, 반복해서 배울 수 있고, 상호작용이 재미있고, 계산이 충분히 싸면 된다.

---

## 10. 결과는 정직하게, 감각은 과장한다

Simulation과 Presentation을 분리한다.

### Simulation Truth

플레이 결과에 실제 영향을 주는 것:

- Matter 이동/변환
- Temperature
- Pressure
- 연소
- 파열
- 상변화
- 전기/방사선/빛 등 미래 gameplay state

### Presentation Effects

시뮬레이션 결과를 더 강하게 느끼게 만드는 것:

- heat haze
- glow
- bloom
- shockwave visual
- debris
- local distortion
- camera impulse
- sound

원칙:

> **결과는 정직하게, 감각은 과장한다.**

실제로 움직이지 않은 Matter가 이동한 것처럼 보여 플레이어가 세계 법칙을 잘못 이해하게 만드는 Presentation은 피한다.

---

## 11. 세계의 장기 계층

장기적으로 세계는 다음 다섯 층을 담을 수 있다.

1. Matter
2. Field
3. Agent
4. Concept
5. Meta

하지만 이것은 장기적인 개념 계약이다.

현재 M0에서는 **Matter와 Field만 구현**한다. Agent/Concept/Meta를 위한 빈 추상화나 코드 구조를 미리 만들지 않는다.

장기적으로는:

```text
물질
→ 에너지
→ 화학
→ 생명
→ 생태계
→ 기계
→ 정보
→ 언어
→ 사회
→ 문명
→ 신앙/신화
→ AI
→ 시간/공간
→ 세계 법칙
```

까지 확장할 수 있다.

이 방향은 약속된 기능 목록이 아니라 **세계 창조 샌드박스가 도달할 수 있는 천장**이다.

---

## 12. 현재 플랫폼과 개발 목표

현재 프로젝트는 범용 배포 제품을 우선하지 않는다.

### 현재 공식 개발 경로

- Windows
- Rust
- winit
- wgpu
- DX12 backend
- NVIDIA RTX 5090을 Primary Performance Target으로 사용

브라우저/macOS 호환을 위해 현재 엔진의 단순성이나 성능을 희생하지 않는다. 과거 Browser/Mac 관련 문서는 초기 연구 단계의 가설로 보존한다.

Production Simulation의 authoritative state는 GPU에 둔다.

CPU Reference는 작은 세계의 이해·테스트·디버그·비교용이며 GPU Production의 bit-exact oracle이 아니다.

---

## 13. 플레이어가 느껴야 하는 것

처음 몇 분 안에도:

- Sand가 떨어지고
- Water가 흐르고
- 다른 density의 Matter가 뜨거나 가라앉고
- Ice/Water/Steam이 온도에 따라 변하고
- Wood/Oil 등이 조건에 따라 타며
- Steam의 팽창이 Pressure를 만들고
- 약한 구조가 부서지고
- 서로 전용으로 작성하지 않은 시스템들이 연쇄반응을 만드는 것

을 직접 볼 수 있어야 한다.

장기적으로 가장 중요한 질문은 이것이다.

> **“이 세계에 이것을 넣으면 대체 무슨 일이 일어날까?”라는 생각을 계속 하게 만드는가?**

그 질문이 계속 생긴다면 Powdergame은 올바른 방향에 있다.
