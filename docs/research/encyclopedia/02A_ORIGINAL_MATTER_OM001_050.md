# Volume 02 — Original Matter OM-001 ~ OM-100

> 이름보다 동사가 중요하다. 각 물질은 작은 국소 규칙 하나가 다른 시스템의 입력이 되도록 설계한다.

## OM-001 — Pyrostor (열 축적 합금 / 열저장석)

**도감** — 열을 잊지 않는 돌. 조용히 삼키다가 한계를 넘으면 기억한 열을 한꺼번에 되돌린다.

- **Family:** 열·상전이·압력
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Latent Thermal Accumulator (흡열 완충 후 지연 폭발)
- **Simulation notes:** 열을 흡수하여 내부 격자에 가두며 팽창하지 않다가, 한계 온도 도달 시 충격파와 함께 축적된 열을 일시에 방출하는 축열 고체.
- **출처:** Original Matter research

## OM-002 — Gelid Silt (흡열성 한기 침전토 / 결빙니)

**도감** — 주변 액체의 온도를 급속히 빼앗아 고체화시키며 자신은 가벼운 부유 기체로 승화하는 흡열성 분말

- **Family:** 열·상전이·압력
- **Movement:** `POWDER`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Contact Endothermic Sublimation (접촉식 흡열 승화)
- **Simulation notes:** 주변 액체의 온도를 급속히 빼앗아 고체화시키며 자신은 가벼운 부유 기체로 승화하는 흡열성 분말.
- **출처:** Original Matter research

## OM-003 — Baroclast (감압 팽창석 / 감압암)

**도감** — 고압 환경에서는 단단한 암석이지만, 외부 압력이 낮아지면 수십 배의 체적으로 팽창하여 다공성 폼이 되는 기압 반응성 광물

- **Family:** 열·상전이·압력
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Pressure-Inverted Density Expansion (저압 팽창)
- **Simulation notes:** 고압 환경에서는 단단한 암석이지만, 외부 압력이 낮아지면 수십 배의 체적으로 팽창하여 다공성 폼이 되는 기압 반응성 광물.
- **출처:** Original Matter research

## OM-004 — Thermovant (열류 추진 기체 / 열추진기)

**도감** — 온도 구배가 높은 방향(열원)을 향해 강한 가속도로 역류 이동하는 이상 열주성 가스

- **Family:** 열·상전이·압력
- **Movement:** `GAS`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Negative Thermal Gradient Chemotaxis (열원 역류 이동)
- **Simulation notes:** 온도 구배가 높은 방향(열원)을 향해 강한 가속도로 역류 이동하는 이상 열주성 가스.
- **출처:** Original Matter research

## OM-005 — Resonac (공진 파쇄 결정 / 음향석)

**도감** — 특정 주파수의 압력 진동(주기적 압력파)을 받으면 내부 결합이 와해되어 미세 파편으로 비산하는 결정

- **Family:** 열·상전이·압력
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Harmonic Shock Resonance Rupture (주기적 충격 파열)
- **Simulation notes:** 특정 주파수의 압력 진동(주기적 압력파)을 받으면 내부 결합이 와해되어 미세 파편으로 비산하는 결정.
- **출처:** Original Matter research

## OM-006 — Null-Ash (불연성 소화재 / 허무회)

**도감** — 불이 먹을 것을 빼앗는 재. 화염의 가장자리를 덮으면 연쇄가 그 자리에서 끊긴다.

- **Family:** 열·상전이·압력
- **Movement:** `POWDER`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Radical Flame Quenching & Oxygen Displacement (라디칼 연소 흡수)
- **Simulation notes:** 인접 8방향에 FIRE, SPARK, LAVA 감지 시 즉시 소멸시키며, $T$를 상온($20^\circ\text{C}$)으로 강제 동결.
- **출처:** Original Matter research

## OM-007 — Phase-Wax (잠열 상변화 파라핀 / 위상랍)

**도감** — 열을 먹고 녹으며, 식으면 그 열을 천천히 돌려주는 온도의 완충재.

