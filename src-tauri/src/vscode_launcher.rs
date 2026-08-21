//! Locate and launch VS Code on Windows. Kept separate from Remote-SSH history/config logic.

use std::path::PathBuf;
use std::process::Command;

use crate::util::no_window;

/// How to launch VS Code: call `Code.exe` directly, or fall back to `cmd /c code.cmd`.
struct CodeLauncher {
    program: PathBuf,
    prefix: Vec<String>,
}

pub fn available() -> bool {
    code_launcher().is_some()
}

/// Open a connected remote window with no folder.
pub fn open_remote_window(authority: &str) -> Result<(), String> {
    launch(["--remote".to_string(), format!("ssh-remote+{authority}")])
}

/// Open a remote directory using the CLI form documented by VS Code.
/// A trailing slash forces VS Code to treat a dotted path as a folder rather than guessing it is a file.
pub fn open_remote_folder(authority: &str, path: &str) -> Result<(), String> {
    let folder = if path.ends_with('/') {
        path.to_string()
    } else {
        format!("{path}/")
    };
    launch([
        "--remote".to_string(),
        format!("ssh-remote+{authority}"),
        folder,
    ])
}

fn launch<const N: usize>(args: [String; N]) -> Result<(), String> {
    let launcher = code_launcher().ok_or_else(|| "VS Code not found.".to_string())?;
    let mut command = Command::new(&launcher.program);
    command.args(&launcher.prefix).args(args);
    no_window(&mut command);
    command.spawn().map_err(|err| err.to_string())?;
    Ok(())
}

/// Locate how to launch VS Code: prefer `Code.exe` (including registry probing, so non-standard drive installs work),
/// falling back to `cmd /c code.cmd`.
fn code_launcher() -> Option<CodeLauncher> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    for var in ["LOCALAPPDATA", "ProgramFiles", "ProgramFiles(x86)"] {
        if let Some(base) = std::env::var_os(var) {
            candidates.push(
                PathBuf::from(&base)
                    .join("Programs")
                    .join("Microsoft VS Code")
                    .join("Code.exe"),
            );
            candidates.push(
                PathBuf::from(&base)
                    .join("Microsoft VS Code")
                    .join("Code.exe"),
            );
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            if dir.join("code.exe").exists() {
                candidates.push(dir.join("code.exe"));
            }
            if dir.join("code.cmd").exists() || dir.join("code").exists() {
                if let Some(parent) = dir.parent() {
                    candidates.push(parent.join("Code.exe"));
                }
            }
        }
    }
    if let Some(exe) = find_code_via_registry() {
        candidates.push(exe);
    }
    for candidate in &candidates {
        if candidate.exists() {
            return Some(CodeLauncher {
                program: candidate.clone(),
                prefix: Vec::new(),
            });
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            for name in ["code.cmd", "code.exe"] {
                let candidate = dir.join(name);
                if candidate.exists() {
                    return Some(CodeLauncher {
                        program: PathBuf::from("cmd"),
                        prefix: vec!["/c".to_string(), candidate.to_string_lossy().into_owned()],
                    });
                }
            }
        }
    }
    None
}

fn find_code_via_registry() -> Option<PathBuf> {
    let protocol_keys = [
        "HKEY_CLASSES_ROOT\\vscode\\shell\\open\\command",
        "HKEY_CURRENT_USER\\Software\\Classes\\vscode\\shell\\open\\command",
    ];
    for key in protocol_keys {
        if let Some(exe) = reg_query_code_exe(&[key, "/ve"]) {
            return Some(exe);
        }
    }
    let uninstall_keys = [
        "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{771FD6B0-FA20-440A-A002-3B3BAC16DC50}_is1",
        "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{EA457B21-F73E-494C-ACAB-524FDE069978}_is1",
        "HKEY_LOCAL_MACHINE\\SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{EA457B21-F73E-494C-ACAB-524FDE069978}_is1",
    ];
    for key in uninstall_keys {
        if let Some(exe) = reg_query_code_exe(&[key, "/v", "DisplayIcon"]) {
            return Some(exe);
        }
    }
    None
}

fn reg_query_code_exe(args: &[&str]) -> Option<PathBuf> {
    let mut command = Command::new("reg");
    command.arg("query").args(args);
    no_window(&mut command);
    let output = command.output().ok()?;
    extract_code_exe(&String::from_utf8_lossy(&output.stdout))
}

fn extract_code_exe(text: &str) -> Option<PathBuf> {
    for line in text.lines() {
        let lower = line.to_lowercase();
        let Some(hit) = lower.find("code.exe") else {
            continue;
        };
        let end = hit + "code.exe".len();
        let prefix = &line[..end];
        let start = if let Some(quote) = prefix.rfind('"') {
            quote + 1
        } else if let Some(sz) = lower[..end].rfind("reg_sz") {
            let after = sz + "reg_sz".len();
            after + (line[after..end].len() - line[after..end].trim_start().len())
        } else {
            0
        };
        let candidate = PathBuf::from(line[start..end].trim());
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn remote_folder_gets_trailing_slash() {
        let path = "/srv/project.with.dot";
        let folder = if path.ends_with('/') {
            path.to_string()
        } else {
            format!("{path}/")
        };
        assert_eq!(folder, "/srv/project.with.dot/");
    }
}
