import { describe, expect, test } from "vitest";
import { classifyFile } from "./classifyFile";

describe("classifyFile", () => {
  test(".test.ts file", () => {
    expect(classifyFile("src/utils/foo.test.ts")).toBe("test");
  });

  test(".spec.tsx file", () => {
    expect(classifyFile("components/Bar.spec.tsx")).toBe("test");
  });

  test("__tests__ directory", () => {
    expect(classifyFile("src/__tests__/helper.ts")).toBe("test");
  });

  test("__test__ directory", () => {
    expect(classifyFile("src/__test__/helper.ts")).toBe("test");
  });

  test("/test/ directory", () => {
    expect(classifyFile("project/test/setup.ts")).toBe("test");
  });

  test("/tests/ directory", () => {
    expect(classifyFile("project/tests/integration.ts")).toBe("test");
  });

  test("test_ prefix", () => {
    expect(classifyFile("test_helper.py")).toBe("test");
  });

  test("_test.go suffix", () => {
    expect(classifyFile("pkg/handler_test.go")).toBe("test");
  });

  test("_test.rs suffix", () => {
    expect(classifyFile("src/parser_test.rs")).toBe("test");
  });

  test("_test.py suffix", () => {
    expect(classifyFile("tests/unit_test.py")).toBe("test");
  });

  test("regular source file", () => {
    expect(classifyFile("src/utils/helper.ts")).toBe("src");
  });

  test("case insensitive", () => {
    expect(classifyFile("SRC/__TESTS__/Foo.ts")).toBe("test");
  });
});
