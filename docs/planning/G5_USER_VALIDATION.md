# G5 Pressure Chain — Visible Boiler User Validation

Status: **PASS / CLOSED — USER VALIDATION APPROVED (2026-08-16)**

Technical prerequisites are frozen:

- G5-A Pressure Field — TECHNICAL PASS / FROZEN
- G5-B Expansion / Confinement → Pressure — TECHNICAL PASS / FROZEN
- G5-C Pressure Stress → Rupture → Opening → Vent — TECHNICAL PASS / FROZEN

---

## 1. Demo Command & Controls

```powershell
cargo run -p powdergame-windows -- --pressure-demo
```
*(or run `run_g5_demo.bat`)*

Controls:
- `SPACE`: Play / Pause (starts paused at 60 TPS when played)
- `N`: Single tick step while paused
- `R`: Reset world simulation and metrics
- `ESC`: Exit

---

## 2. 2×2 Multi-Boiler Stress Lab Architecture

```text
┌─────────────────────────────────────────┬─────────────────────────────────────────┐
│ [A] TOP-LEFT: WOOD RELIEF (CANONICAL)   │ [B] TOP-RIGHT: STONE SEALED (CONTROL)   │
│ • 1x Floor Heater (T=150)               │ • 1x Floor Heater (T=150)               │
│ • 1x Upper Heater (T=110)               │ • 1x Upper Heater (T=110)               │
│ • 9-cell Wood Roof Relief Plug (x=60..68│ • 100% Unbreakable Stone Roof           │
├─────────────────────────────────────────┼─────────────────────────────────────────┤
│ [C] BOT-LEFT: WOOD RELIEF (EXTREME)     │ [D] BOT-RIGHT: DELAYED PRESSURE BREACH  │
│ • 3x Floor Heaters (T=220 Overdrive)    │ • 3x Floor Heaters (T=220 Overdrive)    │
│ • 1x Upper Heater (T=130, y=176)        │ • 1x Upper Heater (T=130, y=176)        │
│ • 9-cell Wood Roof Relief Plug (x=60..68│ • Solid Stone Roof + 9-cell Wood        │
│                                         │   Distant Side Seam (y=214..=222, x=242)│
└─────────────────────────────────────────┴─────────────────────────────────────────┘
```

### Experimental Symmetry Invariant (Panel C vs Panel D)
- **Shared Descriptor**: Both chambers share 100% identical initial thermal & matter conditions (`Water @ T=58.0`, `3x Floor Heaters @ T=220`, `Upper Heater @ T=130` centered at `y=176`).
- **Independent Variable**: Structural relief path only.
  - **Panel C**: Immediate Wood relief plug on roof (`y=170, x=60..=68`, 9 cells).
  - **Panel D**: Solid Stone roof + distant Wood seam on outer wall (`y=214..=222, x=242`, 9 cells) leading to exterior exhaust duct (`x=243..=254`).
- **Natural Propagation Delay**: Overpressure from boiling floor heaters propagates up the fluid medium without any artificial timers, local hot spots, pre-staged pressure, or radial impulse.

---

## 3. Final User Observation Evidence (Interactive Runtime, 2026-08-16)

The user directly observed and approved the 2×2 Stress Lab runtime:

1. **[A] WOOD RELIEF — CANONICAL**:
   - Peak Pressure: `650.0`
   - First Relief: `Tick 40`
   - Relief Plug Wood: `6/9` cells remaining after opening
   - Sustained Steam vent observed; Final state: `RELIEF ACTIVE / VENTING`
   - **Verdict**: Canonical Heat → Steam → confinement → Wood relief rupture → opening → vent chain **PASS**.

2. **[B] STONE SEALED — CONTROL**:
   - Peak Pressure: `650.0`
   - Rupture Event: `NONE`
   - Chamber Integrity: `100% SEALED`
   - Long-run sealed state maintained indefinitely
   - **Verdict**: Stone control remains intact under identical canonical heating **PASS**.

3. **[C] WOOD RELIEF — EXTREME**:
   - Peak Pressure: `1314.4`
   - First Relief: `Tick 35`
   - Sustained high-output upward vent plume observed
   - Faster relief (`35 < 40`) and higher peak pressure (`1314.4 > 650.0`) than Panel A.
   - **Verdict**: Extreme heating provides early relief release **PASS**.

4. **[D] DELAYED PRESSURE BREACH**:
   - Peak Pressure: `1307.7`
   - First Breach: `Tick 135`
   - Weak Seam Wood: `8/9` cells remaining after first rupture
   - Duct Steam Vent: `Tick 170`
   - Final state: `SIDE WALL BREACH -> VENTING`
   - **Relative Separation**: $t_D - t_C = 135 - 35 = 100\text{ ticks}$ ($\approx 1.67\text{ seconds}$ at 60 TPS).
   - **Verdict**: Delayed pressure propagation → structural stress → rupture → opening → duct vent **PASS**.

---

## 4. Automated Contract Fixture Evidence

Small deterministic GPU regression test (`two_by_two_multi_boiler_stress_lab_relative_ordering_contract`):
- `first_relief(C) = Tick 33` <= `first_relief(A) = Tick 36`
- `rupture(B) == NONE` (100% unbreakable sealed control)
- `first_breach(D) = Tick 133` (Separation: `133 - 33 = 100 ticks >= MIN_MEANINGFUL_DELAY (60)`)
- `breach_local_pressure(D) = 80.8 >= 80.0` (Wood threshold exceeded)
- `first_vent(D) = Tick 170 > Tick 133` (Matter vents into exterior exhaust duct)
- `test_c_d_initial_thermal_matter_symmetry`: $t=0$ Panel C & D internal cells 100% identical in Material & Temperature.

---

## 5. Temperature Unit Hygiene

Powdergame Temperature is a **relative gameplay scalar**, not Celsius.
- Water initial: `T = 58.0`
- Floor heaters: `T = 150` (Canonical), `T = 220` (Extreme)
- Upper heaters: `T = 110` (Canonical), `T = 130` (Extreme)
- Wood ignition: `T = 90.0`, Water boil: `T = 60.0`

---

## 6. G5 Production Simulation Invariant

The complete causal chain:
$$\text{Water Heated} \to \text{Steam Transition} \to \text{Insufficient Space} \to \text{Confinement Pressure} \to \text{Propagation} \to \text{Weak Structure Rupture} \to \text{Opening} \to \text{GAS Venting}$$
occurs entirely within the GPU production simulation pipeline without:
- Scripted timer rupture
- Pre-staged Pressure
- Pre-staged Steam
- Radial explosion solvers
- Fake vent animation

---

## 7. Final Gate Conclusion

**G5 — Pressure Chain: PASS / CLOSED**
**G5 User Validation: APPROVED (2026-08-16)**
**Next Milestone Gate: G6 — Parallel Integrity**
