# Glaze Components

Glaze Components aims to be a collection of unique, scriptable, components for [GlazeWM](https://github.com/glzr-io/glazewm).
#### Contents
1. [Showcases](#showcases)
2. [Installation](#installation)
3. [Building](#building)
3. [Usage](#usage)
    - [Daily Note Structure](#daily-note-structure)
    - [Components](#daily-note-structure)
    - [Daily Note Structure](#daily-note-structure)
    - [GlazeWM Component](#glazewm)
### Showcases
![demo](assets/glaze-components-small-128.gif?raw=true)
### Installation
Soon!
### Building
```
git clone https://github.com/oonamo/glaze-components.git
cd glaze-components
cargo run
```
### Usage
#### Configuration
##### Components
```yaml
# C:/User/your_user/.glaze-wm/components/config.yaml
state_path: "C:/User/name/.state" # the active state of the components will live here
daily_note: # daily note component
  # see https://docs.rs/chrono-wasi07/latest/chrono/format/strftime/index.html#specifiers
  # for all the formatting options
  # prepend paths with '$'
  format_style: ["$C:/Users/your_name/your_notes_path/notes", "%b %G", "%G-%m-%d.md"]
  task_list_path: "C:/Users/name/.glaze-wm/components/task_list.log" # where the list of all daily tasks will be kept
  regex: \s* \[ \] (.*) # match file content. The first capture group is what is displayed.
```
##### Daily Note Structure
While this tool aims to allow as much flexibility as possible, their still needs some method of organization to allow this tool to work seamlessly.
The recommended way to organize is of this form.
```
Notes
└── notes/
    ├── dir
    │   └── day + extenstion
    │   └── day + extenstion
    │   └── day + extenstion
    ├── dir
    │   └── day + extenstion
    │   └── day + extenstion
    │   └── day + extenstion
```
###### Example
The example configuration would have this file structure
```
Notes
└── notes/
    ├── Apr 24
    │   └── 2024-04-01.md
    │   └── 2024-04-02.md
    │   └── 2024-04-03.md
    ├── Jun 24
    │   └── 2024-05-01.md
    │   └── 2024-05-02.md
    │   └── 2024-05-03.md
```
##### GlazeWM
```yaml
components_left: # can also be center or right
    - type: "text file"
      file_path: "your_state_path/daily_note.log"

    - type: "text"
      text: " > "
      margin: 0 0 0 0
      left_click_command: "exec path_to_python_script.py"
```
Please see the ![example config](glaze_example.yaml)
