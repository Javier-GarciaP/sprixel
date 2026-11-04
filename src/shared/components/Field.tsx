import { useId, type InputHTMLAttributes } from "react";

type Props = InputHTMLAttributes<HTMLInputElement> & {
  label: string;
  error?: string;
};

// A labeled text input. When `error` is set the field turns red and the
// message floats under it (out of the layout flow, so the form never jumps)
// and is linked to the input for screen readers.
export function Field({ label, error, ...rest }: Props) {
  const id = useId();
  return (
    <div className="relative flex flex-col gap-1">
      <label htmlFor={id} className="text-xs text-text-muted">
        {label}
      </label>
      <input
        id={id}
        aria-invalid={error ? true : undefined}
        aria-describedby={error ? `${id}-error` : undefined}
        className={`rounded-md border bg-bg px-3 py-2 text-sm text-text ${error ? "border-error" : "border-border"}`}
        {...rest}
      />
      {error && (
        <p
          id={`${id}-error`}
          className="absolute left-0 top-full z-10 mt-1 w-max max-w-full rounded-md border border-error bg-surface px-2 py-1 text-xs text-error shadow-lg"
        >
          {error}
        </p>
      )}
    </div>
  );
}
