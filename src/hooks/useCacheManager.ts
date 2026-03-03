import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export function useCacheManager() {
  const [cacheSize, setCacheSize] = useState<string | null>(null);
  const [clearing, setClearing] = useState(false);

  useEffect(() => {
    invoke<string>("get_cache_size")
      .then(setCacheSize)
      .catch(() => {});
  }, []);

  async function clearCache() {
    setClearing(true);
    try {
      await invoke("clear_cache");
      setCacheSize("0 B");
    } catch (e) {
      alert(`Failed to clear cache: ${e}`);
    } finally {
      setClearing(false);
    }
  }

  return { cacheSize, clearing, clearCache };
}
