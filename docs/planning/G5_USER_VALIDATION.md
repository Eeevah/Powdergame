# G5 Pressure Chain — Visible Boiler User Validation

Status: **PENDING USER VALIDATION**

Technical prerequisites are frozen:

- G5-A Pressure Field — TECHNICAL PASS / FROZEN
- G5-B Expansion / Confinement → Pressure — TECHNICAL PASS / FROZEN
- G5-C Pressure Stress → Rupture → Opening → Vent — TECHNICAL PASS / FROZEN

The remaining G5 gate is deliberately visual/product-level. The goal is not to prove another unit invariant; it is to verify that the small-rule causal chain reads as one convincing event to a player.

## Demo command

```powershell
cargo run -p powdergame-windows -- --pressure-demo
```

Controls:

- `SPACE`: Play / Pause
- `N`: single tick while paused
- `R`: reset fixture
- `ESC`: exit

The demo starts paused at 60 TPS when played.

## Fixture

A 128×128 twin-boiler scene uses the same production simulation path for both chambers.

### Left boiler — weak relief structure

- Dense Water charge starts at `T = 58.0`, below Water→Steam threshold.
- Hot Stone floor is staged at `T = 150.0` and heats Water through normal thermal conduction.
- An identical upper Stone heater plate at `T = 110.0` sits five Water rows below each roof plug. It accelerates observation timing through normal conduction; it does not write Pressure or touch the plug directly.
- Roof is Stone except for a one-cell-thick, 9-cell-wide Wood relief plug.
- No Pressure is staged by the demo.
- No vent is pre-opened.

Expected causal chain:

```text
Hot Stone conducts heat
→ Water crosses boiling threshold
→ Water → Steam with matter_yield=2
→ dense chamber cannot satisfy extra-Matter expansion
→ confinement Pressure generated
→ Pressure propagates through Liquid/Gas
→ Wood plug sees neighboring Pressure >= 80
→ Wood self-writes EMPTY
→ opening exists
→ ordinary GAS movement sends Steam through opening
→ plume rises through chimney
```

### Right boiler — sealed control

The right chamber has the same water charge, temperature, hot floor and upper heater plate, but the corresponding roof plug is Stone. Stone is intentionally unbreakable in the M0 G5 contract.

Expected result: it remains sealed while the left Wood relief boiler opens and vents.

## User acceptance criteria

G5 can be marked **PASS / CLOSED** only when the user directly observes and approves the following:

1. The initial scene is understandable before simulation starts.
2. Both boilers visibly heat/transition rather than receiving pre-staged Steam or Pressure.
3. The left Wood plug opens as a consequence of the simulation, not a timed/scripted edit.
4. Steam exits only after the opening is created.
5. The right Stone control remains sealed under the same starting conditions.
6. The sequence reads naturally as `heat → steam → confinement → rupture → vent` without needing hidden special-case explosion logic.
7. No obvious regression, device loss, corruption, or presentation artifact prevents interpreting the event.

If the event is technically correct but too fast, too slow, visually ambiguous, or difficult to read, G5 remains open and only the **demo fixture/presentation** should be tuned. Frozen G5-A/B/C simulation contracts must not be weakened just to make the demo prettier.
