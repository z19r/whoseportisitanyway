use std::collections::HashMap;
use std::path::PathBuf;

use crate::model::{Framework, Project};

pub fn detect_framework(
    _process_name: &str,
    command_line: &str,
    parent_command_line: Option<&str>,
    project: &Option<Project>,
    framework_cache: &mut HashMap<PathBuf, Option<Framework>>,
) -> Option<Framework> {
    if let Some(fw) = detect_from_command(command_line) {
        return Some(fw);
    }
    if let Some(fw) = detect_from_bin_path(command_line) {
        return Some(fw);
    }
    if let Some(fw) = detect_from_token_pairs(command_line) {
        return Some(fw);
    }
    if let Some(parent) = parent_command_line {
        if let Some(fw) = detect_from_command(parent) {
            return Some(fw);
        }
        if let Some(fw) = detect_from_token_pairs(parent) {
            return Some(fw);
        }
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

fn detect_from_bin_path(command_line: &str) -> Option<Framework> {
    let needle = "node_modules/.bin/";
    let pos = command_line.find(needle)?;
    let after = &command_line[pos + needle.len()..];
    let tool = after
        .split(|c: char| c.is_whitespace() || c == '/')
        .next()?;
    match_tool_name(tool)
}

fn detect_from_token_pairs(command_line: &str) -> Option<Framework> {
    let tokens: Vec<&str> = command_line.split_whitespace().collect();
    if tokens.len() < 2 {
        return None;
    }

    let pairs: &[(&[&str], Framework)] = &[
        (&["next", "dev"], Framework::Next),
        (&["next", "start"], Framework::Next),
        (&["nuxt", "dev"], Framework::Nuxt),
        (&["nuxt", "start"], Framework::Nuxt),
        (&["expo", "start"], Framework::Expo),
        (&["gatsby", "develop"], Framework::Gatsby),
        (&["gatsby", "serve"], Framework::Gatsby),
        (&["astro", "dev"], Framework::Astro),
        (&["remix", "dev"], Framework::Remix),
        (&["vite", "dev"], Framework::Vite),
        (&["vite", "build"], Framework::Vite),
        (&["nest", "start"], Framework::Nest),
        (&["hugo", "server"], Framework::Hugo),
        (&["hugo", "serve"], Framework::Hugo),
        (&["rails", "server"], Framework::Rails),
        (&["rails", "s"], Framework::Rails),
        (&["flask", "run"], Framework::Flask),
        (&["uvicorn", "main:app"], Framework::FastAPI),
    ];

    for (pair, fw) in pairs {
        let a = pair[0];
        let b = pair[1];
        for window in tokens.windows(2) {
            let t0 = window[0].rsplit('/').next().unwrap_or(window[0]);
            if t0.eq_ignore_ascii_case(a) && window[1].eq_ignore_ascii_case(b) {
                return Some(fw.clone());
            }
        }
    }
    None
}

fn match_tool_name(tool: &str) -> Option<Framework> {
    match tool.to_lowercase().as_str() {
        "next" => Some(Framework::Next),
        "nuxt" | "nuxi" => Some(Framework::Nuxt),
        "remix" | "remix-serve" => Some(Framework::Remix),
        "astro" => Some(Framework::Astro),
        "vite" => Some(Framework::Vite),
        "gatsby" => Some(Framework::Gatsby),
        "expo" | "expo-cli" => Some(Framework::Expo),
        "storybook" | "start-storybook" => Some(Framework::Storybook),
        "nest" => Some(Framework::Nest),
        "fastify" => Some(Framework::Fastify),
        "hugo" => Some(Framework::Hugo),
        "turbopack" | "next-swc" => Some(Framework::Turbopack),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // detect_from_command — JS/TS meta-frameworks
    #[test]
    fn cmd_next_dev() {
        assert_eq!(detect_from_command("node next dev"), Some(Framework::Next));
    }

    #[test]
    fn cmd_next_start() {
        assert_eq!(
            detect_from_command("node next start"),
            Some(Framework::Next)
        );
    }

    #[test]
    fn cmd_next_server() {
        assert_eq!(
            detect_from_command("/app/node_modules/.bin/next-server"),
            Some(Framework::Next)
        );
    }

    #[test]
    fn cmd_remix() {
        assert_eq!(
            detect_from_command("remix dev --port 3000"),
            Some(Framework::Remix)
        );
    }

    #[test]
    fn cmd_remix_run() {
        assert_eq!(
            detect_from_command("npx @remix-run/dev"),
            Some(Framework::Remix)
        );
    }

    #[test]
    fn cmd_astro() {
        assert_eq!(
            detect_from_command("astro dev --host"),
            Some(Framework::Astro)
        );
    }

    #[test]
    fn cmd_sveltekit_hyphen() {
        assert_eq!(
            detect_from_command("svelte-kit dev"),
            Some(Framework::SvelteKit)
        );
    }

    #[test]
    fn cmd_sveltekit_combined() {
        assert_eq!(
            detect_from_command("sveltekit dev"),
            Some(Framework::SvelteKit)
        );
    }

    #[test]
    fn cmd_svelte_dev() {
        assert_eq!(
            detect_from_command("svelte dev"),
            Some(Framework::SvelteKit)
        );
    }

    #[test]
    fn cmd_nuxt() {
        assert_eq!(
            detect_from_command("nuxt dev --port 3000"),
            Some(Framework::Nuxt)
        );
    }

    #[test]
    fn cmd_gatsby() {
        assert_eq!(
            detect_from_command("gatsby develop"),
            Some(Framework::Gatsby)
        );
    }

    #[test]
    fn cmd_turbopack_direct() {
        assert_eq!(
            detect_from_command("turbopack --watch"),
            Some(Framework::Turbopack)
        );
    }

    #[test]
    fn cmd_webpack_dev_server() {
        assert_eq!(
            detect_from_command("webpack-dev-server --port 8080"),
            Some(Framework::Webpack)
        );
    }

    #[test]
    fn cmd_webpack_serve() {
        assert_eq!(
            detect_from_command("webpack serve"),
            Some(Framework::Webpack)
        );
    }

    #[test]
    fn cmd_vite() {
        assert_eq!(
            detect_from_command("vite --port 5173"),
            Some(Framework::Vite)
        );
    }

    #[test]
    fn cmd_expo() {
        assert_eq!(detect_from_command("expo start"), Some(Framework::Expo));
    }

    #[test]
    fn cmd_storybook() {
        assert_eq!(
            detect_from_command("start-storybook -p 6006"),
            Some(Framework::Storybook)
        );
    }

    #[test]
    fn cmd_nest_start() {
        assert_eq!(
            detect_from_command("nest start --watch"),
            Some(Framework::Nest)
        );
    }

    #[test]
    fn cmd_fastify() {
        assert_eq!(
            detect_from_command("fastify start"),
            Some(Framework::Fastify)
        );
    }

    #[test]
    fn cmd_express() {
        assert_eq!(
            detect_from_command("node express server.js"),
            Some(Framework::Express)
        );
    }

    #[test]
    fn cmd_rails() {
        assert_eq!(
            detect_from_command("rails server -p 3000"),
            Some(Framework::Rails)
        );
    }

    #[test]
    fn cmd_puma() {
        assert_eq!(
            detect_from_command("puma -C config/puma.rb"),
            Some(Framework::Rails)
        );
    }

    #[test]
    fn cmd_fastapi() {
        assert_eq!(
            detect_from_command("uvicorn main:app --reload fastapi"),
            Some(Framework::FastAPI)
        );
    }

    #[test]
    fn cmd_django() {
        assert_eq!(
            detect_from_command("python manage.py runserver"),
            Some(Framework::Django)
        );
    }

    #[test]
    fn cmd_flask() {
        assert_eq!(
            detect_from_command("flask run --host 0.0.0.0"),
            Some(Framework::Flask)
        );
    }

    #[test]
    fn cmd_spring() {
        assert_eq!(
            detect_from_command("java -jar spring-boot-app.jar"),
            Some(Framework::Spring)
        );
    }

    #[test]
    fn cmd_gin_run() {
        assert_eq!(detect_from_command("gin run main.go"), Some(Framework::Gin));
    }

    #[test]
    fn cmd_phoenix_server() {
        assert_eq!(
            detect_from_command("mix phx.server"),
            Some(Framework::Phoenix)
        );
    }

    #[test]
    fn cmd_laravel_artisan() {
        assert_eq!(
            detect_from_command("php artisan serve"),
            Some(Framework::Laravel)
        );
    }

    #[test]
    fn cmd_hugo_server() {
        assert_eq!(detect_from_command("hugo server -D"), Some(Framework::Hugo));
    }

    #[test]
    fn cmd_actix() {
        assert_eq!(
            detect_from_command("./target/debug/actix-web-app"),
            Some(Framework::Actix)
        );
    }

    #[test]
    fn cmd_axum() {
        assert_eq!(
            detect_from_command("./target/release/axum-server"),
            Some(Framework::Axum)
        );
    }

    #[test]
    fn cmd_rocket() {
        assert_eq!(
            detect_from_command("./rocket-app --port 8000"),
            Some(Framework::Rocket)
        );
    }

    #[test]
    fn cmd_no_match() {
        assert_eq!(detect_from_command("python myscript.py"), None);
    }

    // detect_from_bin_path
    #[test]
    fn bin_path_next() {
        assert_eq!(
            detect_from_bin_path("/app/node_modules/.bin/next dev"),
            Some(Framework::Next)
        );
    }

    #[test]
    fn bin_path_vite() {
        assert_eq!(
            detect_from_bin_path("/proj/node_modules/.bin/vite --port 3000"),
            Some(Framework::Vite)
        );
    }

    #[test]
    fn bin_path_no_match() {
        assert_eq!(detect_from_bin_path("/usr/bin/python3 server.py"), None);
    }

    #[test]
    fn bin_path_unknown_tool() {
        assert_eq!(
            detect_from_bin_path("/app/node_modules/.bin/unknown-tool"),
            None
        );
    }

    // detect_from_token_pairs
    #[test]
    fn token_pair_next_dev() {
        assert_eq!(
            detect_from_token_pairs("node next dev"),
            Some(Framework::Next)
        );
    }

    #[test]
    fn token_pair_rails_s() {
        assert_eq!(detect_from_token_pairs("rails s"), Some(Framework::Rails));
    }

    #[test]
    fn token_pair_flask_run() {
        assert_eq!(detect_from_token_pairs("flask run"), Some(Framework::Flask));
    }

    #[test]
    fn token_pair_uvicorn_fastapi() {
        assert_eq!(
            detect_from_token_pairs("uvicorn main:app"),
            Some(Framework::FastAPI)
        );
    }

    #[test]
    fn token_pair_hugo_server() {
        assert_eq!(
            detect_from_token_pairs("hugo server"),
            Some(Framework::Hugo)
        );
    }

    #[test]
    fn token_pair_single_token_no_match() {
        assert_eq!(detect_from_token_pairs("node"), None);
    }

    // match_tool_name
    #[test]
    fn tool_name_all_variants() {
        assert_eq!(match_tool_name("next"), Some(Framework::Next));
        assert_eq!(match_tool_name("nuxt"), Some(Framework::Nuxt));
        assert_eq!(match_tool_name("nuxi"), Some(Framework::Nuxt));
        assert_eq!(match_tool_name("remix"), Some(Framework::Remix));
        assert_eq!(match_tool_name("remix-serve"), Some(Framework::Remix));
        assert_eq!(match_tool_name("astro"), Some(Framework::Astro));
        assert_eq!(match_tool_name("vite"), Some(Framework::Vite));
        assert_eq!(match_tool_name("gatsby"), Some(Framework::Gatsby));
        assert_eq!(match_tool_name("expo"), Some(Framework::Expo));
        assert_eq!(match_tool_name("expo-cli"), Some(Framework::Expo));
        assert_eq!(match_tool_name("storybook"), Some(Framework::Storybook));
        assert_eq!(
            match_tool_name("start-storybook"),
            Some(Framework::Storybook)
        );
        assert_eq!(match_tool_name("nest"), Some(Framework::Nest));
        assert_eq!(match_tool_name("fastify"), Some(Framework::Fastify));
        assert_eq!(match_tool_name("hugo"), Some(Framework::Hugo));
        assert_eq!(match_tool_name("turbopack"), Some(Framework::Turbopack));
        assert_eq!(match_tool_name("next-swc"), Some(Framework::Turbopack));
    }

    #[test]
    fn tool_name_case_insensitive() {
        assert_eq!(match_tool_name("NEXT"), Some(Framework::Next));
        assert_eq!(match_tool_name("Vite"), Some(Framework::Vite));
    }

    #[test]
    fn tool_name_unknown() {
        assert_eq!(match_tool_name("webpack"), None);
        assert_eq!(match_tool_name("esbuild"), None);
    }

    // detect_from_project_files — uses tempdir
    #[test]
    fn project_files_next() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("next.config.js"), "").unwrap();
        let proj = Project {
            name: "test".into(),
            root: dir.path().to_path_buf(),
            framework: None,
        };
        assert_eq!(detect_from_project_files(&proj), Some(Framework::Next));
    }

    #[test]
    fn project_files_remix() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("remix.config.js"), "").unwrap();
        let proj = Project {
            name: "test".into(),
            root: dir.path().to_path_buf(),
            framework: None,
        };
        assert_eq!(detect_from_project_files(&proj), Some(Framework::Remix));
    }

    #[test]
    fn project_files_astro() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("astro.config.mjs"), "").unwrap();
        let proj = Project {
            name: "test".into(),
            root: dir.path().to_path_buf(),
            framework: None,
        };
        assert_eq!(detect_from_project_files(&proj), Some(Framework::Astro));
    }

    #[test]
    fn project_files_sveltekit() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("svelte.config.js"), "").unwrap();
        let proj = Project {
            name: "test".into(),
            root: dir.path().to_path_buf(),
            framework: None,
        };
        assert_eq!(detect_from_project_files(&proj), Some(Framework::SvelteKit));
    }

    #[test]
    fn project_files_nuxt() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("nuxt.config.ts"), "").unwrap();
        let proj = Project {
            name: "test".into(),
            root: dir.path().to_path_buf(),
            framework: None,
        };
        assert_eq!(detect_from_project_files(&proj), Some(Framework::Nuxt));
    }

    #[test]
    fn project_files_gatsby() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("gatsby-config.js"), "").unwrap();
        let proj = Project {
            name: "test".into(),
            root: dir.path().to_path_buf(),
            framework: None,
        };
        assert_eq!(detect_from_project_files(&proj), Some(Framework::Gatsby));
    }

    #[test]
    fn project_files_vite() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("vite.config.ts"), "").unwrap();
        let proj = Project {
            name: "test".into(),
            root: dir.path().to_path_buf(),
            framework: None,
        };
        assert_eq!(detect_from_project_files(&proj), Some(Framework::Vite));
    }

    #[test]
    fn project_files_storybook() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".storybook")).unwrap();
        let proj = Project {
            name: "test".into(),
            root: dir.path().to_path_buf(),
            framework: None,
        };
        assert_eq!(detect_from_project_files(&proj), Some(Framework::Storybook));
    }

    #[test]
    fn project_files_rails() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Gemfile"), "").unwrap();
        std::fs::create_dir_all(dir.path().join("config")).unwrap();
        std::fs::write(dir.path().join("config/routes.rb"), "").unwrap();
        let proj = Project {
            name: "test".into(),
            root: dir.path().to_path_buf(),
            framework: None,
        };
        assert_eq!(detect_from_project_files(&proj), Some(Framework::Rails));
    }

    #[test]
    fn project_files_django() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("manage.py"), "").unwrap();
        std::fs::write(dir.path().join("settings.py"), "").unwrap();
        let proj = Project {
            name: "test".into(),
            root: dir.path().to_path_buf(),
            framework: None,
        };
        assert_eq!(detect_from_project_files(&proj), Some(Framework::Django));
    }

    #[test]
    fn project_files_django_via_requirements() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("manage.py"), "").unwrap();
        std::fs::write(dir.path().join("requirements.txt"), "Django==4.2\n").unwrap();
        let proj = Project {
            name: "test".into(),
            root: dir.path().to_path_buf(),
            framework: None,
        };
        assert_eq!(detect_from_project_files(&proj), Some(Framework::Django));
    }

    #[test]
    fn project_files_laravel() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("artisan"), "").unwrap();
        std::fs::write(dir.path().join("composer.json"), "{}").unwrap();
        let proj = Project {
            name: "test".into(),
            root: dir.path().to_path_buf(),
            framework: None,
        };
        assert_eq!(detect_from_project_files(&proj), Some(Framework::Laravel));
    }

    #[test]
    fn project_files_gin() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("go.mod"),
            "module myapp\nrequire github.com/gin-gonic/gin v1.9\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("main.go"), "package main").unwrap();
        let proj = Project {
            name: "test".into(),
            root: dir.path().to_path_buf(),
            framework: None,
        };
        assert_eq!(detect_from_project_files(&proj), Some(Framework::Gin));
    }

    #[test]
    fn project_files_spring() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("pom.xml"), "<project/>").unwrap();
        std::fs::create_dir_all(dir.path().join("src/main/java")).unwrap();
        let proj = Project {
            name: "test".into(),
            root: dir.path().to_path_buf(),
            framework: None,
        };
        assert_eq!(detect_from_project_files(&proj), Some(Framework::Spring));
    }

    #[test]
    fn project_files_hugo() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("hugo.toml"), "").unwrap();
        let proj = Project {
            name: "test".into(),
            root: dir.path().to_path_buf(),
            framework: None,
        };
        assert_eq!(detect_from_project_files(&proj), Some(Framework::Hugo));
    }

    #[test]
    fn project_files_no_match() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("README.md"), "").unwrap();
        let proj = Project {
            name: "test".into(),
            root: dir.path().to_path_buf(),
            framework: None,
        };
        assert_eq!(detect_from_project_files(&proj), None);
    }

    // detect_framework — integration
    #[test]
    fn detect_framework_command_takes_priority() {
        let mut cache = HashMap::new();
        let result = detect_framework("node", "next dev --port 3000", None, &None, &mut cache);
        assert_eq!(result, Some(Framework::Next));
    }

    #[test]
    fn detect_framework_parent_fallback() {
        let mut cache = HashMap::new();
        let result = detect_framework(
            "node",
            "node server.js",
            Some("rails server -p 3000"),
            &None,
            &mut cache,
        );
        assert_eq!(result, Some(Framework::Rails));
    }

    #[test]
    fn detect_framework_project_fallback() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("vite.config.ts"), "").unwrap();
        let proj = Some(Project {
            name: "test".into(),
            root: dir.path().to_path_buf(),
            framework: None,
        });
        let mut cache = HashMap::new();
        let result = detect_framework("node", "node server.js", None, &proj, &mut cache);
        assert_eq!(result, Some(Framework::Vite));
    }

    #[test]
    fn detect_framework_cache_hit() {
        let dir = tempfile::tempdir().unwrap();
        let proj = Some(Project {
            name: "test".into(),
            root: dir.path().to_path_buf(),
            framework: None,
        });
        let mut cache = HashMap::new();
        cache.insert(dir.path().to_path_buf(), Some(Framework::Remix));
        let result = detect_framework("node", "node server.js", None, &proj, &mut cache);
        assert_eq!(result, Some(Framework::Remix));
    }

    #[test]
    fn detect_framework_none_when_nothing_matches() {
        let mut cache = HashMap::new();
        let result = detect_framework("python", "python myscript.py", None, &None, &mut cache);
        assert_eq!(result, None);
    }

    // has_django_in_requirements
    #[test]
    fn has_django_in_requirements_txt() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("requirements.txt"), "Django>=4.0\n").unwrap();
        assert!(has_django_in_requirements(dir.path()));
    }

    #[test]
    fn no_django_in_requirements() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("requirements.txt"), "flask==2.0\n").unwrap();
        assert!(!has_django_in_requirements(dir.path()));
    }

    #[test]
    fn no_requirements_files() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!has_django_in_requirements(dir.path()));
    }
}
