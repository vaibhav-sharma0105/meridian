import { useEffect, useState } from "react";
import { AlertCircle, Check } from "lucide-react";
import { useUserProfile, useUpdateUserIdentity } from "@/hooks/useRole";

/**
 * Captures who the user is. My Activity is ordered by role, and both the
 * Manager ("team items first") and IC ("my assignments first") rules need a
 * way to match `task.assignee` against the current user. Without this the
 * backend falls back to severity + recency ordering.
 */
export function IdentitySettings() {
  const { data: profile } = useUserProfile();
  const updateIdentity = useUpdateUserIdentity();

  const [displayName, setDisplayName] = useState("");
  const [userEmail, setUserEmail] = useState("");
  const [aliases, setAliases] = useState("");
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    if (!profile) return;
    setDisplayName(profile.display_name ?? "");
    setUserEmail(profile.user_email ?? "");
    setAliases((profile.user_aliases ?? []).join(", "));
  }, [profile]);

  const hasIdentity = Boolean(
    profile?.display_name || profile?.user_email || profile?.user_aliases?.length
  );

  const handleSave = () => {
    updateIdentity.mutate(
      {
        displayName: displayName.trim(),
        userEmail: userEmail.trim(),
        userAliases: aliases
          .split(",")
          .map((a) => a.trim())
          .filter(Boolean),
      },
      {
        onSuccess: () => {
          setSaved(true);
          setTimeout(() => setSaved(false), 2000);
        },
      }
    );
  };

  return (
    <div className="space-y-4">
      <p className="text-[12px] text-zinc-500 leading-relaxed">
        Used to tell your own items apart from your team's when ordering My
        Activity. Names are matched against task assignees.
      </p>

      {!hasIdentity && (
        <div className="flex items-start gap-2 px-3 py-2 rounded-lg bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800">
          <AlertCircle className="w-4 h-4 text-amber-500 flex-shrink-0 mt-0.5" />
          <p className="text-[12px] text-amber-700 dark:text-amber-400 leading-relaxed">
            Without a name or email, My Activity falls back to sorting by
            severity only — role-based ordering stays off.
          </p>
        </div>
      )}

      <div className="space-y-3">
        <div>
          <label className="block text-[11px] text-zinc-400 mb-1">
            Display name
          </label>
          <input
            type="text"
            value={displayName}
            onChange={(e) => setDisplayName(e.target.value)}
            placeholder="Ada Lovelace"
            className="w-full px-3 py-2 text-[13px] bg-zinc-50 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500"
          />
        </div>

        <div>
          <label className="block text-[11px] text-zinc-400 mb-1">Email</label>
          <input
            type="email"
            value={userEmail}
            onChange={(e) => setUserEmail(e.target.value)}
            placeholder="ada@example.com"
            className="w-full px-3 py-2 text-[13px] bg-zinc-50 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500"
          />
        </div>

        <div>
          <label className="block text-[11px] text-zinc-400 mb-1">
            Other names you're assigned by
          </label>
          <input
            type="text"
            value={aliases}
            onChange={(e) => setAliases(e.target.value)}
            placeholder="ada, alovelace"
            className="w-full px-3 py-2 text-[13px] bg-zinc-50 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500"
          />
          <p className="text-[11px] text-zinc-400 mt-1">
            Comma-separated. Useful when tasks use a handle instead of your full
            name.
          </p>
        </div>
      </div>

      <div className="flex items-center gap-3">
        <button
          onClick={handleSave}
          disabled={updateIdentity.isPending}
          className="px-3 py-1.5 text-[13px] font-medium bg-indigo-500 hover:bg-indigo-600 text-white rounded-lg disabled:opacity-50 transition-colors"
        >
          {updateIdentity.isPending ? "Saving…" : "Save"}
        </button>
        {saved && (
          <span className="flex items-center gap-1 text-[12px] text-emerald-500">
            <Check className="w-3.5 h-3.5" />
            Saved
          </span>
        )}
      </div>
    </div>
  );
}
