# Volume 01A — Powdergame Foundation Catalog

### Boundary Block (`boundary_block`)

**도감** — 편집 가능한 세계의 가장자리.

- **Layer / Movement:** `ENGINE_PRIMITIVE` / `STATIC`
- **상태:** `M0_VALIDATED`
- **Simulation identity:** 편집 가능한 세계의 가장자리. 움직이지 않고 일반 반응에 참여하지 않는 구조적 경계다.

### Stone (`stone`)

**도감** — 무겁고 단단한 기본 암석.

- **Layer / Movement:** `MATTER` / `STATIC`
- **상태:** `M0_VALIDATED`
- **Simulation identity:** 무겁고 단단한 기본 암석. 열과 압력을 버티며 다른 물질의 흐름을 가로막는다.

### Sand (`sand`)

**도감** — 쌓이고 무너지고 빈틈으로 흘러드는 가장 기본적인 가루.

- **Layer / Movement:** `MATTER` / `POWDER`
- **상태:** `M0_VALIDATED`
- **Simulation identity:** 쌓이고 무너지고 빈틈으로 흘러드는 가장 기본적인 가루. 충분한 열에서는 유리화 후보가 된다.

### Ice (`ice`)

**도감** — 물의 차가운 고체 상태.

- **Layer / Movement:** `MATTER` / `STATIC`
- **상태:** `M0_VALIDATED`
- **Simulation identity:** 물의 차가운 고체 상태. 열을 받으면 Water로 돌아간다.

### Water (`water`)

**도감** — 흐르고 스며들고 열을 받아 Steam이 되는 세계의 기준 액체

- **Layer / Movement:** `MATTER` / `LIQUID`
- **상태:** `M0_VALIDATED`
- **Simulation identity:** 흐르고 스며들고 열을 받아 Steam이 되는 세계의 기준 액체.

### Steam (`steam`)

**도감** — 열이 만든 가벼운 기체.

- **Layer / Movement:** `MATTER` / `GAS`
- **상태:** `M0_VALIDATED`
- **Simulation identity:** 열이 만든 가벼운 기체. 공간이 막히면 상변화 팽창이 압력 문제로 이어질 수 있다.

### Smoke (`smoke`)

**도감** — 연소가 남긴 떠다니는 흔적.

- **Layer / Movement:** `MATTER` / `GAS`
- **상태:** `M0_VALIDATED`
- **Simulation identity:** 연소가 남긴 떠다니는 흔적. 불이 지나간 경로를 눈에 보이게 만든다.

### Wood (`wood`)

**도감** — 불에 탈 수 있는 구조재.

- **Layer / Movement:** `MATTER` / `STATIC`
- **상태:** `M0_VALIDATED`
- **Simulation identity:** 불에 탈 수 있는 구조재. 연소 문법을 가장 직관적으로 보여주는 고체 연료다.

### Oil (`oil`)

**도감** — 물 위에 층을 만들고 불이 붙으면 표면을 따라 화염을 운반하는 가연성 액체

- **Layer / Movement:** `MATTER` / `LIQUID`
- **상태:** `M0_VALIDATED`
- **Simulation identity:** 물 위에 층을 만들고 불이 붙으면 표면을 따라 화염을 운반하는 가연성 액체.

### Acid (`acid`)

**도감** — 닿은 재료를 선택적으로 약화·용해시키는 반응성 액체.

- **Layer / Movement:** `MATTER` / `LIQUID`
- **상태:** `REGISTERED_DIRECTION`
- **Simulation identity:** 닿은 재료를 선택적으로 약화·용해시키는 반응성 액체. 정확한 현실 산 하나가 아니라 게임용 부식 archetype이다.

### Seed (`seed`)

**도감** — 조건이 맞으면 Plant로 이어지는 잠든 생명.

- **Layer / Movement:** `MATTER` / `POWDER`
- **상태:** `REGISTERED_DIRECTION`
- **Simulation identity:** 조건이 맞으면 Plant로 이어지는 잠든 생명. 흙과 물을 새로운 상호작용 축으로 연결한다.

### Plant (`plant`)

**도감** — 물과 환경을 먹고 공간을 차지하는 가장 단순한 생명 재료

- **Layer / Movement:** `MATTER` / `STATIC`
- **상태:** `REGISTERED_DIRECTION`
- **Simulation identity:** 물과 환경을 먹고 공간을 차지하는 가장 단순한 생명 재료.

### Salt (`salt`)

**도감** — 물에 녹아 Brine을 만들고 동결·부식 같은 새로운 조건을 여는 결정 가루

- **Layer / Movement:** `MATTER` / `POWDER`
- **상태:** `REGISTERED_DIRECTION`
- **Simulation identity:** 물에 녹아 Brine을 만들고 동결·부식 같은 새로운 조건을 여는 결정 가루.

### Lava (`lava`)

**도감** — 흐르는 암석이자 강력한 열원.

- **Layer / Movement:** `MATTER` / `LIQUID`
- **상태:** `REGISTERED_DIRECTION`
- **Simulation identity:** 흐르는 암석이자 강력한 열원. 식으면 Stone 계열로 돌아가고 물과 만나면 급격한 열교환을 만든다.

### Metal (`metal`)

**도감** — 열을 잘 전달하고 압력을 견디며 녹으면 Molten Metal이 되는 구조·공학 기준재

- **Layer / Movement:** `MATTER` / `STATIC`
- **상태:** `REGISTERED_DIRECTION`
- **Simulation identity:** 열을 잘 전달하고 압력을 견디며 녹으면 Molten Metal이 되는 구조·공학 기준재.

### Glass (`glass`)

**도감** — 모래가 열을 거쳐 얻는 투명한 구조재.

- **Layer / Movement:** `MATTER` / `STATIC`
- **상태:** `REGISTERED_DIRECTION`
- **Simulation identity:** 모래가 열을 거쳐 얻는 투명한 구조재. 단단하지만 열충격·압력 파괴의 별도 성격을 줄 수 있다.
