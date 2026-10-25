import type { ReactNode } from "react";
import { useT } from "@/shared/lib/i18n";

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="rounded-lg border border-border bg-surface p-3">
      <h2 className="mb-2 text-xs font-semibold text-text-muted">{title}</h2>
      {children}
    </section>
  );
}

type Props = {
  // The palette and the onion skin controls are made by the editor.
  // The preview that plays the animation (made by the frames feature).
  preview: ReactNode;
  palette: ReactNode;
  onion: ReactNode;
  // What the agent that is drawing now is doing, if there is one.
  agent?: string | null;
};

// The right panel: preview, palette, onion skin and agent activity.
export function RightPanel({ preview, palette, onion, agent }: Props) {
  const t = useT();
  return (
    <aside
      aria-label={t("editor.panel")}
      className="flex w-72 shrink-0 flex-col gap-3 overflow-y-auto border-l border-border bg-bg p-3"
    >
      <Section title={t("editor.preview")}>{preview}</Section>
      <Section title={t("editor.palette")}>{palette}</Section>
      <Section title={t("editor.onionSkin")}>{onion}</Section>
      <Section title={t("editor.agentActivity")}>
        <p className="text-xs text-text-muted">
          {agent ?? t("editor.agentActivityEmpty")}
        </p>
      </Section>
    </aside>
  );
}
