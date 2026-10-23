import { useState } from "react";
import ghost from "@/assets/sprixel-ghost.png";
import { Button, Icon } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";
import type { RecentProject } from "../api/preferences";

type Props = {
  recents: RecentProject[];
  onNew: () => void;
  onOpen: () => void;
  onOpenRecent: (path: string) => void;
  onRemoveRecent: (path: string) => void;
  folderExists: (path: string) => Promise<boolean>;
};

export function WelcomeScreen({
  recents,
  onNew,
  onOpen,
  onOpenRecent,
  onRemoveRecent,
  folderExists,
}: Props) {
  const t = useT();
  // The path of a recent project whose folder was not found.
  const [missing, setMissing] = useState<string | null>(null);

  const openRecent = async (path: string) => {
    if (await folderExists(path)) {
      setMissing(null);
      onOpenRecent(path);
    } else {
      setMissing(path);
    }
  };

  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 p-6">
      <h1 className="font-title text-4xl font-bold">{t("welcome.title")}</h1>
      <p className="text-text-muted">{t("welcome.subtitle")}</p>
      <div className="flex gap-3">
        <Button variant="primary" onClick={onNew}>
          <Icon name="plus" />
          {t("welcome.new")}
        </Button>
        <Button onClick={onOpen}>
          <Icon name="folder" />
          {t("welcome.open")}
        </Button>
      </div>

      <section className="mt-4 w-[520px] max-w-full rounded-lg border border-border bg-surface p-4">
        <h2 className="mb-3 text-sm font-semibold text-text-muted">
          {t("welcome.recent")}
        </h2>
        {recents.length === 0 ? (
          <p className="flex items-center gap-3 text-sm text-text-muted">
            <img
              src={ghost}
              alt=""
              className="h-8 w-8 opacity-40 grayscale [image-rendering:pixelated]"
            />
            {t("welcome.recentEmpty")}
          </p>
        ) : (
          <ul className="flex flex-col gap-1">
            {recents.map((project) => (
              <li key={project.path}>
                <button
                  type="button"
                  onClick={() => openRecent(project.path)}
                  className="flex w-full items-center justify-between rounded-md px-2 py-1.5 text-left text-sm hover:bg-surface-2"
                >
                  <span className="font-semibold">{project.name}</span>
                  <span className="font-mono text-xs text-text-faint">
                    {project.path}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        )}
        {missing && (
          <div
            role="alert"
            className="mt-3 flex items-center justify-between gap-3 rounded-md border border-error px-3 py-2 text-sm text-error"
          >
            <span>{t("welcome.folderMissing", { path: missing })}</span>
            <button
              type="button"
              className="font-semibold underline"
              onClick={() => {
                onRemoveRecent(missing);
                setMissing(null);
              }}
            >
              {t("welcome.removeFromList")}
            </button>
          </div>
        )}
      </section>
    </div>
  );
}
