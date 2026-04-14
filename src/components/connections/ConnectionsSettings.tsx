import { useState, useRef } from "react";
import {
  X,
  RefreshCw,
  Link2,
  Unlink,
  CheckCircle2,
  AlertCircle,
  ChevronDown,
  ChevronUp,
  Copy,
  Check,
  ExternalLink,
  Upload,
  FileText,
  Loader2,
  Info,
  RotateCcw,
} from "lucide-react";
import { useConnections } from "@/hooks/useConnections";
import { useProjectStore } from "@/stores/projectStore";
import { format, isValid } from "date-fns";
import * as api from "@/lib/tauri";
import { open } from "@tauri-apps/plugin-dialog";
import toast from "react-hot-toast";

// ─── Constants ────────────────────────────────────────────────────────────────

const APPS_SCRIPT_CODE = `// ── Meridian Sheets Relay ──────────────────────────────────────────
// Paste this into Extensions → Apps Script, then run setSecretKey()
// once to store your secret, then deploy as a Web App.

var SECRET_PROP = 'MERIDIAN_KEY';

function doGet(e) {
  try {
    var secret = PropertiesService.getScriptProperties()
                   .getProperty(SECRET_PROP);
    if (!secret || !e.parameter.key || e.parameter.key !== secret) {
      return out({ error: 'unauthorized' });
    }
    if (e.parameter.test === '1') {
      return out({ ok: true, message: 'Connected!' });
    }
    var since = parseFloat(e.parameter.since || '0');
    var ss    = SpreadsheetApp.getActiveSpreadsheet();
    var sheet = ss.getSheetByName('meridian_imports') || ss.getSheets()[0];
    var data  = sheet.getDataRange().getValues();
    if (data.length <= 1) return out({ rows: [] });
    var headers = data[0].map(function(h){ return String(h).trim(); });
    var rows = [];
    for (var i = 1; i < data.length; i++) {
      var row = {};
      for (var j = 0; j < headers.length; j++) {
        var val = data[i][j];
        // Convert Date objects to ISO 8601 — Google Sheets stores dates as
        // Date objects; String() produces locale formats that break parsing.
        row[headers[j]] = (val instanceof Date) ? val.toISOString() : (val !== undefined ? String(val) : '');
      }
      var ts = row['created_at'] ? new Date(row['created_at']).getTime() : NaN;
      // Include rows with unparseable dates (NaN) — Meridian deduplicates via import_id.
      // Only apply the since filter when the date is valid.
      if (isNaN(ts) || ts > since) rows.push(row);
    }
    return out({ rows: rows });
  } catch(err) {
    return out({ error: err.toString() });
  }
}

/** Run this once to set your secret key, then never again. */
function setSecretKey() {
  // Replace the value below with your secret from Meridian
  PropertiesService.getScriptProperties()
    .setProperty(SECRET_PROP, 'PASTE_YOUR_SECRET_HERE');
  Logger.log('Secret key saved.');
}

function out(obj) {
  return ContentService.createTextOutput(JSON.stringify(obj))
    .setMimeType(ContentService.MimeType.JSON);
}`;

const SHEET_HEADERS = "import_id\tcreated_at\ttitle\tmeeting_date\tsummary\taction_items\tsource_subject";

// ─── Types ────────────────────────────────────────────────────────────────────

interface Props {
  open: boolean;
  onClose: () => void;
  runSync: () => Promise<void>;
  isSyncing: boolean;
}

// ─── Small helper components ──────────────────────────────────────────────────

function CopyButton({ text, label }: { text: string; label?: string }) {
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };
  return (
    <button
      onClick={copy}
      className="flex items-center gap-1 px-2 py-1 text-xs rounded bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-300 hover:bg-zinc-200 dark:hover:bg-zinc-700 transition-colors"
    >
      {copied ? <Check className="w-3 h-3 text-green-500" /> : <Copy className="w-3 h-3" />}
      {label ?? (copied ? "Copied!" : "Copy")}
    </button>
  );
}

function StepBadge({ n, done }: { n: number; done?: boolean }) {
  return (
    <span className={`flex-shrink-0 w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold ${
      done
        ? "bg-green-500 text-white"
        : "bg-indigo-100 dark:bg-indigo-900/40 text-indigo-700 dark:text-indigo-300"
    }`}>
      {done ? <Check className="w-3 h-3" /> : n}
    </span>
  );
}

