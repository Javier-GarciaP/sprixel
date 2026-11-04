// @vitest-environment jsdom
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "@/shared/lib/i18n";
import { AnimationDialog } from "./AnimationDialog";
import type { Tag } from "./types";

afterEach(cleanup);

function show(
  props: {
    tag?: Tag;
    others?: string[];
    onSave?: (tag: Tag) => Promise<unknown>;
  } = {},
) {
  const onSave = vi.fn<(tag: Tag) => Promise<unknown>>(
    props.onSave ?? (async () => undefined),
  );
  const onClose = vi.fn();
  render(
    <I18nProvider language="en">
      <AnimationDialog
        frameCount={7}
        tag={props.tag}
        others={props.others ?? []}
        onSave={onSave}
        onClose={onClose}
      />
    </I18nProvider>,
  );
  return { onSave, onClose };
}

const type = (label: string, value: string) =>
  fireEvent.change(screen.getByLabelText(label), { target: { value } });
const create = () =>
  screen.getByRole("button", { name: "Create animation" }) as HTMLButtonElement;

describe("AnimationDialog", () => {
  // Checks the "Create an animation" scenario: "walk" from frame 1 to 4 at 12
  // fps repeating is sent to be saved, and the dialog closes.
  it("saves a valid animation", async () => {
    const { onSave, onClose } = show();
    type("Name", "walk");
    type("Last frame", "4");
    fireEvent.click(create());
    await waitFor(() =>
      expect(onSave).toHaveBeenCalledWith({
        name: "walk",
        from: 1,
        to: 4,
        fps: 12,
        mode: "loop",
      }),
    );
    await waitFor(() => expect(onClose).toHaveBeenCalled());
  });

  // Checks that the button is off until there is a name, and that an empty
  // name is not shown as an error before the user types.
  it("starts with the button off and no error", () => {
    show();
    expect(create().disabled).toBe(true);
    expect(screen.queryByText("Write a name")).toBeNull();
  });

  // Checks the "Invalid range" scenario: an end frame below the start frame
  // shows an error next to the field and disables the button.
  it("shows an error and disables the button for a bad range", async () => {
    show();
    type("Name", "walk");
    type("First frame", "4");
    type("Last frame", "2");
    expect(
      await screen.findByText("The last frame cannot be before the first"),
    ).toBeTruthy();
    expect(create().disabled).toBe(true);
    expect(
      screen.getByLabelText("Last frame").getAttribute("aria-invalid"),
    ).toBe("true");
    type("Last frame", "5");
    await waitFor(() => expect(create().disabled).toBe(false));
  });

  // Checks the other errors: a frame that does not exist, a speed outside 1 to
  // 60, and a name that the sprite already uses.
  it("checks the frames, the speed and the name", async () => {
    show({ others: ["idle"] });
    type("Name", "Idle");
    expect(await screen.findByText(/already has an animation/)).toBeTruthy();
    type("Name", "walk");
    type("Last frame", "9");
    expect(await screen.findByText("Use a frame from 1 to 7")).toBeTruthy();
    type("Last frame", "3");
    type("Speed (fps)", "61");
    expect(
      await screen.findByText("The speed is from 1 to 60 fps"),
    ).toBeTruthy();
    expect(create().disabled).toBe(true);
  });

  // Checks the repeat mode: Repeat is the start, and Ping-pong is sent when it
  // is chosen.
  it("sends the repeat mode that is chosen", async () => {
    const { onSave } = show();
    expect(
      screen
        .getByRole("radio", { name: "Repeat" })
        .getAttribute("aria-checked"),
    ).toBe("true");
    type("Name", "walk");
    fireEvent.click(screen.getByRole("radio", { name: "Ping-pong" }));
    fireEvent.click(create());
    await waitFor(() => expect(onSave).toHaveBeenCalled());
    expect(onSave.mock.calls[0][0].mode).toBe("pingpong");
  });

  // Checks the edit mode: it starts with the values of the animation, its
  // own name is not "taken", and the button says Save.
  it("edits an animation", async () => {
    const tag: Tag = { name: "walk", from: 2, to: 5, fps: 24, mode: "once" };
    const { onSave } = show({ tag, others: ["idle"] });
    expect((screen.getByLabelText("Name") as HTMLInputElement).value).toBe(
      "walk",
    );
    expect(
      (screen.getByLabelText("Speed (fps)") as HTMLInputElement).value,
    ).toBe("24");
    expect(
      screen.getByRole("radio", { name: "Once" }).getAttribute("aria-checked"),
    ).toBe("true");
    type("Last frame", "6");
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(onSave).toHaveBeenCalledWith({ ...tag, to: 6 }));
  });

  // Checks that a failure of the core stays in the dialog, which stays open.
  it("shows a failure of the core and stays open", async () => {
    const { onClose } = show({
      onSave: () => Promise.reject({ message: "no room" }),
    });
    type("Name", "walk");
    fireEvent.click(create());
    expect((await screen.findByRole("alert")).textContent).toBe("no room");
    expect(onClose).not.toHaveBeenCalled();
  });
});
