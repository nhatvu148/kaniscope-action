# 🦀 Kaniscope — AI Code Review

A GitHub Action that reviews your pull requests with an AI model and posts a
**line-anchored inline review + a summary comment** — powered by the open-source
[`pr-review-core`](https://crates.io/crates/pr-review-core) Rust engine.

- **Advisory, never blocking** — it comments, it doesn't gate merges or edit code.
- **Cheap** — runs on [OpenRouter](https://openrouter.ai) with the model you choose; pennies per PR on a light model.
- **Self-contained** — a small Docker container action; no external service to trust with your code beyond OpenRouter → the model.

> Try the engine live, no setup, at the **[Kaniscope playground](https://kaniscope.nvnv.app)** — paste a diff or a PR URL and see a review.

## Usage

Add `.github/workflows/kaniscope.yml`:

```yaml
name: Kaniscope review
on:
  pull_request:
    types: [opened, reopened, synchronize]

permissions:
  contents: read
  pull-requests: write        # required to post the review

jobs:
  review:
    runs-on: ubuntu-latest
    steps:
      - uses: nhatvu148/kaniscope-action@v1
        with:
          openrouter-api-key: ${{ secrets.OPENROUTER_API_KEY }}
          # model: moonshotai/kimi-k2-0905   # optional; cheap default
```

Then add your OpenRouter key as a repo secret named `OPENROUTER_API_KEY`
(**Settings → Secrets and variables → Actions**).

## Inputs

| Input | Required | Default | Description |
| --- | --- | --- | --- |
| `openrouter-api-key` | **yes** | — | OpenRouter key that funds the review ([get one](https://openrouter.ai/keys)). |
| `model` | no | `moonshotai/kimi-k2-0905` | OpenRouter model. Use a stronger one (e.g. `anthropic/claude-sonnet-4.5`) for higher quality at higher cost. |
| `github-token` | no | `${{ github.token }}` | Token to read the diff and post the review (needs `pull-requests: write`). |
| `dry-run` | no | `false` | Generate the review and print it to the log **without** posting. |
| `max-diff-chars` | no | engine default | Truncate large diffs before sending to the model (cost cap). |
| `max-tokens` | no | engine default | Max output tokens per review call (cost cap). |

## Notes

- **Permissions:** the job needs `pull-requests: write` (shown above) so the
  default token can post the review.
- **Cost:** each review is one model call, capped by `max-tokens` /
  `max-diff-chars`. A light model keeps it to pennies; re-reviews update the
  bot's own comments rather than stacking.
- **Data:** your PR diff is sent to OpenRouter → the model provider. Fine for
  your own/public repos; clear it with whoever owns security/IP before pointing
  it at proprietary code.

## How it works

The action reads the PR from the Actions context, then calls
`pr_review_core::review::run_review` — which fetches the diff, reviews it, and
posts inline + summary comments. The engine is a separate, reusable crate:
[`pr-review-core`](https://crates.io/crates/pr-review-core).

## License

Dual-licensed under **MIT OR Apache-2.0** (the Rust convention, matching
[`pr-review-core`](https://crates.io/crates/pr-review-core)) — pick whichever you
prefer. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).

© 2026 nhatvu148

<!-- self-test trigger -->
