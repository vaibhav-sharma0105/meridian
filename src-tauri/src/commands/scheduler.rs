use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct SchedulerStatus {
    pub enabled: bool,
    pub platform: String,
    pub service_name: String,
    pub error: Option<String>,
}

const LAUNCHD_LABEL: &str = "com.meridian.daemon";
const TASK_NAME: &str = "MeridianDaemon";

fn get_daemon_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .unwrap_or_default()
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();

    let suffix = if cfg!(windows) { ".exe" } else { "" };
    let binary_name = format!("meridian-daemon{}", suffix);

    exe_dir.join(binary_name)
}

fn get_launchd_plist_path() -> PathBuf {
    dirs_next::home_dir()
        .unwrap_or_default()
        .join("Library/LaunchAgents")
        .join(format!("{}.plist", LAUNCHD_LABEL))
}

fn generate_launchd_plist(daemon_path: &PathBuf) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>
    <key>StandardOutPath</key>
    <string>{}/logs/daemon.log</string>
    <key>StandardErrorPath</key>
    <string>{}/logs/daemon-error.log</string>
    <key>WorkingDirectory</key>
    <string>{}</string>
</dict>
</plist>"#,
        LAUNCHD_LABEL,
        daemon_path.display(),
        crate::db::connection::get_data_dir().display(),
        crate::db::connection::get_data_dir().display(),
        crate::db::connection::get_data_dir().display()
    )
}

#[cfg(target_os = "macos")]
fn is_launchd_registered() -> bool {
    let output = Command::new("launchctl")
        .args(["list", LAUNCHD_LABEL])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

#[cfg(not(target_os = "macos"))]
fn is_launchd_registered() -> bool {
    false
}

#[cfg(target_os = "macos")]
fn register_launchd() -> Result<(), String> {
    let daemon_path = get_daemon_path();
    if !daemon_path.exists() {
        return Err(format!("Daemon binary not found at {:?}", daemon_path));
    }

    let plist_path = get_launchd_plist_path();
    let plist_dir = plist_path.parent().unwrap();

    // Ensure LaunchAgents directory exists
    fs::create_dir_all(plist_dir)
        .map_err(|e| format!("Failed to create LaunchAgents directory: {}", e))?;

    // Ensure logs directory exists
    fs::create_dir_all(crate::db::connection::get_logs_dir())
        .map_err(|e| format!("Failed to create logs directory: {}", e))?;

    // Write plist file
    let plist_content = generate_launchd_plist(&daemon_path);
    fs::write(&plist_path, plist_content)
        .map_err(|e| format!("Failed to write plist: {}", e))?;

    // Load the launch agent
    let output = Command::new("launchctl")
        .args(["load", "-w", plist_path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("Failed to run launchctl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("launchctl load failed: {}", stderr));
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn register_launchd() -> Result<(), String> {
    Err("launchd is only available on macOS".to_string())
}

#[cfg(target_os = "macos")]
fn unregister_launchd() -> Result<(), String> {
    let plist_path = get_launchd_plist_path();

    if plist_path.exists() {
        // Unload the launch agent
        let output = Command::new("launchctl")
            .args(["unload", "-w", plist_path.to_str().unwrap()])
            .output()
            .map_err(|e| format!("Failed to run launchctl: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Ignore "Could not find specified service" error
            if !stderr.contains("Could not find specified service") {
                return Err(format!("launchctl unload failed: {}", stderr));
            }
        }

        // Remove plist file
        fs::remove_file(&plist_path)
            .map_err(|e| format!("Failed to remove plist: {}", e))?;
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn unregister_launchd() -> Result<(), String> {
    Err("launchd is only available on macOS".to_string())
}

#[cfg(target_os = "windows")]
fn is_task_scheduler_registered() -> bool {
    let output = Command::new("schtasks")
        .args(["/Query", "/TN", TASK_NAME])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

#[cfg(not(target_os = "windows"))]
fn is_task_scheduler_registered() -> bool {
    false
}

#[cfg(target_os = "windows")]
fn register_task_scheduler() -> Result<(), String> {
    let daemon_path = get_daemon_path();
    if !daemon_path.exists() {
        return Err(format!("Daemon binary not found at {:?}", daemon_path));
    }

    // Create scheduled task that runs at logon
    let output = Command::new("schtasks")
        .args([
            "/Create",
            "/TN", TASK_NAME,
            "/TR", daemon_path.to_str().unwrap(),
            "/SC", "ONLOGON",
            "/RL", "LIMITED",
            "/F",  // Force overwrite if exists
        ])
        .output()
        .map_err(|e| format!("Failed to run schtasks: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("schtasks create failed: {}", stderr));
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn register_task_scheduler() -> Result<(), String> {
    Err("Task Scheduler is only available on Windows".to_string())
}

#[cfg(target_os = "windows")]
fn unregister_task_scheduler() -> Result<(), String> {
    let output = Command::new("schtasks")
        .args(["/Delete", "/TN", TASK_NAME, "/F"])
        .output()
        .map_err(|e| format!("Failed to run schtasks: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Ignore "does not exist" error
        if !stderr.contains("does not exist") {
            return Err(format!("schtasks delete failed: {}", stderr));
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn unregister_task_scheduler() -> Result<(), String> {
    Err("Task Scheduler is only available on Windows".to_string())
}

#[tauri::command]
pub async fn get_scheduler_status() -> Result<SchedulerStatus, String> {
    #[cfg(target_os = "macos")]
    {
        Ok(SchedulerStatus {
            enabled: is_launchd_registered(),
            platform: "macos".to_string(),
            service_name: LAUNCHD_LABEL.to_string(),
            error: None,
        })
    }

    #[cfg(target_os = "windows")]
    {
        Ok(SchedulerStatus {
            enabled: is_task_scheduler_registered(),
            platform: "windows".to_string(),
            service_name: TASK_NAME.to_string(),
            error: None,
        })
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Ok(SchedulerStatus {
            enabled: false,
            platform: "unsupported".to_string(),
            service_name: "".to_string(),
            error: Some("System scheduler not supported on this platform".to_string()),
        })
    }
}

#[tauri::command]
pub async fn enable_system_scheduler() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        register_launchd()
    }

    #[cfg(target_os = "windows")]
    {
        register_task_scheduler()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err("System scheduler not supported on this platform".to_string())
    }
}

#[tauri::command]
pub async fn disable_system_scheduler() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        unregister_launchd()
    }

    #[cfg(target_os = "windows")]
    {
        unregister_task_scheduler()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err("System scheduler not supported on this platform".to_string())
    }
}
