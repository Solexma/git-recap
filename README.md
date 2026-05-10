# git-recap

Hook-driven, local-first activity reporter for git repos. Foundation data layer for [gitnapped](https://github.com/Solexma/gitnapped).

## What it does

Installs a post-commit hook that snapshots your last N commits into a structured TOML report. Run `git recap digest` to get an instant overview of all your repos — no scanning, no runtime git log.

## Install

```sh
cargo install git-recap
```

Or build from source:

```sh
git clone https://github.com/Solexma/git-recap
cd git-recap
cargo install --path .
```

## Quick start

```sh
# In any git repo:
git recap install      # installs post-commit hook, registers repo

# Make some commits... the hook runs automatically.
# Or run manually:
git recap this         # recap last 10 commits

# Check this repo:
git recap status

# Check all repos:
git recap digest all
git recap digest since yesterday
git recap digest since last-week --json
```

## Commands

### Per-repo

```
git recap install              # install post-commit hook, register repo
git recap uninstall            # remove hook, deregister
git recap touch                # manual "I'm here" ping (no commit needed)
git recap this                 # recap last N commits into report
git recap this --count 20      # recap last 20, save count for this repo
git recap this --default       # clear per-repo count, fall back to default
git recap status               # show this repo's report
git recap info                 # version, author, project links
```

### Cross-repo

```
git recap digest all                       # summary of all registered repos
git recap digest all --json                # machine-readable output
git recap digest since yesterday           # filter by activity date
git recap digest since today
git recap digest since last-week           # last 7 days
git recap digest since last-month          # last 30 days
git recap digest since 7d                  # last N days
git recap digest since 20260101            # exact date (YYYYMMDD)
git recap digest since 2026-01-01          # exact date (YYYY-MM-DD)
git recap digest since last-week --json    # any filter + JSON
```

## How it works

```
git commit (any repo)
    → post-commit hook fires
    → git-recap this
    → snapshots last N commits to ~/.config/git-recap/<sha>.toml

git recap digest since yesterday
    → reads all report files (no git operations)
    → outputs compact summary to stdout
```

Reports are stored at platform-specific locations:
- **macOS:** `~/Library/Application Support/git-recap/`
- **Linux:** `~/.local/share/git-recap/` (data), `~/.config/git-recap/` (config)

No files are created inside your repos (only `.git/hooks/post-commit` is modified).

## Report format

```toml
[repo]
path = "/path/to/repo"
name = "my-project"
sha = "a1b2c3d4"

[activity]
last_touched = 2026-03-25T16:45:00+01:00

[[activity.commits]]
sha = "abc123"
date = 2026-03-25T15:30:00+01:00
message = "fix: something"
branch = "main"
author = "Marco Orlandin"
```

## Registry

Per-repo commit count can be configured:

```toml
default_count = 10

[[repos]]
sha = "a1b2c3d4"
path = "/path/to/repo"

[[repos]]
sha = "f5e6d7c8"
path = "/other/repo"
count = 20
```

## Tech

- Rust, shells out to git CLI (no git2 library)
- Reuses patterns from [git-side](https://github.com/Solexma/git-side)
- MIT License — [Solexma](https://github.com/Solexma)

## License

MIT
