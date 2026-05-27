use std::path::Path;

use crate::model::Project;

const PROJECT_MARKERS: &[&str] = &[
    "package.json",
    "Cargo.toml",
    "go.mod",
    "Gemfile",
    "requirements.txt",
    "pyproject.toml",
    "setup.py",
    "setup.cfg",
    "Pipfile",
    "mix.exs",
    "build.gradle",
    "build.gradle.kts",
    "pom.xml",
    "composer.json",
    "pubspec.yaml",
    "deno.json",
    "deno.jsonc",
    "bun.lockb",
    "Makefile",
    ".git",
    "CMakeLists.txt",
    "meson.build",
    "flake.nix",
    "cabal.project",
    "stack.yaml",
    "shard.yml",
    "Project.toml",
    "rebar.config",
];

pub fn detect_project(cwd: &Path) -> Option<Project> {
    let mut dir = cwd.to_path_buf();
    loop {
        for marker in PROJECT_MARKERS {
            if dir.join(marker).exists() {
                let name = dir
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                return Some(Project {
                    name,
                    root: dir,
                    framework: None,
                });
            }
        }
        if !dir.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn detects_cargo_project() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let project = detect_project(dir.path()).unwrap();
        assert_eq!(project.root, dir.path());
        assert!(project.framework.is_none());
    }

    #[test]
    fn detects_node_project() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("package.json"), "{}").unwrap();
        let project = detect_project(dir.path()).unwrap();
        assert_eq!(project.root, dir.path());
    }

    #[test]
    fn detects_go_project() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("go.mod"), "").unwrap();
        let project = detect_project(dir.path()).unwrap();
        assert_eq!(project.root, dir.path());
    }

    #[test]
    fn detects_python_project() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("requirements.txt"), "").unwrap();
        let project = detect_project(dir.path()).unwrap();
        assert_eq!(project.root, dir.path());
    }

    #[test]
    fn detects_git_repo() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        let project = detect_project(dir.path()).unwrap();
        assert_eq!(project.root, dir.path());
    }

    #[test]
    fn walks_up_to_parent() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let sub = dir.path().join("src");
        std::fs::create_dir(&sub).unwrap();
        let project = detect_project(&sub).unwrap();
        assert_eq!(project.root, dir.path());
    }

    #[test]
    fn project_name_from_dir_name() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("package.json"), "").unwrap();
        let project = detect_project(dir.path()).unwrap();
        assert!(!project.name.is_empty());
    }

    #[test]
    fn returns_none_when_no_markers() {
        let dir = tempdir().unwrap();
        assert!(detect_project(dir.path()).is_none());
    }

    #[test]
    fn all_markers_recognized() {
        for marker in PROJECT_MARKERS {
            let dir = tempdir().unwrap();
            let path = dir.path().join(marker);
            if marker.contains('.') || !marker.starts_with('.') {
                std::fs::write(&path, "").unwrap();
            } else {
                std::fs::create_dir(&path).unwrap();
            }
            assert!(
                detect_project(dir.path()).is_some(),
                "marker {} not detected",
                marker
            );
        }
    }
}
