import { useEffect, useState } from "react";
import { checkName, type NameProblem } from "@/shared/lib/core";

export type NameState =
  // The core has not answered yet for this name.
  | { status: "checking" }
  | { status: "ok" }
  | { status: "problem"; problem: NameProblem };

// Asks the core if a name can be used in a folder, so the dialogs show the
// same rules as the core. An answer for an older name is ignored.
export function useNameCheck(
  parent: string[],
  name: string,
  current?: string,
): NameState {
  const [answer, setAnswer] = useState<{
    asked: string;
    problem: NameProblem | null;
  } | null>(null);
  const asked = JSON.stringify([parent, name, current ?? null]);

  useEffect(() => {
    let stopped = false;
    checkName(parent, name, current)
      .then((problem) => {
        if (!stopped) setAnswer({ asked, problem });
      })
      // Without an answer the button stays disabled; the core still checks on create.
      .catch(() => {});
    return () => {
      stopped = true;
    };
    // `asked` stands for parent, name and current.
  }, [asked]);

  if (!answer || answer.asked !== asked) return { status: "checking" };
  return answer.problem
    ? { status: "problem", problem: answer.problem }
    : { status: "ok" };
}
