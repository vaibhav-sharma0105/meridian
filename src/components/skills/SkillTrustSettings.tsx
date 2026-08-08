import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Shield,
  ShieldAlert,
  ShieldCheck,
  ShieldOff,
  Globe,
  Lock,
  Network,
  Plus,
  X,
  AlertTriangle,
} from "lucide-react";

interface Skill {
  id: string;
  name: string;
  description: string | null;
  category: string | null;
}

interface TrustState {
  trust_state: string;
  trust_granted_at: string | null;
  network_mode: string;
  network_allowlist: string | null;
}

interface SkillTrustSettingsProps {
  className?: string;
}

export function SkillTrustSettings({ className }: SkillTrustSettingsProps) {
  const [skills, setSkills] = useState<Skill[]>([]);
  const [trustStates, setTrustStates] = useState<Record<string, TrustState>>({});
  const [selectedSkill, setSelectedSkill] = useState<Skill | null>(null);
  const [showDetails, setShowDetails] = useState(false);
  const [networkMode, setNetworkMode] = useState("none");
  const [allowlist, setAllowlist] = useState<string[]>([]);
  const [newHost, setNewHost] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadSkills();
  }, []);

  const loadSkills = async () => {
    try {
      const result = await invoke<Skill[]>("list_skills", {});
      setSkills(result);

      const states: Record<string, TrustState> = {};
      for (const skill of result) {
        try {
          const state = await invoke<TrustState>("get_skill_trust_state", {
            skillId: skill.id,
          });
          states[skill.id] = state;
        } catch {
          states[skill.id] = {
            trust_state: "untrusted",
            trust_granted_at: null,
            network_mode: "none",
            network_allowlist: null,
          };
        }
      }
      setTrustStates(states);
    } catch (err) {
      console.error("Failed to load skills:", err);
    }
  };

  const handleGrantTrust = async (skillId: string, mode: string = "none", hosts?: string[]) => {
    setLoading(true);
    try {
      await invoke("grant_skill_trust", { skillId, networkMode: mode, allowlist: hosts });
      await loadSkills();
    } catch (err) {
      console.error("Failed to grant trust:", err);
    } finally {
      setLoading(false);
    }
  };

  const handleRevokeTrust = async (skillId: string) => {
    setLoading(true);
    try {
      await invoke("revoke_skill_trust", { skillId });
      await loadSkills();
    } catch (err) {
      console.error("Failed to revoke trust:", err);
    } finally {
      setLoading(false);
    }
  };

  const openDetails = (skill: Skill) => {
    setSelectedSkill(skill);
    const state = trustStates[skill.id];
    if (state) {
      setNetworkMode(state.network_mode || "none");
      try {
        setAllowlist(state.network_allowlist ? JSON.parse(state.network_allowlist) : []);
      } catch {
        setAllowlist([]);
      }
    }
    setShowDetails(true);
  };

  const addToAllowlist = () => {
    if (newHost && !allowlist.includes(newHost)) {
      setAllowlist([...allowlist, newHost]);
      setNewHost("");
    }
  };

  const removeFromAllowlist = (host: string) => {
    setAllowlist(allowlist.filter((h) => h !== host));
  };

  const handleSaveDetails = async () => {
    if (!selectedSkill) return;
    await handleGrantTrust(selectedSkill.id, networkMode, networkMode === "allowlist" ? allowlist : undefined);
    setShowDetails(false);
  };

  const getTrustIcon = (state: string) => {
    switch (state) {
      case "trusted":
        return <ShieldCheck className="h-4 w-4 text-green-500" />;
      case "revoked":
        return <ShieldOff className="h-4 w-4 text-red-500" />;
      default:
        return <ShieldAlert className="h-4 w-4 text-amber-500" />;
    }
  };

  const getTrustBadge = (state: string) => {
    switch (state) {
      case "trusted":
        return (
          <span className="px-2 py-0.5 text-xs rounded-full bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400">
            Trusted
          </span>
        );
      case "revoked":
        return (
          <span className="px-2 py-0.5 text-xs rounded-full bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400">
            Revoked
          </span>
        );
      default:
        return (
          <span className="px-2 py-0.5 text-xs rounded-full bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300">
            Untrusted
          </span>
        );
    }
  };

  const getNetworkIcon = (mode: string) => {
    switch (mode) {
      case "full":
        return <Globe className="h-4 w-4 text-blue-500" />;
      case "allowlist":
        return <Network className="h-4 w-4 text-amber-500" />;
      default:
        return <Lock className="h-4 w-4 text-gray-400" />;
    }
  };

  return (
    <div className={className}>
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
        <div className="flex items-center gap-2 p-4 border-b border-gray-200 dark:border-gray-700">
          <Shield className="h-5 w-5 text-blue-500" />
          <h3 className="font-semibold text-gray-900 dark:text-gray-100">Skill Trust Settings</h3>
        </div>
        <div className="p-4">
          {skills.length === 0 ? (
            <p className="text-sm text-gray-500 dark:text-gray-400">
              No skills require trust management.
            </p>
          ) : (
            <div className="space-y-3">
              {skills.map((skill) => {
                const state = trustStates[skill.id];
                if (!state) return null;

                return (
                  <div
                    key={skill.id}
                    className="flex items-center justify-between p-3 rounded-lg border border-gray-200 dark:border-gray-700"
                  >
                    <div className="flex items-center gap-3">
                      {getTrustIcon(state.trust_state)}
                      <div>
                        <div className="font-medium text-gray-900 dark:text-gray-100">{skill.name}</div>
                        <div className="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400">
                          {getNetworkIcon(state.network_mode)}
                          <span>
                            {state.network_mode === "full"
                              ? "Full network"
                              : state.network_mode === "allowlist"
                              ? "Limited network"
                              : "No network"}
                          </span>
                        </div>
                      </div>
                    </div>

                    <div className="flex items-center gap-2">
                      {getTrustBadge(state.trust_state)}

                      {state.trust_state === "trusted" ? (
                        <button
                          onClick={() => handleRevokeTrust(skill.id)}
                          disabled={loading}
                          className="px-3 py-1 text-sm border border-gray-200 dark:border-gray-700 rounded hover:bg-gray-50 dark:hover:bg-gray-700/50 disabled:opacity-50"
                        >
                          Revoke
                        </button>
                      ) : (
                        <button
                          onClick={() => handleGrantTrust(skill.id)}
                          disabled={loading}
                          className="px-3 py-1 text-sm border border-gray-200 dark:border-gray-700 rounded hover:bg-gray-50 dark:hover:bg-gray-700/50 disabled:opacity-50"
                        >
                          Grant Trust
                        </button>
                      )}

                      <button
                        onClick={() => openDetails(skill)}
                        className="px-3 py-1 text-sm text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300"
                      >
                        Details
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>

      {/* Details Dialog */}
      {showDetails && selectedSkill && (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
          <div className="fixed inset-0 bg-black/50" onClick={() => setShowDetails(false)} />
          <div className="relative bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full mx-4">
            <div className="p-4 border-b border-gray-200 dark:border-gray-700">
              <h3 className="font-semibold text-gray-900 dark:text-gray-100">
                Trust Settings: {selectedSkill.name}
              </h3>
              <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                Configure network permissions and trust for this skill.
              </p>
            </div>

            <div className="p-4 space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Network Access
                </label>
                <select
                  value={networkMode}
                  onChange={(e) => setNetworkMode(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-200 dark:border-gray-700 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100"
                >
                  <option value="none">No Network Access</option>
                  <option value="allowlist">Allowlist Only</option>
                  <option value="full">Full Network Access</option>
                </select>
              </div>

              {networkMode === "allowlist" && (
                <div className="space-y-2">
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Allowed Hosts
                  </label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={newHost}
                      onChange={(e) => setNewHost(e.target.value)}
                      placeholder="api.example.com"
                      onKeyDown={(e) => e.key === "Enter" && addToAllowlist()}
                      className="flex-1 px-3 py-2 border border-gray-200 dark:border-gray-700 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100"
                    />
                    <button
                      onClick={addToAllowlist}
                      className="px-3 py-2 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50"
                    >
                      <Plus className="h-4 w-4" />
                    </button>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {allowlist.map((host) => (
                      <span
                        key={host}
                        className="inline-flex items-center gap-1 px-2 py-1 text-sm bg-gray-100 dark:bg-gray-700 rounded"
                      >
                        {host}
                        <button onClick={() => removeFromAllowlist(host)}>
                          <X className="h-3 w-3" />
                        </button>
                      </span>
                    ))}
                  </div>
                </div>
              )}

              {networkMode === "full" && (
                <div className="p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg flex items-start gap-2">
                  <AlertTriangle className="h-4 w-4 text-amber-500 mt-0.5" />
                  <p className="text-sm text-amber-700 dark:text-amber-400">
                    Full network access allows the skill to connect to any host.
                    Only grant this to skills you fully trust.
                  </p>
                </div>
              )}
            </div>

            <div className="flex justify-end gap-2 p-4 border-t border-gray-200 dark:border-gray-700">
              <button
                onClick={() => setShowDetails(false)}
                className="px-4 py-2 text-sm border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50"
              >
                Cancel
              </button>
              <button
                onClick={handleSaveDetails}
                className="px-4 py-2 text-sm bg-blue-500 text-white rounded-lg hover:bg-blue-600"
              >
                Save Changes
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
