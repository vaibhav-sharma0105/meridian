import { useEffect, useState } from "react";
import { Command } from "cmdk";
import { useUIStore } from "@/stores/uiStore";
import { useProjectStore } from "@/stores/projectStore";
import { useTranslation } from "react-i18next";
import {
  LayoutList, Plus, Settings, Moon, Sun, FolderOpen,
  MessageSquare, BarChart2, FileText, Upload, Search
} from "lucide-react";

export default function CommandPalette() {
  const { t } = useTranslation();
  const { commandPaletteOpen, setCommandPaletteOpen, theme, setTheme, setActiveView, setIngestModalOpen } = useUIStore();
  const { projects, setActiveProject } = useProjectStore();
  const [search, setSearch] = useState("");

  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setCommandPaletteOpen(!commandPaletteOpen);
      }
    };
    document.addEventListener("keydown", down);
    return () => document.removeEventListener("keydown", down);
  }, [commandPaletteOpen]);

  if (!commandPaletteOpen) return null;

  const run = (fn: () => void) => {
    fn();
    setCommandPaletteOpen(false);
    setSearch("");
  };

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh]">
      <div className="absolute inset-0 bg-black/40" onClick={() => setCommandPaletteOpen(false)} />
      <div className="relative w-full max-w-lg mx-4">
        <Command
          className="bg-white dark:bg-zinc-900 rounded-xl shadow-2xl border border-zinc-200 dark:border-zinc-700 overflow-hidden"
          onKeyDown={(e) => e.key === "Escape" && setCommandPaletteOpen(false)}
        >
          <div className="flex items-center gap-2 px-4 py-3 border-b border-zinc-100 dark:border-zinc-800">
            <Search className="w-4 h-4 text-zinc-400 flex-shrink-0" />
            <Command.Input
              value={search}
              onValueChange={setSearch}
              placeholder={t("commandPalette.placeholder")}
              className="flex-1 bg-transparent text-sm text-zinc-900 dark:text-zinc-50 outline-none placeholder:text-zinc-400"
              autoFocus
            />
            <kbd className="hidden sm:inline-block text-xs text-zinc-400 bg-zinc-100 dark:bg-zinc-800 px-1.5 py-0.5 rounded">esc</kbd>
          </div>

          <Command.List className="max-h-80 overflow-y-auto p-2">
            <Command.Empty className="py-8 text-center text-sm text-zinc-400">
              {t("commandPalette.noResults")}
            </Command.Empty>

            <Command.Group heading={t("commandPalette.navigation")} className="[&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:font-semibold [&_[cmdk-group-heading]]:text-zinc-400 [&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1.5 [&_[cmdk-group-heading]]:uppercase [&_[cmdk-group-heading]]:tracking-wide">
              <CmdItem icon={<LayoutList className="w-4 h-4" />} label={t("tasks.title")} onSelect={() => run(() => setActiveView("tasks"))} />
              <CmdItem icon={<FileText className="w-4 h-4" />} label={t("meetings.title")} onSelect={() => run(() => setActiveView("meetings"))} />
              <CmdItem icon={<Upload className="w-4 h-4" />} label={t("documents.title")} onSelect={() => run(() => setActiveView("documents"))} />
              <CmdItem icon={<BarChart2 className="w-4 h-4" />} label={t("analytics.title")} onSelect={() => run(() => setActiveView("analytics"))} />
              <CmdItem icon={<MessageSquare className="w-4 h-4" />} label={t("ai.title")} onSelect={() => run(() => setActiveView("chat"))} />
            </Command.Group>

            <Command.Group heading={t("commandPalette.actions")} className="[&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:font-semibold [&_[cmdk-group-heading]]:text-zinc-400 [&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1.5 [&_[cmdk-group-heading]]:uppercase [&_[cmdk-group-heading]]:tracking-wide">
              <CmdItem icon={<Plus className="w-4 h-4" />} label={t("meetings.ingest")} onSelect={() => run(() => setIngestModalOpen(true))} />
              <CmdItem
                icon={theme === "dark" ? <Sun className="w-4 h-4" /> : <Moon className="w-4 h-4" />}
                label={theme === "dark" ? t("settings.lightMode") : t("settings.darkMode")}
                onSelect={() => run(() => {
                  const next = theme === "dark" ? "light" : "dark";
                  setTheme(next);
                  document.documentElement.classList.toggle("dark", next === "dark");
                })}
              />
            </Command.Group>

            {projects.length > 0 && (
              <Command.Group heading={t("commandPalette.projects")} className="[&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:font-semibold [&_[cmdk-group-heading]]:text-zinc-400 [&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1.5 [&_[cmdk-group-heading]]:uppercase [&_[cmdk-group-heading]]:tracking-wide">
                {projects.map((project) => (
                  <CmdItem
                    key={project.id}
                    icon={<FolderOpen className="w-4 h-4" style={{ color: project.color ?? "#6366f1" }} />}
                    label={project.name}
                    onSelect={() => run(() => setActiveProject(project.id))}
                  />
                ))}
              </Command.Group>
            )}
          </Command.List>
        </Command>
      </div>
    </div>
  );
}

function CmdItem({ icon, label, onSelect }: { icon: React.ReactNode; label: string; onSelect: () => void }) {
  return (
    <Command.Item
      onSelect={onSelect}
      className="flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-zinc-700 dark:text-zinc-300 cursor-pointer data-[selected=true]:bg-zinc-100 dark:data-[selected=true]:bg-zinc-800 transition-colors"
    >
      <span className="text-zinc-400">{icon}</span>
      {label}
    </Command.Item>
  );
}
