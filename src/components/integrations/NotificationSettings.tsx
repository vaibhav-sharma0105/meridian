import { useState, useEffect } from "react";
import {
  X,
  Bell,
  BellOff,
  Volume2,
  VolumeX,
  AlertCircle,
  Info,
  Check,
  RefreshCw,
} from "lucide-react";
import * as api from "@/lib/tauri";
import toast from "react-hot-toast";

interface NotificationSettingsProps {
  onClose: () => void;
}

interface NotificationPreferences {
  desktop_enabled: boolean;
  sound_enabled: boolean;
  info_show_desktop: boolean;
  warning_show_desktop: boolean;
  critical_show_desktop: boolean;
  critical_play_sound: boolean;
}

const DEFAULT_PREFS: NotificationPreferences = {
  desktop_enabled: true,
  sound_enabled: true,
  info_show_desktop: false,
  warning_show_desktop: true,
  critical_show_desktop: true,
  critical_play_sound: true,
};

export function NotificationSettings({ onClose }: NotificationSettingsProps) {
  const [prefs, setPrefs] = useState<NotificationPreferences>(DEFAULT_PREFS);
  const [hasPermission, setHasPermission] = useState<boolean | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const permission = await api.checkNotificationPermission();
      setHasPermission(permission);

      const settings = await api.getAppSettings();
      setPrefs({
        desktop_enabled: settings.desktop_notifications_enabled !== "false",
        sound_enabled: settings.notification_sound_enabled !== "false",
        info_show_desktop: settings.notification_info_desktop === "true",
        warning_show_desktop: settings.notification_warning_desktop !== "false",
        critical_show_desktop: settings.notification_critical_desktop !== "false",
        critical_play_sound: settings.notification_critical_sound !== "false",
      });
    } catch (e) {
      console.error("Failed to load notification settings:", e);
    } finally {
      setIsLoading(false);
    }
  };

  const handleRequestPermission = async () => {
    try {
      await api.requestNotificationPermission();
      const permission = await api.checkNotificationPermission();
      setHasPermission(permission);
      if (permission) {
        toast.success("Notification permission granted");
      } else {
        toast.error("Notification permission denied");
      }
    } catch (e) {
      toast.error("Failed to request permission");
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await api.setAppSetting("desktop_notifications_enabled", String(prefs.desktop_enabled));
      await api.setAppSetting("notification_sound_enabled", String(prefs.sound_enabled));
      await api.setAppSetting("notification_info_desktop", String(prefs.info_show_desktop));
      await api.setAppSetting("notification_warning_desktop", String(prefs.warning_show_desktop));
      await api.setAppSetting("notification_critical_desktop", String(prefs.critical_show_desktop));
      await api.setAppSetting("notification_critical_sound", String(prefs.critical_play_sound));
      toast.success("Notification settings saved");
    } catch (e) {
      toast.error("Failed to save settings");
    } finally {
      setIsSaving(false);
    }
  };

  const handleTestNotification = async () => {
    try {
      await api.createNotificationWithOptions(
        "test",
        "Test Notification",
        "This is a test notification from Meridian",
        { severity: "info", desktop: true }
      );
      toast.success("Test notification sent");
    } catch (e) {
      toast.error("Failed to send test notification");
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 overflow-y-auto py-8">
      <div
        className="w-full max-w-md bg-white dark:bg-zinc-900 rounded-xl shadow-2xl mx-4"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-200 dark:border-zinc-700">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-indigo-100 dark:bg-indigo-900/30 flex items-center justify-center">
              <Bell className="w-5 h-5 text-indigo-600 dark:text-indigo-400" />
            </div>
            <div>
              <h3 className="font-semibold text-zinc-900 dark:text-zinc-100">
                Notification Settings
              </h3>
              <p className="text-xs text-zinc-500">
                Configure desktop notifications
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 rounded-md hover:bg-zinc-100 dark:hover:bg-zinc-800"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {isLoading ? (
          <div className="p-6 flex items-center justify-center">
            <RefreshCw className="w-5 h-5 animate-spin text-zinc-400" />
          </div>
        ) : (
          <div className="p-6 space-y-6">
            {/* Permission Status */}
            {hasPermission === false && (
              <div className="flex items-start gap-3 p-4 bg-amber-50 dark:bg-amber-900/20 rounded-lg">
                <AlertCircle className="w-5 h-5 text-amber-500 mt-0.5 flex-shrink-0" />
                <div>
                  <div className="text-sm font-medium text-amber-800 dark:text-amber-200">
                    Notifications Disabled
                  </div>
                  <p className="text-xs text-amber-700 dark:text-amber-300 mt-1">
                    Desktop notifications require system permission.
                  </p>
                  <button
                    onClick={handleRequestPermission}
                    className="mt-2 px-3 py-1.5 text-xs font-medium bg-amber-600 hover:bg-amber-700 text-white rounded"
                  >
                    Enable Notifications
                  </button>
                </div>
              </div>
            )}

            {hasPermission && (
              <div className="flex items-center gap-2 p-3 bg-green-50 dark:bg-green-900/20 rounded-lg">
                <Check className="w-4 h-4 text-green-500" />
                <span className="text-sm text-green-700 dark:text-green-300">
                  Desktop notifications enabled
                </span>
              </div>
            )}

            {/* Global Settings */}
            <div className="space-y-3">
              <h4 className="text-sm font-medium text-zinc-700 dark:text-zinc-300">
                Global Settings
              </h4>
              <label className="flex items-center justify-between p-3 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg cursor-pointer">
                <div className="flex items-center gap-3">
                  {prefs.desktop_enabled ? (
                    <Bell className="w-5 h-5 text-zinc-500" />
                  ) : (
                    <BellOff className="w-5 h-5 text-zinc-400" />
                  )}
                  <div>
                    <div className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
                      Desktop Notifications
                    </div>
                    <div className="text-xs text-zinc-500">
                      Show notifications in system tray
                    </div>
                  </div>
                </div>
                <input
                  type="checkbox"
                  checked={prefs.desktop_enabled}
                  onChange={(e) =>
                    setPrefs((p) => ({ ...p, desktop_enabled: e.target.checked }))
                  }
                  className="w-4 h-4 rounded border-zinc-300 dark:border-zinc-600 text-indigo-600"
                />
              </label>

              <label className="flex items-center justify-between p-3 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg cursor-pointer">
                <div className="flex items-center gap-3">
                  {prefs.sound_enabled ? (
                    <Volume2 className="w-5 h-5 text-zinc-500" />
                  ) : (
                    <VolumeX className="w-5 h-5 text-zinc-400" />
                  )}
                  <div>
                    <div className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
                      Notification Sounds
                    </div>
                    <div className="text-xs text-zinc-500">
                      Play sound for critical notifications
                    </div>
                  </div>
                </div>
                <input
                  type="checkbox"
                  checked={prefs.sound_enabled}
                  onChange={(e) =>
                    setPrefs((p) => ({ ...p, sound_enabled: e.target.checked }))
                  }
                  className="w-4 h-4 rounded border-zinc-300 dark:border-zinc-600 text-indigo-600"
                />
              </label>
            </div>

            {/* Per-Severity Settings */}
            <div className="space-y-3">
              <h4 className="text-sm font-medium text-zinc-700 dark:text-zinc-300">
                Severity-Based Settings
              </h4>
              <div className="border border-zinc-200 dark:border-zinc-700 rounded-lg divide-y divide-zinc-200 dark:divide-zinc-700">
                <div className="flex items-center justify-between px-4 py-3">
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-blue-500" />
                    <span className="text-sm text-zinc-700 dark:text-zinc-300">
                      Info
                    </span>
                  </div>
                  <label className="flex items-center gap-2 text-xs text-zinc-500 cursor-pointer">
                    Desktop
                    <input
                      type="checkbox"
                      checked={prefs.info_show_desktop}
                      onChange={(e) =>
                        setPrefs((p) => ({ ...p, info_show_desktop: e.target.checked }))
                      }
                      disabled={!prefs.desktop_enabled}
                      className="w-3.5 h-3.5 rounded border-zinc-300 dark:border-zinc-600 text-indigo-600"
                    />
                  </label>
                </div>
                <div className="flex items-center justify-between px-4 py-3">
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-amber-500" />
                    <span className="text-sm text-zinc-700 dark:text-zinc-300">
                      Warning
                    </span>
                  </div>
                  <label className="flex items-center gap-2 text-xs text-zinc-500 cursor-pointer">
                    Desktop
                    <input
                      type="checkbox"
                      checked={prefs.warning_show_desktop}
                      onChange={(e) =>
                        setPrefs((p) => ({ ...p, warning_show_desktop: e.target.checked }))
                      }
                      disabled={!prefs.desktop_enabled}
                      className="w-3.5 h-3.5 rounded border-zinc-300 dark:border-zinc-600 text-indigo-600"
                    />
                  </label>
                </div>
                <div className="flex items-center justify-between px-4 py-3">
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-red-500" />
                    <span className="text-sm text-zinc-700 dark:text-zinc-300">
                      Critical
                    </span>
                  </div>
                  <div className="flex items-center gap-4">
                    <label className="flex items-center gap-2 text-xs text-zinc-500 cursor-pointer">
                      Desktop
                      <input
                        type="checkbox"
                        checked={prefs.critical_show_desktop}
                        onChange={(e) =>
                          setPrefs((p) => ({ ...p, critical_show_desktop: e.target.checked }))
                        }
                        disabled={!prefs.desktop_enabled}
                        className="w-3.5 h-3.5 rounded border-zinc-300 dark:border-zinc-600 text-indigo-600"
                      />
                    </label>
                    <label className="flex items-center gap-2 text-xs text-zinc-500 cursor-pointer">
                      Sound
                      <input
                        type="checkbox"
                        checked={prefs.critical_play_sound}
                        onChange={(e) =>
                          setPrefs((p) => ({ ...p, critical_play_sound: e.target.checked }))
                        }
                        disabled={!prefs.sound_enabled}
                        className="w-3.5 h-3.5 rounded border-zinc-300 dark:border-zinc-600 text-indigo-600"
                      />
                    </label>
                  </div>
                </div>
              </div>
            </div>

            {/* Test Button */}
            <button
              onClick={handleTestNotification}
              disabled={!hasPermission || !prefs.desktop_enabled}
              className="w-full px-4 py-2 text-sm text-zinc-600 dark:text-zinc-400 border border-zinc-200 dark:border-zinc-700 rounded-lg hover:bg-zinc-50 dark:hover:bg-zinc-800 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Send Test Notification
            </button>
          </div>
        )}

        {/* Footer */}
        <div className="px-6 py-4 border-t border-zinc-200 dark:border-zinc-700 flex items-center justify-end gap-2">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={isSaving}
            className="px-4 py-2 text-sm font-medium bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg transition-colors disabled:opacity-50"
          >
            {isSaving ? "Saving..." : "Save Settings"}
          </button>
        </div>
      </div>
    </div>
  );
}
