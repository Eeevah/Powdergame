from pathlib import Path

p = Path('apps/windows/src/main.rs')
s = p.read_text(encoding='utf-8')

repls = [
("//!   `--thermal-demo`  — G4 thermal lab (PHASE: sequential Ice melt by\n//!                       distance from the hot source; HEAT FLOW: sealed\n//!                       Water vs Oil conduction tubes; COMBUSTION: Wood\n//!                       ignition front travelling along a strip).\n//! Forest scene is unused by the G3/G4 demos.",
 "//!   `--thermal-demo`  — G4 thermal lab (PHASE: sequential Ice melt by\n//!                       distance from the hot source; HEAT FLOW: sealed\n//!                       Water vs Oil conduction tubes; COMBUSTION: Wood\n//!                       ignition front travelling along a strip),\n//!   `--pressure-demo` — G5 twin-boiler user-validation scene: identical\n//!                       heated Water chambers, weak Wood relief plug on the\n//!                       left and unbreakable Stone control on the right.\n//! Forest scene is unused by the G3/G4/G5 demos."),
("const THERMAL_DEMO_TPS: u32 = 60;", "const THERMAL_DEMO_TPS: u32 = 60;\nconst PRESSURE_DEMO_TPS: u32 = 60;"),
("const THERMAL_DEMO_TITLE: &str =\n    \"Powdergame G4 Thermal Observatory | 4 Large Panels + Live Metrics\";",
 "const THERMAL_DEMO_TITLE: &str =\n    \"Powdergame G4 Thermal Observatory | 4 Large Panels + Live Metrics\";\nconst PRESSURE_DEMO_TITLE: &str =\n    \"Powdergame G5 Pressure Chain | WOOD RELIEF vs STONE SEALED | Heat → Steam → Pressure → Rupture → Vent\";"),
("    Thermal,\n}", "    Thermal,\n    Pressure,\n}"),
("            DemoMode::Thermal => THERMAL_DEMO_TPS,", "            DemoMode::Thermal => THERMAL_DEMO_TPS,\n            DemoMode::Pressure => PRESSURE_DEMO_TPS,"),
("            DemoMode::Thermal => THERMAL_DEMO_TITLE,\n            DemoMode::None => \"Powdergame — G0 Runtime\",",
 "            DemoMode::Thermal => THERMAL_DEMO_TITLE,\n            DemoMode::Pressure => PRESSURE_DEMO_TITLE,\n            DemoMode::None => \"Powdergame — G0 Runtime\","),
("                DemoMode::Thermal => (320, 192),\n                _ => (128, 128),",
 "                DemoMode::Thermal => (320, 192),\n                DemoMode::Pressure => (128, 128),\n                _ => (128, 128),"),
("            DemoMode::Thermal => {\n                stage_thermal_demo(&simulation)?;\n                println!(\"[powdergame] thermal demo: 4-panel large observatory staged\");\n            }",
 "            DemoMode::Thermal => {\n                stage_thermal_demo(&simulation)?;\n                println!(\"[powdergame] thermal demo: 4-panel large observatory staged\");\n            }\n            DemoMode::Pressure => {\n                stage_pressure_demo(&simulation)?;\n                println!(\"[powdergame] pressure demo: twin boilers staged (Wood relief vs Stone control)\");\n            }"),
("                DemoMode::Thermal => PresentationPalette::ThermalLab,\n                _ => PresentationPalette::Forest,",
 "                DemoMode::Thermal | DemoMode::Pressure => PresentationPalette::ThermalLab,\n                _ => PresentationPalette::Forest,"),
("        DemoMode::Thermal => stage_thermal_demo(simulation),\n        DemoMode::None => Ok(()),",
 "        DemoMode::Thermal => stage_thermal_demo(simulation),\n        DemoMode::Pressure => stage_pressure_demo(simulation),\n        DemoMode::None => Ok(()),"),
("/// `--thermal-demo` (or their `POWDERGAME_*_DEMO=1` env equivalents).",
 "/// `--thermal-demo` / `--pressure-demo` (or their `POWDERGAME_*_DEMO=1` env equivalents)."),
("            \"--thermal-demo\" => return DemoMode::Thermal,", "            \"--thermal-demo\" => return DemoMode::Thermal,\n            \"--pressure-demo\" => return DemoMode::Pressure,"),
("    if std::env::var(\"POWDERGAME_THERMAL_DEMO\").as_deref() == Ok(\"1\") {\n        return DemoMode::Thermal;\n    }\n    DemoMode::None",
 "    if std::env::var(\"POWDERGAME_THERMAL_DEMO\").as_deref() == Ok(\"1\") {\n        return DemoMode::Thermal;\n    }\n    if std::env::var(\"POWDERGAME_PRESSURE_DEMO\").as_deref() == Ok(\"1\") {\n        return DemoMode::Pressure;\n    }\n    DemoMode::None"),
("        DemoMode::Thermal => println!(\n            \"[powdergame] thermal demo: 320×192 thermal observatory \\\n             (4 large panels + live diagnostic metrics), 60 TPS, starts PAUSED \\\n             (SPACE play | N step | R reset | ESC quit)\"\n        ),\n        DemoMode::None => {}",
 "        DemoMode::Thermal => println!(\n            \"[powdergame] thermal demo: 320×192 thermal observatory \\\n             (4 large panels + live diagnostic metrics), 60 TPS, starts PAUSED \\\n             (SPACE play | N step | R reset | ESC quit)\"\n        ),\n        DemoMode::Pressure => println!(\n            \"[powdergame] pressure demo: 128×128 twin boilers, 60 TPS. \\\n             LEFT Wood relief plug should rupture/vent; RIGHT Stone control stays sealed. \\\n             Starts PAUSED (SPACE play | N step | R reset | ESC quit)\"\n        ),\n        DemoMode::None => {}")
]