// ─── Main component ───────────────────────────────────────────────────────────

export default function ConnectionsSettings({ open: isOpen, onClose, runSync, isSyncing }: Props) {
  const {
    zoom,
    sheetsRelay,
    connectZoom,
    isConnectingZoom,
    saveSheetRelayConfig,
    isSavingSheetRelay,
    testSheetsRelay,
    isTestingSheetsRelay,
    sheetsRelayTestResult,
    sheetsRelayTestError,
    disconnect,
    zoomError,
    sheetsRelayError,
  } = useConnections();

  const { projects } = useProjectStore();

  // Sheets relay setup form state
  const [showRelaySetup, setShowRelaySetup] = useState(false);
  const [scriptUrl, setScriptUrl] = useState("");
  const [secretKey, setSecretKey] = useState(() => crypto.randomUUID());
  const [testMessage, setTestMessage] = useState<{ ok: boolean; text: string } | null>(null);

  // Manual import state
  const [pasteText, setPasteText] = useState("");
  const [manualTitle, setManualTitle] = useState("");
  const [manualProjectId, setManualProjectId] = useState("");
  const [isImporting, setIsImporting] = useState(false);
  const [importError, setImportError] = useState<string | null>(null);
  const [importSuccess, setImportSuccess] = useState(false);

  // Disconnect confirm guard
  const [disconnectConfirm, setDisconnectConfirm] = useState<string | null>(null);

  const fileInputRef = useRef<HTMLInputElement>(null);

  if (!isOpen) return null;

  // ── Zoom helpers ─────────────────────────────────────────────────────────

  const handleDisconnect = async (provider: string) => {
    if (disconnectConfirm !== provider) {
      setDisconnectConfirm(provider);
      return;
    }
    await disconnect(provider);
    setDisconnectConfirm(null);
    if (provider === "sheets_relay") {
      setShowRelaySetup(false);
      setTestMessage(null);
    }
  };

  // ── Sheets relay helpers ─────────────────────────────────────────────────

  const regenerateSecret = () => setSecretKey(crypto.randomUUID());

  const handleSaveAndTest = async () => {
    setTestMessage(null);
    try {
      await saveSheetRelayConfig({ scriptUrl: scriptUrl.trim(), secretKey });
      const msg = await testSheetsRelay();
      setTestMessage({ ok: true, text: msg });
      toast.success("Sheets relay connected!");
    } catch (err) {
      setTestMessage({ ok: false, text: String(err) });
    }
  };

  const handleTestExisting = async () => {
    setTestMessage(null);
    try {
      const msg = await testSheetsRelay();
      setTestMessage({ ok: true, text: msg });
    } catch (err) {
      setTestMessage({ ok: false, text: String(err) });
    }
  };

  // ── Manual import helpers ────────────────────────────────────────────────

  const handleImportText = async () => {
    if (!pasteText.trim() || !manualProjectId) return;
    setIsImporting(true);
    setImportError(null);
    setImportSuccess(false);
    try {
      await api.ingestMeeting({
        projectId: manualProjectId,
        title: manualTitle.trim() || "Zoom Meeting (manual import)",
        platform: "zoom",
        rawTranscript: pasteText.trim(),
      });
      setImportSuccess(true);
      setPasteText("");
      setManualTitle("");
      toast.success("Meeting imported and analysed!");
    } catch (err) {
      setImportError(String(err));
    } finally {
      setIsImporting(false);
    }
  };

  const handleImportFile = async () => {
    if (!manualProjectId) {
      setImportError("Please select a project first.");
      return;
    }
    setImportError(null);
    setImportSuccess(false);
    try {
      const filePath = await open({
        multiple: false,
        filters: [{ name: "Transcript / Summary", extensions: ["txt", "vtt", "md", "pdf"] }],
      });
      if (!filePath) return;
      const path = typeof filePath === "string" ? filePath : filePath[0];
      setIsImporting(true);
      await api.ingestMeetingFromFile({
        projectId: manualProjectId,
        filePath: path,
        platform: "zoom",
      });
      setImportSuccess(true);
      toast.success("Meeting imported and analysed!");
    } catch (err) {
      setImportError(String(err));
    } finally {
      setIsImporting(false);
    }
  };

  // ── Render ───────────────────────────────────────────────────────────────

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/40 overflow-y-auto py-8"
      onClick={onClose}
    >
      <div
        className="relative w-full max-w-lg bg-white dark:bg-zinc-900 rounded-xl shadow-2xl border border-zinc-200 dark:border-zinc-800 p-6 mx-4"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between mb-5">
          <h2 className="text-base font-semibold text-zinc-900 dark:text-zinc-50">
            Connections
          </h2>
          <button
            onClick={onClose}
            className="p-1.5 text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-300 rounded-md hover:bg-zinc-100 dark:hover:bg-zinc-800"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="space-y-4">

          {/* ── ZOOM ─────────────────────────────────────────────────────── */}
          <div className="border border-zinc-200 dark:border-zinc-700 rounded-lg p-4">
            <div className="flex items-center gap-2 mb-3">
              <span className="text-lg">🎥</span>
              <span className="font-medium text-zinc-900 dark:text-zinc-50">Zoom</span>
              {zoom && <CheckCircle2 className="w-4 h-4 text-green-500 ml-auto" />}
            </div>

            {zoom ? (
              <div className="space-y-3">
                <div>
                  <p className="text-sm text-zinc-600 dark:text-zinc-400">
                    Connected as{" "}
                    <span className="font-medium text-zinc-900 dark:text-zinc-50">
                      {zoom.account_email}
                    </span>
                  </p>
                  <p className="text-xs text-zinc-400 mt-0.5">
                    {zoom.last_sync_at && (() => {
                      const d = new Date(zoom.last_sync_at!);
                      return isValid(d) ? `Last synced: ${format(d, "MMM d, h:mm a")}` : null;
                    })() || "Never synced"}
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    onClick={runSync}
                    disabled={isSyncing}
                    className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md bg-indigo-50 text-indigo-700 hover:bg-indigo-100 dark:bg-indigo-900/30 dark:text-indigo-300 disabled:opacity-50 transition-colors"
                  >
                    <RefreshCw className={`w-3 h-3 ${isSyncing ? "animate-spin" : ""}`} />
                    {isSyncing ? "Syncing…" : "Sync Now"}
                  </button>
                  <button
                    onClick={() => handleDisconnect("zoom")}
                    className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/20 transition-colors"
                  >
                    <Unlink className="w-3 h-3" />
                    {disconnectConfirm === "zoom" ? "Click again to confirm" : "Disconnect"}
                  </button>
                </div>
              </div>
            ) : (
              <div className="space-y-3">
                <p className="text-sm text-zinc-500 dark:text-zinc-400">
                  Requires a Zoom Marketplace app (Pro plan or higher). If your plan
                  doesn't support it, use the Gmail relay below instead.
                </p>
                {zoomError && (
                  <div className="flex items-start gap-2 p-2 bg-red-50 dark:bg-red-900/20 rounded-md">
                    <AlertCircle className="w-4 h-4 text-red-500 flex-shrink-0 mt-0.5" />
                    <p className="text-xs text-red-600 dark:text-red-400">{String(zoomError)}</p>
                  </div>
                )}
                <button
                  onClick={() => connectZoom()}
                  disabled={isConnectingZoom}
                  className="flex items-center gap-2 px-3 py-2 text-sm font-medium rounded-md bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50 transition-colors"
                >
                  <Link2 className="w-4 h-4" />
                  {isConnectingZoom ? "Connecting… (check your browser)" : "Connect Zoom"}
                </button>
              </div>
            )}
          </div>

          {/* ── GMAIL VIA SHEETS RELAY ───────────────────────────────────── */}
          <div className="border border-zinc-200 dark:border-zinc-700 rounded-lg p-4">
            <div className="flex items-center gap-2 mb-1">
              <span className="text-lg">✉️</span>
              <span className="font-medium text-zinc-900 dark:text-zinc-50">
                Gmail → Sheets Relay
              </span>
              {sheetsRelay
                ? <CheckCircle2 className="w-4 h-4 text-green-500 ml-auto" />
                : <span className="ml-auto text-xs text-indigo-600 dark:text-indigo-400 bg-indigo-50 dark:bg-indigo-900/30 px-2 py-0.5 rounded-full font-medium">No API key needed</span>
              }
            </div>
            <p className="text-xs text-zinc-400 dark:text-zinc-500 mb-3">
              Studio watches Gmail for Zoom summaries → writes to a private Sheet → Meridian polls every 15 min.
            </p>

            {sheetsRelay ? (
              /* ── Connected state ── */
              <div className="space-y-3">
                <div>
                  <p className="text-sm text-zinc-600 dark:text-zinc-400">
                    Relay active
                    {sheetsRelay.last_sync_at && (() => {
                      const d = new Date(sheetsRelay.last_sync_at!);
                      return isValid(d) ? ` · Last synced: ${format(d, "MMM d, h:mm a")}` : "";
                    })()}
                  </p>
                  <p className="text-xs text-zinc-400 mt-0.5 truncate">
                    {sheetsRelay.account_email}
                  </p>
                </div>

                {testMessage && (
                  <div className={`flex items-start gap-2 p-2 rounded-md text-xs ${
                    testMessage.ok
                      ? "bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-300"
                      : "bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400"
                  }`}>
                    {testMessage.ok
                      ? <CheckCircle2 className="w-3.5 h-3.5 flex-shrink-0 mt-0.5" />
                      : <AlertCircle className="w-3.5 h-3.5 flex-shrink-0 mt-0.5" />}
                    {testMessage.text}
                  </div>
                )}

                <div className="flex items-center gap-2 flex-wrap">
                  <button
                    onClick={runSync}
                    disabled={isSyncing}
                    className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md bg-indigo-50 text-indigo-700 hover:bg-indigo-100 dark:bg-indigo-900/30 dark:text-indigo-300 disabled:opacity-50 transition-colors"
                  >
                    <RefreshCw className={`w-3 h-3 ${isSyncing ? "animate-spin" : ""}`} />
                    {isSyncing ? "Syncing…" : "Sync Now"}
                  </button>
                  <button
                    onClick={handleTestExisting}
                    disabled={isTestingSheetsRelay}
                    className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-300 hover:bg-zinc-200 dark:hover:bg-zinc-700 disabled:opacity-50 transition-colors"
                  >
                    {isTestingSheetsRelay
                      ? <Loader2 className="w-3 h-3 animate-spin" />
                      : <CheckCircle2 className="w-3 h-3" />}
                    Test connection
                  </button>
                  <button
                    onClick={() => handleDisconnect("sheets_relay")}
                    className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/20 transition-colors"
                  >
                    <Unlink className="w-3 h-3" />
                    {disconnectConfirm === "sheets_relay" ? "Click again to confirm" : "Disconnect"}
                  </button>
                </div>
              </div>
            ) : (
              /* ── Not configured state ── */
              <div className="space-y-3">
                {!showRelaySetup ? (
                  <div className="space-y-3">
                    <div className="bg-zinc-50 dark:bg-zinc-800/60 rounded-md p-3 space-y-1.5">
                      <p className="text-xs font-medium text-zinc-700 dark:text-zinc-300">How it works</p>
                      <ul className="text-xs text-zinc-500 dark:text-zinc-400 space-y-1">
                        <li className="flex items-start gap-1.5"><span className="text-indigo-500 font-bold mt-0.5">1.</span> Studio detects a new Zoom summary email in Gmail</li>
                        <li className="flex items-start gap-1.5"><span className="text-indigo-500 font-bold mt-0.5">2.</span> Studio appends a row to your private Google Sheet</li>
                        <li className="flex items-start gap-1.5"><span className="text-indigo-500 font-bold mt-0.5">3.</span> Meridian polls the Sheet every 15 min and imports new meetings</li>
                      </ul>
                    </div>
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => setShowRelaySetup(true)}
                        className="flex items-center gap-2 px-3 py-2 text-sm font-medium rounded-md bg-indigo-600 text-white hover:bg-indigo-700 transition-colors"
                      >
                        <Link2 className="w-4 h-4" />
                        Set up Sheets Relay
                      </button>
                      <a
                        href="#"
                        onClick={(e) => { e.preventDefault(); api.openUrl("https://github.com/vaibhav-sharma0105/meridian/blob/main/docs/GMAIL_SHEETS_RELAY_SETUP.md"); }}
                        className="flex items-center gap-1 text-xs text-indigo-600 dark:text-indigo-400 hover:underline"
                      >
                        <ExternalLink className="w-3 h-3" />
                        Full setup guide
                      </a>
                    </div>
                  </div>
                ) : (
                  /* ── Setup form ── */
                  <div className="space-y-4">
                    <div className="flex items-center justify-between">
                      <p className="text-xs font-semibold text-zinc-700 dark:text-zinc-300 uppercase tracking-wide">
                        Setup steps
                      </p>
                      <button
                        onClick={() => setShowRelaySetup(false)}
                        className="text-xs text-zinc-400 hover:text-zinc-600"
                      >
                        Cancel
                      </button>
                    </div>

                    {/* Step 1 */}
                    <div className="space-y-2">
                      <div className="flex items-center gap-2">
                        <StepBadge n={1} />
                        <span className="text-sm font-medium text-zinc-800 dark:text-zinc-200">
                          Create the Google Sheet
                        </span>
                      </div>
                      <div className="ml-8 space-y-2">
                        <p className="text-xs text-zinc-500 dark:text-zinc-400">
                          Create a new Google Sheet. Rename the first tab to{" "}
                          <code className="bg-zinc-100 dark:bg-zinc-800 px-1 rounded">meridian_imports</code>.
                          Then add these headers in row 1 (one per column):
                        </p>
                        <div className="flex items-center gap-2">
                          <code className="text-xs bg-zinc-100 dark:bg-zinc-800 px-2 py-1 rounded text-zinc-700 dark:text-zinc-300 flex-1 overflow-x-auto whitespace-nowrap">
                            import_id · created_at · title · meeting_date · summary · action_items · source_subject
                          </code>
                          <CopyButton text={SHEET_HEADERS} label="Copy" />
                        </div>
                      </div>
                    </div>

                    {/* Step 2 */}
                    <div className="space-y-2">
                      <div className="flex items-center gap-2">
                        <StepBadge n={2} />
                        <span className="text-sm font-medium text-zinc-800 dark:text-zinc-200">
                          Add the Apps Script
                        </span>
                      </div>
                      <div className="ml-8 space-y-2">
                        <p className="text-xs text-zinc-500 dark:text-zinc-400">
                          In your Sheet click <strong>Extensions → Apps Script</strong>. Delete
                          any existing code and paste the script below.
                        </p>
                        <div className="relative">
                          <pre className="text-xs bg-zinc-950 text-green-400 p-3 rounded-md overflow-x-auto max-h-36 overflow-y-auto leading-relaxed">
                            {APPS_SCRIPT_CODE}
                          </pre>
                          <div className="absolute top-2 right-2">
                            <CopyButton text={APPS_SCRIPT_CODE} />
                          </div>
                        </div>
                      </div>
                    </div>

                    {/* Step 3 */}
                    <div className="space-y-2">
                      <div className="flex items-center gap-2">
                        <StepBadge n={3} />
                        <span className="text-sm font-medium text-zinc-800 dark:text-zinc-200">
                          Set the secret key
                        </span>
                      </div>
                      <div className="ml-8 space-y-2">
                        <p className="text-xs text-zinc-500 dark:text-zinc-400">
                          This key proves requests come from Meridian. Copy it, then replace{" "}
                          <code className="bg-zinc-100 dark:bg-zinc-800 px-1 rounded">PASTE_YOUR_SECRET_HERE</code>{" "}
                          in the script with your key. Then in the Apps Script editor click
                          <strong> Run → setSecretKey</strong> once.
                        </p>
                        <div className="flex items-center gap-2">
                          <input
                            value={secretKey}
                            readOnly
                            className="flex-1 px-2 py-1.5 text-xs font-mono rounded-md border border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-800 text-zinc-700 dark:text-zinc-300"
                          />
                          <CopyButton text={secretKey} />
                          <button
                            onClick={regenerateSecret}
                            title="Generate new key"
                            className="p-1.5 rounded-md text-zinc-400 hover:text-zinc-700 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
                          >
                            <RotateCcw className="w-3.5 h-3.5" />
                          </button>
                        </div>
                      </div>
                    </div>

                    {/* Step 4 */}
                    <div className="space-y-2">
                      <div className="flex items-center gap-2">
                        <StepBadge n={4} />
                        <span className="text-sm font-medium text-zinc-800 dark:text-zinc-200">
                          Deploy the Web App
                        </span>
                      </div>
                      <div className="ml-8">
                        <p className="text-xs text-zinc-500 dark:text-zinc-400">
                          In Apps Script click <strong>Deploy → New deployment</strong>.
                          Set type to <strong>Web app</strong>, execute as <strong>Me</strong>,
                          who has access <strong>Anyone</strong>. Click Deploy and copy the URL.
                        </p>
                      </div>
                    </div>

                    {/* Step 5 — URL input + connect */}
                    <div className="space-y-2">
                      <div className="flex items-center gap-2">
                        <StepBadge n={5} />
                        <span className="text-sm font-medium text-zinc-800 dark:text-zinc-200">
                          Paste the Web App URL
                        </span>
                      </div>
                      <div className="ml-8 space-y-2">
                        <input
                          type="url"
                          placeholder="https://script.google.com/macros/s/…/exec"
                          value={scriptUrl}
                          onChange={(e) => setScriptUrl(e.target.value)}
                          className="w-full px-3 py-2 text-xs rounded-md border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100 placeholder-zinc-400 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        />

                        {(testMessage || sheetsRelayError) && (
                          <div className={`flex items-start gap-2 p-2 rounded-md text-xs ${
                            testMessage?.ok
                              ? "bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-300"
                              : "bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400"
                          }`}>
                            {testMessage?.ok
                              ? <CheckCircle2 className="w-3.5 h-3.5 flex-shrink-0 mt-0.5" />
                              : <AlertCircle className="w-3.5 h-3.5 flex-shrink-0 mt-0.5" />}
                            {testMessage?.text ?? String(sheetsRelayError)}
                          </div>
                        )}

                        <button
                          onClick={handleSaveAndTest}
                          disabled={isSavingSheetRelay || isTestingSheetsRelay || !scriptUrl.trim()}
                          className="flex items-center gap-2 px-4 py-2 text-sm font-medium rounded-md bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50 transition-colors"
                        >
                          {(isSavingSheetRelay || isTestingSheetsRelay)
                            ? <Loader2 className="w-4 h-4 animate-spin" />
                            : <CheckCircle2 className="w-4 h-4" />}
                          Save & Test Connection
                        </button>
                      </div>
                    </div>

                    {/* Step 6 — Studio */}
                    <div className="space-y-2">
                      <div className="flex items-center gap-2">
                        <StepBadge n={6} />
                        <span className="text-sm font-medium text-zinc-800 dark:text-zinc-200">
                          Create the Studio workflow
                        </span>
                      </div>
                      <div className="ml-8 space-y-1.5">
                        <p className="text-xs text-zinc-500 dark:text-zinc-400">
                          In{" "}
                          <a
                            href="https://studio.workspace.google.com"
                            target="_blank"
                            rel="noreferrer"
                            className="text-indigo-600 dark:text-indigo-400 hover:underline"
                          >
                            Google Workspace AI Studio
                          </a>{" "}
                          create a workflow with:
                        </p>
                        <ul className="text-xs text-zinc-500 dark:text-zinc-400 space-y-1 list-disc list-inside ml-1">
                          <li><strong>Trigger:</strong> Gmail — new email matching <code className="bg-zinc-100 dark:bg-zinc-800 px-1 rounded">from:zoom.us</code></li>
                          <li><strong>Action:</strong> Append row to your Sheet — map email subject → title, body → summary, date → meeting_date</li>
                          <li>For <code className="bg-zinc-100 dark:bg-zinc-800 px-1 rounded">import_id</code> use a unique formula (e.g. email message ID if available)</li>
                          <li>For <code className="bg-zinc-100 dark:bg-zinc-800 px-1 rounded">created_at</code> use today's timestamp</li>
                        </ul>
                        <a
                          href="#"
                          onClick={(e) => {
                            e.preventDefault();
                            api.openUrl("https://github.com/vaibhav-sharma0105/meridian/blob/main/docs/GMAIL_SHEETS_RELAY_SETUP.md");
                          }}
                          className="inline-flex items-center gap-1 text-xs text-indigo-600 dark:text-indigo-400 hover:underline mt-1"
                        >
                          <ExternalLink className="w-3 h-3" />
                          Open full setup guide with screenshots
                        </a>
                      </div>
                    </div>

                  </div>
                )}
              </div>
            )}
          </div>

          {/* ── MANUAL IMPORT ────────────────────────────────────────────── */}
          <div className="border border-zinc-200 dark:border-zinc-700 rounded-lg p-4">
            <div className="flex items-center gap-2 mb-3">
              <span className="text-lg">📎</span>
              <span className="font-medium text-zinc-900 dark:text-zinc-50">Manual Import</span>
              <span className="ml-auto text-xs text-zinc-400 bg-zinc-100 dark:bg-zinc-800 px-2 py-0.5 rounded-full">
                Always available
              </span>
            </div>
            <p className="text-sm text-zinc-500 dark:text-zinc-400 mb-3">
              Paste a transcript or AI summary, or upload a <code className="bg-zinc-100 dark:bg-zinc-800 px-1 rounded text-xs">.txt</code> / <code className="bg-zinc-100 dark:bg-zinc-800 px-1 rounded text-xs">.vtt</code> file. Meridian will extract tasks and create a meeting entry.
            </p>

            <div className="space-y-3">
              {/* Project picker */}
              <div>
                <label className="block text-xs font-medium text-zinc-600 dark:text-zinc-400 mb-1">
                  Import into project
                </label>
                <select
                  value={manualProjectId}
                  onChange={(e) => setManualProjectId(e.target.value)}
                  className="w-full px-3 py-2 text-sm rounded-md border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                >
                  <option value="">Select a project…</option>
                  {projects.filter(p => !p.archived_at).map((p) => (
                    <option key={p.id} value={p.id}>{p.name}</option>
                  ))}
                </select>
              </div>

              {/* Title (optional) */}
              <div>
                <label className="block text-xs font-medium text-zinc-600 dark:text-zinc-400 mb-1">
                  Meeting title <span className="text-zinc-400 font-normal">(optional)</span>
                </label>
                <input
                  type="text"
                  placeholder="e.g. Q2 Planning — defaults to 'Zoom Meeting'"
                  value={manualTitle}
                  onChange={(e) => setManualTitle(e.target.value)}
                  className="w-full px-3 py-2 text-sm rounded-md border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100 placeholder-zinc-400 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                />
              </div>

              {/* Paste area */}
              <div>
                <label className="block text-xs font-medium text-zinc-600 dark:text-zinc-400 mb-1">
                  Paste transcript or AI summary
                </label>
                <textarea
                  value={pasteText}
                  onChange={(e) => setPasteText(e.target.value)}
                  placeholder="Paste the full Zoom AI summary, transcript, or any meeting notes here…"
                  rows={5}
                  className="w-full px-3 py-2 text-sm rounded-md border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100 placeholder-zinc-400 focus:outline-none focus:ring-2 focus:ring-indigo-500 resize-y"
                />
              </div>

              {/* Error / success feedback */}
              {importError && (
                <div className="flex items-start gap-2 p-2 bg-red-50 dark:bg-red-900/20 rounded-md">
                  <AlertCircle className="w-4 h-4 text-red-500 flex-shrink-0 mt-0.5" />
                  <p className="text-xs text-red-600 dark:text-red-400">{importError}</p>
                </div>
              )}
              {importSuccess && (
                <div className="flex items-start gap-2 p-2 bg-green-50 dark:bg-green-900/20 rounded-md">
                  <CheckCircle2 className="w-4 h-4 text-green-500 flex-shrink-0 mt-0.5" />
                  <p className="text-xs text-green-700 dark:text-green-300">
                    Meeting imported successfully! Tasks have been extracted.
                  </p>
                </div>
              )}

              {/* Action buttons */}
              <div className="flex items-center gap-2 flex-wrap">
                <button
                  onClick={handleImportText}
                  disabled={isImporting || !pasteText.trim() || !manualProjectId}
                  className="flex items-center gap-2 px-4 py-2 text-sm font-medium rounded-md bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50 transition-colors"
                >
                  {isImporting
                    ? <Loader2 className="w-4 h-4 animate-spin" />
                    : <FileText className="w-4 h-4" />}
                  {isImporting ? "Analysing…" : "Import Pasted Text"}
                </button>
                <button
                  onClick={handleImportFile}
                  disabled={isImporting || !manualProjectId}
                  className="flex items-center gap-2 px-4 py-2 text-sm font-medium rounded-md border border-zinc-300 dark:border-zinc-600 text-zinc-700 dark:text-zinc-300 hover:bg-zinc-50 dark:hover:bg-zinc-800 disabled:opacity-50 transition-colors"
                >
                  <Upload className="w-4 h-4" />
                  Upload File (.txt / .vtt)
                </button>
              </div>
              {!manualProjectId && (
                <p className="text-xs text-amber-600 dark:text-amber-400 flex items-center gap-1">
                  <Info className="w-3 h-3" />
                  Select a project above to enable import
                </p>
              )}
            </div>
          </div>

        </div>
      </div>
    </div>
  );
}
