# Init
1. `cargo install --path .`

# Usage

Use with `plugin-cli [COMMAND] [OPTIONS]` example `plugin-cli create` creates a new Plugin Repo.

# Global Options

| Option            | Description                                               |
| -----------       | --------------------------------------------------------- |
| **-h, --help**    | Prints this Tables in the Console                         |
| **-v, --version** | Shows the current SemVer of this Package                  |

# Commands

## help
Print this message or the help of the given subcommand(s)
`plugin-cli help`

## create
 Initialize code for a plugin    
 `plugin-cli create`

## publish
Publish Package on the plugin store
`plugin-cli publish [OPTIONS]`

| Options     | Description                                               | Example                                              |
| ----------- | --------------------------------------------------------- | ---------------------------------------------------- |
| **-r [URL]**  | set the remote `URL` of your PluginStore.               | `plugin-cli publish -r https://plugins.simpl-cms.de` |

## package
Package the plugin into a distributable package
`plugin-cli package`





