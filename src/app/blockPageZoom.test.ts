// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { blockPageZoom } from "./blockPageZoom";

let stop = () => {};
afterEach(() => stop());

const wheel = (ctrlKey: boolean) => {
  const event = new WheelEvent("wheel", {
    ctrlKey,
    cancelable: true,
    bubbles: true,
  });
  document.body.dispatchEvent(event);
  return event.defaultPrevented;
};

describe("blockPageZoom", () => {
  // Checks that Control and the wheel anywhere in the app is prevented, so
  // the webview never zooms.
  it("prevents Control and the wheel", () => {
    stop = blockPageZoom();
    expect(wheel(true)).toBe(true);
  });

  // Checks that the plain wheel still scrolls.
  it("leaves the plain wheel alone", () => {
    stop = blockPageZoom();
    expect(wheel(false)).toBe(false);
  });

  // Checks that stopping removes the listener.
  it("stops when asked", () => {
    blockPageZoom()();
    expect(wheel(true)).toBe(false);
  });
});
