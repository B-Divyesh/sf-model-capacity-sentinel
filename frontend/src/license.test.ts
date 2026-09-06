import { beforeEach, describe, expect, it } from "vitest";
import { consumeLicenseFromUrl, storedToken, storeToken } from "./license";
describe("license handoff", () => {
  beforeEach(() => {
    localStorage.clear();
    history.replaceState({}, "", "/?license=abc123&view=atlas");
  });
  it("@claim:license-restore stores and strips a returned token", () => {
    consumeLicenseFromUrl();
    expect(storedToken()).toBe("abc123");
    expect(location.search).toBe("?view=atlas");
  });
  it("restores a pasted token", () => {
    storeToken("  restored  ");
    expect(storedToken()).toBe("restored");
  });
});
