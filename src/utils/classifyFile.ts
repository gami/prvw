export type FileCategory = "test" | "src";

export function classifyFile(filePath: string): FileCategory {
  const p = filePath.toLowerCase();
  const base = p.split("/").pop() ?? "";

  if (
    p.includes("__tests__") ||
    p.includes("__test__") ||
    p.includes("/test/") ||
    p.includes("/tests/") ||
    p.includes("/spec/") ||
    p.includes("/specs/") ||
    base.endsWith(".test.ts") ||
    base.endsWith(".test.tsx") ||
    base.endsWith(".test.js") ||
    base.endsWith(".test.jsx") ||
    base.endsWith(".spec.ts") ||
    base.endsWith(".spec.tsx") ||
    base.endsWith(".spec.js") ||
    base.endsWith(".spec.jsx") ||
    base.startsWith("test_") ||
    base.endsWith("_test.go") ||
    base.endsWith("_test.rs") ||
    base.endsWith("_test.py")
  )
    return "test";

  return "src";
}
