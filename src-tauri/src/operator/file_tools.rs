// File Tools - Safe file operations for Hermes Game Operator

use crate::operator::security::PathGuard;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// ============================================================================
// File Read Tool
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadResult {
    pub path: String,
    pub content: String,
    pub size_bytes: u64,
    pub line_count: usize,
}

/// Safely read a file with path validation
pub fn file_read(path: &Path, path_guard: &PathGuard) -> Result<FileReadResult, FileToolError> {
    // Validate path
    path_guard
        .validate_file(path)
        .map_err(|e| FileToolError::PathError(e.to_string()))?;

    // Read file
    let content = fs::read_to_string(path).map_err(|e| FileToolError::IoError(e.to_string()))?;

    let size_bytes = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    let line_count = content.lines().count();

    Ok(FileReadResult {
        path: path.to_string_lossy().to_string(),
        content,
        size_bytes,
        line_count,
    })
}

// ============================================================================
// File Patch Tool
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePatch {
    pub path: String,
    pub original_content: String,
    pub new_content: String,
    pub diff: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePatchResult {
    pub path: String,
    pub success: bool,
    pub lines_changed: usize,
    pub backup_path: Option<String>,
}

/// Generate a diff between original and new content
fn generate_diff(original: &str, new: &str) -> String {
    let mut diff = String::new();

    let orig_lines: Vec<&str> = original.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    // Simple line-by-line diff (production would use a proper diff algorithm)
    let max_lines = orig_lines.len().max(new_lines.len());

    for i in 0..max_lines {
        let orig_line = orig_lines.get(i).copied().unwrap_or("");
        let new_line = new_lines.get(i).copied().unwrap_or("");

        if orig_line != new_line {
            if i < orig_lines.len() {
                diff.push_str(&format!("-{}\n", orig_line));
            }
            if i < new_lines.len() {
                diff.push_str(&format!("+{}\n", new_line));
            }
        } else {
            diff.push_str(&format!(" {}\n", orig_line));
        }
    }

    diff
}

/// Apply a patch to a file with backup
pub fn file_patch(
    path: &Path,
    new_content: &str,
    path_guard: &PathGuard,
    create_backup: bool,
) -> Result<FilePatchResult, FileToolError> {
    // Validate path
    path_guard
        .validate_write(path)
        .map_err(|e| FileToolError::PathError(e.to_string()))?;

    // Read original content
    let original_content =
        fs::read_to_string(path).map_err(|e| FileToolError::IoError(e.to_string()))?;

    // Generate backup if requested
    let backup_path = if create_backup {
        let backup = path.with_extension(format!(
            "{}.bak",
            path.extension().and_then(|e| e.to_str()).unwrap_or("txt")
        ));
        fs::copy(path, &backup).map_err(|e| FileToolError::IoError(e.to_string()))?;
        Some(backup.to_string_lossy().to_string())
    } else {
        None
    };

    // Write new content
    fs::write(path, new_content).map_err(|e| FileToolError::IoError(e.to_string()))?;

    // Calculate lines changed
    let orig_lines: Vec<&str> = original_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();
    let lines_changed = orig_lines
        .iter()
        .zip(new_lines.iter())
        .filter(|(a, b)| a != b)
        .count()
        + orig_lines.len().abs_diff(new_lines.len());

    Ok(FilePatchResult {
        path: path.to_string_lossy().to_string(),
        success: true,
        lines_changed,
        backup_path,
    })
}

/// Generate a patch without applying it
pub fn file_patch_preview(
    path: &Path,
    new_content: &str,
    path_guard: &PathGuard,
) -> Result<FilePatch, FileToolError> {
    // Validate path
    path_guard
        .validate_file(path)
        .map_err(|e| FileToolError::PathError(e.to_string()))?;

    // Read original content
    let original_content =
        fs::read_to_string(path).map_err(|e| FileToolError::IoError(e.to_string()))?;

    // Generate diff
    let diff = generate_diff(&original_content, new_content);

    Ok(FilePatch {
        path: path.to_string_lossy().to_string(),
        original_content,
        new_content: new_content.to_string(),
        diff,
    })
}

// ============================================================================
// File List Tool
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileListResult {
    pub path: String,
    pub files: Vec<String>,
    pub directories: Vec<String>,
}

/// List files and directories in a path
pub fn file_list(path: &Path, path_guard: &PathGuard) -> Result<FileListResult, FileToolError> {
    // Validate path
    path_guard
        .validate_directory(path)
        .map_err(|e| FileToolError::PathError(e.to_string()))?;

    let mut files = Vec::new();
    let mut directories = Vec::new();

    for entry in fs::read_dir(path).map_err(|e| FileToolError::IoError(e.to_string()))? {
        let entry = entry.map_err(|e| FileToolError::IoError(e.to_string()))?;
        let entry_path = entry.path();
        let name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if entry_path.is_file() {
            files.push(name);
        } else if entry_path.is_dir() {
            directories.push(name);
        }
    }

    files.sort();
    directories.sort();

    Ok(FileListResult {
        path: path.to_string_lossy().to_string(),
        files,
        directories,
    })
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug)]
pub enum FileToolError {
    PathError(String),
    IoError(String),
    PermissionDenied(String),
}

impl std::fmt::Display for FileToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileToolError::PathError(msg) => write!(f, "Path error: {}", msg),
            FileToolError::IoError(msg) => write!(f, "IO error: {}", msg),
            FileToolError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
        }
    }
}

impl std::error::Error for FileToolError {}
