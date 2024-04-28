# Glaze Components

Glaze Components aims to be a collection of unique, scriptable, components for [GlazeWM](https://github.com/glzr-io/glazewm).
#### Contents
1. [Showcases](#showcases)
2. [Installation](#installation)
3. [Building](#building)
3. [Usage](#usage)
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
```yaml
state_path: "C:/User/name/.state" # the active state of the components will live here
daily_note:     # daily note component
  base_dir: "C:/Users/name/Desktop/notes_path" # the base path of your notes
  task_list_path: "C:/Users/name/.glaze-wm/components/task_list.log" # where the list of all daily tasks will be kept
  dir_format: "%b %G" # see https://docs.rs/chrono-wasi07/latest/chrono/format/strftime/index.html#specifiers
  day_format: "%G-%m-%d" # see https://docs.rs/chrono-wasi07/latest/chrono/format/strftime/index.html#specifiers
  file_extension: ".md"
  regex: \s* \[ \] (.*)
```
Currently, only the "daily note" component is implemented.
While the "daily note" component has been tested primarily on GlazeWM, their is no reason why it could not work for [Zebar](https://github.com/glzr-io/zebar)