- **Family:** 열·상전이·압력
- **Movement:** `STATIC / LIQUID (가역)`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Isothermal Latent Heat Storage (정온 상전이 버퍼)
- **Simulation notes:** $50^\circ\text{C}$ 도달 시 액화되면서 주변의 열을 대량 흡수하여 계의 온도를 $50^\circ\text{C}$로 장시간 고정. 냉각 시 다시 굳으며 $50^\circ\text{C}$ 열 방출.
- **출처:** Original Matter research

## OM-008 — Ignis-Gel (점착성 자기발열 젤 / 화지)

**도감** — 벽에 달라붙어 스스로 뜨거워지는 젤. 흘러내리지 않는 불씨가 된다.

- **Family:** 열·상전이·압력
- **Movement:** `LIQUID`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Shear-Adhesive Exothermic Viscosity (벽면 부착 발열 유체)
- **Simulation notes:** 수직 고체 벽면에 닿으면 흘러내리지 않고 고정. 공기와 접촉 시 서서히 산화 발열($150^\circ\text{C}$ 지속).
- **출처:** Original Matter research

## OM-009 — Cryo-Brine (부동 냉각수 / 극저온 염수)

**도감** — 얼지 않는 차가운 물. 흐르면서 다른 물을 얼려 길과 벽을 만든다.

- **Family:** 열·상전이·압력
- **Movement:** `LIQUID`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Sub-Zero Liquid Convection (영하 액상 대류)
- **Simulation notes:** $-70^\circ\text{C}$까지 얼지 않고 액체 상태로 흐르며, 접촉하는 WATER를 즉시 ICE로 동결시킴.
- **출처:** Original Matter research

## OM-010 — Fulgur-Crust (압전 암석 / 낙뢰각)

**도감** — 누르면 번개를 토하는 돌. 압력이 곧 점화 신호가 된다.

- **Family:** 열·상전이·압력
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Compression Spark Discharge (고압 방전)
- **Simulation notes:** 상단에서 $P > 8\text{ atm}$의 압력이 가해지면 하단으로 SPARK 입자를 사출.
- **출처:** Original Matter research

## OM-011 — Steam-Sponge (다공성 증기 흡착재)

**도감** — 기체 STEAM을 흡수하여 체적 내에 격리, 압축 후 냉각 시 물방울로 배출

- **Family:** 열·상전이·압력
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 기체 STEAM을 흡수하여 체적 내에 격리, 압축 후 냉각 시 물방울로 배출.
- **Simulation notes:** 기체 STEAM을 흡수하여 체적 내에 격리, 압축 후 냉각 시 물방울로 배출.
- **출처:** Original Matter research

## OM-012 — Pyrophoric Brass (마찰 발화 황동 분말)

**도감** — 고속 낙하 및 표면 마찰 시 불꽃 입자 연속 튀김

- **Family:** 열·상전이·압력
- **Movement:** `POWDER`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 고속 낙하 및 표면 마찰 시 불꽃 입자 연속 튀김.
- **Simulation notes:** 고속 낙하 및 표면 마찰 시 불꽃 입자 연속 튀김.
- **출처:** Original Matter research

## OM-013 — Aerobaryte (부력 역전 분말)

**도감** — 기온이 상승하면 비중이 공기보다 가벼워져 거꾸로 상승하는 역중력 모래

- **Family:** 열·상전이·압력
- **Movement:** `POWDER`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 기온이 상승하면 비중이 공기보다 가벼워져 거꾸로 상승하는 역중력 모래.
- **Simulation notes:** 기온이 상승하면 비중이 공기보다 가벼워져 거꾸로 상승하는 역중력 모래.
- **출처:** Original Matter research

## OM-014 — Vitriolic Slag (지연 발열 슬래그)

**도감** — 물과 닿으면 100틱 후 극초고온으로 끓어오르는 시차 반응재

- **Family:** 열·상전이·압력
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 물과 닿으면 100틱 후 극초고온($900^\circ\text{C}$)으로 끓어오르는 시차 반응재.
- **Simulation notes:** 물과 닿으면 100틱 후 극초고온($900^\circ\text{C}$)으로 끓어오르는 시차 반응재.
- **출처:** Original Matter research

