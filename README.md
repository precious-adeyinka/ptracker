# ptracker

A CLI tool for tracking projects, file activity, and hours for freelancers.

## Installation

```bash
cargo install project-tracker
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

ptracker watches your project folders in real time, logging every file creation, modification, rename, and deletion. All activity is saved to a JSON project file and a human-readable log file so you can prove to clients exactly what you built and when.

## Data

Each project creates two files in your current directory:

- `projectname.json` — structured project data
- `projectname.log` — timestamped activity log

## License

MIT
