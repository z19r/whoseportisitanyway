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