## OM-015 — Quench-Glass (급랭 강화 유리)

**도감** — 균일 냉각 시 극강의 강도, 국소 열충격 시 미세 분말로 폭쇄

- **Family:** 열·상전이·압력
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 균일 냉각 시 극강의 강도, 국소 열충격($\Delta T > 200^\circ\text{C}$) 시 미세 분말로 폭쇄.
- **Simulation notes:** 균일 냉각 시 극강의 강도, 국소 열충격($\Delta T > 200^\circ\text{C}$) 시 미세 분말로 폭쇄.
- **출처:** Original Matter research

## OM-016 — Slambog (비뉴턴 충격 흡수 슬라임)

**도감** — 고속 낙하물 충돌 시 순간 경화하여 관통 저지, 저속 침강물은 천천히 통과시킴

- **Family:** 열·상전이·압력
- **Movement:** `LIQUID`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 고속 낙하물 충돌 시 순간 경화하여 관통 저지, 저속 침강물은 천천히 통과시킴.
- **Simulation notes:** 고속 낙하물 충돌 시 순간 경화하여 관통 저지, 저속 침강물은 천천히 통과시킴.
- **출처:** Original Matter research

## OM-017 — Hydro-Calcite (수화 팽창 석회)

**도감** — 물을 1:1로 흡수하여 3배 부피의 단단한 콘크리트 고체로 급결

- **Family:** 열·상전이·압력
- **Movement:** `POWDER`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 물을 1:1로 흡수하여 3배 부피의 단단한 콘크리트 고체로 급결.
- **Simulation notes:** 물을 1:1로 흡수하여 3배 부피의 단단한 콘크리트 고체로 급결.
- **출처:** Original Matter research

## OM-018 — Zephyr-Puff (극저밀도 부유 포자)

**도감** — 극미한 압력 차이에도 반응하여 고기압에서 저기압으로 초고속 분출

- **Family:** 열·상전이·압력
- **Movement:** `GAS/POWDER`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 극미한 압력 차이에도 반응하여 고기압에서 저기압으로 초고속 분출.
- **Simulation notes:** 극미한 압력 차이에도 반응하여 고기압에서 저기압으로 초고속 분출.
- **출처:** Original Matter research

## OM-019 — Pyro-Oil (저온 착화유)

**도감** — 에서 자발 발화하며 물보다 가벼워 수면 전체를 화염으로 뒤덮음

- **Family:** 열·상전이·압력
- **Movement:** `LIQUID`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** $30^\circ\text{C}$에서 자발 발화하며 물보다 가벼워 수면 전체를 화염으로 뒤덮음.
- **Simulation notes:** $30^\circ\text{C}$에서 자발 발화하며 물보다 가벼워 수면 전체를 화염으로 뒤덮음.
- **출처:** Original Matter research

## OM-020 — Frost-Marrow (냉기 방출 골수 결정)

**도감** — 주위 공기에서 열을 지속 추출하여 자신 주위에 고드름 구조를 아래로 성장시킴

- **Family:** 열·상전이·압력
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 주위 공기에서 열을 지속 추출하여 자신 주위에 고드름 구조를 아래로 성장시킴.
- **Simulation notes:** 주위 공기에서 열을 지속 추출하여 자신 주위에 고드름 구조를 아래로 성장시킴.
- **출처:** Original Matter research

## OM-021 — Litho-Mycelium (암석 섭식 균사 / 석균)

**도감** — 무기질 암석과 모래를 분해하여 유기 섬유로 전환하며, 사멸 시 다공성 숯을 남기는 암석 침식 균류

- **Family:** 증식·결정·침식
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Substrate Mineral Digestion (무기물 부식 증식)
- **Simulation notes:** 무기질 암석과 모래를 분해하여 유기 섬유로 전환하며, 사멸 시 다공성 숯을 남기는 암석 침식 균류.
- **출처:** Original Matter research

