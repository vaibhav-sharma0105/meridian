import { useState } from "react";
import { useTranslation } from "react-i18next";
import { FolderOpen, Search, Upload } from "lucide-react";
import { useDocuments } from "@/hooks/useDocuments";
import DocCard from "./DocCard";
import DocUpload from "./DocUpload";
import DocSearch from "./DocSearch";
import EmptyState from "@/components/shared/EmptyState";

interface Props {
  projectId: string | null;
}

type Tab = "browse" | "upload" | "search";

export default function DocFolder({ projectId }: Props) {
  const { t } = useTranslation();
  const { documents, refetch } = useDocuments(projectId);
  const [tab, setTab] = useState<Tab>("browse");

  if (!projectId) {
    return (
      <EmptyState
        title={t("documents.noProject")}
        icon={<FolderOpen className="w-10 h-10 text-zinc-400" />}
      />
    );
  }

  const TABS: { id: Tab; label: string; icon: typeof FolderOpen }[] = [
    { id: "browse", label: t("documents.browse"), icon: FolderOpen },
    { id: "upload", label: t("documents.upload"), icon: Upload },
    { id: "search", label: t("documents.search"), icon: Search },
  ];

  return (
    <div className="flex flex-col h-full">
      {/* Sub-tabs */}
      <div className="flex gap-1 px-4 py-3 border-b border-zinc-100 dark:border-zinc-800">
        {TABS.map((t_) => (
          <button
            key={t_.id}
            onClick={() => setTab(t_.id)}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm transition-colors ${tab === t_.id ? "bg-zinc-100 dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 font-medium" : "text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300"}`}
          >
            <t_.icon className="w-3.5 h-3.5" />
            {t_.label}
          </button>
        ))}
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        {tab === "browse" && (
          <div className="space-y-2">
            {documents.length === 0 ? (
              <EmptyState
                title={t("documents.noDocuments")}
                description={t("documents.noDocumentsDesc")}
                icon={<FolderOpen className="w-10 h-10 text-zinc-400" />}
                action={{ label: t("documents.upload"), onClick: () => setTab("upload") }}
              />
            ) : (
              documents.map((doc) => (
                <DocCard key={doc.id} doc={doc} onDeleted={refetch} />
              ))
            )}
          </div>
        )}

        {tab === "upload" && (
          <DocUpload projectId={projectId} onUploaded={() => { refetch(); setTab("browse"); }} />
        )}

        {tab === "search" && (
          <DocSearch projectId={projectId} />
        )}
      </div>
    </div>
  );
}
