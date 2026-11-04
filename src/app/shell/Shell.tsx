import type { ReactNode } from "react";
import { BottomBar } from "./BottomBar";
import { TopBar } from "./TopBar";

type Props = {
  topBar: Parameters<typeof TopBar>[0];
  settingsOpen: boolean;
  onSettings: () => void;
  children: ReactNode;
};

// Fixed regions: top bar, bottom bar and a body that scrolls inside.
// Features plug into the body; the shell does not know their content.
export function Shell({ topBar, settingsOpen, onSettings, children }: Props) {
  return (
    <div className="flex h-screen flex-col bg-bg text-text">
      <TopBar {...topBar} />
      <main className="min-h-0 flex-1 overflow-auto">{children}</main>
      <BottomBar settingsOpen={settingsOpen} onSettings={onSettings} />
    </div>
  );
}
