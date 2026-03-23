import { useState } from "react";
import { useTranslation } from "react-i18next";
import { X, Trash2 } from "lucide-react";
import { updateProject, archiveProject } from "@/lib/tauri";
import { useProjectStore } from "@/stores/projectStore";
import { PROJECT_COLORS } from "@/lib/constants";
import ConfirmDialog from "@/components/shared/ConfirmDialog";
import type { Project } from "@/lib/tauri";

interface Props {
  project: Project;
  open: boolean;
  onClose: () => void;
}

export default function ProjectSettings({ project, open, onClose }: Props) {
  const { t } = useTranslation();
  const { loadProjects, setActiveProject } = useProjectStore();
  const [name, setName] = useState(project.name);
  const [description, setDescription] = useState(project.description ?? "");
  const [color, setColor] = useState(project.color ?? PROJECT_COLORS[0]);
  const [saving, setSaving] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);

  if (!open) return null;

  const handleSave = async () => {
    setSaving(true);
    try {
      await updateProject({ id: project.id, name, description, color });
      await loadProjects();
      onClose();
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    await archiveProject(project.id);
    await loadProjects();
    setActiveProject(null);
    onClose();
  };

  return (
    <>
      <div className="fixed inset-0 z-50 flex items-center justify-center">
        <div className="absolute inset-0 bg-black/40" onClick={onClose} />
        <div className="relative bg-white dark:bg-zinc-900 rounded-xl shadow-xl p-6 max-w-md w-full mx-4 border border-zinc-200 dark:border-zinc-700">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-base font-semibold text-zinc-900 dark:text-zinc-50">{t("projects.settings")}</h2>
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
                    className={`w-7 h-7 rounded-full transition-transform ${color === c ? "scale-125 ring-2 ring-offset-2 ring-zinc-400" : ""}`}
                    style={{ backgroundColor: c }}
                  />
                ))}
              </div>
            </div>
          </div>

          <div className="flex items-center justify-between">
            <button
              onClick={() => setConfirmDelete(true)}
              className="flex items-center gap-1.5 px-3 py-2 text-sm text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
            >
              <Trash2 className="w-4 h-4" />
              {t("projects.archive")}
            </button>

            <div className="flex gap-3">
              <button
                onClick={onClose}
                className="px-4 py-2 border border-zinc-200 dark:border-zinc-700 rounded-lg text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800"
              >
                {t("common.cancel")}
              </button>
              <button
                onClick={handleSave}
                disabled={saving || !name.trim()}
                className="px-4 py-2 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white rounded-lg text-sm font-medium"
              >
                {saving ? t("common.loading") : t("common.save")}
              </button>
            </div>
          </div>
        </div>
      </div>

      <ConfirmDialog
        open={confirmDelete}
        title={t("projects.archiveConfirmTitle")}
        message={t("projects.archiveConfirmMessage")}
        confirmLabel={t("projects.archive")}
        onConfirm={handleDelete}
        onCancel={() => setConfirmDelete(false)}
        danger
      />
    </>
  );
}
