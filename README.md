# ptracker

A CLI tool for tracking projects, file activity, and hours for freelancers.

### Features

- `init` — initialize a new project
- `watch` — real-time file system monitoring with deduplication
- `log` — manually log hours to a project
- `summary` — view project details and activity logs in a table
- `rename` — rename a project and its associated files
- `delete` — remove a project and its logs

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

# Watch a folder and track file activity
ptracker watch "My Project" "./src"

# Log hours manually
ptracker log "My Project" 3.5

# View project summary
ptracker summary "My Project"

# Rename a project
ptracker rename "My Project" "New Name"

# Delete a project
ptracker delete "My Project"
```

## How it works

ptracker watches your project folders in real time, logging every file creation,
modification, rename, and deletion. All activity is saved to a JSON project file
and a human-readable log file so you can prove to clients exactly what you built
and when.

## Data

Each project creates two files in your current directory:

- `projectname.json` — structured project data
- `projectname.log` — timestamped activity log

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
