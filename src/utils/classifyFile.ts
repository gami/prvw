export type FileCategory = "generated" | "test" | "docs" | "config" | "src";

export function classifyFile(filePath: string): FileCategory {
  const p = filePath.toLowerCase();
  const base = p.split("/").pop() ?? "";

  if (
    p.includes("generated") ||
    p.includes("__generated__") ||
    p.includes("/gen/") ||
    p.startsWith("gen/") ||
    p.includes("/api/out/") ||
    p.startsWith("api/out/") ||
    base === "package-lock.json" ||
    base === "yarn.lock" ||
    base === "pnpm-lock.yaml" ||
    base === "cargo.lock" ||
    base === "go.sum" ||
    base === "poetry.lock" ||
    base === "composer.lock" ||
    base === "gemfile.lock" ||
    p.endsWith(".min.js") ||
    p.endsWith(".min.css") ||
    p.endsWith(".pb.go") ||
    p.endsWith(".pb.ts") ||
    p.endsWith(".g.dart") ||
    p.endsWith(".generated.ts") ||
    p.endsWith(".generated.js")
  )
    return "generated";

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

  if (
    p.endsWith(".md") ||
    p.endsWith(".mdx") ||
    p.endsWith(".rst") ||
    p.endsWith(".txt") ||
    p.includes("/docs/") ||
    p.includes("/doc/") ||
    base === "changelog" ||
    base === "license" ||
    base === "licence"
  )
    return "docs";

  if (
    base.startsWith(".") ||
    base.endsWith(".toml") ||
    base.endsWith(".yaml") ||
    base.endsWith(".yml") ||
    base.endsWith(".json") ||
    base.endsWith(".ini") ||
    base.endsWith(".cfg") ||
    base.endsWith(".conf") ||
    base.endsWith(".config.js") ||
    base.endsWith(".config.ts") ||
    base.endsWith(".config.mjs") ||
    base === "dockerfile" ||
    base === "makefile" ||
    base === "rakefile" ||
    base === "procfile" ||
    p.includes("/.github/") ||
    p.includes("/.circleci/") ||
    p.includes("/.vscode/")
  )
    return "config";

  return "src";
}
