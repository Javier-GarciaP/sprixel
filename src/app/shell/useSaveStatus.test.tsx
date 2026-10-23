// @vitest-environment jsdom
import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { projectStatus, retrySave } = vi.hoisted(() => ({
  projectStatus: vi.fn(),
  retrySave: vi.fn(),
}));
vi.mock("../api/project", () => ({ projectStatus, retrySave }));

import { useSaveStatus } from "./useSaveStatus";

beforeEach(() => {
  vi.useFakeTimers();
  projectStatus.mockReset();
  retrySave.mockReset();
});
afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

describe("useSaveStatus", () => {
  // Checks that the status is read right away and again on every interval
  // while a project is open.
  it("polls the status while a project is open", async () => {
    projectStatus.mockResolvedValue({ status: "saving" });
    const { result } = renderHook(() => useSaveStatus(true, 500));
    await act(async () => {});
    expect(result.current.state).toEqual({ status: "saving" });

    projectStatus.mockResolvedValue({ status: "saved" });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(500);
    });
    expect(result.current.state).toEqual({ status: "saved" });
  });

  // Checks that nothing is polled when no project is open, and that the
  // polling stops when the project closes.
  it("does not poll without a project and stops on close", async () => {
    projectStatus.mockResolvedValue({ status: "saved" });
    const { rerender } = renderHook(({ on }) => useSaveStatus(on, 500), {
      initialProps: { on: false },
    });
    await vi.advanceTimersByTimeAsync(2000);
    expect(projectStatus).not.toHaveBeenCalled();

    rerender({ on: true });
    await act(async () => {});
    const calls = projectStatus.mock.calls.length;
    rerender({ on: false });
    await vi.advanceTimersByTimeAsync(2000);
    expect(projectStatus.mock.calls.length).toBe(calls);
  });

  // Checks that an error status is passed on with its reason, and that Retry
  // asks the core to save again and shows the answer.
  it("shows an error and retries", async () => {
    projectStatus.mockResolvedValue({ status: "error", reason: "disk full" });
    retrySave.mockResolvedValue({ status: "saved" });
    const { result } = renderHook(() => useSaveStatus(true, 500));
    await act(async () => {});
    expect(result.current.state).toEqual({
      status: "error",
      reason: "disk full",
    });

    await act(async () => result.current.retry());
    expect(retrySave).toHaveBeenCalledTimes(1);
    expect(result.current.state).toEqual({ status: "saved" });
  });
});
