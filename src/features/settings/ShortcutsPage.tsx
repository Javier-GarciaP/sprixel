export type ShortcutsGroup = {
  title: string;
  rows: {
    id: string;
    label: string;
    keys: string[];
    // Another way to do it, shown after a word like "or".
    alternative?: { word: string; keys: string[] };
  }[];
};

function KeyCaps({ keys }: { keys: string[] }) {
  return keys.map((key) => (
    <kbd
      key={key}
      className="rounded border border-border bg-surface-2 px-1.5 font-mono text-xs text-text-muted"
    >
      {key}
    </kbd>
  ));
}

// The project menu names end in "..." when they open a dialog. Here they are
// plain names: "Open", not "Open...".
const plain = (label: string) => label.replace(/\s*(\.{3}|…)$/, "");

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
                <span>{plain(row.label)}</span>
                <span className="flex items-center gap-1">
                  <KeyCaps keys={row.keys} />
                  {row.alternative && (
                    <>
                      <span className="px-0.5 text-xs text-text-faint">
                        {row.alternative.word}
                      </span>
                      <KeyCaps keys={row.alternative.keys} />
                    </>
                  )}
                </span>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}
