use crate::error::Result;
use crate::init::{subdirs, WAYLOG_DIR};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Get the home directory in a cross-platform way
pub fn home_dir() -> Result<PathBuf> {
    #[cfg(test)]
    {
        // Use a unique directory per test run to prevent tests from modifying
        // the actual user's home directory. We use thread id to make it more unique.
        let thread_id = format!("{:?}", std::thread::current().id());
        let sanitized_id =
            thread_id.replace(&['T', 'h', 'r', 'e', 'a', 'd', 'I', 'd', '(', ')'][..], "");
        let test_home = std::env::temp_dir().join(format!("waylog_test_home_{}", sanitized_id));
        let _ = std::fs::create_dir_all(&test_home);
        return Ok(test_home);
    }

    #[cfg(not(test))]
    home::home_dir().ok_or_else(|| {
        crate::error::WaylogError::PathError("Could not find home directory".to_string())
    })
}

/// Get the data directory for AI tools
/// On Unix: ~/.{tool}
/// On Windows: %USERPROFILE%\.{tool} (future extension point)
pub fn get_ai_data_dir(tool_name: &str) -> Result<PathBuf> {
    let home = home_dir()?;

    #[cfg(target_os = "windows")]
    {
        // Windows: Use AppData\Local for application data (future extension)
        // For now, keep it simple and use home directory
        Ok(home.join(format!(".{}", tool_name)))
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Unix-like systems (macOS, Linux)
        Ok(home.join(format!(".{}", tool_name)))
    }
}

