# Changelog

## [0.2.0] - 2026-06-28

### Added

- Global `~/.ptracker` home directory storage — projects accessible from any terminal
- `projects.json` global registry tracking all your projects in one place
- `list` command — shows all projects in a formatted table
- Automatic session tracking — start time, end time, and duration saved on every `watch`
- Per-day session files (`sessions/YYYY-MM-DD.json`) with full event breakdown
- Per-day activity logs (`logs/YYYY-MM-DD.log`) with timestamped entries
- File event totals per session — created, modified, deleted, renamed counts
- Graceful Ctrl+C shutdown — session is saved automatically when you stop watching
- `--path` flag on `init` and `watch` to specify or override the watch folder
- `--version` / `-V` flag
- Duplicate project guard on `init`

### Changed

- Projects now stored in `~/.ptracker/projects/{slug}/` instead of current directory
- `summary` now shows today's session breakdown in addition to project totals
- `watch` now accepts `--path` as an optional flag instead of a positional argument

### Removed

- Per-project `.json` and `.log` files in the working directory — all data now lives in `~/.ptracker`

## [0.1.0] - 2026-06-27

The initial release.
