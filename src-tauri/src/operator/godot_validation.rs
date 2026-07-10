use serde::{Deserialize, Serialize};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::operator::{d_drive_path_from_env, is_d_drive_path, workspace_root};

const DEFAULT_TIMEOUT_SECONDS: u64 = 120;
const MAX_TIMEOUT_SECONDS: u64 = 15 * 60;
const OUTPUT_LIMIT_BYTES: usize = 64 * 1024;
const POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GodotValidationStatus {
    Passed,
    Failed,
    TimedOut,
    Cancelled,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodotValidationResult {
    pub status: GodotValidationStatus,
    pub executable: Option<String>,
    pub args: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub output_truncated: bool,
    pub duration_ms: u128,
    pub message: String,
}

impl GodotValidationResult {
    pub fn skipped(reason: impl Into<String>) -> Self {
        Self {
            status: GodotValidationStatus::Skipped,
            executable: None,
            args: Vec::new(),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            output_truncated: false,
            duration_ms: 0,
            message: reason.into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GodotValidationError {
    #[error("invalid Godot validation input: {0}")]
    InvalidInput(String),
    #[error("failed to start Godot validation: {0}")]
    Spawn(String),
    #[error("failed while waiting for Godot validation: {0}")]
    Wait(String),
    #[error("failed to capture Godot validation output: {0}")]
    Capture(String),
}

pub fn discover_godot_executable() -> Option<PathBuf> {
    d_drive_path_from_env("GODOT_BIN")
        .or_else(|| d_drive_path_from_env("GODOT4_BIN"))
        .filter(|path| path.is_file())
        .or_else(|| {
            let root = workspace_root();
            [
                root.join("bin").join("godot.exe"),
                root.join("bin").join("godot4.exe"),
                root.join("bin").join("Godot.exe"),
                PathBuf::from("D:/dev-tools/godot/godot.exe"),
                PathBuf::from("D:/dev-tools/godot/Godot.exe"),
                PathBuf::from("D:/Godot/godot.exe"),
                PathBuf::from("D:/Godot/Godot.exe"),
            ]
            .into_iter()
            .find(|path| is_d_drive_path(path) && path.is_file())
        })
        .or_else(|| find_godot_executable_in(Path::new("D:/dev-tools/godot")))
        .or_else(|| find_godot_executable_in(Path::new("D:/Godot")))
}

fn find_godot_executable_in(directory: &Path) -> Option<PathBuf> {
    if !is_d_drive_path(directory) || !directory.is_dir() {
        return None;
    }
    let mut candidates = fs::read_dir(directory)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .map(|extension| extension.eq_ignore_ascii_case("exe"))
                    .unwrap_or(false)
                && path
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .map(|name| name.to_ascii_lowercase().starts_with("godot"))
                    .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.into_iter().next()
}

pub fn run_godot_validation<F>(
    executable: &Path,
    project_root: &Path,
    should_cancel: F,
) -> Result<GodotValidationResult, GodotValidationError>
where
    F: FnMut() -> bool,
{
    run_godot_validation_with_timeout(
        executable,
        project_root,
        validation_timeout(),
        should_cancel,
    )
}

fn run_godot_validation_with_timeout<F>(
    executable: &Path,
    project_root: &Path,
    timeout: Duration,
    should_cancel: F,
) -> Result<GodotValidationResult, GodotValidationError>
where
    F: FnMut() -> bool,
{
    let executable = canonical_d_file(executable, "Godot executable")?;
    let project_root = canonical_d_directory(project_root, "Godot project")?;
    if !project_root.join("project.godot").is_file() {
        return Err(GodotValidationError::InvalidInput(format!(
            "project.godot was not found under {}",
            project_root.display()
        )));
    }

    let args = godot_validation_args(&project_root);
    let mut command = Command::new(&executable);
    command.args(&args);
    apply_d_drive_godot_env(&mut command, &project_root);
    run_bounded_command(command, &executable, &args, timeout, should_cancel)
}

fn godot_validation_args(project_root: &Path) -> Vec<OsString> {
    vec![
        OsString::from("--headless"),
        OsString::from("--editor"),
        OsString::from("--path"),
        project_root.as_os_str().to_os_string(),
        OsString::from("--quit-after"),
        OsString::from("1"),
    ]
}

fn canonical_d_file(path: &Path, label: &str) -> Result<PathBuf, GodotValidationError> {
    if !is_d_drive_path(path) {
        return Err(GodotValidationError::InvalidInput(format!(
            "{} must be on D: {}",
            label,
            path.display()
        )));
    }
    let canonical = path.canonicalize().map_err(|error| {
        GodotValidationError::InvalidInput(format!(
            "{} cannot be canonicalized ({}): {}",
            label,
            path.display(),
            error
        ))
    })?;
    if !is_d_drive_path(&canonical) || !canonical.is_file() {
        return Err(GodotValidationError::InvalidInput(format!(
            "{} must resolve to a D: file: {}",
            label,
            canonical.display()
        )));
    }
    Ok(canonical)
}

fn canonical_d_directory(path: &Path, label: &str) -> Result<PathBuf, GodotValidationError> {
    if !is_d_drive_path(path) {
        return Err(GodotValidationError::InvalidInput(format!(
            "{} must be on D: {}",
            label,
            path.display()
        )));
    }
    let canonical = path.canonicalize().map_err(|error| {
        GodotValidationError::InvalidInput(format!(
            "{} cannot be canonicalized ({}): {}",
            label,
            path.display(),
            error
        ))
    })?;
    if !is_d_drive_path(&canonical) || !canonical.is_dir() {
        return Err(GodotValidationError::InvalidInput(format!(
            "{} must resolve to a D: directory: {}",
            label,
            canonical.display()
        )));
    }
    Ok(canonical)
}

fn apply_d_drive_godot_env(command: &mut Command, project_root: &Path) {
    let runtime_root = workspace_root()
        .join(".operator")
        .join("runtime")
        .join("godot");
    let temp = runtime_root.join("tmp");
    let data = runtime_root.join("data");
    let config = runtime_root.join("config");
    let cache = runtime_root.join("cache");
    let appdata = runtime_root.join("appdata");
    let local_appdata = runtime_root.join("localappdata");
    for directory in [
        &runtime_root,
        &temp,
        &data,
        &config,
        &cache,
        &appdata,
        &local_appdata,
    ] {
        let _ = fs::create_dir_all(directory);
    }

    command
        .current_dir(project_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("TEMP", &temp)
        .env("TMP", &temp)
        .env("HOME", &runtime_root)
        .env("USERPROFILE", &runtime_root)
        .env("APPDATA", &appdata)
        .env("LOCALAPPDATA", &local_appdata)
        .env("XDG_DATA_HOME", &data)
        .env("XDG_CONFIG_HOME", &config)
        .env("XDG_CACHE_HOME", &cache);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
}

fn run_bounded_command<F>(
    mut command: Command,
    executable: &Path,
    args: &[OsString],
    timeout: Duration,
    mut should_cancel: F,
) -> Result<GodotValidationResult, GodotValidationError>
where
    F: FnMut() -> bool,
{
    let started = Instant::now();
    let mut child = command
        .spawn()
        .map_err(|error| GodotValidationError::Spawn(error.to_string()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| GodotValidationError::Capture("stdout was not piped".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| GodotValidationError::Capture("stderr was not piped".to_string()))?;
    let stdout_reader = thread::spawn(move || read_bounded(stdout, OUTPUT_LIMIT_BYTES));
    let stderr_reader = thread::spawn(move || read_bounded(stderr, OUTPUT_LIMIT_BYTES));

    let (status, process_status) = loop {
        if should_cancel() {
            let _ = child.kill();
            let status = child
                .wait()
                .map_err(|error| GodotValidationError::Wait(error.to_string()))?;
            break (GodotValidationStatus::Cancelled, Some(status));
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let status = child
                .wait()
                .map_err(|error| GodotValidationError::Wait(error.to_string()))?;
            break (GodotValidationStatus::TimedOut, Some(status));
        }
        match child
            .try_wait()
            .map_err(|error| GodotValidationError::Wait(error.to_string()))?
        {
            Some(status) => break (classify_exit_status(&status), Some(status)),
            None => thread::sleep(POLL_INTERVAL),
        }
    };

    let stdout = stdout_reader
        .join()
        .map_err(|_| GodotValidationError::Capture("stdout reader panicked".to_string()))?
        .map_err(|error| GodotValidationError::Capture(error.to_string()))?;
    let stderr = stderr_reader
        .join()
        .map_err(|_| GodotValidationError::Capture("stderr reader panicked".to_string()))?
        .map_err(|error| GodotValidationError::Capture(error.to_string()))?;
    let stdout_text = String::from_utf8_lossy(&stdout.bytes).into_owned();
    let stderr_text = String::from_utf8_lossy(&stderr.bytes).into_owned();
    let status = if status == GodotValidationStatus::Passed
        && contains_script_diagnostic(&stdout_text, &stderr_text)
    {
        GodotValidationStatus::Failed
    } else {
        status
    };
    let exit_code = process_status.and_then(|status| status.code());
    let message = validation_message(&status, exit_code, timeout);

    Ok(GodotValidationResult {
        status,
        executable: Some(executable.to_string_lossy().to_string()),
        args: args
            .iter()
            .map(|argument| argument.to_string_lossy().to_string())
            .collect(),
        exit_code,
        stdout: stdout_text,
        stderr: stderr_text,
        output_truncated: stdout.truncated || stderr.truncated,
        duration_ms: started.elapsed().as_millis(),
        message,
    })
}

fn classify_exit_status(status: &ExitStatus) -> GodotValidationStatus {
    if status.success() {
        GodotValidationStatus::Passed
    } else {
        GodotValidationStatus::Failed
    }
}

fn contains_script_diagnostic(stdout: &str, stderr: &str) -> bool {
    let combined = format!("{}\n{}", stdout, stderr).to_ascii_lowercase();
    [
        "script error:",
        "parse error:",
        "error at res://",
        "failed to load script",
    ]
    .iter()
    .any(|marker| combined.contains(marker))
}

fn validation_message(
    status: &GodotValidationStatus,
    exit_code: Option<i32>,
    timeout: Duration,
) -> String {
    match status {
        GodotValidationStatus::Passed => "Godot headless validation passed".to_string(),
        GodotValidationStatus::Failed => format!(
            "Godot headless validation failed with exit code {:?}",
            exit_code
        ),
        GodotValidationStatus::TimedOut => format!(
            "Godot headless validation timed out after {} seconds",
            timeout.as_secs()
        ),
        GodotValidationStatus::Cancelled => {
            "Godot headless validation was cancelled by operator state".to_string()
        }
        GodotValidationStatus::Skipped => "Godot headless validation was skipped".to_string(),
    }
}

fn read_bounded<R: Read>(mut reader: R, limit: usize) -> io::Result<CapturedOutput> {
    let mut captured = CapturedOutput::default();
    let mut buffer = [0u8; 8192];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let remaining = limit.saturating_sub(captured.bytes.len());
        let keep = read.min(remaining);
        captured.bytes.extend_from_slice(&buffer[..keep]);
        if keep < read {
            captured.truncated = true;
        }
    }
    Ok(captured)
}

#[derive(Default)]
struct CapturedOutput {
    bytes: Vec<u8>,
    truncated: bool,
}

fn validation_timeout() -> Duration {
    let seconds = env::var("GODOT_VALIDATION_TIMEOUT_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|seconds| *seconds > 0)
        .unwrap_or(DEFAULT_TIMEOUT_SECONDS)
        .min(MAX_TIMEOUT_SECONDS);
    Duration::from_secs(seconds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    const CHILD_TEST: &str = "operator::godot_validation::tests::validation_child_entrypoint";

    #[test]
    fn validation_child_entrypoint() {
        let Ok(mode) = env::var("ACP_GODOT_VALIDATION_CHILD_MODE") else {
            return;
        };
        match mode.as_str() {
            "success" => println!("Godot Engine v4 test validation passed"),
            "script_error" => eprintln!("SCRIPT ERROR: Parse error at res://scripts/player.gd"),
            "nonzero" => std::process::exit(7),
            "sleep" => thread::sleep(Duration::from_secs(10)),
            "flood" => println!("{}", "x".repeat(OUTPUT_LIMIT_BYTES * 4)),
            other => panic!("unknown child mode: {}", other),
        }
    }

    fn child_command(mode: &str) -> (Command, PathBuf, Vec<OsString>) {
        let executable = env::current_exe().expect("current test executable");
        assert!(is_d_drive_path(&executable));
        let args = vec![
            OsString::from("--exact"),
            OsString::from(CHILD_TEST),
            OsString::from("--nocapture"),
            OsString::from("--test-threads=1"),
        ];
        let mut command = Command::new(&executable);
        command
            .args(&args)
            .env("ACP_GODOT_VALIDATION_CHILD_MODE", mode)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        (command, executable, args)
    }

    #[test]
    fn bounded_process_reports_success() {
        let (command, executable, args) = child_command("success");
        let result =
            run_bounded_command(command, &executable, &args, Duration::from_secs(5), || {
                false
            })
            .expect("run child");

        assert_eq!(result.status, GodotValidationStatus::Passed);
        assert_eq!(result.exit_code, Some(0));
        assert!(result.stdout.contains("validation passed"));
    }

    #[test]
    fn bounded_process_reports_nonzero_exit() {
        let (command, executable, args) = child_command("nonzero");
        let result =
            run_bounded_command(command, &executable, &args, Duration::from_secs(5), || {
                false
            })
            .expect("run child");

        assert_eq!(result.status, GodotValidationStatus::Failed);
        assert_eq!(result.exit_code, Some(7));
    }

    #[test]
    fn script_diagnostics_fail_even_with_zero_exit() {
        let (command, executable, args) = child_command("script_error");
        let result =
            run_bounded_command(command, &executable, &args, Duration::from_secs(5), || {
                false
            })
            .expect("run child");

        assert_eq!(result.exit_code, Some(0));
        assert_eq!(result.status, GodotValidationStatus::Failed);
        assert!(result.stderr.contains("SCRIPT ERROR"));
    }

    #[test]
    fn bounded_process_times_out_and_is_reaped() {
        let (command, executable, args) = child_command("sleep");
        let started = Instant::now();
        let result = run_bounded_command(
            command,
            &executable,
            &args,
            Duration::from_millis(150),
            || false,
        )
        .expect("run child");

        assert_eq!(result.status, GodotValidationStatus::TimedOut);
        assert!(started.elapsed() < Duration::from_secs(3));
    }

    #[test]
    fn bounded_process_honors_cancellation() {
        let (command, executable, args) = child_command("sleep");
        let cancel = Arc::new(AtomicBool::new(false));
        let setter = cancel.clone();
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(120));
            setter.store(true, Ordering::SeqCst);
        });
        let result =
            run_bounded_command(command, &executable, &args, Duration::from_secs(5), || {
                cancel.load(Ordering::SeqCst)
            })
            .expect("run child");
        handle.join().expect("cancel setter");

        assert_eq!(result.status, GodotValidationStatus::Cancelled);
    }

    #[test]
    fn bounded_process_truncates_but_drains_output() {
        let (command, executable, args) = child_command("flood");
        let result =
            run_bounded_command(command, &executable, &args, Duration::from_secs(5), || {
                false
            })
            .expect("run child");

        assert_eq!(result.status, GodotValidationStatus::Passed);
        assert!(result.output_truncated);
        assert!(result.stdout.len() <= OUTPUT_LIMIT_BYTES);
    }

    #[test]
    fn godot_arguments_are_fixed_and_do_not_include_model_validation_text() {
        let project = Path::new("D:/games/example");
        let args = godot_validation_args(project)
            .into_iter()
            .map(|argument| argument.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert_eq!(
            args,
            vec![
                "--headless",
                "--editor",
                "--path",
                "D:/games/example",
                "--quit-after",
                "1"
            ]
        );
    }

    #[test]
    fn skipped_result_never_claims_validation_passed() {
        let result = GodotValidationResult::skipped("Godot was not found on D:");

        assert_eq!(result.status, GodotValidationStatus::Skipped);
        assert!(!result.message.to_ascii_lowercase().contains("passed"));
    }

    #[test]
    #[ignore = "requires ACP_REAL_GODOT_BIN and ACP_REAL_GODOT_PROJECT on D:"]
    fn validates_external_real_godot_project() {
        let executable =
            PathBuf::from(env::var("ACP_REAL_GODOT_BIN").expect("ACP_REAL_GODOT_BIN must be set"));
        let project = PathBuf::from(
            env::var("ACP_REAL_GODOT_PROJECT").expect("ACP_REAL_GODOT_PROJECT must be set"),
        );

        let result = run_godot_validation_with_timeout(
            &executable,
            &project,
            Duration::from_secs(60),
            || false,
        )
        .expect("run real Godot validation");
        eprintln!("Godot stdout:\n{}", result.stdout);
        eprintln!("Godot stderr:\n{}", result.stderr);

        assert_eq!(result.status, GodotValidationStatus::Passed);
        assert!(!result.output_truncated);
    }
}
