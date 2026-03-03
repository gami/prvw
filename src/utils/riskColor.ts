export function riskColor(risk: string): string {
  switch (risk) {
    case "high":
      return "#e74c3c";
    case "medium":
      return "#f39c12";
    case "low":
      return "#27ae60";
    default:
      return "#888";
  }
}