## OM-022 — Crys-Tendril (과냉각 결정 덩굴 / 빙권)

**도감** — 준안정 상태의 액체에 닿는 즉시 바늘 모양의 수지상(Dendrite) 결정을 폭발적으로 뻗어내는 결정핵 물질

- **Family:** 증식·결정·침식
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Nucleation Cascade Propagation (결정핵 연쇄 전파)
- **Simulation notes:** 준안정 상태의 액체에 닿는 즉시 바늘 모양의 수지상(Dendrite) 결정을 폭발적으로 뻗어내는 결정핵 물질.
- **출처:** Original Matter research

## OM-023 — Viridian Rust (녹청성 부식액 / 청청록)

**도감** — 금속 격자만을 선택적으로 먹어 치우며 인화성 고압 수소 가스를 내뿜는 자기증식 산화 촉매

- **Family:** 증식·결정·침식
- **Movement:** `LIQUID`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Metal-Consuming Autocatalysis (금속 분해 수소 발생)
- **Simulation notes:** 금속 격자만을 선택적으로 먹어 치우며 인화성 고압 수소 가스를 내뿜는 자기증식 산화 촉매.
- **출처:** Original Matter research

## OM-024 — Scoria-Weaver (용암 섬유 거미줄 / 암사)

**도감** — 액체 용암과 접촉하면 표면을 굳혀 신축성 있는 내열성 로프망을 엮어내는 광물 분말

- **Family:** 증식·결정·침식
- **Movement:** `POWDER`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Magma Filamentation (용융물 섬유화)
- **Simulation notes:** 액체 용암과 접촉하면 표면을 굳혀 신축성 있는 내열성 로프망을 엮어내는 광물 분말.
- **출처:** Original Matter research

## OM-025 — Bloom-Peat (폭발성 증식 토탄 / 팽창탄)

**도감** — 물을 흡수하면 부피가 5배로 부풀며, 건조 후 점화 시 극도로 높은 열량으로 타오르는 연료 토양

- **Family:** 증식·결정·침식
- **Movement:** `POWDER`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Hygroscopic Mass Multiplication (흡수 증식 탄화)
- **Simulation notes:** 물을 흡수하면 부피가 5배로 부풀며, 건조 후 점화 시 극도로 높은 열량으로 타오르는 연료 토양.
- **출처:** Original Matter research

## OM-026 — Coral-Lime (석회질 산호 골격)

**도감** — 이산화탄소 기체를 흡수하여 바다물 속에서 수직 분기 암초 구조 성장

- **Family:** 증식·결정·침식
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 이산화탄소 기체를 흡수하여 바다물 속에서 수직 분기 암초 구조 성장.
- **Simulation notes:** 이산화탄소 기체를 흡수하여 바다물 속에서 수직 분기 암초 구조 성장.
- **출처:** Original Matter research

## OM-027 — Spore-Pod (압력 분출 포자낭)

**도감** — 외부 충격파 감지 시 사방으로 독성 포자 가스를 고속 분사

- **Family:** 증식·결정·침식
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 외부 충격파 감지 시 사방으로 독성 포자 가스를 고속 분사.
- **Simulation notes:** 외부 충격파 감지 시 사방으로 독성 포자 가스를 고속 분사.
- **출처:** Original Matter research

## OM-028 — Devourer-Slime (유기물 섭식 점균)

**도감** — 죽은 생체, 나무, 기름을 흡수하여 점성 액적을 늘려가는 육식 유체

- **Family:** 증식·결정·침식
- **Movement:** `LIQUID`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 죽은 생체, 나무, 기름을 흡수하여 점성 액적을 늘려가는 육식 유체.
- **Simulation notes:** 죽은 생체, 나무, 기름을 흡수하여 점성 액적을 늘려가는 육식 유체.
- **출처:** Original Matter research

## OM-029 — Sinter-Dust (자기소결 세라믹 분말)

**도감** — 이상 가열 시 입자 간 경계가 사라지며 단단한 벽돌 타일로 융착

