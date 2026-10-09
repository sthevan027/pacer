import { ACCENT_PRESETS } from "../../lib/accent";

export function AccentPicker({ value, onChange }: { value: string; onChange: (hex: string) => void }) {
  return (
    <div className="accentc" role="radiogroup" aria-label="Cor de destaque">
      {ACCENT_PRESETS.map((p) => (
        <button
          key={p.hex}
          type="button"
          role="radio"
          aria-checked={p.hex === value}
          aria-label={p.name}
          title={p.name}
          className={`accentb${p.hex === value ? " on" : ""}`}
          style={{ backgroundColor: p.hex }}
          onClick={() => onChange(p.hex)}
        />
      ))}
    </div>
  );
}
