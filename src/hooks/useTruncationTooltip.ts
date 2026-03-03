import { useRef, useState } from "react";

export function useTruncationTooltip() {
  const [tooltip, setTooltip] = useState<{ x: number; y: number } | null>(null);
  const ref = useRef<HTMLSpanElement>(null);

  function onMouseEnter() {
    const el = ref.current;
    if (!el || el.scrollWidth <= el.clientWidth) return;
    const rect = el.getBoundingClientRect();
    setTooltip({ x: rect.left, y: rect.bottom + 4 });
  }

  function onMouseLeave() {
    setTooltip(null);
  }

  return { tooltip, ref, onMouseEnter, onMouseLeave };
}