- **Family:** 증식·결정·침식
- **Movement:** `POWDER`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** $400^\circ\text{C}$ 이상 가열 시 입자 간 경계가 사라지며 단단한 벽돌 타일로 융착.
- **Simulation notes:** $400^\circ\text{C}$ 이상 가열 시 입자 간 경계가 사라지며 단단한 벽돌 타일로 융착.
- **출처:** Original Matter research

## OM-030 — Alkali-Tendril (알칼리 침상 결정)

**도감** — 산성 액체와 반응하여 중화염 기포를 뿜으며 반대 방향으로 침상 성장

- **Family:** 증식·결정·침식
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 산성 액체와 반응하여 중화염 기포를 뿜으며 반대 방향으로 침상 성장.
- **Simulation notes:** 산성 액체와 반응하여 중화염 기포를 뿜으며 반대 방향으로 침상 성장.
- **출처:** Original Matter research

## OM-031 — Midas-Precipitate (황금 침전 촉매)

**도감** — 특정 독성 슬러지와 구리가 섞인 액체에서 금 분말을 결정화하여 석출

- **Family:** 증식·결정·침식
- **Movement:** `LIQUID`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 특정 독성 슬러지와 구리가 섞인 액체에서 금 분말을 결정화하여 석출.
- **Simulation notes:** 특정 독성 슬러지와 구리가 섞인 액체에서 금 분말을 결정화하여 석출.
- **출처:** Original Matter research

## OM-032 — Chitin-Scale (키틴질 각피 분말)

**도감** — 산성 환경에서 부식되지 않고 방수 피막 층을 형성하는 생체 분말

- **Family:** 증식·결정·침식
- **Movement:** `POWDER`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 산성 환경에서 부식되지 않고 방수 피막 층을 형성하는 생체 분말.
- **Simulation notes:** 산성 환경에서 부식되지 않고 방수 피막 층을 형성하는 생체 분말.
- **출처:** Original Matter research

## OM-033 — Ash-Bramble (재 가시덤불)

**도감** — 화재 현장의 재(ASH)를 영양분 삼아 잿더미 속에서 뻗어나오는 불연성 가시덤불

- **Family:** 증식·결정·침식
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 화재 현장의 재(ASH)를 영양분 삼아 잿더미 속에서 뻗어나오는 불연성 가시덤불.
- **Simulation notes:** 화재 현장의 재(ASH)를 영양분 삼아 잿더미 속에서 뻗어나오는 불연성 가시덤불.
- **출처:** Original Matter research

## OM-034 — Leach-Root (지하수 흡인 근계)

**도감** — 모세관 현상으로 하층의 액체를 상층으로 10셀 이상 끌어올리는 목질 섬유

- **Family:** 증식·결정·침식
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 모세관 현상으로 하층의 액체를 상층으로 10셀 이상 끌어올리는 목질 섬유.
- **Simulation notes:** 모세관 현상으로 하층의 액체를 상층으로 10셀 이상 끌어올리는 목질 섬유.
- **출처:** Original Matter research

## OM-035 — Glass-Pox (유리 규폐 감염체)

**도감** — 모래(SAND) 및 석영에 닿으면 투명한 유리 결정으로 전염 증식시키는 침식균

- **Family:** 증식·결정·침식
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 모래(SAND) 및 석영에 닿으면 투명한 유리 결정으로 전염 증식시키는 침식균.
- **Simulation notes:** 모래(SAND) 및 석영에 닿으면 투명한 유리 결정으로 전염 증식시키는 침식균.
- **출처:** Original Matter research

## OM-036 — Tar-Tumor (타르 종양 결절)

**도감** — 원유(OIL) 속에 잠겨 있을 때 크기를 키우며 가연성 메탄 방출

- **Family:** 증식·결정·침식
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 원유(OIL) 속에 잠겨 있을 때 크기를 키우며 가연성 메탄 방출.
- **Simulation notes:** 원유(OIL) 속에 잠겨 있을 때 크기를 키우며 가연성 메탄 방출.
- **출처:** Original Matter research

