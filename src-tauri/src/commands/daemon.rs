use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub jobs_processed: Option<u64>,
    pub uptime_seconds: Option<i64>,
    pub last_error: Option<String>,
}

fn get_data_dir() -> PathBuf {
    dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("meridian")
}

fn get_socket_path() -> PathBuf {
    get_data_dir().join("daemon.sock")
}

fn get_pid_path() -> PathBuf {
    get_data_dir().join("daemon.pid")
}

fn read_pid() -> Option<u32> {
    let pid_path = get_pid_path();
    if pid_path.exists() {
        std::fs::read_to_string(&pid_path)
            .ok()
            .and_then(|s| s.trim().parse().ok())
    } else {
        None
    }
}

fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
    #[cfg(not(unix))]
    {
        // On Windows, we'd need to use OpenProcess
        false
    }
}

#[cfg(unix)]
fn send_ipc_request(request: &str) -> Result<serde_json::Value, String> {
    let socket_path = get_socket_path();

    let mut stream = UnixStream::connect(&socket_path)
        .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

    stream.set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;

    writeln!(stream, "{}", request)
        .map_err(|e| format!("Failed to send request: {}", e))?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response)
        .map_err(|e| format!("Failed to read response: {}", e))?;

    serde_json::from_str(&response)
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[cfg(not(unix))]
fn send_ipc_request(_request: &str) -> Result<serde_json::Value, String> {
    Err("IPC not supported on this platform".to_string())
}

#[tauri::command]
pub async fn get_daemon_status() -> Result<DaemonStatus, String> {
    let pid = read_pid();

    // Check if process is running
    let running = pid.map(is_process_running).unwrap_or(false);

    if !running {
        return Ok(DaemonStatus {
            running: false,
            pid: None,
            jobs_processed: None,
            uptime_seconds: None,
            last_error: None,
        });
    }

    // Try to get status via IPC
    match send_ipc_request(r#"{"type":"status"}"#) {
        Ok(response) => {
            Ok(DaemonStatus {
                running: true,
                pid,
                jobs_processed: response["data"]["jobs_processed"].as_u64(),
                uptime_seconds: response["data"]["uptime_seconds"].as_i64(),
                last_error: response["data"]["last_error"].as_str().map(String::from),
            })
        }
        Err(e) => {
            Ok(DaemonStatus {
                running: true,
                pid,
                jobs_processed: None,
                uptime_seconds: None,
                last_error: Some(format!("IPC error: {}", e)),
            })
        }
    }
}

#[tauri::command]
pub async fn start_daemon() -> Result<DaemonStatus, String> {
    // Check if already running
    if let Some(pid) = read_pid() {
        if is_process_running(pid) {
            return Err("Daemon is already running".to_string());
        }
    }

    // Find the daemon binary
    let daemon_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get current exe: {}", e))?
        .parent()
        .ok_or("Failed to get exe directory")?
        .join("meridian-daemon");

    if !daemon_path.exists() {
        return Err(format!("Daemon binary not found at {:?}", daemon_path));
    }

    // Start the daemon
    let _child = Command::new(&daemon_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start daemon: {}", e))?;

    // Wait a bit for it to start
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Return current status
    get_daemon_status().await
}

#[tauri::command]
pub async fn stop_daemon() -> Result<(), String> {
    // Try graceful shutdown via IPC
    match send_ipc_request(r#"{"type":"shutdown"}"#) {
        Ok(_) => {
            // Wait for shutdown
            for _ in 0..10 {
                std::thread::sleep(std::time::Duration::from_millis(200));
                if let Some(pid) = read_pid() {
                    if !is_process_running(pid) {
                        return Ok(());
                    }
                } else {
                    return Ok(());
                }
            }
            Err("Daemon did not stop in time".to_string())
        }
        Err(e) => {
            // If IPC failed, try to kill the process directly
            if let Some(pid) = read_pid() {
                #[cfg(unix)]
                {
                    unsafe { libc::kill(pid as i32, libc::SIGTERM); }
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    if is_process_running(pid) {
                        unsafe { libc::kill(pid as i32, libc::SIGKILL); }
                    }
                }
                Ok(())
            } else {
                Err(format!("Failed to stop daemon: {}", e))
            }
        }
    }
}

#[tauri::command]
pub async fn daemon_health_check() -> Result<bool, String> {
    match send_ipc_request(r#"{"type":"health"}"#) {
        Ok(response) => {
            Ok(response["data"]["healthy"].as_bool().unwrap_or(false))
        }
        Err(_) => Ok(false),
    }
}
