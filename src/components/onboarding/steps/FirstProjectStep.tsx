import { useState } from "react";
import { useTranslation } from "react-i18next";
import { createProject } from "@/lib/tauri";
import { PROJECT_COLORS } from "@/lib/constants";

interface Props {
  onNext: (projectId: string) => void;
  onSkip: () => void;
}

export default function FirstProjectStep({ onNext, onSkip }: Props) {
  const { t } = useTranslation();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [color, setColor] = useState<string>(PROJECT_COLORS[0]);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  const handleCreate = async () => {
    if (!name.trim()) return;
    setSaving(true);
    setError("");
    try {
      const project = await createProject({ name: name.trim(), description, color });
      onNext(project.id);
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="flex-1 flex flex-col items-center justify-center p-8 max-w-lg mx-auto w-full">
      <h2 className="text-2xl font-bold text-zinc-900 dark:text-zinc-50 mb-1">
        {t("onboarding.firstProject.title")}
      </h2>
      <p className="text-zinc-500 text-sm mb-8">{t("onboarding.firstProject.step")}</p>

      <div className="w-full space-y-4">
        <div>
          <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
            {t("onboarding.firstProject.name")}
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder={t("onboarding.firstProject.namePlaceholder")}
            className="w-full px-3 py-2 rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 text-sm"
            autoFocus
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
            {t("onboarding.firstProject.description")}
          </label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder={t("onboarding.firstProject.descriptionPlaceholder")}
            rows={3}
            className="w-full px-3 py-2 rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 text-sm resize-none"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-2">
            {t("onboarding.firstProject.color")}
          </label>
          <div className="flex gap-2 flex-wrap">
            {PROJECT_COLORS.map((c) => (
              <button
                key={c}
                onClick={() => setColor(c)}
                className={`w-8 h-8 rounded-full transition-transform ${color === c ? "scale-125 ring-2 ring-offset-2 ring-zinc-400" : ""}`}
                style={{ backgroundColor: c }}
              />
            ))}
          </div>
        </div>

        {error && <p className="text-sm text-red-600">{error}</p>}

        <div className="flex gap-3 pt-2">
          <button
            onClick={onSkip}
            className="flex-1 px-4 py-2 border border-zinc-200 dark:border-zinc-700 rounded-lg text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800"
          >
            {t("common.skip")}
          </button>
          <button
            onClick={handleCreate}
            disabled={saving || !name.trim()}
            className="flex-1 px-4 py-2 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white rounded-lg text-sm font-medium"
          >
            {saving ? t("common.loading") : t("common.next")}
          </button>
        </div>
      </div>
    </div>
  );
}
