use std::collections::HashMap;
use std::path::PathBuf;

use crate::model::{Framework, Project};

pub fn detect_framework(
    _process_name: &str,
    command_line: &str,
    project: &Option<Project>,
    framework_cache: &mut HashMap<PathBuf, Option<Framework>>,
) -> Option<Framework> {
    if let Some(fw) = detect_from_command(command_line) {
        return Some(fw);
    }
    if let Some(proj) = project {
        return framework_cache
            .entry(proj.root.clone())
            .or_insert_with(|| detect_from_project_files(proj))
            .clone();
    }
    None
}

fn detect_from_command(command_line: &str) -> Option<Framework> {
    let cmd = command_line.to_lowercase();

    // JS/TS meta-frameworks — order matters, check specific before generic
    if cmd.contains("next")
        && (cmd.contains("next dev") || cmd.contains("next start") || cmd.contains("next-server"))
    {
        return Some(Framework::Next);
    }
    if cmd.contains("remix") || cmd.contains("@remix-run") {
        return Some(Framework::Remix);
    }
    if cmd.contains("astro") {
        return Some(Framework::Astro);
    }
    if cmd.contains("svelte-kit")
        || cmd.contains("sveltekit")
        || (cmd.contains("svelte") && cmd.contains("dev"))
    {
        return Some(Framework::SvelteKit);
    }
    if cmd.contains("nuxt") {
        return Some(Framework::Nuxt);
    }
    if cmd.contains("gatsby") {
        return Some(Framework::Gatsby);
    }
    if cmd.contains("turbopack") || cmd.contains("--turbo") {
        return Some(Framework::Turbopack);
    }
    if cmd.contains("webpack-dev-server") || cmd.contains("webpack serve") {
        return Some(Framework::Webpack);
    }
    if cmd.contains("vite") {
        return Some(Framework::Vite);
    }
    if cmd.contains("expo") {
        return Some(Framework::Expo);
    }
    if cmd.contains("storybook") {
        return Some(Framework::Storybook);
    }

    // JS/TS server frameworks
    if cmd.contains("nest") && cmd.contains("start") {
        return Some(Framework::Nest);
    }
    if cmd.contains("fastify") {
        return Some(Framework::Fastify);
    }
    if cmd.contains("express") {
        return Some(Framework::Express);
    }

    // Ruby
    if cmd.contains("rails") || cmd.contains("puma") || cmd.contains("unicorn") {
        return Some(Framework::Rails);
    }

    // Python
    if cmd.contains("uvicorn") && cmd.contains("fastapi") || cmd.contains("fastapi") {
        return Some(Framework::FastAPI);
    }
    if cmd.contains("django") || cmd.contains("manage.py") {
        return Some(Framework::Django);
    }
    if cmd.contains("flask") {
        return Some(Framework::Flask);
    }

    // JVM
    if cmd.contains("spring") || cmd.contains("spring-boot") || cmd.contains("org.springframework")
    {
        return Some(Framework::Spring);
    }

    // Go
    if cmd.contains("gin") && cmd.contains("run") {
        return Some(Framework::Gin);
    }

    // Elixir
    if cmd.contains("phx.server") || cmd.contains("phoenix") {
        return Some(Framework::Phoenix);
    }

    // PHP
    if cmd.contains("artisan") || cmd.contains("laravel") {
        return Some(Framework::Laravel);
    }

    // Static site generators
    if cmd.contains("hugo") && (cmd.contains("server") || cmd.contains("serve")) {
        return Some(Framework::Hugo);
    }

    // Rust web frameworks
    if cmd.contains("actix") || cmd.contains("actix-web") {
        return Some(Framework::Actix);
    }
    if cmd.contains("axum") {
        return Some(Framework::Axum);
    }
    if cmd.contains("rocket") {
        return Some(Framework::Rocket);
    }

    None
}

pub(crate) fn detect_from_project_files(project: &Project) -> Option<Framework> {
    let root = &project.root;

    // JS/TS meta-frameworks
    if root.join("next.config.js").exists()
        || root.join("next.config.mjs").exists()
        || root.join("next.config.ts").exists()
    {
        return Some(Framework::Next);
    }
    if root.join("remix.config.js").exists()
        || root.join("remix.config.ts").exists()
        || root.join("app/root.tsx").exists()
    {
        return Some(Framework::Remix);
    }
    if root.join("astro.config.mjs").exists() || root.join("astro.config.ts").exists() {
        return Some(Framework::Astro);
    }
    if root.join("svelte.config.js").exists() || root.join("svelte.config.ts").exists() {
        return Some(Framework::SvelteKit);
    }
    if root.join("nuxt.config.ts").exists() || root.join("nuxt.config.js").exists() {
        return Some(Framework::Nuxt);
    }
    if root.join("gatsby-config.js").exists() || root.join("gatsby-config.ts").exists() {
        return Some(Framework::Gatsby);
    }

    // Build tools / bundlers
    if root.join("vite.config.ts").exists() || root.join("vite.config.js").exists() {
        return Some(Framework::Vite);
    }

    // JS/TS — expo, storybook, nest
    if root.join("app.json").exists() && root.join("node_modules/expo").exists() {
        return Some(Framework::Expo);
    }
    if root.join(".storybook").exists() {
        return Some(Framework::Storybook);
    }
    if root.join("nest-cli.json").exists() {
        return Some(Framework::Nest);
    }

    // Ruby
    if root.join("Gemfile").exists() && root.join("config/routes.rb").exists() {
        return Some(Framework::Rails);
    }

    // Python
    if root.join("manage.py").exists()
        && (root.join("settings.py").exists()
            || root.join("wsgi.py").exists()
            || root.join("urls.py").exists()
            || has_django_in_requirements(root))
    {
        return Some(Framework::Django);
    }

    // PHP
    if root.join("artisan").exists() && root.join("composer.json").exists() {
        return Some(Framework::Laravel);
    }

    // Elixir
    if root.join("mix.exs").exists() && root.join("lib").join("_web").exists() {
        return Some(Framework::Phoenix);
    }

    // Go — harder to detect without parsing imports
    if root.join("go.mod").exists() && root.join("main.go").exists() {
        if let Ok(content) = std::fs::read_to_string(root.join("go.mod")) {
            if content.contains("github.com/gin-gonic/gin") {
                return Some(Framework::Gin);
            }
        }
    }

    // JVM
    if (root.join("pom.xml").exists() || root.join("build.gradle").exists())
        && root.join("src/main/java").exists()
    {
        return Some(Framework::Spring);
    }

    // Static site generators
    if root.join("hugo.toml").exists()
        || root.join("hugo.yaml").exists()
        || root.join("config.toml").exists()
            && root.join("content").exists()
            && root.join("themes").exists()
    {
        return Some(Framework::Hugo);
    }

    None
}

fn has_django_in_requirements(root: &std::path::Path) -> bool {
    for file in &["requirements.txt", "Pipfile", "pyproject.toml"] {
        if let Ok(content) = std::fs::read_to_string(root.join(file)) {
            if content.to_lowercase().contains("django") {
                return true;
            }
        }
    }
    false
}