## OM-037 — Silk-Vapor (응결 섬유 증기)

**도감** — 공기 중에서 서서히 낙하하며 입자끼리 엉겨 붙어 얇은 실크 직물 발판을 형성

- **Family:** 증식·결정·침식
- **Movement:** `GAS`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 공기 중에서 서서히 낙하하며 입자끼리 엉겨 붙어 얇은 실크 직물 발판을 형성.
- **Simulation notes:** 공기 중에서 서서히 낙하하며 입자끼리 엉겨 붙어 얇은 실크 직물 발판을 형성.
- **출처:** Original Matter research

## OM-038 — Rust-Mite (녹 갉아먹는 분말)

**도감** — 산화된 철 표면의 녹을 섭취하고 고순도 철 미분말로 정제 배출

- **Family:** 증식·결정·침식
- **Movement:** `POWDER`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 산화된 철 표면의 녹을 섭취하고 고순도 철 미분말로 정제 배출.
- **Simulation notes:** 산화된 철 표면의 녹을 섭취하고 고순도 철 미분말로 정제 배출.
- **출처:** Original Matter research

## OM-039 — Cryo-Algae (빙설 조류 포자)

**도감** — 눈과 얼음 표면에서 햇빛을 받아 광합성하며 붉은색 보온 발열막 형성

- **Family:** 증식·결정·침식
- **Movement:** `POWDER`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 눈과 얼음 표면에서 햇빛을 받아 광합성하며 붉은색 보온 발열막 형성.
- **Simulation notes:** 눈과 얼음 표면에서 햇빛을 받아 광합성하며 붉은색 보온 발열막 형성.
- **출처:** Original Matter research

## OM-040 — Pest-Resin (해충 유인 수지)

**도감** — 끈적이는 송진으로 공기 중 유기 포자를 흡착하여 호박(AMBER) 화석으로 고화

- **Family:** 증식·결정·침식
- **Movement:** `LIQUID`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 끈적이는 송진으로 공기 중 유기 포자를 흡착하여 호박(AMBER) 화석으로 고화.
- **Simulation notes:** 끈적이는 송진으로 공기 중 유기 포자를 흡착하여 호박(AMBER) 화석으로 고화.
- **출처:** Original Matter research

## OM-041 — Vector-Glass (운동량 편향 유리 / 편향경)

**도감** — 표면에 충돌한 모든 입자의 입사각을 무시하고 설정된 단일 방향(격자 각도)으로 강제 튕겨내는 키네틱 편향판

- **Family:** 이동·유동·밀도
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Deterministic Velocity Reflection (단방향 운동량 투사)
- **Simulation notes:** 표면에 충돌한 모든 입자의 입사각을 무시하고 설정된 단일 방향(격자 각도)으로 강제 튕겨내는 키네틱 편향판.
- **출처:** Original Matter research

## OM-042 — Siphon-Mercury (역모세관 수은 / 역류홍)

**도감** — 좁은 1픽셀 수직 틈새를 감지하면 중력을 거슬러 벽을 타고 초고속으로 기어오르는 고밀도 금속 유체

- **Family:** 이동·유동·밀도
- **Movement:** `LIQUID`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Negative Gravity Meniscus Creeping (수직 모세관 자가 부상)
- **Simulation notes:** 좁은 1픽셀 수직 틈새를 감지하면 중력을 거슬러 벽을 타고 초고속으로 기어오르는 고밀도 금속 유체.
- **출처:** Original Matter research

## OM-043 — Buoy-Sand (공기 부유사 / 부유사)

**도감** — 정지해 있을 때는 모래처럼 쌓이지만, 바람이나 기체 기류를 만나면 비중이 기체 이하로 떨어져 떠오르는 분말

- **Family:** 이동·유동·밀도
- **Movement:** `POWDER`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Gas-Coupled Fluidized Levitation (기류 반응 부유)
- **Simulation notes:** 정지해 있을 때는 모래처럼 쌓이지만, 바람이나 기체 기류를 만나면 비중이 기체 이하로 떨어져 떠오르는 분말.
- **출처:** Original Matter research

