import type { NameProblem } from "@/shared/lib/core";
import type { TranslationKey } from "@/shared/lib/i18n";
import type { NameState } from "./useNameCheck";

const keys: Record<NameProblem, TranslationKey> = {
  empty: "name.empty",
  invalid_name: "name.invalid",
  folder_exists: "name.folderExists",
  sprite_exists: "name.spriteExists",
};

// The text to show under a name field, or undefined when there is nothing to
// say. An empty name is only an error after the person typed in the field.
export function nameErrorKey(
  state: NameState,
  typed: boolean,
): TranslationKey | undefined {
  if (state.status !== "problem") return undefined;
  if (state.problem === "empty" && !typed) return undefined;
  return keys[state.problem];
}
