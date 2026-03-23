import { useState } from "react";
import { useTranslation } from "react-i18next";
import { X } from "lucide-react";
import { createProject } from "@/lib/tauri";
import { useProjectStore } from "@/stores/projectStore";
import { useQueryClient } from "@tanstack/react-query";
import { PROJECT_COLORS } from "@/lib/constants";

interface Props {
  open?: boolean;
  onClose: () => void;
}

export default function ProjectCreate({ open, onClose }: Props) {
  const { t } = useTranslation();
  const { loadProjects, setActiveProject } = useProjectStore();
  const qc = useQueryClient();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [color, setColor] = useState<string>(PROJECT_COLORS[0]);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  if (open === false) return null;

  const handleSubmit = async () => {
    if (!name.trim()) return;
    setSaving(true);
    setError("");
    try {
      const project = await createProject({ name: name.trim(), description, color });
      await qc.invalidateQueries({ queryKey: ["projects"] });
      await loadProjects();
      setActiveProject(project.id);
      setName("");
      setDescription("");
      setColor(PROJECT_COLORS[0]);
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/40" onClick={onClose} />
      <div className="relative bg-white dark:bg-zinc-900 rounded-xl shadow-xl p-6 max-w-md w-full mx-4 border border-zinc-200 dark:border-zinc-700">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-base font-semibold text-zinc-900 dark:text-zinc-50">{t("projects.create")}</h2>
          <button onClick={onClose} className="p-1 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded">
            <X className="w-4 h-4 text-zinc-500" />
          </button>
        </div>

        <div className="space-y-4 mb-6">
          <div>
            <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
              {t("projects.name")}
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={t("projects.namePlaceholder")}
              autoFocus
              onKeyDown={(e) => e.key === "Enter" && handleSubmit()}
              className="w-full px-3 py-2 rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 text-sm"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
              {t("projects.description")}
            </label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={2}
              className="w-full px-3 py-2 rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 text-sm resize-none"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-2">
              {t("projects.color")}
            </label>
            <div className="flex gap-2 flex-wrap">
              {PROJECT_COLORS.map((c) => (
                <button
                  key={c}
                  onClick={() => setColor(c)}
                  className={`w-7 h-7 rounded-full transition-transform ${color === c ? "scale-125 ring-2 ring-offset-2 ring-zinc-400 dark:ring-zinc-600" : ""}`}
                  style={{ backgroundColor: c }}
                />
              ))}
            </div>
          </div>

          {error && <p className="text-sm text-red-600">{error}</p>}
        </div>

        <div className="flex gap-3 justify-end">
          <button
            onClick={onClose}
            className="px-4 py-2 border border-zinc-200 dark:border-zinc-700 rounded-lg text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800"
          >
            {t("common.cancel")}
          </button>
          <button
            onClick={handleSubmit}
            disabled={saving || !name.trim()}
            className="px-4 py-2 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white rounded-lg text-sm font-medium"
          >
            {saving ? t("common.loading") : t("common.create")}
          </button>
        </div>
      </div>
    </div>
  );
}
