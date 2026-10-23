type Props = {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
};

export function Switch({ checked, onChange, label }: Props) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      onClick={() => onChange(!checked)}
      className={`flex h-5 w-9 items-center rounded-full p-0.5 ${checked ? "bg-amber justify-end" : "bg-border-strong justify-start"}`}
    >
      <span className="h-4 w-4 rounded-full bg-ink" />
    </button>
  );
}
