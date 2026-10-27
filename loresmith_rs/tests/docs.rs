//! Documentation guards.
//!
//! Relative links in the README and the manuals have to keep resolving when
//! files move, and the repository root links are what the release page and the
//! Installation Guide rely on.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn documentation_files() -> Vec<PathBuf> {
    let root = repo_root();
    let mut files = vec![
        root.join("README.md"),
        root.join("LAUNCH.md"),
        root.join("release/README.md"),
        root.join("examples/README.md"),
    ];
    let mut docs: Vec<PathBuf> = std::fs::read_dir(root.join("docs"))
        .expect("docs directory should exist")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
        .collect();
    docs.sort();
    files.extend(docs);
    files
}

/// Strip fenced code blocks so example snippets are not treated as links.
fn prose(text: &str) -> String {
    let mut out = String::new();
    let mut in_fence = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

fn markdown_targets(text: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut rest = text;
    while let Some(index) = rest.find("](") {
        let after = &rest[index + 2..];
        match after.find(')') {
            Some(end) => {
                targets.push(after[..end].to_string());
                rest = &after[end..];
            }
            None => break,
        }
    }
    targets
}

fn is_external(target: &str) -> bool {
    target.is_empty()
        || target.starts_with('#')
        || target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("mailto:")
        || target.starts_with('<')
}

#[test]
fn relative_markdown_links_resolve() {
    let mut broken: Vec<String> = Vec::new();
    let mut checked = 0;

    for path in documentation_files() {
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
        let base = path.parent().expect("document parent");
        for target in markdown_targets(&prose(&text)) {
            let target = target.split_whitespace().next().unwrap_or("").to_string();
            let target = target.split('#').next().unwrap_or("").to_string();
            if is_external(&target) {
                continue;
            }
            checked += 1;
            if !base.join(&target).exists() {
                broken.push(format!("{} -> {target}", path.display()));
            }
        }
    }

    assert!(
        checked > 20,
        "expected the documentation to contain relative links, saw {checked}"
    );
    assert!(
        broken.is_empty(),
        "these documentation links point at missing files:\n  {}",
        broken.join("\n  ")
    );
}
