import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Download,
  FolderGit2,
  AlertTriangle,
  CheckCircle,
  Loader2,
  FileCode,
  Shield,
  X,
  ChevronLeft,
} from "lucide-react";

interface ImportableSkill {
  path: string;
  name: string;
  description: string | null;
}

interface Integration {
  id: string;
  name: string;
  integration_type: string;
  status: string;
  config?: {
    repositories?: string[];
  };
}

interface SkillImportWizardProps {
  open: boolean;
  onClose: () => void;
  onImported: () => void;
}

type WizardStep = "select-repo" | "browse-skills" | "confirm-import" | "importing" | "complete";

export function SkillImportWizard({ open, onClose, onImported }: SkillImportWizardProps) {
  const [step, setStep] = useState<WizardStep>("select-repo");
  const [integrations, setIntegrations] = useState<Integration[]>([]);
  const [selectedIntegration, setSelectedIntegration] = useState<string | null>(null);
  const [selectedRepo, setSelectedRepo] = useState<{ owner: string; repo: string } | null>(null);
  const [skills, setSkills] = useState<ImportableSkill[]>([]);
  const [selectedSkill, setSelectedSkill] = useState<ImportableSkill | null>(null);
  const [localName, setLocalName] = useState("");
  const [nameConflict, setNameConflict] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      loadIntegrations();
    }
  }, [open]);

  const loadIntegrations = async () => {
    try {
      const result = await invoke<Integration[]>("list_integrations");
      const githubIntegrations = result.filter(
        (i) => i.integration_type === "github" && i.status === "connected"
      );
      setIntegrations(githubIntegrations);
    } catch (err) {
      setError(String(err));
    }
  };

  const loadSkillsFromRepo = async (integrationId: string, owner: string, repo: string) => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<ImportableSkill[]>("list_importable_skills", {
        integrationId,
        owner,
        repo,
      });
      setSkills(result);
      setSelectedRepo({ owner, repo });
      setStep("browse-skills");
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const checkNameConflict = async (name: string) => {
    if (!name) {
      setNameConflict(false);
      return;
    }
    try {
      const existingSkills = await invoke<{ name: string }[]>("list_skills", {});
      setNameConflict(existingSkills.some((s) => s.name.toLowerCase() === name.toLowerCase()));
    } catch {
      setNameConflict(false);
    }
  };

  const handleSelectSkill = (skill: ImportableSkill) => {
    setSelectedSkill(skill);
    setLocalName(skill.name);
    checkNameConflict(skill.name);
    setStep("confirm-import");
  };

  const handleImport = async () => {
    if (!selectedIntegration || !selectedSkill) return;

    setStep("importing");
    setError(null);

    try {
      await invoke("import_skill_from_repo", {
        integrationId: selectedIntegration,
        skillPath: selectedSkill.path,
        localName: localName || undefined,
      });
      setStep("complete");
    } catch (err) {
      setError(String(err));
      setStep("confirm-import");
    }
  };

  const handleComplete = () => {
    onImported();
    onClose();
    resetState();
  };

  const resetState = () => {
    setStep("select-repo");
    setSelectedIntegration(null);
    setSelectedRepo(null);
    setSelectedSkill(null);
    setLocalName("");
    setNameConflict(false);
    setError(null);
    setSkills([]);
  };

  const handleClose = () => {
    onClose();
    resetState();
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="fixed inset-0 bg-black/50" onClick={handleClose} />
      <div className="relative bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-xl w-full mx-4 max-h-[80vh] overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
          <div className="flex items-center gap-2">
            <Download className="h-5 w-5 text-blue-500" />
            <h2 className="text-lg font-semibold">Import Skill from Repository</h2>
          </div>
          <button
            onClick={handleClose}
            className="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded"
          >
            <X className="h-5 w-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-4 overflow-y-auto max-h-[60vh]">
          {error && (
            <div className="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg flex items-center gap-2 text-red-700 dark:text-red-400">
              <AlertTriangle className="h-4 w-4" />
              <span className="text-sm">{error}</span>
            </div>
          )}

          {/* Step 1: Select Repository */}
          {step === "select-repo" && (
            <div className="space-y-4">
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Select a connected GitHub repository to browse skills.
              </p>

              {integrations.length === 0 ? (
                <div className="p-4 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg">
                  <p className="text-sm text-amber-700 dark:text-amber-400">
                    No GitHub integrations connected. Connect a GitHub integration first
                    to import skills from repositories.
                  </p>
                </div>
              ) : (
                <div className="space-y-2">
                  {integrations.map((integration) => (
                    <div key={integration.id} className="space-y-2">
                      <div className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300">
                        <FolderGit2 className="h-4 w-4" />
                        {integration.name}
                      </div>
                      {integration.config?.repositories?.map((repoPath) => {
                        const [owner, repo] = repoPath.split("/");
                        return (
                          <button
                            key={repoPath}
                            onClick={() => {
                              setSelectedIntegration(integration.id);
                              loadSkillsFromRepo(integration.id, owner, repo);
                            }}
                            className="w-full flex items-center gap-3 p-3 ml-6 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
                          >
                            <FileCode className="h-4 w-4 text-gray-400" />
                            <span className="text-sm">{repoPath}</span>
                          </button>
                        );
                      })}
                    </div>
                  ))}
                </div>
              )}

              {loading && (
                <div className="flex items-center justify-center py-4">
                  <Loader2 className="h-6 w-6 animate-spin text-blue-500" />
                </div>
              )}
            </div>
          )}

          {/* Step 2: Browse Skills */}
          {step === "browse-skills" && (
            <div className="space-y-4">
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Select a skill to import. Skills with scripts will run in a sandbox.
              </p>

              {skills.length === 0 ? (
                <div className="p-4 bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-lg">
                  <p className="text-sm text-gray-600 dark:text-gray-400">
                    No skills found in .claude/skills/ or .agents/skills/ directories.
                  </p>
                </div>
              ) : (
                <div className="space-y-2 max-h-80 overflow-y-auto">
                  {skills.map((skill) => (
                    <button
                      key={skill.path}
                      onClick={() => handleSelectSkill(skill)}
                      className="w-full flex items-start gap-3 p-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors text-left"
                    >
                      <FileCode className="h-5 w-5 text-gray-400 mt-0.5" />
                      <div className="flex-1 min-w-0">
                        <div className="font-medium text-gray-900 dark:text-gray-100">{skill.name}</div>
                        {skill.description && (
                          <div className="text-sm text-gray-500 dark:text-gray-400 truncate">
                            {skill.description}
                          </div>
                        )}
                        <div className="text-xs text-gray-400 dark:text-gray-500 mt-1">
                          {skill.path}
                        </div>
                      </div>
                    </button>
                  ))}
                </div>
              )}

              <button
                onClick={() => setStep("select-repo")}
                className="flex items-center gap-1 text-sm text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300"
              >
                <ChevronLeft className="h-4 w-4" />
                Back
              </button>
            </div>
          )}

          {/* Step 3: Confirm Import */}
          {step === "confirm-import" && selectedSkill && (
            <div className="space-y-4">
              <div className="p-4 rounded-lg border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900/50">
                <h3 className="font-medium text-gray-900 dark:text-gray-100">{selectedSkill.name}</h3>
                {selectedSkill.description && (
                  <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                    {selectedSkill.description}
                  </p>
                )}
                <p className="text-xs text-gray-400 dark:text-gray-500 mt-2">{selectedSkill.path}</p>
              </div>

              <div className="space-y-2">
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                  Local Name
                </label>
                <input
                  type="text"
                  value={localName}
                  onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                    setLocalName(e.target.value);
                    checkNameConflict(e.target.value);
                  }}
                  placeholder="Enter a name for the skill"
                  className="w-full px-3 py-2 border border-gray-200 dark:border-gray-700 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
                {nameConflict && (
                  <p className="text-sm text-amber-500 flex items-center gap-1">
                    <AlertTriangle className="h-3 w-3" />
                    A skill with this name already exists
                  </p>
                )}
              </div>

              <div className="p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg flex items-start gap-2">
                <Shield className="h-4 w-4 text-blue-500 mt-0.5" />
                <p className="text-sm text-blue-700 dark:text-blue-400">
                  Imported skills start untrusted. You will need to grant trust
                  before running any scripts. Scripts run in a sandbox with no network access by default.
                </p>
              </div>

              <div className="flex gap-2">
                <button
                  onClick={() => setStep("browse-skills")}
                  className="px-4 py-2 text-sm border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50"
                >
                  Back
                </button>
                <button
                  onClick={handleImport}
                  disabled={nameConflict || !localName}
                  className="px-4 py-2 text-sm bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Import Skill
                </button>
              </div>
            </div>
          )}

          {/* Step 4: Importing */}
          {step === "importing" && (
            <div className="flex flex-col items-center justify-center py-8">
              <Loader2 className="h-8 w-8 animate-spin text-blue-500" />
              <p className="mt-4 text-gray-500 dark:text-gray-400">Importing skill...</p>
            </div>
          )}

          {/* Step 5: Complete */}
          {step === "complete" && (
            <div className="flex flex-col items-center justify-center py-8">
              <CheckCircle className="h-12 w-12 text-green-500" />
              <h3 className="mt-4 font-medium text-gray-900 dark:text-gray-100">Skill Imported Successfully</h3>
              <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">
                The skill has been imported and is ready to use.
              </p>
              <button
                onClick={handleComplete}
                className="mt-6 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
              >
                Done
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
