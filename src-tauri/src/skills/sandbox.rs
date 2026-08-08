use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::process::Command as AsyncCommand;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SandboxBackend {
    Docker,
    MacOSSandbox,
    Firejail,
    Bubblewrap,
    ProcessIsolation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkMode {
    None,
    Allowlist(Vec<String>),
    Full,
}

impl Default for NetworkMode {
    fn default() -> Self {
        NetworkMode::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub timeout_secs: u64,
    pub memory_mb: u64,
    pub network_mode: NetworkMode,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 60,
            memory_mb: 512,
            network_mode: NetworkMode::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFile {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub output_files: Vec<OutputFile>,
    pub duration_ms: u64,
    pub backend_used: SandboxBackend,
}

pub fn detect_backend() -> SandboxBackend {
    // Check for Docker first
    if is_docker_available() {
        return SandboxBackend::Docker;
    }

    // Platform-specific fallbacks
    #[cfg(target_os = "macos")]
    {
        SandboxBackend::MacOSSandbox
    }

    #[cfg(target_os = "linux")]
    {
        if is_firejail_available() {
            SandboxBackend::Firejail
        } else if is_bubblewrap_available() {
            SandboxBackend::Bubblewrap
        } else {
            SandboxBackend::ProcessIsolation
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        SandboxBackend::ProcessIsolation
    }
}

fn is_docker_available() -> bool {
    Command::new("docker")
        .arg("info")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn is_firejail_available() -> bool {
    Command::new("which")
        .arg("firejail")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn is_bubblewrap_available() -> bool {
    Command::new("which")
        .arg("bwrap")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub async fn execute_in_sandbox(
    skill_path: &Path,
    script_name: &str,
    inputs: &serde_json::Value,
    config: &SandboxConfig,
) -> Result<SandboxExecutionResult, String> {
    let backend = detect_backend();
    let start = std::time::Instant::now();

    // Create output directory
    let output_dir = super::sync::get_created_files_dir()?;

    let result = match backend {
        SandboxBackend::Docker => {
            execute_docker(skill_path, script_name, inputs, config, &output_dir).await
        }
        SandboxBackend::MacOSSandbox => {
            execute_macos_sandbox(skill_path, script_name, inputs, config, &output_dir).await
        }
        SandboxBackend::Firejail => {
            execute_firejail(skill_path, script_name, inputs, config, &output_dir).await
        }
        SandboxBackend::Bubblewrap => {
            execute_bubblewrap(skill_path, script_name, inputs, config, &output_dir).await
        }
        SandboxBackend::ProcessIsolation => {
            execute_process_isolation(skill_path, script_name, inputs, config, &output_dir).await
        }
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok((stdout, stderr, exit_code, output_files)) => Ok(SandboxExecutionResult {
            stdout,
            stderr,
            exit_code,
            output_files,
            duration_ms,
            backend_used: backend,
        }),
        Err(e) => Err(e),
    }
}

async fn execute_docker(
    skill_path: &Path,
    script_name: &str,
    inputs: &serde_json::Value,
    config: &SandboxConfig,
    output_dir: &Path,
) -> Result<(String, String, i32, Vec<OutputFile>), String> {
    let script_path = skill_path.join(script_name);
    let extension = script_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let image = match extension {
        "py" => "python:3.11-slim",
        "js" | "ts" => "node:20-slim",
        "sh" => "alpine:latest",
        _ => "alpine:latest",
    };

    let network_arg = match &config.network_mode {
        NetworkMode::None => "--network=none",
        NetworkMode::Allowlist(_) => "--network=bridge", // Would need custom network setup
        NetworkMode::Full => "--network=bridge",
    };

    let mut cmd = AsyncCommand::new("docker");
    cmd.arg("run")
        .arg("--rm")
        .arg(format!("--memory={}m", config.memory_mb))
        .arg("--cpus=1")
        .arg("--pids-limit=10")
        .arg(network_arg)
        .arg("--read-only")
        .arg("--tmpfs=/tmp:size=100m")
        .arg("-v")
        .arg(format!("{}:/skill:ro", skill_path.display()))
        .arg("-v")
        .arg(format!("{}:/output:rw", output_dir.display()))
        .arg("-e")
        .arg(format!("INPUTS={}", inputs.to_string()))
        .arg(image);

    // Add the appropriate command
    match extension {
        "py" => {
            cmd.arg("python").arg(format!("/skill/{}", script_name));
        }
        "js" => {
            cmd.arg("node").arg(format!("/skill/{}", script_name));
        }
        "sh" => {
            cmd.arg("sh").arg(format!("/skill/{}", script_name));
        }
        _ => {
            cmd.arg("sh").arg(format!("/skill/{}", script_name));
        }
    }

    let output = tokio::time::timeout(
        std::time::Duration::from_secs(config.timeout_secs),
        cmd.output(),
    )
    .await
    .map_err(|_| "Script execution timed out")?
    .map_err(|e| format!("Failed to execute Docker: {}", e))?;

    let output_files = collect_output_files(output_dir)?;

    Ok((
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.code().unwrap_or(-1),
        output_files,
    ))
}

#[cfg(target_os = "macos")]
async fn execute_macos_sandbox(
    skill_path: &Path,
    script_name: &str,
    inputs: &serde_json::Value,
    config: &SandboxConfig,
    output_dir: &Path,
) -> Result<(String, String, i32, Vec<OutputFile>), String> {
    let script_path = skill_path.join(script_name);
    let extension = script_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    // Create sandbox profile
    let profile = format!(
        r#"
(version 1)
(deny default)
(allow process-exec)
(allow file-read* (subpath "{}"))
(allow file-read* (subpath "/usr"))
(allow file-read* (subpath "/System"))
(allow file-read* (subpath "/Library"))
(allow file-read* (subpath "/private/var"))
(allow file-write* (subpath "{}"))
(allow file-write* (subpath "/private/var/folders"))
{}
"#,
        skill_path.display(),
        output_dir.display(),
        if matches!(config.network_mode, NetworkMode::None) {
            "(deny network*)"
        } else {
            "(allow network*)"
        }
    );

    let profile_path = std::env::temp_dir().join(format!("meridian_sandbox_{}.sb", uuid::Uuid::new_v4()));
    std::fs::write(&profile_path, &profile)
        .map_err(|e| format!("Failed to write sandbox profile: {}", e))?;

    let interpreter = match extension {
        "py" => "python3",
        "js" => "node",
        "sh" => "sh",
        _ => "sh",
    };

    let mut cmd = AsyncCommand::new("sandbox-exec");
    cmd.arg("-f")
        .arg(&profile_path)
        .arg(interpreter)
        .arg(&script_path)
        .env("INPUTS", inputs.to_string())
        .env("OUTPUT_DIR", output_dir);

    let output = tokio::time::timeout(
        std::time::Duration::from_secs(config.timeout_secs),
        cmd.output(),
    )
    .await
    .map_err(|_| "Script execution timed out")?
    .map_err(|e| format!("Failed to execute sandbox-exec: {}", e))?;

    // Cleanup profile
    let _ = std::fs::remove_file(&profile_path);

    let output_files = collect_output_files(output_dir)?;

    Ok((
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.code().unwrap_or(-1),
        output_files,
    ))
}

#[cfg(not(target_os = "macos"))]
async fn execute_macos_sandbox(
    _skill_path: &Path,
    _script_name: &str,
    _inputs: &serde_json::Value,
    _config: &SandboxConfig,
    _output_dir: &Path,
) -> Result<(String, String, i32, Vec<OutputFile>), String> {
    Err("macOS sandbox not available on this platform".to_string())
}

async fn execute_firejail(
    skill_path: &Path,
    script_name: &str,
    inputs: &serde_json::Value,
    config: &SandboxConfig,
    output_dir: &Path,
) -> Result<(String, String, i32, Vec<OutputFile>), String> {
    let script_path = skill_path.join(script_name);
    let extension = script_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let interpreter = match extension {
        "py" => "python3",
        "js" => "node",
        "sh" => "sh",
        _ => "sh",
    };

    let mut cmd = AsyncCommand::new("firejail");
    cmd.arg("--quiet")
        .arg(format!("--whitelist={}", skill_path.display()))
        .arg(format!("--whitelist={}", output_dir.display()))
        .arg("--read-only=/")
        .arg(format!("--read-write={}", output_dir.display()));

    if matches!(config.network_mode, NetworkMode::None) {
        cmd.arg("--net=none");
    }

    cmd.arg("--")
        .arg(interpreter)
        .arg(&script_path)
        .env("INPUTS", inputs.to_string())
        .env("OUTPUT_DIR", output_dir);

    let output = tokio::time::timeout(
        std::time::Duration::from_secs(config.timeout_secs),
        cmd.output(),
    )
    .await
    .map_err(|_| "Script execution timed out")?
    .map_err(|e| format!("Failed to execute firejail: {}", e))?;

    let output_files = collect_output_files(output_dir)?;

    Ok((
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.code().unwrap_or(-1),
        output_files,
    ))
}

async fn execute_bubblewrap(
    skill_path: &Path,
    script_name: &str,
    inputs: &serde_json::Value,
    config: &SandboxConfig,
    output_dir: &Path,
) -> Result<(String, String, i32, Vec<OutputFile>), String> {
    let script_path = skill_path.join(script_name);
    let extension = script_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let interpreter = match extension {
        "py" => "python3",
        "js" => "node",
        "sh" => "sh",
        _ => "sh",
    };

    let mut cmd = AsyncCommand::new("bwrap");
    cmd.arg("--ro-bind").arg("/usr").arg("/usr")
        .arg("--ro-bind").arg("/lib").arg("/lib")
        .arg("--ro-bind").arg("/lib64").arg("/lib64")
        .arg("--ro-bind").arg(skill_path).arg("/skill")
        .arg("--bind").arg(output_dir).arg("/output")
        .arg("--tmpfs").arg("/tmp")
        .arg("--proc").arg("/proc")
        .arg("--dev").arg("/dev");

    if matches!(config.network_mode, NetworkMode::None) {
        cmd.arg("--unshare-net");
    }

    cmd.arg("--")
        .arg(interpreter)
        .arg(format!("/skill/{}", script_name))
        .env("INPUTS", inputs.to_string())
        .env("OUTPUT_DIR", "/output");

    let output = tokio::time::timeout(
        std::time::Duration::from_secs(config.timeout_secs),
        cmd.output(),
    )
    .await
    .map_err(|_| "Script execution timed out")?
    .map_err(|e| format!("Failed to execute bwrap: {}", e))?;

    let output_files = collect_output_files(output_dir)?;

    Ok((
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.code().unwrap_or(-1),
        output_files,
    ))
}

async fn execute_process_isolation(
    skill_path: &Path,
    script_name: &str,
    inputs: &serde_json::Value,
    config: &SandboxConfig,
    output_dir: &Path,
) -> Result<(String, String, i32, Vec<OutputFile>), String> {
    let script_path = skill_path.join(script_name);
    let extension = script_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let interpreter = match extension {
        "py" => "python3",
        "js" => "node",
        "sh" => "sh",
        _ => "sh",
    };

    // Basic process isolation - restricted environment
    let mut cmd = AsyncCommand::new(interpreter);
    cmd.arg(&script_path)
        .current_dir(skill_path)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", "/tmp")
        .env("INPUTS", inputs.to_string())
        .env("OUTPUT_DIR", output_dir);

    let output = tokio::time::timeout(
        std::time::Duration::from_secs(config.timeout_secs),
        cmd.output(),
    )
    .await
    .map_err(|_| "Script execution timed out")?
    .map_err(|e| format!("Failed to execute script: {}", e))?;

    let output_files = collect_output_files(output_dir)?;

    Ok((
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.code().unwrap_or(-1),
        output_files,
    ))
}

fn collect_output_files(output_dir: &Path) -> Result<Vec<OutputFile>, String> {
    let mut files = Vec::new();

    if !output_dir.exists() {
        return Ok(files);
    }

    for entry in std::fs::read_dir(output_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_file() {
            let metadata = std::fs::metadata(&path).map_err(|e| e.to_string())?;
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let mime_type = guess_mime_type(&path);

            files.push(OutputFile {
                name,
                path,
                size: metadata.len(),
                mime_type,
            });
        }
    }

    Ok(files)
}

fn guess_mime_type(path: &Path) -> Option<String> {
    let extension = path.extension()?.to_str()?;
    let mime = match extension.to_lowercase().as_str() {
        "txt" => "text/plain",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        "csv" => "text/csv",
        "md" => "text/markdown",
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        _ => return None,
    };
    Some(mime.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_backend() {
        // Just ensure it doesn't panic
        let backend = detect_backend();
        println!("Detected backend: {:?}", backend);
    }

    #[test]
    fn test_default_config() {
        let config = SandboxConfig::default();
        assert_eq!(config.timeout_secs, 60);
        assert_eq!(config.memory_mb, 512);
        assert!(matches!(config.network_mode, NetworkMode::None));
    }

    #[test]
    fn test_network_mode_default() {
        let mode = NetworkMode::default();
        assert!(matches!(mode, NetworkMode::None));
    }

    #[test]
    fn test_guess_mime_type() {
        assert_eq!(guess_mime_type(Path::new("test.json")), Some("application/json".to_string()));
        assert_eq!(guess_mime_type(Path::new("test.png")), Some("image/png".to_string()));
        assert_eq!(guess_mime_type(Path::new("test.md")), Some("text/markdown".to_string()));
        assert_eq!(guess_mime_type(Path::new("test.unknown")), None);
        assert_eq!(guess_mime_type(Path::new("noextension")), None);
    }

    #[test]
    fn test_output_file_struct() {
        let file = OutputFile {
            name: "test.txt".to_string(),
            path: PathBuf::from("/tmp/test.txt"),
            size: 1024,
            mime_type: Some("text/plain".to_string()),
        };
        assert_eq!(file.name, "test.txt");
        assert_eq!(file.size, 1024);
    }

    #[test]
    fn test_sandbox_execution_result() {
        let result = SandboxExecutionResult {
            stdout: "output".to_string(),
            stderr: "".to_string(),
            exit_code: 0,
            output_files: vec![],
            duration_ms: 100,
            backend_used: SandboxBackend::ProcessIsolation,
        };
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.duration_ms, 100);
    }

    #[test]
    fn test_network_mode_allowlist() {
        let mode = NetworkMode::Allowlist(vec![
            "api.github.com".to_string(),
            "api.openai.com".to_string(),
        ]);
        if let NetworkMode::Allowlist(hosts) = mode {
            assert_eq!(hosts.len(), 2);
            assert!(hosts.contains(&"api.github.com".to_string()));
        } else {
            panic!("Expected Allowlist mode");
        }
    }

    #[test]
    fn test_network_mode_from_trust_state() {
        // Test conversion from skill trust state strings
        let none_mode = NetworkMode::None;
        let full_mode = NetworkMode::Full;

        assert!(matches!(none_mode, NetworkMode::None));
        assert!(matches!(full_mode, NetworkMode::Full));
    }

    #[test]
    fn test_sandbox_config_with_network() {
        let config = SandboxConfig {
            timeout_secs: 30,
            memory_mb: 256,
            network_mode: NetworkMode::Allowlist(vec!["example.com".to_string()]),
        };
        assert_eq!(config.timeout_secs, 30);
        assert!(matches!(config.network_mode, NetworkMode::Allowlist(_)));
    }
}
