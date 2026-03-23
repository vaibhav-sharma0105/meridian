import { useState } from "react";
import { useTranslation } from "react-i18next";
import { ingestMeeting } from "@/lib/tauri";
import { PLATFORMS } from "@/lib/constants";
import { CheckCircle, Loader } from "lucide-react";

interface Props {
  projectId: string | null;
  onFinish: () => void;
  onSkip: () => void;
}

export default function FirstTranscriptStep({ projectId, onFinish, onSkip }: Props) {
  const { t } = useTranslation();
  const [platform, setPlatform] = useState("zoom");
  const [transcript, setTranscript] = useState("");
  const [processing, setProcessing] = useState(false);
  const [done, setDone] = useState(false);
  const [error, setError] = useState("");

  const handleIngest = async () => {
    if (!transcript.trim() || !projectId) return;
    setProcessing(true);
    setError("");
    try {
      await ingestMeeting({
        projectId,
        platform,
        rawTranscript: transcript,
        title: "My First Meeting",
      });
      setDone(true);
      setTimeout(() => onFinish(), 1500);
    } catch (e) {
      setError(String(e));
    } finally {
      setProcessing(false);
    }
  };

  return (
    <div className="flex-1 flex flex-col items-center justify-center p-8 max-w-lg mx-auto w-full">
      <h2 className="text-2xl font-bold text-zinc-900 dark:text-zinc-50 mb-1">
        {t("onboarding.firstTranscript.title")}
      </h2>
      <p className="text-zinc-500 text-sm mb-8">{t("onboarding.firstTranscript.step")}</p>

      {done ? (
        <div className="flex flex-col items-center gap-4">
          <CheckCircle className="w-12 h-12 text-green-500" />
          <p className="text-zinc-700 dark:text-zinc-300 font-medium">{t("onboarding.firstTranscript.success")}</p>
        </div>
      ) : (
        <div className="w-full space-y-4">
          {!projectId && (
            <div className="p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg">
              <p className="text-sm text-amber-700 dark:text-amber-400">{t("onboarding.firstTranscript.noProject")}</p>
            </div>
          )}

          <div>
            <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
              {t("onboarding.firstTranscript.platform")}
            </label>
            <select
              value={platform}
              onChange={(e) => setPlatform(e.target.value)}
              className="w-full px-3 py-2 rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 text-sm"
            >
              {PLATFORMS.map((p) => (
                <option key={p.value} value={p.value}>{p.label}</option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
              {t("onboarding.firstTranscript.transcript")}
            </label>
            <textarea
              value={transcript}
              onChange={(e) => setTranscript(e.target.value)}
              placeholder={t("onboarding.firstTranscript.transcriptPlaceholder")}
              rows={8}
              className="w-full px-3 py-2 rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 text-sm resize-none font-mono text-xs"
            />
            <p className="text-xs text-zinc-400 mt-1">{t("onboarding.firstTranscript.minWords")}</p>
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
              onClick={handleIngest}
              disabled={processing || !transcript.trim() || !projectId}
              className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white rounded-lg text-sm font-medium"
            >
              {processing && <Loader className="w-4 h-4 animate-spin" />}
              {processing ? t("onboarding.firstTranscript.processing") : t("onboarding.firstTranscript.analyze")}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
