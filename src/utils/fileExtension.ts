export function getFileExtension(filePath: string): string {
  const base = filePath.split("/").pop() ?? "";
  const dotIdx = base.lastIndexOf(".");
  if (dotIdx <= 0) return "(no ext)";
  return base.slice(dotIdx);
}
