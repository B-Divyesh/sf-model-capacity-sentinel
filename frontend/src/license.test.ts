import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  consumeLicenseFromUrl,
  storeToken,
  storedToken,
  verifyLicense,
} from "./license";

describe("Atlas license lifecycle", () => {
  beforeEach(() => {
    localStorage.clear();
    history.replaceState({}, "", "/?license=returned-token&view=atlas");
  });

  afterEach(() => vi.unstubAllGlobals());

  it("@claim:atlas-license-lifecycle stores returned and pasted licenses, caches verification, and fails closed", async () => {
    consumeLicenseFromUrl();
    expect(storedToken()).toBe("returned-token");
    expect(location.search).toBe("?view=atlas");

    storeToken("  pasted-token  ");
    expect(storedToken()).toBe("pasted-token");

    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(new Response(JSON.stringify({ valid: true, reason: "ok" })))
      .mockResolvedValueOnce(new Response(JSON.stringify({ valid: true, reason: "ok" })))
      .mockResolvedValueOnce(new Response(JSON.stringify({ valid: false, reason: "invalid" })))
      .mockResolvedValueOnce(new Response(JSON.stringify({ valid: false, reason: "revoked" })))
      .mockRejectedValueOnce(new Error("offline"));
    vi.stubGlobal("fetch", fetchMock);

    const valid = await verifyLicense("pasted-token");
    expect(valid.unlocked).toBe(true);
    expect(valid.notice).toBe("Atlas license active");
    expect(fetchMock).toHaveBeenCalledTimes(1);
    expect(String(fetchMock.mock.calls[0][0])).toContain("license=pasted-token");

    const cached = await verifyLicense("pasted-token");
    expect(cached.unlocked).toBe(true);
    expect(fetchMock).toHaveBeenCalledTimes(1);

    localStorage.setItem(
      "sb_license:model-capacity-sentinel:verdict",
      JSON.stringify({ valid: true, at: Date.now() - 86_400_001, token: "pasted-token" }),
    );
    const refreshed = await verifyLicense("pasted-token");
    expect(refreshed.unlocked).toBe(true);
    expect(fetchMock).toHaveBeenCalledTimes(2);

    const invalid = await verifyLicense("pasted-token", true);
    expect(invalid.unlocked).toBe(false);
    expect(invalid.notice).toBe("License no longer active");
    expect(fetchMock).toHaveBeenCalledTimes(3);

    const revoked = await verifyLicense("pasted-token", true);
    expect(revoked.unlocked).toBe(false);
    expect(revoked.notice).toBe("License no longer active");
    expect(fetchMock).toHaveBeenCalledTimes(4);

    storeToken("offline-token");
    const offline = await verifyLicense("offline-token", true);
    expect(offline.unlocked).toBe(false);
    expect(offline.notice).toBe("License check will retry when online");
    expect(fetchMock).toHaveBeenCalledTimes(5);
  });
});
