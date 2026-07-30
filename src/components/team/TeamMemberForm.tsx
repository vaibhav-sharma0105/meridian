import { useState } from "react";
import { X, Plus, Trash2 } from "lucide-react";
import { useCreateTeamMember, useUpdateTeamMember } from "@/hooks/useTeam";
import type { TeamMember, CreateTeamMemberInput, UpdateTeamMemberInput } from "@/lib/tauri";

interface TeamMemberFormProps {
  member?: TeamMember | null;
  onClose: () => void;
}

export function TeamMemberForm({ member, onClose }: TeamMemberFormProps) {
  const createMember = useCreateTeamMember();
  const updateMember = useUpdateTeamMember();

  const [name, setName] = useState(member?.name || "");
  const [email, setEmail] = useState(member?.email || "");
  const [role, setRole] = useState(member?.role || "member");
  const [expertiseInput, setExpertiseInput] = useState("");
  const [expertise, setExpertise] = useState<string[]>(member?.expertise || []);

  const isEditing = !!member;
  const isSubmitting = createMember.isPending || updateMember.isPending;

  const handleAddExpertise = () => {
    const trimmed = expertiseInput.trim();
    if (trimmed && !expertise.includes(trimmed)) {
      setExpertise([...expertise, trimmed]);
      setExpertiseInput("");
    }
  };

  const handleRemoveExpertise = (skill: string) => {
    setExpertise(expertise.filter((s) => s !== skill));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!name.trim()) return;

    try {
      if (isEditing && member) {
        const input: UpdateTeamMemberInput = {
          id: member.id,
          name: name.trim(),
          email: email.trim() || undefined,
          role: role || undefined,
          expertise: expertise.length > 0 ? expertise : undefined,
        };
        await updateMember.mutateAsync(input);
      } else {
        const input: CreateTeamMemberInput = {
          name: name.trim(),
          email: email.trim() || undefined,
          source: "manual",
          role: role || undefined,
          expertise: expertise.length > 0 ? expertise : undefined,
        };
        await createMember.mutateAsync(input);
      }
      onClose();
    } catch (err) {
      console.error("Failed to save member:", err);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="w-full max-w-md bg-white dark:bg-zinc-900 rounded-xl shadow-xl">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-200 dark:border-zinc-800">
          <h2 className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
            {isEditing ? "Edit Team Member" : "Add Team Member"}
          </h2>
          <button
            onClick={onClose}
            className="p-1 text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          {/* Name */}
          <div>
            <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
              Name <span className="text-red-500">*</span>
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="John Doe"
              required
              className="w-full px-3 py-2 text-sm bg-zinc-50 dark:bg-zinc-800
                       border border-zinc-200 dark:border-zinc-700 rounded-lg
                       focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none"
            />
          </div>

          {/* Email */}
          <div>
            <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
              Email
            </label>
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="john@example.com"
              className="w-full px-3 py-2 text-sm bg-zinc-50 dark:bg-zinc-800
                       border border-zinc-200 dark:border-zinc-700 rounded-lg
                       focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none"
            />
          </div>

          {/* Role */}
          <div>
            <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
              Role
            </label>
            <select
              value={role}
              onChange={(e) => setRole(e.target.value)}
              className="w-full px-3 py-2 text-sm bg-zinc-50 dark:bg-zinc-800
                       border border-zinc-200 dark:border-zinc-700 rounded-lg
                       focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none"
            >
              <option value="member">Member</option>
              <option value="lead">Lead</option>
              <option value="manager">Manager</option>
              <option value="admin">Admin</option>
            </select>
          </div>

          {/* Expertise */}
          <div>
            <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
              Expertise / Skills
            </label>
            <div className="flex gap-2 mb-2">
              <input
                type="text"
                value={expertiseInput}
                onChange={(e) => setExpertiseInput(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    handleAddExpertise();
                  }
                }}
                placeholder="Add a skill..."
                className="flex-1 px-3 py-2 text-sm bg-zinc-50 dark:bg-zinc-800
                         border border-zinc-200 dark:border-zinc-700 rounded-lg
                         focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none"
              />
              <button
                type="button"
                onClick={handleAddExpertise}
                className="px-3 py-2 text-sm text-zinc-600 dark:text-zinc-400
                         border border-zinc-200 dark:border-zinc-700 rounded-lg
                         hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
              >
                <Plus className="w-4 h-4" />
              </button>
            </div>
            {expertise.length > 0 && (
              <div className="flex flex-wrap gap-1">
                {expertise.map((skill) => (
                  <span
                    key={skill}
                    className="inline-flex items-center gap-1 text-xs px-2 py-1
                             bg-indigo-100 dark:bg-indigo-900/30 text-indigo-700 dark:text-indigo-400 rounded"
                  >
                    {skill}
                    <button
                      type="button"
                      onClick={() => handleRemoveExpertise(skill)}
                      className="hover:text-red-500"
                    >
                      <X className="w-3 h-3" />
                    </button>
                  </span>
                ))}
              </div>
            )}
          </div>

          {/* Source Info (read-only for synced members) */}
          {isEditing && member?.source !== "manual" && (
            <div className="text-xs text-zinc-500 bg-zinc-50 dark:bg-zinc-800 px-3 py-2 rounded-lg">
              This member is synced from {member?.source}. Some fields may be overwritten on next sync.
            </div>
          )}

          {/* Actions */}
          <div className="flex justify-end gap-3 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm text-zinc-600 dark:text-zinc-400
                       hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={!name.trim() || isSubmitting}
              className="px-4 py-2 text-sm font-medium text-white bg-indigo-600
                       hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed
                       rounded-lg transition-colors"
            >
              {isSubmitting ? "Saving..." : isEditing ? "Save Changes" : "Add Member"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
