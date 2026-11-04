import type { TranslationKey } from "@/shared/lib/i18n";

type Translate = (
  key: TranslationKey,
  values?: Record<string, string | number>,
) => string;

// The message for an error of a power tool: a clear text for the usual codes,
// and the message of the core for the others.
export function powerErrorText(
  t: Translate,
  error: { code?: string; message?: string } | undefined,
): string {
  switch (error?.code) {
    case "not_square":
      return t("power.notSquare");
    case "frame_busy":
      return t("power.busy");
    case "palette_mismatch":
      return t("power.paletteMismatch");
    case "region_out":
      return t("power.regionOut");
    default:
      return t("power.error", { message: error?.message ?? "" });
  }
}
