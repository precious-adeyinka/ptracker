# ptracker

A CLI tool for tracking projects, file activity, and hours for freelancers.

### Features

- `init` — initialize a new project with global storage in `~/.ptracker`
- `watch` — real-time file system monitoring with automatic session tracking
- `log` — manually log hours to a project
- `summary` — view project details, hours, and today's session activity
- `list` — show all projects across your machine in a table
- `rename` — rename a project globally
- `delete` — remove a project and all its data

|                 |                                             |
| --------------: | ------------------------------------------- |
|  Latest Version | [![Latest version][badge-version]][crate]   |
| Crate Downloads | [![Crate downloads][badge-crate-dl]][crate] |
|         License | [![Crate license][badge-license]][github]   |

<details>
<summary><strong>Table of Contents</strong></summary>

- [Installation](#installation)
- [Usage](#usage)
- [How it works](#how-it-works)
- [Data](#data)
- [Release History](#release-history)
- [Authors](#authors)
- [License](#license)

</details>

## Installation

```bash
cargo install ptracker
```

## Usage

```bash
# Initialize a new project
ptracker init "My Project"

# Initialize with a specific folder to watch
ptracker init "My Project" --path ./src

# Watch a folder and track file activity
ptracker watch "My Project"

# Watch a specific folder for this session
ptracker watch "My Project" --path ./src

# Log hours manually
ptracker log "My Project" 3.5

# View project summary with session breakdown
ptracker summary "My Project"

# List all projects
ptracker list

# Rename a project
ptracker rename "My Project" "New Name"

# Delete a project
ptracker delete "My Project"

# Show version
ptracker --version
```

## How it works

ptracker watches your project folders in real time, logging every file creation,
modification, rename, and deletion. Each work session is automatically timed —
start with `watch`, stop with Ctrl+C, and ptracker saves the session duration,
file event totals, and activity log automatically.

All data lives in `~/.ptracker` so your projects are accessible from any
terminal on your machine.

## Data

All project data is stored globally in your home directory:

```
~/.ptracker/
├── projects.json               # global project registry
└── projects/
    └── my-project/
        ├── project.json        # project metadata and totals
        ├── sessions/
        │   └── 2026-06-28.json # per-day session file
        └── logs/
            └── 2026-06-28.log  # timestamped activity log
```

## Release History

See the [changelog](https://github.com/precious-adeyinka/ptracker/blob/main/CHANGELOG.md) for a full release history.

## Authors

Created and maintained by [Precious Adeyinka](https://github.com/precious-adeyinka).

## License

Licensed under the MIT license — see [LICENSE](https://github.com/precious-adeyinka/ptracker/blob/main/LICENSE.txt) for details.

[badge-crate-dl]: https://img.shields.io/crates/d/ptracker.svg?style=flat-square
[badge-license]: https://img.shields.io/crates/l/ptracker.svg?style=flat-square
[badge-version]: https://img.shields.io/crates/v/ptracker.svg?style=flat-square
[crate]: https://crates.io/crates/ptracker
[github]: https://github.com/precious-adeyinka/ptracker
