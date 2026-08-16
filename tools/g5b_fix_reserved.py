from pathlib import Path

path = Path("engine/gpu/src/expansion_pressure.wgsl")
text = path.read_text(encoding="utf-8")
old = "active"
if old not in text:
    raise RuntimeError("expected G5-B generated shader to contain reserved field 'active'")
text = text.replace("active: u32", "enabled: u32")
text = text.replace("effect.active = 1u", "effect.enabled = 1u")
text = text.replace("effect.active != 0u", "effect.enabled != 0u")
path.write_text(text, encoding="utf-8", newline="\n")
print("G5-B WGSL reserved keyword corrected: active -> enabled")
