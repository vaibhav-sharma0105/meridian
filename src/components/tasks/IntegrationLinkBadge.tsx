import { ExternalLink, Github, Trello } from "lucide-react";
import type { IntegrationLink } from "@/lib/tauri";
import { useLinksForTask } from "@/hooks/useIntegrationLinks";

interface IntegrationLinkBadgeProps {
  taskId: string;
}

export function IntegrationLinkBadge({ taskId }: IntegrationLinkBadgeProps) {
  const { data: links = [] } = useLinksForTask(taskId);

  if (links.length === 0) return null;

  return (
    <div className="flex items-center gap-1">
      {links.map((link) => (
        <LinkBadge key={link.id} link={link} />
      ))}
    </div>
  );
}

function LinkBadge({ link }: { link: IntegrationLink }) {
  const icon = {
    issue: <Github className="w-3 h-3" />,
    pr: <Github className="w-3 h-3" />,
    jira_issue: <Trello className="w-3 h-3" />,
  }[link.external_type] || <ExternalLink className="w-3 h-3" />;

  const label = {
    issue: `#${link.external_id.slice(-4)}`,
    pr: `PR#${link.external_id.slice(-4)}`,
    jira_issue: link.external_id,
  }[link.external_type] || link.external_id;

  const handleClick = () => {
    if (link.external_url) {
      window.open(link.external_url, "_blank");
    }
  };

  return (
    <button
      onClick={handleClick}
      className="inline-flex items-center gap-1 px-1.5 py-0.5 text-[10px] font-medium text-zinc-600 dark:text-zinc-400 bg-zinc-100 dark:bg-zinc-800 rounded hover:bg-zinc-200 dark:hover:bg-zinc-700 transition-colors"
      title={`Open ${link.external_type} in browser`}
    >
      {icon}
      {label}
    </button>
  );
}
