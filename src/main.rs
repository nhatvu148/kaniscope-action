//! GitHub Action entrypoint for **Kaniscope** AI code review.
//!
//! Runs inside a `pull_request` workflow: it reads the PR from the Actions
//! context, reviews its unified diff with [`pr_review_core`], and posts a
//! line-anchored inline review plus a summary comment. Action inputs (`INPUT_*`)
//! are mapped onto the env vars the engine reads, then [`Config::from_env`] loads
//! the rest of the configuration.

use pr_review_core::config::Config;
use pr_review_core::review::{run_review, RunReviewInput};

/// Copy a GitHub Action input into the env var `pr_review_core` reads (when the
/// input is present and non-empty). GitHub exposes inputs as `INPUT_<NAME>` with
/// the name upper-cased (hyphens preserved).
fn map_input(input: &str, env: &str) {
    if let Ok(v) = std::env::var(format!("INPUT_{}", input.to_uppercase())) {
        let v = v.trim();
        if !v.is_empty() {
            // Safe on the 2021 edition; runs before anything reads the env
            // concurrently (single-threaded prologue of `main`).
            std::env::set_var(env, v);
        }
    }
}

/// Read a boolean-ish action input (`true`/`1`/`yes`/`on`).
fn input_flag(input: &str) -> bool {
    std::env::var(format!("INPUT_{}", input.to_uppercase()))
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

/// The PR number, from the Actions event payload (preferred) or `GITHUB_REF`.
fn pr_number() -> Option<u64> {
    if let Ok(path) = std::env::var("GITHUB_EVENT_PATH") {
        if let Ok(txt) = std::fs::read_to_string(&path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                for ptr in ["/pull_request/number", "/number", "/issue/number"] {
                    if let Some(n) = v.pointer(ptr).and_then(serde_json::Value::as_u64) {
                        return Some(n);
                    }
                }
            }
        }
    }
    // Fallback: `pull_request` events set GITHUB_REF = refs/pull/<n>/merge.
    std::env::var("GITHUB_REF").ok().and_then(|r| {
        r.strip_prefix("refs/pull/")
            .and_then(|s| s.split_once('/'))
            .and_then(|(n, _)| n.parse().ok())
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    map_input("openrouter-api-key", "OPENROUTER_API_KEY");
    map_input("model", "OPENROUTER_MODEL");
    map_input("github-token", "GH_TOKEN");
    map_input("max-diff-chars", "MAX_DIFF_CHARS");
    map_input("max-tokens", "OPENROUTER_MAX_TOKENS");
    map_input("comment-marker", "COMMENT_MARKER");
    let dry_run = input_flag("dry-run");

    let repo = std::env::var("GITHUB_REPOSITORY").map_err(|_| {
        anyhow::anyhow!("GITHUB_REPOSITORY is unset — this action must run inside GitHub Actions.")
    })?;
    let pr = pr_number().ok_or_else(|| {
        anyhow::anyhow!(
            "Couldn't determine the PR number — run this action on `pull_request` events."
        )
    })?;

    let cfg = Config::from_env();
    if cfg.openrouter_api_key.is_empty() {
        anyhow::bail!(
            "Missing `openrouter-api-key` input (OpenRouter key). Add it as a repo secret."
        );
    }

    println!(
        "Kaniscope: reviewing {repo}#{pr}{}…",
        if dry_run { " (dry-run)" } else { "" }
    );
    let out = match run_review(
        &cfg,
        RunReviewInput {
            provider: "github".into(),
            repo: repo.clone(),
            pr,
            dry_run,
            placeholder: !dry_run,
        },
    )
    .await
    {
        Ok(out) => out,
        Err(e) => {
            // A non-dry-run posted a "⏳ Reviewing this PR…" placeholder before the
            // slow call. Bailing here would leave it on the PR promising a review
            // that will never arrive — and unlike the long-running bots, this action
            // never runs again to replace it. Say what happened instead.
            //
            // Only when a placeholder was actually posted: core's upsert CREATES the
            // summary comment when none exists, so a dry-run retraction would post a
            // failure notice onto a PR that was never commented on.
            if !dry_run {
                pr_review_core::review::post_review_failure(
                    "github",
                    &cfg,
                    &repo,
                    pr,
                    &format!("{e:#}"),
                )
                .await;
            }
            return Err(e);
        }
    };

    println!(
        "Kaniscope: {} finding(s) · recommendation: {} · model {}",
        out.findings, out.recommendation, out.model
    );
    if dry_run {
        println!("\n{}", out.summary_markdown);
    } else if let Some(url) = out.comment_url {
        println!("Posted: {url}");
    }
    Ok(())
}
