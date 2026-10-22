import type { ButtonHTMLAttributes } from "react";

type Variant = "primary" | "secondary" | "danger";

const styles: Record<Variant, string> = {
  // Amber with a hard offset shadow, like the design.
  primary:
    "bg-amber text-ink border-transparent shadow-[0_2px_0_var(--color-warning)]",
  secondary: "bg-surface text-text border-border",
  danger: "bg-transparent text-error border-error",
};

type Props = ButtonHTMLAttributes<HTMLButtonElement> & { variant?: Variant };

export function Button({ variant = "secondary", className, ...rest }: Props) {
  return (
    <button
      type="button"
      className={`inline-flex items-center gap-2 rounded-md border px-3 py-1.5 text-sm font-semibold disabled:cursor-not-allowed disabled:opacity-40 ${styles[variant]} ${className ?? ""}`}
      {...rest}
    />
  );
}