for old, new in repls:
    if old not in s:
        raise SystemExit('missing anchor: ' + old[:100])
    s = s.replace(old, new, 1)

anchor = "/// Stages the G2 stylized-forest movement scene on the 128×128 demo world.\nfn stage_movement_demo"
if anchor not in s:
    raise SystemExit('missing stage_movement_demo anchor')

stage = r'''/// Stages the G5 twin-boiler user-validation scene on the 128×128 demo world.
///
/// This fixture does not inject Pressure and does not open any vent. Both
/// boilers start with the same dense Water charge at T=58, just below the
/// Water→Steam threshold. A real hot-Stone floor conducts heat into the
/// Water. The left boiler has a one-cell Wood relief plug; the right uses
/// Stone at the corresponding location as an unbreakable control.
///
/// Expected emergent chain on the left:
/// thermal conduction → Water boils → yield=2 expansion is blocked by dense
/// Matter → confinement Pressure accumulates/propagates → Wood threshold 80
/// is exceeded → Wood self-writes EMPTY → ordinary GAS movement vents Steam.
/// The right-hand Stone control should remain sealed under the same rules.
fn stage_pressure_demo(simulation: &Simulation) -> Result<(), GpuError> {
    let q = &simulation.context.queue;
    let set = |x: i64, y: i64, id: u32| simulation.world.write_material(q, x, y, id);
    let set_t = |x: i64, y: i64, t: f32| simulation.world.write_temperature(q, x, y, t);

    // Central divider / visual baseline.
    for y in 8..=119 {
        set(63, y, MATERIAL_STONE)?;
    }

    // Build one boiler. Geometry is identical except for the center roof plug.
    let build_boiler = |x0: i64, x1: i64, plug_material: u32| -> Result<(), GpuError> {
        let roof_y = 44i64;
        let bottom_y = 108i64;
        let plug_l = (x0 + x1) / 2 - 4;
        let plug_r = (x0 + x1) / 2 + 4;

        // Side walls and base shell.
        for y in roof_y..=bottom_y {
            set(x0, y, MATERIAL_STONE)?;
            set(x1, y, MATERIAL_STONE)?;
        }
        for x in x0..=x1 {
            set(x, bottom_y, MATERIAL_STONE)?;
            set_t(x, bottom_y, 150.0)?;
        }

        // One-cell roof. Only the 9-cell center plug differs between boilers.
        for x in (x0 + 1)..x1 {
            let mat = if x >= plug_l && x <= plug_r {
                plug_material
            } else {
                MATERIAL_STONE
            };
            set(x, roof_y, mat)?;
            set_t(x, roof_y, 20.0)?;
        }

        // Dense water charge. No EMPTY neighbor is available inside the shell,
        // so boiling yield requests must either win a newly opened plug or
        // become confinement Pressure.
        for y in (roof_y + 1)..bottom_y {
            for x in (x0 + 1)..x1 {
                set(x, y, MATERIAL_WATER)?;
                set_t(x, y, 58.0)?;
            }
        }

        // Chimney rails above the plug make the vent plume easy to read while
        // leaving the center fully EMPTY. They are presentation geometry only.
        for y in 8..roof_y {
            set(plug_l - 2, y, MATERIAL_STONE)?;
            set(plug_r + 2, y, MATERIAL_STONE)?;
        }
        Ok(())
    };

    // LEFT: weak Wood relief plug. RIGHT: Stone control.
    build_boiler(8, 57, MATERIAL_WOOD)?;
    build_boiler(70, 119, MATERIAL_STONE)?;

    // Two small pedestal marks distinguish the chambers even without text.
    for x in 24..=41 {
        set(x, 116, MATERIAL_WOOD)?;
    }
    for x in 86..=103 {
        set(x, 116, MATERIAL_STONE)?;
    }

    Ok(())
}

/// Stages the G2 stylized-forest movement scene on the 128×128 demo world.
fn stage_movement_demo'''
s = s.replace(anchor, stage, 1)

p.write_text(s, encoding='utf-8', newline='\n')
print('G5 pressure demo patch applied')
