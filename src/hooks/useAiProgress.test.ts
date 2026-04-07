import { renderHook } from "@testing-library/react";
import { listen } from "@tauri-apps/api/event";
import { useAiProgress } from "./useAiProgress";

describe("useAiProgress", () => {
  it("subscribes to ai://progress on mount", () => {
    const onProgress = vi.fn();
    renderHook(() => useAiProgress(onProgress));

    expect(listen).toHaveBeenCalledWith("ai://progress", expect.any(Function));
  });

  it("calls onProgress callback with event payload", async () => {
    const onProgress = vi.fn();
    const mockUnlisten = vi.fn();
    const mockPayload = {
      completed: 5,
      total: 10,
      currentUrl: "https://example.com/page",
      tokensUsed: 1500,
      budgetRemaining: 8500,
    };

    vi.mocked(listen).mockImplementation(async (_event, handler) => {
      // Simulate an event arriving
      (handler as (event: { payload: unknown }) => void)({ payload: mockPayload });
      return mockUnlisten;
    });

    renderHook(() => useAiProgress(onProgress));

    // Wait for the async subscribe to complete
    await vi.waitFor(() => {
      expect(onProgress).toHaveBeenCalledWith(mockPayload);
    });
  });

  it("unsubscribes on unmount", async () => {
    const onProgress = vi.fn();
    const mockUnlisten = vi.fn();
    vi.mocked(listen).mockResolvedValue(mockUnlisten);

    const { unmount } = renderHook(() => useAiProgress(onProgress));

    // Wait for subscribe to complete
    await vi.waitFor(() => {
      expect(listen).toHaveBeenCalled();
    });

    unmount();
    expect(mockUnlisten).toHaveBeenCalled();
  });
});
