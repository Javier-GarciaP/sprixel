import type { useT } from "@/shared/lib/i18n";

type T = ReturnType<typeof useT>;

// "1 sprite", "4 sprites".
export const spritesText = (t: T, count: number) =>
  count === 1 ? t("export.sprites.one") : t("export.sprites.other", { count });

export const framesText = (t: T, count: number) =>
  count === 1 ? t("export.frames.one") : t("export.frames.other", { count });

export const filesText = (t: T, count: number) =>
  count === 1 ? t("export.files.one") : t("export.files.other", { count });
