import { describe, expect, test } from "vitest";
import { riskColor } from "./riskColor";

describe("riskColor", () => {
  test("high returns red", () => {
    expect(riskColor("high")).toBe("#e74c3c");
  });

  test("medium returns orange", () => {
    expect(riskColor("medium")).toBe("#f39c12");
  });

  test("low returns green", () => {
    expect(riskColor("low")).toBe("#27ae60");
  });

  test("unknown returns gray", () => {
    expect(riskColor("unknown")).toBe("#888");
  });

  test("empty string returns gray", () => {
    expect(riskColor("")).toBe("#888");
  });
});
