import { Button, Dialog } from "@/shared/components";
import { useT } from "@/shared/lib/i18n";

// How many names the list shows before "and N more".
const MAX_LISTED = 8;

type Props = {
  title: string;
  message: string;
  // What will go to the trash, by name.
  items: string[];
  confirmLabel: string;
  onConfirm: () => void;
  onClose: () => void;
};

// Asks before sending a folder, or several sprites, to the trash. The focus
// starts on Cancel, so pressing Enter by mistake does not delete anything.
export function ConfirmDeleteDialog({
  title,
  message,
  items,
  confirmLabel,
  onConfirm,
  onClose,
}: Props) {
  const t = useT();
  const listed = items.slice(0, MAX_LISTED);
  const more = items.length - listed.length;
  return (
    <Dialog title={title} closeLabel={t("dialog.close")} onClose={onClose}>
      <p className="text-sm text-text-muted">{message}</p>
      {listed.length > 0 && (
        <ul className="my-3 max-h-40 overflow-auto rounded-md border border-border bg-bg p-2 font-mono text-xs">
          {listed.map((name, index) => (
            <li key={`${name}:${index}`}>{name}</li>
          ))}
          {more > 0 && (
            <li className="text-text-faint">
              {t("confirm.more", { count: more })}
            </li>
          )}
        </ul>
      )}
      <div className="mt-4 flex justify-end gap-2">
        <Button autoFocus onClick={onClose}>
          {t("dialog.cancel")}
        </Button>
        <Button variant="danger" onClick={onConfirm}>
          {confirmLabel}
        </Button>
      </div>
    </Dialog>
  );
}
