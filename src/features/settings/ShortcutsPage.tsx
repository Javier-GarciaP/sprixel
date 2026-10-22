export type ShortcutsGroup = {
  title: string;
  rows: { id: string; label: string; keys: string[] }[];
};

// The "Keyboard shortcuts" page: one card per area, one row per command.
// The text and the key caps come ready from the caller.
export function ShortcutsPage({ groups }: { groups: ShortcutsGroup[] }) {
  return (
    <div className="grid grid-cols-3 gap-5">
      {groups.map((group) => (
        <section
          key={group.title}
          aria-label={group.title}
          className="rounded-lg border border-border bg-surface p-4"
        >
          <h3 className="mb-2 text-sm font-semibold text-text-muted">
            {group.title}
          </h3>
          <ul className="flex flex-col gap-2">
            {group.rows.map((row) => (
              <li
                key={row.id}
                className="flex items-center justify-between text-sm"
              >
                <span>{row.label}</span>
                <span className="flex gap-1">
                  {row.keys.map((key) => (
                    <kbd
                      key={key}
                      className="rounded border border-border bg-surface-2 px-1.5 font-mono text-xs text-text-muted"
                    >
                      {key}
                    </kbd>
                  ))}
                </span>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}
