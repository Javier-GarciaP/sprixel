import { Toast } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";

export type SaveState =
  | { status: "saved" }
  | { status: "saving" }
  | { status: "error"; reason: string };

type Props = { state: SaveState; onRetry: () => void };

// The save status next to the project name. The state comes from outside, so
// the storage layer can feed it later.
export function SaveStatus({ state, onRetry }: Props) {
  const t = useT();

  if (state.status === "error") {
    return (
      <>
        <span className="flex items-center gap-2 text-xs">
          <span className="font-semibold text-error">{t("save.error")}</span>
          <button type="button" onClick={onRetry} className="font-semibold">
            {t("save.retry")}
          </button>
        </span>
        <Toast
          tone="error"
          message={t("save.errorMessage", { reason: state.reason })}
          actionLabel={t("save.retry")}
          onAction={onRetry}
        />
      </>
    );
  }

  return (
    <span className="text-xs text-text-faint">
      {state.status === "saving" ? t("save.saving") : t("save.saved")}
    </span>
  );
}
