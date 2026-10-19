# Terraform Plan Format

[![CI](https://github.com/ovsds/tf-plan-format/workflows/Check%20PR/badge.svg)](https://github.com/ovsds/tf-plan-format/actions?query=workflow%3A%22%22Check+PR%22%22)

Terraform Plan Format

## Installation

### Install script

[setup.sh](setup.sh) downloads the release archive for your platform (Linux or macOS, amd64 or arm64), checks it against the release's `checksums.txt` and installs `tf_plan_format` into `~/.local/bin`, or into the directory given as the second argument. The first argument is the version, the latest release when omitted or `latest`:

```sh
curl -fsSL https://raw.githubusercontent.com/ovsds/tf-plan-format/main/setup.sh | bash
curl -fsSL https://raw.githubusercontent.com/ovsds/tf-plan-format/main/setup.sh | bash -s -- 0.2.0 "$HOME/bin"
```

## Usage

## Development

### Global dependencies

- [mise](https://mise.jdx.dev/installing-mise.html), activated in the shell (`eval "$(mise activate zsh)"`)

All other tools are pinned in [mise.toml](mise.toml).

### Taskfile commands

For all commands see [Taskfile](Taskfile.yaml) or `task --list-all`.

## License

[MIT](LICENSE)
