from pathlib import Path

main = Path('apps/windows/src/main.rs')
s = main.read_text(encoding='utf-8')
old = '''        // Chimney rails above the plug make the vent plume easy to read while
        // leaving the center fully EMPTY. They are presentation geometry only.
        for y in 8..roof_y {
            set(plug_l - 2, y, MATERIAL_STONE)?;
            set(plug_r + 2, y, MATERIAL_STONE)?;
        }
'''
new = '''        // Identical upper Stone heater plate in both boilers. It remains five
        // Water rows below the roof plug, so the plug is never directly heated
        // or scripted. Nearby Water crosses the boil threshold quickly through
        // ordinary thermal conduction, making the pressure-chain readable in
        // a short user-validation run without changing frozen G5 physics.
        let heater_y = roof_y + 6;
        for x in (plug_l - 2)..=(plug_r + 2) {
            set(x, heater_y, MATERIAL_STONE)?;
            set_t(x, heater_y, 110.0)?;
        }

        // Chimney rails above the plug make the vent plume easy to read while
        // leaving the center fully EMPTY. They are presentation geometry only.
        for y in 8..roof_y {
            set(plug_l - 2, y, MATERIAL_STONE)?;
            set(plug_r + 2, y, MATERIAL_STONE)?;
        }
'''
if old not in s:
    raise SystemExit('main heater anchor missing')
s = s.replace(old, new, 1)
s = s.replace(
    '/// Water. The left boiler has a one-cell Wood relief plug; the right uses\n/// Stone at the corresponding location as an unbreakable control.',
    '/// Water. An identical upper Stone heater plate is placed five Water rows\n/// below each roof so the visible event occurs promptly without injecting\n/// Pressure. The left boiler has a one-cell Wood relief plug; the right uses\n/// Stone at the corresponding location as an unbreakable control.',
    1,
)
main.write_text(s, encoding='utf-8', newline='\n')

doc = Path('docs/planning/G5_USER_VALIDATION.md')
d = doc.read_text(encoding='utf-8')
d = d.replace(
    '- Hot Stone floor is staged at `T = 150.0` and heats Water through normal thermal conduction.\n- Roof is Stone except for a one-cell-thick, 9-cell-wide Wood relief plug.',
    '- Hot Stone floor is staged at `T = 150.0` and heats Water through normal thermal conduction.\n- An identical upper Stone heater plate at `T = 110.0` sits five Water rows below each roof plug. It accelerates observation timing through normal conduction; it does not write Pressure or touch the plug directly.\n- Roof is Stone except for a one-cell-thick, 9-cell-wide Wood relief plug.',
    1,
)
d = d.replace(
    'The right chamber has the same water charge, temperature and hot floor, but the corresponding roof plug is Stone.',
    'The right chamber has the same water charge, temperature, hot floor and upper heater plate, but the corresponding roof plug is Stone.',
    1,
)
doc.write_text(d, encoding='utf-8', newline='\n')
print('pressure demo timing tuned with symmetric upper heaters')
