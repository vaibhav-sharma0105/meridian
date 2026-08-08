import { useState, useEffect } from "react";
import { Zap, RotateCcw, RefreshCw, Shield } from "lucide-react";
import type { Skill } from "@/lib/tauri";
import { useUIStore } from "@/stores/uiStore";
import { useResetBuiltinSkills, useCheckSkillUpdates, useSyncSkill } from "@/hooks/useSkills";
import { SkillsList } from "./SkillsList";
import { SkillEditorModal } from "./SkillEditorModal";
import { SkillHistoryPanel } from "./SkillHistoryPanel";
import { SkillTrustSettings } from "./SkillTrustSettings";

export function SkillsPage() {
  const [editorOpen, setEditorOpen] = useState(false);
  const [editingSkill, setEditingSkill] = useState<Skill | null>(null);
  const [historySkill, setHistorySkill] = useState<Skill | null>(null);
  const [trustSkill, setTrustSkill] = useState<Skill | null>(null);
  const [syncSkill, setSyncSkill] = useState<Skill | null>(null);

  const skillEditorData = useUIStore((s) => s.skillEditorData);
  const resetBuiltin = useResetBuiltinSkills();
  const checkUpdates = useCheckSkillUpdates();
  const doSync = useSyncSkill();

  useEffect(() => {
    if (skillEditorData) {
      setEditingSkill(null);
      setEditorOpen(true);
    }
  }, [skillEditorData]);

  const handleCreateSkill = () => {
    setEditingSkill(null);
    setEditorOpen(true);
  };

  const handleEditSkill = (skill: Skill) => {
    setEditingSkill(skill);
    setEditorOpen(true);
  };

  const handleViewHistory = (skill: Skill) => {
    setHistorySkill(skill);
  };

  const handleSyncSkill = async (skill: Skill) => {
    try {
      const result = await checkUpdates.mutateAsync(skill.id);
      if (result.status === "update_available" || result.status === "conflict") {
        setSyncSkill(skill);
      } else {
        alert("Skill is already up to date.");
      }
    } catch (err) {
      alert(`Failed to check for updates: ${err}`);
    }
  };

  const handleManageTrust = (skill: Skill) => {
    setTrustSkill(skill);
  };

  const handleAcceptRemote = async () => {
    if (!syncSkill) return;
    try {
      await doSync.mutateAsync({ skillId: syncSkill.id, strategy: "use_remote" });
      setSyncSkill(null);
    } catch (err) {
      alert(`Failed to sync: ${err}`);
    }
  };

  const handleKeepLocal = async () => {
    if (!syncSkill) return;
    try {
      await doSync.mutateAsync({ skillId: syncSkill.id, strategy: "keep_local" });
      setSyncSkill(null);
    } catch (err) {
      alert(`Failed to sync: ${err}`);
    }
  };

  const handleCloseEditor = () => {
    setEditorOpen(false);
    setEditingSkill(null);
  };

  const handleResetDefaults = () => {
    if (
      window.confirm(
        "This will re-create the built-in skill templates. Your custom skills won't be affected. Continue?"
      )
    ) {
      resetBuiltin.mutate();
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-200 dark:border-zinc-800">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-indigo-100 dark:bg-indigo-900/30 flex items-center justify-center">
            <Zap className="w-4 h-4 text-indigo-600 dark:text-indigo-400" />
          </div>
          <div>
            <h1 className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
              Skills
            </h1>
            <p className="text-[13px] text-zinc-500">
              Automate tasks with scheduled or event-triggered workflows
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleResetDefaults}
            disabled={resetBuiltin.isPending}
            className="flex items-center gap-1.5 px-3 py-1.5 text-[12px] text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300 border border-zinc-200 dark:border-zinc-700 rounded-lg hover:bg-zinc-50 dark:hover:bg-zinc-800 transition-colors disabled:opacity-50"
            title="Re-create built-in skill templates"
          >
            <RotateCcw className={`w-3.5 h-3.5 ${resetBuiltin.isPending ? "animate-spin" : ""}`} />
            Reset defaults
          </button>
        </div>
      </div>

      <div className="flex-1 flex overflow-hidden">
        <div className={`flex-1 ${historySkill ? "border-r border-zinc-200 dark:border-zinc-800" : ""}`}>
          <SkillsList
            onCreateSkill={handleCreateSkill}
            onEditSkill={handleEditSkill}
            onViewHistory={handleViewHistory}
            onSyncSkill={handleSyncSkill}
            onManageTrust={handleManageTrust}
          />
        </div>

        {historySkill && (
          <div className="w-96 flex-shrink-0">
            <SkillHistoryPanel
              skill={historySkill}
              onClose={() => setHistorySkill(null)}
            />
          </div>
        )}
      </div>

      {editorOpen && (
        <SkillEditorModal
          skill={editingSkill}
          onClose={handleCloseEditor}
        />
      )}

      {/* Trust Settings Dialog */}
      {trustSkill && (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
          <div className="fixed inset-0 bg-black/50" onClick={() => setTrustSkill(null)} />
          <div className="relative bg-white dark:bg-zinc-900 rounded-xl shadow-2xl w-[500px] max-h-[80vh] overflow-auto">
            <SkillTrustSettings className="p-0" />
            <div className="flex justify-end px-4 py-3 border-t border-zinc-200 dark:border-zinc-800">
              <button
                onClick={() => setTrustSkill(null)}
                className="px-4 py-2 text-[13px] bg-indigo-500 hover:bg-indigo-600 text-white rounded-lg"
              >
                Done
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Sync Confirmation Dialog */}
      {syncSkill && (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
          <div className="fixed inset-0 bg-black/50" onClick={() => setSyncSkill(null)} />
          <div className="relative bg-white dark:bg-zinc-900 rounded-xl shadow-2xl w-[450px] m-4">
            <div className="p-5 border-b border-zinc-200 dark:border-zinc-800">
              <h3 className="text-[15px] font-semibold text-zinc-900 dark:text-zinc-100">
                Update Available: {syncSkill.name}
              </h3>
              <p className="text-[13px] text-zinc-500 mt-1">
                A newer version is available from the remote repository.
              </p>
            </div>
            <div className="p-5">
              <p className="text-[13px] text-zinc-600 dark:text-zinc-400 mb-4">
                How would you like to handle this update?
              </p>
              <div className="space-y-2">
                <button
                  onClick={handleAcceptRemote}
                  disabled={doSync.isPending}
                  className="w-full flex items-center gap-3 p-3 rounded-lg border border-zinc-200 dark:border-zinc-700 hover:bg-zinc-50 dark:hover:bg-zinc-800 transition-colors text-left disabled:opacity-50"
                >
                  <RefreshCw className={`w-4 h-4 text-blue-500 ${doSync.isPending ? "animate-spin" : ""}`} />
                  <div>
                    <div className="text-[13px] font-medium text-zinc-900 dark:text-zinc-100">Use Remote Version</div>
                    <div className="text-[11px] text-zinc-500">Replace local with the latest from repository. Trust will be revoked.</div>
                  </div>
                </button>
                <button
                  onClick={handleKeepLocal}
                  disabled={doSync.isPending}
                  className="w-full flex items-center gap-3 p-3 rounded-lg border border-zinc-200 dark:border-zinc-700 hover:bg-zinc-50 dark:hover:bg-zinc-800 transition-colors text-left disabled:opacity-50"
                >
                  <Shield className="w-4 h-4 text-zinc-400" />
                  <div>
                    <div className="text-[13px] font-medium text-zinc-900 dark:text-zinc-100">Keep Local Version</div>
                    <div className="text-[11px] text-zinc-500">Mark as synced without changing local content.</div>
                  </div>
                </button>
              </div>
            </div>
            <div className="flex justify-end px-5 py-3 border-t border-zinc-200 dark:border-zinc-800">
              <button
                onClick={() => setSyncSkill(null)}
                className="px-4 py-2 text-[13px] text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-100"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
