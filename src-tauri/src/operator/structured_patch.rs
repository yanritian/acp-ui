use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::operator::PathGuard;

const PATCH_SCHEMA_VERSION: u32 = 1;
const MAX_CHANGES: usize = 64;
const MAX_FILE_BYTES: usize = 256 * 1024;
const MAX_TOTAL_CONTENT_BYTES: usize = 2 * 1024 * 1024;
const MAX_SUMMARY_BYTES: usize = 4 * 1024;
const MAX_VALIDATION_ITEMS: usize = 32;
const MAX_VALIDATION_ITEM_BYTES: usize = 2 * 1024;
const MAX_RELATIVE_PATH_BYTES: usize = 512;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PatchOperation {
    Create,
    Replace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuredFileChange {
    pub path: String,
    pub operation: PatchOperation,
    pub content: String,
    #[serde(default)]
    pub expected_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuredPatchSet {
    pub version: u32,
    pub summary: String,
    pub changes: Vec<StructuredFileChange>,
    #[serde(default)]
    pub validation: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PreparedFileChange {
    pub path: String,
    pub operation: PatchOperation,
    pub target_path: PathBuf,
    pub new_content: String,
    pub original_sha256: Option<String>,
    pub new_sha256: String,
    pub diff: String,
    pub lines_changed: usize,
}

#[derive(Debug, Clone)]
pub struct PreparedPatchSet {
    pub patch_id: String,
    pub summary: String,
    pub validation: Vec<String>,
    pub project_root: PathBuf,
    pub changes: Vec<PreparedFileChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedFileChange {
    pub path: String,
    pub operation: PatchOperation,
    pub backup_path: Option<String>,
    pub lines_changed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedPatchSet {
    pub patch_id: String,
    pub backup_root: String,
    pub files: Vec<AppliedFileChange>,
}

#[derive(Debug, thiserror::Error)]
pub enum StructuredPatchError {
    #[error("invalid structured patch JSON: {0}")]
    InvalidJson(String),
    #[error("unsupported structured patch version: {0}")]
    UnsupportedVersion(u32),
    #[error("invalid structured patch: {0}")]
    InvalidPatch(String),
    #[error("invalid patch path '{path}': {reason}")]
    InvalidPath { path: String, reason: String },
    #[error("stale patch target '{path}': {reason}")]
    StaleTarget { path: String, reason: String },
    #[error("patch apply failed: {0}")]
    ApplyFailed(String),
}

pub fn prepare_structured_patch(
    artifact: &str,
    project_root: &Path,
) -> Result<PreparedPatchSet, StructuredPatchError> {
    if artifact.len() > MAX_TOTAL_CONTENT_BYTES * 2 {
        return Err(StructuredPatchError::InvalidPatch(
            "artifact exceeds the maximum JSON size".to_string(),
        ));
    }

    let patch: StructuredPatchSet = serde_json::from_str(artifact)
        .map_err(|error| StructuredPatchError::InvalidJson(error.to_string()))?;
    validate_patch_metadata(&patch)?;

    let project_root = project_root.canonicalize().map_err(|error| {
        StructuredPatchError::InvalidPatch(format!(
            "project root cannot be canonicalized: {}",
            error
        ))
    })?;
    if !project_root.is_dir() {
        return Err(StructuredPatchError::InvalidPatch(
            "project root is not a directory".to_string(),
        ));
    }

    let path_guard = PathGuard::new(vec![project_root.clone()]);
    let mut seen_paths = HashSet::new();
    let mut prepared_changes = Vec::with_capacity(patch.changes.len());
    let mut total_content_bytes = 0usize;

    for change in patch.changes {
        let relative_path = validate_relative_patch_path(&change.path)?;
        let portable_key = change.path.to_ascii_lowercase();
        if !seen_paths.insert(portable_key) {
            return Err(StructuredPatchError::InvalidPath {
                path: change.path,
                reason: "duplicate path in patch set".to_string(),
            });
        }

        validate_text_content(&change.path, &change.content)?;
        total_content_bytes = total_content_bytes
            .checked_add(change.content.len())
            .ok_or_else(|| {
                StructuredPatchError::InvalidPatch(
                    "total proposed content size overflowed".to_string(),
                )
            })?;
        if total_content_bytes > MAX_TOTAL_CONTENT_BYTES {
            return Err(StructuredPatchError::InvalidPatch(format!(
                "total proposed content exceeds {} bytes",
                MAX_TOTAL_CONTENT_BYTES
            )));
        }

        ensure_no_symlink_components(&project_root, &relative_path, &change.path)?;
        let target_path = project_root.join(&relative_path);
        let (original_content, original_sha256) = match change.operation {
            PatchOperation::Create => {
                if change.expected_sha256.is_some() {
                    return Err(StructuredPatchError::InvalidPatch(format!(
                        "create change '{}' cannot declare expected_sha256",
                        change.path
                    )));
                }
                if target_path.exists() {
                    return Err(StructuredPatchError::StaleTarget {
                        path: change.path,
                        reason: "create target already exists".to_string(),
                    });
                }
                let parent =
                    target_path
                        .parent()
                        .ok_or_else(|| StructuredPatchError::InvalidPath {
                            path: change.path.clone(),
                            reason: "target has no parent directory".to_string(),
                        })?;
                if !parent.is_dir() {
                    return Err(StructuredPatchError::InvalidPath {
                        path: change.path,
                        reason: "parent directory does not exist".to_string(),
                    });
                }
                path_guard.validate_write(&target_path).map_err(|error| {
                    StructuredPatchError::InvalidPath {
                        path: change.path.clone(),
                        reason: error.to_string(),
                    }
                })?;
                (String::new(), None)
            }
            PatchOperation::Replace => {
                path_guard.validate_write(&target_path).map_err(|error| {
                    StructuredPatchError::InvalidPath {
                        path: change.path.clone(),
                        reason: error.to_string(),
                    }
                })?;
                if !target_path.is_file() {
                    return Err(StructuredPatchError::StaleTarget {
                        path: change.path,
                        reason: "replace target is not an existing regular file".to_string(),
                    });
                }
                let original = read_bounded_text_file(&target_path, &change.path)?;
                let digest = sha256_hex(original.as_bytes());
                if let Some(expected) = change
                    .expected_sha256
                    .as_deref()
                    .filter(|expected| !expected.trim().is_empty())
                {
                    validate_sha256(expected, &change.path)?;
                    if !digest.eq_ignore_ascii_case(expected) {
                        return Err(StructuredPatchError::StaleTarget {
                            path: change.path,
                            reason: "expected_sha256 does not match the current file".to_string(),
                        });
                    }
                }
                (original, Some(digest))
            }
        };

        if original_content == change.content {
            return Err(StructuredPatchError::InvalidPatch(format!(
                "change '{}' does not modify content",
                change.path
            )));
        }

        let (diff, lines_changed) =
            generate_review_diff(&change.path, &original_content, &change.content);
        prepared_changes.push(PreparedFileChange {
            path: change.path,
            operation: change.operation,
            target_path,
            new_sha256: sha256_hex(change.content.as_bytes()),
            new_content: change.content,
            original_sha256,
            diff,
            lines_changed,
        });
    }

    let artifact_digest = sha256_hex(artifact.as_bytes());
    Ok(PreparedPatchSet {
        patch_id: format!("patch_{}", &artifact_digest[..16]),
        summary: patch.summary.trim().to_string(),
        validation: patch
            .validation
            .into_iter()
            .map(|item| item.trim().to_string())
            .collect(),
        project_root,
        changes: prepared_changes,
    })
}

pub fn apply_prepared_patch(
    prepared: &PreparedPatchSet,
    backup_root: &Path,
) -> Result<AppliedPatchSet, StructuredPatchError> {
    apply_prepared_patch_inner(prepared, backup_root, None)
}

#[cfg(test)]
fn apply_prepared_patch_with_failure(
    prepared: &PreparedPatchSet,
    backup_root: &Path,
    fail_after_writes: usize,
) -> Result<AppliedPatchSet, StructuredPatchError> {
    apply_prepared_patch_inner(prepared, backup_root, Some(fail_after_writes))
}

fn validate_patch_metadata(patch: &StructuredPatchSet) -> Result<(), StructuredPatchError> {
    if patch.version != PATCH_SCHEMA_VERSION {
        return Err(StructuredPatchError::UnsupportedVersion(patch.version));
    }
    let summary = patch.summary.trim();
    if summary.is_empty() || summary.len() > MAX_SUMMARY_BYTES {
        return Err(StructuredPatchError::InvalidPatch(format!(
            "summary must contain 1..={} bytes",
            MAX_SUMMARY_BYTES
        )));
    }
    if patch.changes.is_empty() || patch.changes.len() > MAX_CHANGES {
        return Err(StructuredPatchError::InvalidPatch(format!(
            "changes must contain 1..={} entries",
            MAX_CHANGES
        )));
    }
    if patch.validation.len() > MAX_VALIDATION_ITEMS {
        return Err(StructuredPatchError::InvalidPatch(format!(
            "validation must contain at most {} entries",
            MAX_VALIDATION_ITEMS
        )));
    }
    if patch
        .validation
        .iter()
        .any(|item| item.trim().is_empty() || item.len() > MAX_VALIDATION_ITEM_BYTES)
    {
        return Err(StructuredPatchError::InvalidPatch(format!(
            "validation entries must contain 1..={} bytes",
            MAX_VALIDATION_ITEM_BYTES
        )));
    }
    Ok(())
}

fn validate_relative_patch_path(path: &str) -> Result<PathBuf, StructuredPatchError> {
    let invalid = |reason: &str| StructuredPatchError::InvalidPath {
        path: path.to_string(),
        reason: reason.to_string(),
    };

    if path.is_empty() || path.trim() != path {
        return Err(invalid(
            "path must be non-empty and have no surrounding whitespace",
        ));
    }
    if path.len() > MAX_RELATIVE_PATH_BYTES {
        return Err(invalid("path exceeds the maximum length"));
    }
    if path.contains('\\') {
        return Err(invalid("use forward slashes in portable relative paths"));
    }
    if path.starts_with('/') || path.contains(':') {
        return Err(invalid("absolute or drive-qualified paths are forbidden"));
    }

    let forbidden = [
        ".git",
        ".godot",
        ".operator",
        "node_modules",
        "target",
        "build",
        "dist",
    ];
    let mut normalized = PathBuf::new();
    for component in path.split('/') {
        if component.is_empty() || component == "." || component == ".." {
            return Err(invalid(
                "empty, current, and parent path components are forbidden",
            ));
        }
        if component.ends_with(' ') || component.ends_with('.') {
            return Err(invalid("path components cannot end with a space or dot"));
        }
        if forbidden
            .iter()
            .any(|entry| component.eq_ignore_ascii_case(entry))
        {
            return Err(invalid("path enters a generated or protected directory"));
        }
        if is_windows_reserved_name(component) {
            return Err(invalid("path contains a Windows reserved device name"));
        }
        normalized.push(component);
    }

    let extension = normalized
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let allowed_extensions = [
        "gd", "tscn", "tres", "godot", "cfg", "ini", "json", "csv", "txt", "md", "shader",
        "gdshader", "glsl", "yaml", "yml", "toml",
    ];
    if !allowed_extensions
        .iter()
        .any(|allowed| extension.eq_ignore_ascii_case(allowed))
    {
        return Err(invalid("only allowlisted Godot text files can be patched"));
    }

    Ok(normalized)
}

fn is_windows_reserved_name(component: &str) -> bool {
    let stem = component
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

fn validate_text_content(path: &str, content: &str) -> Result<(), StructuredPatchError> {
    if content.is_empty() {
        return Err(StructuredPatchError::InvalidPatch(format!(
            "change '{}' has empty content",
            path
        )));
    }
    if content.len() > MAX_FILE_BYTES {
        return Err(StructuredPatchError::InvalidPatch(format!(
            "change '{}' exceeds {} bytes",
            path, MAX_FILE_BYTES
        )));
    }
    if content.contains('\0') {
        return Err(StructuredPatchError::InvalidPatch(format!(
            "change '{}' contains a NUL byte",
            path
        )));
    }
    Ok(())
}

fn validate_sha256(value: &str, path: &str) -> Result<(), StructuredPatchError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(StructuredPatchError::InvalidPatch(format!(
            "change '{}' has an invalid expected_sha256",
            path
        )));
    }
    Ok(())
}

fn ensure_no_symlink_components(
    project_root: &Path,
    relative_path: &Path,
    display_path: &str,
) -> Result<(), StructuredPatchError> {
    let mut current = project_root.to_path_buf();
    for component in relative_path.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(StructuredPatchError::InvalidPath {
                    path: display_path.to_string(),
                    reason: "symbolic links are forbidden in patch paths".to_string(),
                });
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => {
                return Err(StructuredPatchError::InvalidPath {
                    path: display_path.to_string(),
                    reason: format!("cannot inspect path component: {}", error),
                });
            }
        }
    }
    Ok(())
}

fn read_bounded_text_file(path: &Path, display_path: &str) -> Result<String, StructuredPatchError> {
    let metadata = fs::metadata(path).map_err(|error| StructuredPatchError::StaleTarget {
        path: display_path.to_string(),
        reason: format!("cannot inspect target: {}", error),
    })?;
    if metadata.len() > MAX_FILE_BYTES as u64 {
        return Err(StructuredPatchError::InvalidPatch(format!(
            "existing file '{}' exceeds {} bytes",
            display_path, MAX_FILE_BYTES
        )));
    }
    fs::read_to_string(path).map_err(|error| {
        StructuredPatchError::InvalidPatch(format!(
            "existing file '{}' is not readable UTF-8 text: {}",
            display_path, error
        ))
    })
}

fn sha256_hex(content: &[u8]) -> String {
    let digest = Sha256::digest(content);
    digest.iter().map(|byte| format!("{:02x}", byte)).collect()
}

fn generate_review_diff(path: &str, original: &str, proposed: &str) -> (String, usize) {
    let original_lines = original.lines().collect::<Vec<_>>();
    let proposed_lines = proposed.lines().collect::<Vec<_>>();
    let mut prefix = 0usize;
    while prefix < original_lines.len()
        && prefix < proposed_lines.len()
        && original_lines[prefix] == proposed_lines[prefix]
    {
        prefix += 1;
    }

    let mut suffix = 0usize;
    while suffix < original_lines.len().saturating_sub(prefix)
        && suffix < proposed_lines.len().saturating_sub(prefix)
        && original_lines[original_lines.len() - 1 - suffix]
            == proposed_lines[proposed_lines.len() - 1 - suffix]
    {
        suffix += 1;
    }

    let original_changed_end = original_lines.len().saturating_sub(suffix);
    let proposed_changed_end = proposed_lines.len().saturating_sub(suffix);
    let context_start = prefix.saturating_sub(3);
    let context_suffix = suffix.min(3);
    let mut diff = format!("--- a/{0}\n+++ b/{0}\n@@ preview @@\n", path);

    for line in &original_lines[context_start..prefix] {
        diff.push(' ');
        diff.push_str(line);
        diff.push('\n');
    }
    for line in &original_lines[prefix..original_changed_end] {
        diff.push('-');
        diff.push_str(line);
        diff.push('\n');
    }
    for line in &proposed_lines[prefix..proposed_changed_end] {
        diff.push('+');
        diff.push_str(line);
        diff.push('\n');
    }
    for line in &original_lines[original_changed_end..original_changed_end + context_suffix] {
        diff.push(' ');
        diff.push_str(line);
        diff.push('\n');
    }

    let lines_changed =
        original_changed_end.saturating_sub(prefix) + proposed_changed_end.saturating_sub(prefix);
    (diff, lines_changed)
}

fn apply_prepared_patch_inner(
    prepared: &PreparedPatchSet,
    backup_root: &Path,
    fail_after_writes: Option<usize>,
) -> Result<AppliedPatchSet, StructuredPatchError> {
    if prepared.changes.is_empty() {
        return Err(StructuredPatchError::InvalidPatch(
            "prepared patch contains no changes".to_string(),
        ));
    }
    if !backup_root.is_absolute() {
        return Err(StructuredPatchError::ApplyFailed(
            "backup root must be an absolute path".to_string(),
        ));
    }

    let current_root = prepared.project_root.canonicalize().map_err(|error| {
        StructuredPatchError::ApplyFailed(format!("project root cannot be revalidated: {}", error))
    })?;
    if current_root != prepared.project_root {
        return Err(StructuredPatchError::StaleTarget {
            path: prepared.project_root.display().to_string(),
            reason: "project root identity changed after preview".to_string(),
        });
    }

    let path_guard = PathGuard::new(vec![prepared.project_root.clone()]);
    for change in &prepared.changes {
        verify_prepared_target(change, &prepared.project_root, &path_guard)?;
    }

    if backup_root.exists() {
        return Err(StructuredPatchError::ApplyFailed(format!(
            "backup root already exists: {}",
            backup_root.display()
        )));
    }
    fs::create_dir_all(backup_root).map_err(|error| {
        StructuredPatchError::ApplyFailed(format!("cannot create backup root: {}", error))
    })?;

    let mut backup_paths = vec![None; prepared.changes.len()];
    for (index, change) in prepared.changes.iter().enumerate() {
        if matches!(change.operation, PatchOperation::Replace) {
            let backup_path = backup_root.join(Path::new(&change.path));
            let parent = backup_path.parent().ok_or_else(|| {
                StructuredPatchError::ApplyFailed("backup path has no parent".to_string())
            })?;
            fs::create_dir_all(parent).map_err(|error| {
                StructuredPatchError::ApplyFailed(format!(
                    "cannot create backup directory for '{}': {}",
                    change.path, error
                ))
            })?;
            fs::copy(&change.target_path, &backup_path).map_err(|error| {
                StructuredPatchError::ApplyFailed(format!(
                    "cannot back up '{}': {}",
                    change.path, error
                ))
            })?;
            backup_paths[index] = Some(backup_path);
        }
    }

    let mut touched = Vec::new();
    for (index, change) in prepared.changes.iter().enumerate() {
        if let Err(error) = verify_prepared_target(change, &prepared.project_root, &path_guard) {
            return rollback_after_failure(
                prepared,
                &backup_paths,
                &touched,
                format!("target revalidation failed: {}", error),
            );
        }

        touched.push(index);
        if let Err(error) = write_change(change) {
            return rollback_after_failure(prepared, &backup_paths, &touched, error);
        }
        let actual = match fs::read(&change.target_path) {
            Ok(content) => sha256_hex(&content),
            Err(error) => {
                return rollback_after_failure(
                    prepared,
                    &backup_paths,
                    &touched,
                    format!("cannot verify '{}': {}", change.path, error),
                );
            }
        };
        if actual != change.new_sha256 {
            return rollback_after_failure(
                prepared,
                &backup_paths,
                &touched,
                format!("written content hash mismatch for '{}'", change.path),
            );
        }
        if fail_after_writes == Some(touched.len()) {
            return rollback_after_failure(
                prepared,
                &backup_paths,
                &touched,
                "injected batch write failure".to_string(),
            );
        }
    }

    let files = prepared
        .changes
        .iter()
        .enumerate()
        .map(|(index, change)| AppliedFileChange {
            path: change.path.clone(),
            operation: change.operation.clone(),
            backup_path: backup_paths[index]
                .as_ref()
                .map(|path| path.to_string_lossy().to_string()),
            lines_changed: change.lines_changed,
        })
        .collect();

    Ok(AppliedPatchSet {
        patch_id: prepared.patch_id.clone(),
        backup_root: backup_root.to_string_lossy().to_string(),
        files,
    })
}

fn verify_prepared_target(
    change: &PreparedFileChange,
    project_root: &Path,
    path_guard: &PathGuard,
) -> Result<(), StructuredPatchError> {
    let relative_path = validate_relative_patch_path(&change.path)?;
    let expected_target = project_root.join(&relative_path);
    if expected_target != change.target_path {
        return Err(StructuredPatchError::StaleTarget {
            path: change.path.clone(),
            reason: "prepared target path changed".to_string(),
        });
    }
    ensure_no_symlink_components(project_root, &relative_path, &change.path)?;
    path_guard
        .validate_write(&change.target_path)
        .map_err(|error| StructuredPatchError::InvalidPath {
            path: change.path.clone(),
            reason: error.to_string(),
        })?;

    match change.operation {
        PatchOperation::Create => {
            if change.target_path.exists() {
                return Err(StructuredPatchError::StaleTarget {
                    path: change.path.clone(),
                    reason: "create target appeared after preview".to_string(),
                });
            }
            let parent =
                change
                    .target_path
                    .parent()
                    .ok_or_else(|| StructuredPatchError::InvalidPath {
                        path: change.path.clone(),
                        reason: "target has no parent directory".to_string(),
                    })?;
            if !parent.is_dir() {
                return Err(StructuredPatchError::StaleTarget {
                    path: change.path.clone(),
                    reason: "parent directory disappeared after preview".to_string(),
                });
            }
        }
        PatchOperation::Replace => {
            if !change.target_path.is_file() {
                return Err(StructuredPatchError::StaleTarget {
                    path: change.path.clone(),
                    reason: "replace target disappeared after preview".to_string(),
                });
            }
            let current = read_bounded_text_file(&change.target_path, &change.path)?;
            let current_sha256 = sha256_hex(current.as_bytes());
            if change.original_sha256.as_deref() != Some(current_sha256.as_str()) {
                return Err(StructuredPatchError::StaleTarget {
                    path: change.path.clone(),
                    reason: "file content changed after preview".to_string(),
                });
            }
        }
    }
    Ok(())
}

fn write_change(change: &PreparedFileChange) -> Result<(), String> {
    let mut options = fs::OpenOptions::new();
    options.write(true);
    match change.operation {
        PatchOperation::Create => {
            options.create_new(true);
        }
        PatchOperation::Replace => {
            options.truncate(true);
        }
    }
    let mut file = options
        .open(&change.target_path)
        .map_err(|error| format!("cannot open '{}' for writing: {}", change.path, error))?;
    file.write_all(change.new_content.as_bytes())
        .map_err(|error| format!("cannot write '{}': {}", change.path, error))?;
    file.sync_all()
        .map_err(|error| format!("cannot sync '{}': {}", change.path, error))?;
    Ok(())
}

fn rollback_after_failure(
    prepared: &PreparedPatchSet,
    backup_paths: &[Option<PathBuf>],
    touched: &[usize],
    reason: String,
) -> Result<AppliedPatchSet, StructuredPatchError> {
    let mut rollback_errors = Vec::new();
    for index in touched.iter().rev().copied() {
        let change = &prepared.changes[index];
        match change.operation {
            PatchOperation::Create => {
                if change.target_path.exists() {
                    if let Err(error) = fs::remove_file(&change.target_path) {
                        rollback_errors.push(format!(
                            "cannot remove created file '{}': {}",
                            change.path, error
                        ));
                    }
                }
            }
            PatchOperation::Replace => {
                if let Some(backup_path) = backup_paths[index].as_ref() {
                    if change.target_path.exists() {
                        if let Err(error) = fs::remove_file(&change.target_path) {
                            rollback_errors.push(format!(
                                "cannot remove failed replacement '{}': {}",
                                change.path, error
                            ));
                            continue;
                        }
                    }
                    if let Err(error) = fs::copy(backup_path, &change.target_path) {
                        rollback_errors
                            .push(format!("cannot restore '{}': {}", change.path, error));
                    }
                } else {
                    rollback_errors.push(format!("missing backup for '{}'", change.path));
                }
            }
        }
    }

    let rollback_status = if rollback_errors.is_empty() {
        "rollback completed".to_string()
    } else {
        format!("rollback errors: {}", rollback_errors.join("; "))
    };
    Err(StructuredPatchError::ApplyFailed(format!(
        "{}; {}",
        reason, rollback_status
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operator::is_d_drive_path;
    use serde_json::json;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestProject {
        root: PathBuf,
    }

    impl TestProject {
        fn new(name: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time")
                .as_nanos();
            let root = std::env::current_dir()
                .expect("current dir")
                .join(".tmp-tests")
                .join(format!(
                    "structured_patch_{}_{}_{}",
                    name,
                    std::process::id(),
                    unique
                ));
            fs::create_dir_all(root.join("scripts")).expect("create test project");
            fs::write(
                root.join("project.godot"),
                "[application]\nconfig/name=\"Patch Test\"\n",
            )
            .expect("write project marker");
            Self { root }
        }

        fn root(&self) -> &Path {
            &self.root
        }

        fn backup_root(&self, name: &str) -> PathBuf {
            self.root.join(".operator-test-backups").join(name)
        }
    }

    impl Drop for TestProject {
        fn drop(&mut self) {
            if is_d_drive_path(&self.root) {
                let _ = fs::remove_dir_all(&self.root);
            }
        }
    }

    fn artifact(changes: serde_json::Value) -> String {
        json!({
            "version": 1,
            "summary": "Implement a controlled game change",
            "changes": changes,
            "validation": ["Open the main scene and verify it loads"]
        })
        .to_string()
    }

    #[test]
    fn prepares_replace_and_create_without_touching_project_files() {
        let project = TestProject::new("preview");
        let player = project.root().join("scripts/player.gd");
        fs::write(&player, "extends Node\nvar jumps = 1\n").expect("write original");
        let new_file = project.root().join("scripts/jump_state.gd");
        let artifact = artifact(json!([
            {
                "path": "scripts/player.gd",
                "operation": "replace",
                "content": "extends Node\nvar jumps = 2\n"
            },
            {
                "path": "scripts/jump_state.gd",
                "operation": "create",
                "content": "class_name JumpState\n"
            }
        ]));

        let prepared = prepare_structured_patch(&artifact, project.root()).expect("valid patch");

        assert_eq!(prepared.changes.len(), 2);
        assert!(prepared.changes[0].diff.contains("-var jumps = 1"));
        assert!(prepared.changes[0].diff.contains("+var jumps = 2"));
        assert!(prepared.changes[0].original_sha256.is_some());
        assert!(prepared.changes[1].original_sha256.is_none());
        assert_eq!(
            fs::read_to_string(player).expect("read original"),
            "extends Node\nvar jumps = 1\n"
        );
        assert!(!new_file.exists());
    }

    #[test]
    fn treats_empty_expected_sha256_as_an_omitted_guard() {
        let project = TestProject::new("empty-expected-sha");
        let player = project.root().join("scripts/player.gd");
        fs::write(&player, "extends Node\nvar jumps = 1\n").expect("write original");
        let artifact = artifact(json!([{
            "path": "scripts/player.gd",
            "operation": "replace",
            "content": "extends Node\nvar jumps = 2\n",
            "expected_sha256": ""
        }]));

        let prepared = prepare_structured_patch(&artifact, project.root())
            .expect("empty optional hash should be accepted");
        assert_eq!(prepared.changes.len(), 1);
        assert!(prepared.changes[0].original_sha256.is_some());
    }

    #[test]
    fn rejects_non_json_unknown_fields_delete_and_unsafe_paths() {
        let project = TestProject::new("invalid");
        fs::write(project.root().join("scripts/player.gd"), "extends Node\n")
            .expect("write original");

        let cases = [
            "```json\n{\"version\":1}\n```".to_string(),
            json!({
                "version": 1,
                "summary": "unknown field",
                "changes": [],
                "unexpected": true
            })
            .to_string(),
            artifact(json!([{
                "path": "scripts/player.gd",
                "operation": "delete",
                "content": ""
            }])),
            artifact(json!([{
                "path": "../outside.gd",
                "operation": "create",
                "content": "bad"
            }])),
            artifact(json!([{
                "path": "scripts\\outside.gd",
                "operation": "create",
                "content": "bad"
            }])),
            artifact(json!([{
                "path": ".godot/generated.gd",
                "operation": "create",
                "content": "bad"
            }])),
            artifact(json!([{
                "path": "scripts/new.gd",
                "operation": "create",
                "content": "extends Node\n",
                "expected_sha256": "0000000000000000000000000000000000000000000000000000000000000000"
            }])),
        ];

        for case in cases {
            assert!(prepare_structured_patch(&case, project.root()).is_err());
        }
    }

    #[test]
    fn rejects_duplicate_paths_and_model_hash_mismatch() {
        let project = TestProject::new("duplicates");
        fs::write(project.root().join("scripts/player.gd"), "extends Node\n")
            .expect("write original");

        let duplicate = artifact(json!([
            {
                "path": "scripts/player.gd",
                "operation": "replace",
                "content": "extends Node2D\n"
            },
            {
                "path": "scripts/PLAYER.gd",
                "operation": "replace",
                "content": "extends CharacterBody2D\n"
            }
        ]));
        assert!(prepare_structured_patch(&duplicate, project.root()).is_err());

        let stale_model_hash = artifact(json!([{
            "path": "scripts/player.gd",
            "operation": "replace",
            "content": "extends Node2D\n",
            "expected_sha256": "0000000000000000000000000000000000000000000000000000000000000000"
        }]));
        assert!(matches!(
            prepare_structured_patch(&stale_model_hash, project.root()),
            Err(StructuredPatchError::StaleTarget { .. })
        ));
    }

    #[test]
    fn refuses_to_overwrite_a_file_changed_after_preview() {
        let project = TestProject::new("stale");
        let player = project.root().join("scripts/player.gd");
        fs::write(&player, "extends Node\nvar jumps = 1\n").expect("write original");
        let prepared = prepare_structured_patch(
            &artifact(json!([{
                "path": "scripts/player.gd",
                "operation": "replace",
                "content": "extends Node\nvar jumps = 2\n"
            }])),
            project.root(),
        )
        .expect("prepare patch");

        fs::write(&player, "extends Node\nvar jumps = 3 # operator edit\n").expect("operator edit");

        let error = apply_prepared_patch(&prepared, &project.backup_root("stale"))
            .expect_err("stale patch must fail");
        assert!(matches!(error, StructuredPatchError::StaleTarget { .. }));
        assert!(fs::read_to_string(player)
            .expect("read operator edit")
            .contains("operator edit"));
    }

    #[test]
    fn applies_all_changes_and_keeps_a_versioned_backup() {
        let project = TestProject::new("apply");
        let player = project.root().join("scripts/player.gd");
        fs::write(&player, "extends Node\nvar jumps = 1\n").expect("write original");
        let prepared = prepare_structured_patch(
            &artifact(json!([
                {
                    "path": "scripts/player.gd",
                    "operation": "replace",
                    "content": "extends Node\nvar jumps = 2\n"
                },
                {
                    "path": "scripts/jump_state.gd",
                    "operation": "create",
                    "content": "class_name JumpState\n"
                }
            ])),
            project.root(),
        )
        .expect("prepare patch");
        let backup_root = project.backup_root("apply");

        let applied = apply_prepared_patch(&prepared, &backup_root).expect("apply patch");

        assert_eq!(applied.files.len(), 2);
        assert!(fs::read_to_string(&player)
            .expect("read replacement")
            .contains("jumps = 2"));
        assert_eq!(
            fs::read_to_string(project.root().join("scripts/jump_state.gd"))
                .expect("read created file"),
            "class_name JumpState\n"
        );
        assert_eq!(
            fs::read_to_string(backup_root.join("scripts/player.gd")).expect("read backup"),
            "extends Node\nvar jumps = 1\n"
        );
        assert!(applied.files[0].backup_path.is_some());
        assert!(applied.files[1].backup_path.is_none());
    }

    #[test]
    fn rolls_back_previous_writes_when_a_batch_apply_fails() {
        let project = TestProject::new("rollback");
        let player = project.root().join("scripts/player.gd");
        fs::write(&player, "extends Node\nvar jumps = 1\n").expect("write original");
        let created = project.root().join("scripts/jump_state.gd");
        let prepared = prepare_structured_patch(
            &artifact(json!([
                {
                    "path": "scripts/player.gd",
                    "operation": "replace",
                    "content": "extends Node\nvar jumps = 2\n"
                },
                {
                    "path": "scripts/jump_state.gd",
                    "operation": "create",
                    "content": "class_name JumpState\n"
                }
            ])),
            project.root(),
        )
        .expect("prepare patch");

        let error =
            apply_prepared_patch_with_failure(&prepared, &project.backup_root("rollback"), 1)
                .expect_err("injected write failure");

        assert!(matches!(error, StructuredPatchError::ApplyFailed(_)));
        assert_eq!(
            fs::read_to_string(player).expect("read restored file"),
            "extends Node\nvar jumps = 1\n"
        );
        assert!(!created.exists());
    }

    #[test]
    #[ignore = "requires ACP_REAL_HERMES_PATCH_ARTIFACT, ACP_REAL_HERMES_PROJECT, and ACP_REAL_HERMES_BACKUP_ROOT"]
    fn validates_and_applies_external_real_hermes_artifact() {
        let artifact_path = PathBuf::from(
            std::env::var("ACP_REAL_HERMES_PATCH_ARTIFACT")
                .expect("ACP_REAL_HERMES_PATCH_ARTIFACT must be set"),
        );
        let project_root = PathBuf::from(
            std::env::var("ACP_REAL_HERMES_PROJECT").expect("ACP_REAL_HERMES_PROJECT must be set"),
        );
        let backup_root = PathBuf::from(
            std::env::var("ACP_REAL_HERMES_BACKUP_ROOT")
                .expect("ACP_REAL_HERMES_BACKUP_ROOT must be set"),
        );
        assert!(is_d_drive_path(&artifact_path));
        assert!(is_d_drive_path(&project_root));
        assert!(is_d_drive_path(&backup_root));

        let artifact = fs::read_to_string(&artifact_path).expect("read real Hermes artifact");
        let prepared =
            prepare_structured_patch(&artifact, &project_root).expect("validate real artifact");
        assert!(!prepared.changes.is_empty());

        let applied =
            apply_prepared_patch(&prepared, &backup_root).expect("apply real Hermes artifact");
        assert_eq!(applied.files.len(), prepared.changes.len());
        for change in &prepared.changes {
            assert!(project_root.join(&change.path).is_file());
        }
    }
}