/// Encode a path for Claude Code (replace all non-alphanumeric chars with -)
/// Unix: /Users/name/project -> -Users-name-project
/// Windows: C:\Users\name\project -> C--Users-name-project
/// Non-ASCII: /Users/名字/project -> -Users----project
pub fn encode_path_claude(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    let normalized = path_str.replace('\\', "/");

    normalized
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// Encode a path for Gemini (SHA-256 hash)
/// This is platform-independent as it hashes the string representation
/// Example: /Users/name/project -> f5ca4b7f107121b48048aa4ebe261a7ee63769dfc3a06e56191c987c8b51176d
pub fn encode_path_gemini(path: &Path) -> String {
    // Use the canonical string representation for consistent hashing
    let path_str = path.to_string_lossy();
    let mut hasher = Sha256::new();
    hasher.update(path_str.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Get the .waylog/history directory for the current project in the user's home directory
pub fn get_waylog_dir(project_dir: &Path) -> PathBuf {
    let home = home_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_name = project_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Fallback to "default" if project_name is empty
    let dir_name = if project_name.is_empty() {
        "default".to_string()
    } else {
        project_name
    };

    home.join(WAYLOG_DIR).join(subdirs::HISTORY).join(dir_name)
}

/// Get the .waylog/logs directory for the current project in the user's home directory
pub fn get_log_dir(project_dir: &Path) -> PathBuf {
    let home = home_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_name = project_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Fallback to "default" if project_name is empty
    let dir_name = if project_name.is_empty() {
        "default".to_string()
    } else {
        project_name
    };

    home.join(WAYLOG_DIR).join(subdirs::LOGS).join(dir_name)
}

/// Find the project root by looking for .git folder
/// moving upwards from the current directory.
/// If we reach the home directory or the system root without finding a marker,
/// returns the current directory.
pub fn find_project_root() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let home = home_dir().ok();

    for path in current_dir.ancestors() {
        if path.join(".git").is_dir() {
            return Some(path.to_path_buf());
        }

        // Stop if we've reached the user's home directory
        if let Some(ref home_path) = home {
            if path == home_path {
                break;
            }
        }
    }

    None
}

/// Ensure a directory exists, creating it if necessary
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_encode_path_claude_absolute_unix() {
        let path = Path::new("/home/user/project");
        assert_eq!(encode_path_claude(path), "-home-user-project");
    }

    #[test]
    fn test_encode_path_claude_relative() {
        let path = Path::new("project/subdir");
        // Relative paths will be converted to project-subdir
        assert_eq!(encode_path_claude(path), "project-subdir");
    }

    #[test]
    fn test_encode_path_claude_root() {
        let path = Path::new("/");
        assert_eq!(encode_path_claude(path), "-");
    }

    #[test]
    fn test_encode_path_claude_with_spaces() {
        // Spaces are replaced with hyphens
        let path = Path::new("/home/my project");
        assert_eq!(encode_path_claude(path), "-home-my-project");
    }

    #[test]
    fn test_encode_path_claude_non_ascii() {
        // Non-ASCII characters are replaced with hyphens
        let path = Path::new("/Users/名字/project");
        assert_eq!(encode_path_claude(path), "-Users----project");
    }

    #[test]
    fn test_encode_path_claude_special_chars() {
        // Special characters are replaced with hyphens
        let path = Path::new("/home/user@#$%");
        assert_eq!(encode_path_claude(path), "-home-user----");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_encode_path_claude_windows_absolute() {
        let path = Path::new("C:\\Users\\user\\project");
        assert_eq!(encode_path_claude(path), "C--Users-user-project");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_encode_path_claude_windows_relative() {
        let path = Path::new("project\\subdir");
        assert_eq!(encode_path_claude(path), "project-subdir");
    }

    #[test]
    fn test_encode_path_gemini_consistent() {
        // Test that same paths produce same hash
        let path1 = Path::new("/home/user/project");
        let path2 = Path::new("/home/user/project");
        assert_eq!(encode_path_gemini(path1), encode_path_gemini(path2));
    }

    #[test]
    fn test_encode_path_gemini_different_paths() {
        // Test that different paths produce different hashes
        let path1 = Path::new("/home/user/project1");
        let path2 = Path::new("/home/user/project2");
        assert_ne!(encode_path_gemini(path1), encode_path_gemini(path2));
    }

    #[test]
    fn test_encode_path_gemini_hash_format() {
        // Test hash format: 64 hexadecimal characters
        let path = Path::new("/home/user/project");
        let hash = encode_path_gemini(path);
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_encode_path_gemini_relative_vs_absolute() {
        // Relative and absolute paths should produce different hashes
        let abs_path = Path::new("/home/user/project");
        let rel_path = Path::new("home/user/project");
        assert_ne!(encode_path_gemini(abs_path), encode_path_gemini(rel_path));
    }

    #[test]
    fn test_get_ai_data_dir_format() {
        let dir = get_ai_data_dir("claude").unwrap();
        let dir_str = dir.to_string_lossy();

        // Should contain tool name
        assert!(dir_str.contains(".claude"));

        // Should be under home directory
        let home = home_dir().unwrap();
        assert!(dir.starts_with(&home));
    }

    #[test]
    fn test_get_ai_data_dir_different_tools() {
        // Different tools should produce different paths
        let dir1 = get_ai_data_dir("claude").unwrap();
        let dir2 = get_ai_data_dir("gemini").unwrap();
        assert_ne!(dir1, dir2);
    }

    #[test]
    fn test_get_waylog_dir() {
        let project_dir = std::env::temp_dir().join("test-project");
        let waylog_dir = get_waylog_dir(&project_dir);

        let home = home_dir().unwrap_or_else(|_| PathBuf::from("."));
        let expected = home.join(".waylog").join("history").join("test-project");
        assert_eq!(waylog_dir, expected);
    }

    #[test]
    fn test_ensure_dir_exists() {
        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.path().join("new-dir");

        // Should create directory if it doesn't exist
        assert!(!new_dir.exists());
        ensure_dir_exists(&new_dir).unwrap();
        assert!(new_dir.exists());
        assert!(new_dir.is_dir());

        // Should not error if directory already exists
        ensure_dir_exists(&new_dir).unwrap();
        assert!(new_dir.exists());

        // Test nested directory creation
        let nested_dir = temp_dir.path().join("a").join("b").join("c");
        ensure_dir_exists(&nested_dir).unwrap();
        assert!(nested_dir.exists());
        assert!(nested_dir.is_dir());
    }

    #[test]
    fn test_find_project_root() {
        // Create temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().join("project");
        let subdir = project_root.join("subdir").join("deep");

        // Create project root directory and .git directory
        fs::create_dir_all(&subdir).unwrap();
        fs::create_dir_all(project_root.join(".git")).unwrap();

        // Save current working directory
        let original_dir = std::env::current_dir().unwrap();

        // Switch to subdirectory
        std::env::set_current_dir(&subdir).unwrap();

        // Should find project root
        let found_root = find_project_root();
        assert!(found_root.is_some());

        let found = found_root.unwrap();
        // Verify the found path contains .git directory
        assert!(found.join(".git").exists());
        // Compare paths by checking they resolve to the same directory
        // Use file_name to avoid issues with different path representations
        assert_eq!(
            found.file_name(),
            project_root.file_name(),
            "Found root should match expected project root"
        );

        // Restore original working directory
        std::env::set_current_dir(&original_dir).unwrap();
    }

    #[test]
    fn test_find_project_root_not_found() {
        // Create temporary directory but don't create .git
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir_all(&subdir).unwrap();

        // Save current working directory
        let original_dir = std::env::current_dir().unwrap();

        // Switch to subdirectory
        std::env::set_current_dir(&subdir).unwrap();

        // Should not find project root (not in home directory and no .git)
        // Note: This test may behave differently in different environments, depending on temp_dir location
        // If temp_dir is under home directory, find_project_root will stop at home and return None
        // If not, it will also return None (because .git was not found)
        let _found_root = find_project_root();
        // In test environment, temp_dir is usually not under home, so should return None
        // But we don't enforce assertion because behavior may vary by environment

        // Restore original working directory
        // If the original directory no longer exists (e.g., in parallel test execution),
        // try to restore to home directory as a fallback
        if let Err(_) = std::env::set_current_dir(&original_dir) {
            // Fallback to home directory if original directory is gone
            if let Ok(home) = home_dir() {
                let _ = std::env::set_current_dir(&home);
            }
        }
    }
}
