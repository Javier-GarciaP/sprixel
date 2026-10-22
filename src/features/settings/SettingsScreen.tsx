import { useState } from "react";
import { useT } from "@/shared/lib/i18n";
import { AppearancePage } from "./AppearancePage";
import { ShortcutsPage, type ShortcutsGroup } from "./ShortcutsPage";

type Page = "appearance" | "shortcuts";

// The Settings screen: a list of pages on the left, the chosen page on the right.
export function SettingsScreen({
  shortcutGroups,
}: {
  shortcutGroups: ShortcutsGroup[];
}) {
  const t = useT();
  const [page, setPage] = useState<Page>("appearance");

  const pages = [
    { id: "appearance" as const, label: t("settings.page.appearance") },
    { id: "shortcuts" as const, label: t("shortcuts.title") },
  ];

  return (
    <div className="flex h-full">
      <nav className="w-64 shrink-0 border-r border-border p-3">
        <h2 className="mb-2 px-2 text-xs font-semibold text-text-muted">
          {t("settings.title")}
        </h2>
        {pages.map(({ id, label }) => (
          <button
            key={id}
            type="button"
            aria-current={page === id ? "page" : undefined}
            onClick={() => setPage(id)}
            className={`mb-1 flex w-full rounded-md border-l-2 px-3 py-2 text-left text-sm ${page === id ? "border-amber bg-surface-2 font-semibold" : "border-transparent text-text-muted"}`}
          >
            {label}
          </button>
        ))}
      </nav>
      <section className="min-w-0 flex-1 p-10">
        <h1 className="mb-6 font-title text-3xl font-bold">
          {pages.find((p) => p.id === page)?.label}
        </h1>
        {page === "appearance" ? (
          <AppearancePage />
        ) : (
          <ShortcutsPage groups={shortcutGroups} />
        )}
      </section>
    </div>
  );
}
