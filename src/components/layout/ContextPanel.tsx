import { useProjectStore } from "@/stores/projectStore";
import AIChatPanel from "@/components/ai/AIChatPanel";

export default function ContextPanel() {
  const { activeProjectId } = useProjectStore();

  return (
    <div className="flex flex-col h-full bg-white dark:bg-[#0f0f12] overflow-hidden">
      <AIChatPanel projectId={activeProjectId} />
    </div>
  );
}
