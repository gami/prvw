import { describe, expect, test } from "vitest";
import { getFileExtension } from "./fileExtension";

describe("getFileExtension", () => {
  test(".ts extension", () => {
    expect(getFileExtension("src/main.ts")).toBe(".ts");
  });

  test(".tsx extension", () => {
    expect(getFileExtension("components/App.tsx")).toBe(".tsx");
  });

  test("multiple dots uses last", () => {
    expect(getFileExtension("archive.tar.gz")).toBe(".gz");
  });

  test("no extension", () => {
    expect(getFileExtension("Makefile")).toBe("(no ext)");
  });

  test("dotfile returns (no ext)", () => {
    expect(getFileExtension(".gitignore")).toBe("(no ext)");
  });

  test("nested path", () => {
    expect(getFileExtension("a/b/c/file.rs")).toBe(".rs");
  });
});