## OM-044 — Void-Plug (압력 흡입 진공전 / 진공전)

**도감** — 파괴되는 순간 주변 반경 5셀 내의 모든 기체와 유체를 중심점으로 강하게 빨아들이며 사라지는 함몰성 구체

- **Family:** 이동·유동·밀도
- **Movement:** `STATIC / POWDER`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Implosive Matter Sink (파열형 순간 진공 형성)
- **Simulation notes:** 파괴되는 순간 주변 반경 5셀 내의 모든 기체와 유체를 중심점으로 강하게 빨아들이며 사라지는 함몰성 구체.
- **출처:** Original Matter research

## OM-045 — Drift-Oil (역경사 활주유 / 활유)

**도감** — 아래로 흐르지 않고 가장 높은 곳을 향해 경사면을 거슬러 오르며 인화성을 유지하는 이상 오일

- **Family:** 이동·유동·밀도
- **Movement:** `LIQUID`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** Inverted Surface Slope Climbing (역경사 유동)
- **Simulation notes:** 아래로 흐르지 않고 가장 높은 곳을 향해 경사면을 거슬러 오르며 인화성을 유지하는 이상 오일.
- **출처:** Original Matter research

## OM-046 — Heavy-Air (초중량 산소 기체)

**도감** — 밀도가 물보다 높아 액체 바닥으로 가라앉으며 수중 호흡 기포 제공

- **Family:** 이동·유동·밀도
- **Movement:** `GAS`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 밀도가 물보다 높아 액체 바닥으로 가라앉으며 수중 호흡 기포 제공.
- **Simulation notes:** 밀도가 물보다 높아 액체 바닥으로 가라앉으며 수중 호흡 기포 제공.
- **출처:** Original Matter research

## OM-047 — Bouncy-Resin (고탄성 반발 수지)

**도감** — 낙하 속도에 비례하여 입자를 1.5배 높이로 튕겨내는 고무 트램펄린

- **Family:** 이동·유동·밀도
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 낙하 속도에 비례하여 입자를 1.5배 높이로 튕겨내는 고무 트램펄린.
- **Simulation notes:** 낙하 속도에 비례하여 입자를 1.5배 높이로 튕겨내는 고무 트램펄린.
- **출처:** Original Matter research

## OM-048 — Tunnel-Worm (중력 침강 관통탄)

**도감** — 오직 수직 아래 방향의 단단한 블록만을 지우며 무한히 파고드는 초고밀도 탄자

- **Family:** 이동·유동·밀도
- **Movement:** `POWDER`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 오직 수직 아래 방향의 단단한 블록만을 지우며 무한히 파고드는 초고밀도 탄자.
- **Simulation notes:** 오직 수직 아래 방향의 단단한 블록만을 지우며 무한히 파고드는 초고밀도 탄자.
- **출처:** Original Matter research

## OM-049 — Phase-Sieve (분자 체 격자)

**도감** — 기체와 물은 통과시키고 기름과 무거운 모래 분말은 걸러내는 나노 필터

- **Family:** 이동·유동·밀도
- **Movement:** `STATIC`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 기체와 물은 통과시키고 기름과 무거운 모래 분말은 걸러내는 나노 필터.
- **Simulation notes:** 기체와 물은 통과시키고 기름과 무거운 모래 분말은 걸러내는 나노 필터.
- **출처:** Original Matter research

## OM-050 — Crawling-Tar (온도 추적 타르)

**도감** — 주변에서 가장 온도가 차가운 셀을 향해 서서히 기어가는 점성 타르

- **Family:** 이동·유동·밀도
- **Movement:** `LIQUID`
- **상태:** `NEAR_TERM_CANDIDATE`
- **핵심 행동:** 주변에서 가장 온도가 차가운 셀을 향해 서서히 기어가는 점성 타르.
- **Simulation notes:** 주변에서 가장 온도가 차가운 셀을 향해 서서히 기어가는 점성 타르.
- **출처:** Original Matter research
