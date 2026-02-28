import { useState } from "react";

export function useSettings() {
  const [codexModel, setCodexModel] = useState(
    () => localStorage.getItem("prvw:codexModel") ?? ""
  );
  const [lang, setLang] = useState(
    () => localStorage.getItem("prvw:lang") ?? "ja"
  );
  const [hasSettings, setHasSettings] = useState(
    () => localStorage.getItem("prvw:lang") !== null
  );

  function saveSettings(s: { codexModel: string; lang: string }) {
    localStorage.setItem("prvw:codexModel", s.codexModel);
    localStorage.setItem("prvw:lang", s.lang);
    setCodexModel(s.codexModel);
    setLang(s.lang);
    setHasSettings(true);
  }

  return { codexModel, lang, hasSettings, saveSettings };
}
