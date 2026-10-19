# Terraform Plan Format

[![CI](https://github.com/ovsds/tf-plan-format/workflows/Check%20PR/badge.svg)](https://github.com/ovsds/tf-plan-format/actions?query=workflow%3A%22%22Check+PR%22%22)

Renders Terraform and OpenTofu plans (the JSON from `show -json`) as GitHub markdown or through your own template.

## Installation

Release tags are plain versions without a `v` prefix (`0.1.0`), matching the version in `Cargo.toml`. Pick a version from the [releases page](https://github.com/ovsds/tf-plan-format/releases); `0.2.0` below is a placeholder: it is the first release whose `github` output carries the comment marker.

### Install script

[setup.sh](setup.sh) downloads the release archive for your platform (Linux or macOS, amd64 or arm64), checks it against the release's `checksums.txt` and installs `tf_plan_format` into `~/.local/bin`, or into the directory given as the second argument. The first argument is the version, the latest release when omitted or `latest`:

```sh
curl -fsSL https://raw.githubusercontent.com/ovsds/tf-plan-format/main/setup.sh | bash
curl -fsSL https://raw.githubusercontent.com/ovsds/tf-plan-format/main/setup.sh | bash -s -- 0.2.0 "$HOME/bin"
```

### mise

```sh
mise use github:ovsds/tf-plan-format@0.2.0
```

This installs the `tf_plan_format` executable from the release archive for your platform.

### Release archive

Each release has an archive per platform, `tf_plan_format-<version>-<os>-<arch>.tar.gz`, for `linux-amd64`, `linux-arm64`, `darwin-amd64` and `darwin-arm64`, plus a `checksums.txt` with their SHA-256 sums. The archive holds the single `tf_plan_format` executable. Linux binaries are built for glibc; on musl-based systems use the container image.

```sh
mkdir -p ~/.local/bin
curl -fsSL https://github.com/ovsds/tf-plan-format/releases/download/0.2.0/tf_plan_format-0.2.0-linux-amd64.tar.gz | tar -xz -C ~/.local/bin tf_plan_format
```

### Container image

Each release also publishes `ghcr.io/ovsds/tf-plan-format:<release tag>` for `linux/amd64` and `linux/arm64`: a statically linked binary in an empty (`scratch`) image whose entrypoint is `tf_plan_format`.

```sh
docker run --rm -v "$PWD:/infra:ro" ghcr.io/ovsds/tf-plan-format:0.2.0 github --file /infra/plan.json
```

## Usage

Produce the plan JSON with OpenTofu or Terraform:

```sh
tofu plan -out=plan.tfplan && tofu show -json plan.tfplan > plan.json
terraform plan -out=plan.tfplan && terraform show -json plan.tfplan > plan.json
```

> [!WARNING]
> The plan file and its JSON contain sensitive values in clear text. Do not commit, publish or upload them as CI artifacts; keep them in a temporary directory.

Both commands read plans with `-f`, `--file`: a path or a glob (quote it so the tool expands it). Repeat the flag or pass several values to read several plans; they are rendered in path order. Without `--file` there is nothing to render: `github` prints only its marker line and an empty line, and `custom` gets no plans. A path or glob that matches no file, a file that cannot be read, or JSON that does not have the shape of a plan fails the command with exit code 65. Pass the output of `show -json` on a plan file: `show -json` on the state is read as a plan without changes.

Actions are shown as icons: ✅ create, ♻️ replace (delete and create), 🔍 read, 🔄 update, ❌ delete, 🟰 no-op, ❓ unknown.

### `github`

Renders GitHub markdown for a PR comment:

- a hidden `<!-- tf-plan-format -->` first line (from 0.2.0), to find the comment again when updating it;
- one collapsible block per plan, titled from its path: the file name is dropped when all plans share it, then the leading directories all plans share, so `plans/prod/plan.json` and `plans/stage/plan.json` become `prod` and `stage`, and a single `infra/plan.json` becomes `infra`;
- the block's summary shows the icons of the changes it shows and of its output changes, the title and `Plan: X to add, Y to change, Z to destroy`, or `No changes`;
- inside, a collapsible block per resource change with its address and values, and an `Outputs` block when outputs change.

Sensitive values are shown as `sensitive`, `sensitive (changed)` or `sensitive (known after apply)`, values known only after apply as `(known after apply)`.

| Flag                     | Default | Effect                                                                                                                                                               |
| ------------------------ | ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `-f`, `--file`           |         | Plan path or glob, repeatable                                                                                                                                        |
| `-c`, `--changed-values` | off     | Also list the attributes and array elements that did not change; by default updates and replacements list only what changed (skipped array elements show as `- ...`) |
| `--show-no-op`           | off     | Render no-op changes                                                                                                                                                 |
| `--show-read`            | off     | Render data source reads                                                                                                                                             |
| `--max-length`           | none    | Limit the output length, see below                                                                                                                                   |

With `--max-length`, output longer than the limit (in bytes, the trailing newline not counted) is shortened: the largest values blocks are replaced with `_Values not shown to fit the length limit_` until it fits, and if that is not enough the last plans are dropped. A closing note says what was left out, for example `_Output limited to 1500 bytes: the last 3 plans, values of 3 changes and outputs of 1 plan not shown_`. The marker line counts towards the limit; if even the marker and the note do not fit, only they are printed.

```sh
tf_plan_format github --file tests/data/plans/outputs/opentofu.tfplan.json
```

````markdown
<!-- tf-plan-format -->
<details>
<summary>✅🔄❌outputs - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<details>
<summary>🔄terraform_data.foo-bar
</summary>

```
input: "foo" -> "bar"
output: "foo" -> (known after apply)
```

</details>
<details>
<summary>✅🔄❌Outputs
</summary>

```
added: null -> "added"
changed: "before" -> "after"
removed: "removed" -> null
secret: sensitive -> sensitive (changed)
unknown: "foo" -> (known after apply)
```

</details>
</details>
````

### `custom`

Renders your own template:

| Flag               | Default | Effect                                                                        |
| ------------------ | ------- | ----------------------------------------------------------------------------- |
| `-e`, `--engine`   | `tera`  | Template engine; only [Tera](https://keats.github.io/tera/docs/) is supported |
| `-f`, `--file`     |         | Plan path or glob, repeatable                                                 |
| `-t`, `--template` |         | The template text (required)                                                  |

#### Template context

`data.plans` maps each plan's file path to the plan, sorted by path. A plan has:

- `title`: the short title described for `github`;
- `changes`: the resource changes in plan order, each with `address`, `name`, `action`, `before` and `after` (attribute name to value, `null` when the resource does not exist on that side) and `raw`, the parts of the change the tool reads: `address`, `name` and `change` with its `actions`, `before`, `after`, `after_unknown`, `before_sensitive` and `after_sensitive` (other fields such as `type` are not kept, missing ones are `null`);
- `unique_actions`: the distinct actions of `changes`;
- `summary`: the `add`, `change` and `destroy` counts of `Plan: X to add, ...`, where a replacement counts in both `add` and `destroy`;
- `outputs`: the changed outputs, with `before` and `after` (output name to value) and `actions`, their distinct actions;
- `raw`: the plan's `resource_changes` and `output_changes` (`null` when the plan has none), limited to the fields the tool reads.

`raw` is not masked: it holds sensitive values in clear text.

An action is one of `Create`, `DeleteCreate` (a replacement, in either order), `Read`, `Update`, `Delete`, `NoOp` or `Unknown` (any other combination); lists of actions are sorted in this order.

A value is an object with its type in `op` and its content in `path`, such as `{"op": "String", "path": "foo"}`. The types with content are `String`, `Integer`, `Float`, `Boolean`, `Array` (a list of values) and `Object` (a map of values). `Null`, `Sensitive`, `SensitiveChanged` (in `after` only: a sensitive value that differs from `before`), `Unknown` (known after apply) and `SensitiveUnknown` have no `path`. In a template: `{% if value.op == "String" %}{{ value.path }}{% endif %}`.

#### Functions and filters

All arguments are named, for example `render_action(action=change.action)`.

- `render_values(before, after, show_changed_values=true)`: renders two attribute maps as the `github` command does, a `name: value` or `name: before -> after` line per attribute; with one side `null`, lists the other side's values. `show_changed_values=false` leaves out what did not change.
- `render_action(action)`: the action's icon.
- `render_actions(actions)`: the icons of a list of actions, deduplicated and sorted.
- `filter_actions(include=[...])` or `filter_actions(exclude=[...])`: a filter that keeps the items of a list, such as `changes`, whose `action` is (or is not) one of the given actions; exactly one of the two arguments is required.

#### Example

`summary.tera`:

```jinja
{% for path, plan in data.plans -%}
{{ render_actions(actions=plan.unique_actions) }} {{ plan.title }}: {{ plan.summary.add }} to add, {{ plan.summary.change }} to change, {{ plan.summary.destroy }} to destroy
{% for change in plan.changes | filter_actions(exclude=["Read", "NoOp"]) -%}
{{ render_action(action=change.action) }} {{ change.address }}
{{ render_values(before=change.before, after=change.after, show_changed_values=false) }}
{% endfor -%}
{% endfor -%}
```

```sh
tf_plan_format custom \
  --file tests/data/plans/read/opentofu.tfplan.json \
  --file tests/data/plans/sensitive/opentofu.tfplan.json \
  --template "$(cat summary.tera)"
```

```text
🔍🔄 read: 0 to add, 1 to change, 0 to destroy
🔄 terraform_data.foo-bar
input: "foo" -> "bar"
output: "foo" -> (known after apply)
♻️ sensitive: 1 to add, 0 to change, 1 to destroy
♻️ random_bytes.test
base64: sensitive -> sensitive (known after apply)
hex: sensitive -> sensitive (known after apply)
length: 4 -> 8
```

### GitHub Actions

[tf-plan-format-action](https://github.com/ovsds/tf-plan-format-action) installs a `tf_plan_format` release, renders the given plans with the `github` command and returns the markdown as the `rendered-plan` output and as a file at `rendered-plan-path`. It does not cap the output (there is no `--max-length`), while GitHub comments stop at 65536 characters: for plans that large, run the CLI with `--max-length` instead.

Pin OpenTofu in the repository's `mise.toml` (`mise use` writes it):

```toml
[tools]
opentofu = "1.13.1"
```

Then plan on every pull request and keep one plan comment up to date with [create-or-update-unique-comment-action](https://github.com/ovsds/create-or-update-unique-comment-action), which finds the comment again by the hidden `<!-- tf-plan-format -->` line the `github` output carries (from 0.2.0). The plan and its JSON stay in the runner's temporary directory and are never uploaded.

```yaml
on: pull_request

permissions:
  contents: read
  pull-requests: write

jobs:
  plan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false

      - uses: jdx/mise-action@9b200bcecbfd6303e69a4a261b2e09e3757440a7 # v5.2.0

      - name: Plan
        run: |
          mkdir -p "$RUNNER_TEMP/infra"
          tofu init -input=false
          tofu plan -input=false -out="$RUNNER_TEMP/infra/plan.tfplan"
          tofu show -json "$RUNNER_TEMP/infra/plan.tfplan" > "$RUNNER_TEMP/infra/plan.json"

      - id: tf-plan-format
        uses: ovsds/tf-plan-format-action@a1111c18db907140d366a097c30f359e85cfebc5 # v1.0.0
        with:
          plan-files: ${{ runner.temp }}/infra/plan.json

      - uses: ovsds/create-or-update-unique-comment-action@df30f5f96aa71679deb9eb697c98f39ee9e26651 # v1.1.0
        with:
          issue-number: ${{ github.event.pull_request.number }}
          body-path: ${{ steps.tf-plan-format.outputs.rendered-plan-path }}
          unique-body-includes: "<!-- tf-plan-format -->"
```

## Development

### Global dependencies

- [mise](https://mise.jdx.dev/installing-mise.html), activated in the shell (`eval "$(mise activate zsh)"`)

All other tools are pinned in [mise.toml](mise.toml).

### mise tasks

For all commands see [mise.toml](mise.toml), [mise-tasks](mise-tasks), [build](build) or `mise tasks ls --all`.

### Template

The repository is generated from [template-repository](https://github.com/ovsds/template-repository);
`mise run update-from-template` re-applies it. Keep these local changes when merging its output:

- `mise.toml` - the Rust, Terraform and OpenTofu tools, the cargo steps in `lint`, `lint-fix`, `test`, `clean` and
  `dependencies-update`.
- `.github/workflows/release.yaml` - binaries and release archives besides the image, built through `build` tasks
  instead of `ci-image-push`: the version check, a concurrency group per release tag, packaging and upload.
- `.github/workflows/release-pr.yaml` - beta binaries besides the image, built through `build` tasks; runs only on
  `paths` that change the build; the image is `ghcr.io/<owner>/<repo>-beta:<PR>-<head sha>`.
- `.github/workflows/check-pr.yaml` - runs on `main` too, with Rust cache and coverage.
- `.github/workflows/release-pr-cleanup.yaml`, `release-scan.yaml` - `${{ github.workflow }}` concurrency group, not the
  template's `$${{ github.workflow }}`.
- `.editorconfig`, `.gitignore` - the Rust sections.
- `README.md` - this one.

## License

[MIT](LICENSE)
