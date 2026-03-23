import { useState } from "react";
import { useTranslation } from "react-i18next";
import { ChevronDown, ChevronRight, ExternalLink } from "lucide-react";

const GUIDES = [
  {
    id: "zoom",
    name: "Zoom",
    steps: [
      "Open the Zoom desktop client and sign in.",
      "Click Settings → Recording.",
      "Enable Cloud Recording or Local Recording.",
      "After the meeting, go to Recordings and download the transcript (.vtt or .txt).",
      "Paste or import the transcript into Meridian.",
    ],
  },
  {
    id: "google-meet",
    name: "Google Meet",
    steps: [
      "Start a Google Meet session.",
      "Click the three-dot menu → Record meeting (requires Workspace plan).",
      "After the meeting ends, the transcript is emailed to the host.",
      "Copy the transcript text and paste it into Meridian.",
    ],
  },
  {
    id: "teams",
    name: "Microsoft Teams",
    steps: [
      "Start or join a Teams meeting.",
      "Click the three-dot menu → Start recording.",
      "After the meeting, go to the chat tab and find the recording link.",
      "Open the recording, click the '...' menu → Open transcript.",
      "Copy the transcript and paste it into Meridian.",
    ],
  },
] as const;

export default function IntegrationGuide() {
  const { t } = useTranslation();
  const [open, setOpen] = useState<string | null>(null);

  return (
    <div className="space-y-2">
      <h3 className="text-sm font-semibold text-zinc-700 dark:text-zinc-300">{t("meetings.integrationGuide")}</h3>
      {GUIDES.map((guide) => (
        <div key={guide.id} className="border border-zinc-200 dark:border-zinc-700 rounded-lg overflow-hidden">
          <button
            onClick={() => setOpen(open === guide.id ? null : guide.id)}
            className="flex items-center justify-between w-full px-4 py-3 hover:bg-zinc-50 dark:hover:bg-zinc-800 transition-colors"
          >
            <span className="text-sm font-medium text-zinc-800 dark:text-zinc-200">{guide.name}</span>
            {open === guide.id ? (
              <ChevronDown className="w-4 h-4 text-zinc-400" />
            ) : (
              <ChevronRight className="w-4 h-4 text-zinc-400" />
            )}
          </button>
          {open === guide.id && (
            <div className="px-4 pb-4 border-t border-zinc-100 dark:border-zinc-800 pt-3">
              <ol className="space-y-2">
                {guide.steps.map((step, i) => (
                  <li key={i} className="flex gap-3 text-sm text-zinc-600 dark:text-zinc-400">
                    <span className="flex-shrink-0 w-5 h-5 bg-indigo-100 dark:bg-indigo-900/30 text-indigo-600 dark:text-indigo-400 rounded-full flex items-center justify-center text-xs font-bold">
                      {i + 1}
                    </span>
                    {step}
                  </li>
                ))}
              </ol>
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
