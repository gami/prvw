import { useState } from "react";

interface Props {
  initialModel: string;
  initialLang: string;
  force: boolean;
  onSave: (settings: { codexModel: string; lang: string }) => void;
  onClose: () => void;
}

export function SettingsModal({
  initialModel,
  initialLang,
  force,
  onSave,
  onClose,
}: Props) {
  const [model, setModel] = useState(initialModel);
  const [lang, setLang] = useState(initialLang || "ja");

  function handleSave() {
    onSave({ codexModel: model, lang });
  }

  return (
    <div className="modal-overlay">
      <div className="modal">
        <div className="modal-header">
          <h3>Settings</h3>
          {!force && (
            <button className="btn-close" onClick={onClose}>
              ×
            </button>
          )}
        </div>
        <div className="modal-body">
          <div className="modal-field">
            <label className="modal-label">Codex Model</label>
            <input
              className="input"
              placeholder="empty = use config default"
              value={model}
              onChange={(e) => setModel(e.target.value)}
              style={{ width: "100%" }}
            />
          </div>
          <div className="modal-field">
            <label className="modal-label">Language</label>
            <input
              className="input"
              placeholder="ja"
              value={lang}
              onChange={(e) => setLang(e.target.value)}
              style={{ width: "100%" }}
            />
          </div>
        </div>
        <div className="modal-footer">
          {!force && (
            <button className="btn btn-ghost" onClick={onClose}>
              Cancel
            </button>
          )}
          <button className="btn btn-primary" onClick={handleSave}>
            Save
          </button>
        </div>
      </div>
    </div>
  );
}
