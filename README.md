> Part of [mt_stack](https://github.com/mikaeltorni/mt_stack), a Linux setup stack of independently installable projects.

<p align="center"><strong>Codex CLI</strong> is a coding agent from OpenAI that runs locally on your computer.
<p align="center">
  <img src="https://github.com/openai/codex/blob/main/.github/codex-cli-splash.png" alt="Codex CLI splash" width="80%" />
</p>
</br>
If you want Codex in your code editor (VS Code, Cursor, Windsurf), <a href="https://developers.openai.com/codex/ide">install in your IDE.</a>
</br>If you want the desktop app experience, run <code>codex app</code> or visit <a href="https://chatgpt.com/codex?app-landing-page=true">the Codex App page</a>.
</br>If you are looking for the <em>cloud-based agent</em> from OpenAI, <strong>Codex Web</strong>, go to <a href="https://chatgpt.com/codex">chatgpt.com/codex</a>.</p>

[![Last commit](https://img.shields.io/github/last-commit/mikaeltorni/codex)](https://github.com/mikaeltorni/codex/commits/main)
[![Commit activity](https://img.shields.io/github/commit-activity/m/mikaeltorni/codex)](https://github.com/mikaeltorni/codex/graphs/commit-activity)
[![Issues](https://img.shields.io/github/issues/mikaeltorni/codex)](https://github.com/mikaeltorni/codex/issues)

---

## Quickstart

### Installing and running Codex CLI

Run the following on Mac or Linux to install Codex CLI:

```shell
curl -fsSL https://chatgpt.com/codex/install.sh | sh
```

Run the following on Windows to install Codex CLI:

```shell
powershell -ExecutionPolicy ByPass -c "irm https://chatgpt.com/codex/install.ps1 | iex"
```

The standalone installers download from `https://releases.openai.com/codex` by default and fall back to GitHub Releases if a metadata or asset download is unavailable. To force GitHub Releases, set `CODEX_INSTALLER_USE_RELEASES_OPENAI_COM` to `false` (`0` and `no` are also accepted):

```shell
curl -fsSL https://chatgpt.com/codex/install.sh | CODEX_INSTALLER_USE_RELEASES_OPENAI_COM=false sh
```

```powershell
$env:CODEX_INSTALLER_USE_RELEASES_OPENAI_COM='false'; irm https://chatgpt.com/codex/install.ps1 | iex
```

Codex CLI can also be installed via the following package managers:

```shell
# Install using npm
npm install -g @openai/codex
```

```shell
# Install using Homebrew
brew install --cask codex
```

Then simply run `codex` to get started.

<details>
<summary>You can also go to the <a href="https://github.com/openai/codex/releases/latest">latest GitHub Release</a> and download the appropriate binary for your platform.</summary>

Each GitHub Release contains many executables, but in practice, you likely want one of these:

- macOS
  - Apple Silicon/arm64: `codex-aarch64-apple-darwin.tar.gz`
  - x86_64 (older Mac hardware): `codex-x86_64-apple-darwin.tar.gz`
- Linux
  - x86_64: `codex-x86_64-unknown-linux-musl.tar.gz`
  - arm64: `codex-aarch64-unknown-linux-musl.tar.gz`

Each archive contains a single entry with the platform baked into the name (e.g., `codex-x86_64-unknown-linux-musl`), so you likely want to rename it to `codex` after extracting it.

</details>

### Using Codex with your ChatGPT plan

Run `codex` and select **Sign in with ChatGPT**. We recommend signing into your ChatGPT account to use Codex as part of your Plus, Pro, Business, Edu, or Enterprise plan. [Learn more about what's included in your ChatGPT plan](https://help.openai.com/en/articles/11369540-codex-in-chatgpt).

You can also use Codex with an API key, but this requires [additional setup](https://developers.openai.com/codex/auth#sign-in-with-an-api-key).

## Image prompts

Attach local images to the interactive CLI with `--image` (repeatable):

```shell
codex --image screenshot.png -- '/goal inspect this screenshot'
```

An initial `/goal` prompt creates a native active Goal before work begins. Codex
copies each attached image into its goal attachments and includes a file reference
in the objective. This also applies to ACC's `ca -ic "inspect this screenshot"`.
Ordinary prompts send their text before the image attachments. Existing active
goals require confirmation before replacement.

## Goal shortcuts

Start the native TUI with `codex --enable goals`, or enable Goals in `~/.codex/config.toml` as shown below. With the composer focused:

| Shortcut | Behavior |
| --- | --- |
| **Alt+G** | Toggle the leading `/goal` without submitting. Add `/goal ` when absent; remove an existing `/goal` and one separating whitespace character when present. Keep the remaining text, pastes and attachments, and adjust the cursor for the prefix edit. A bare `/goal` becomes an empty draft. |
| **Alt+Shift+G** | Immediately resume the current thread's paused, blocked or usage-limited Goal through the existing `/goal resume` lifecycle and continuation, preserving the composer and requiring no confirmation. |

Use `/goal <objective>` to set a Goal and `/goal resume` to resume it by command. Resume does nothing without an eligible Goal; active, complete and budget-limited Goals are ineligible. Resumption yields to popups, searches and modal views. Server-side Goal validation and usage limits still apply.

Both actions appear in `/keymap` and support the native binding settings:

```toml
[features]
goals = true

[tui.keymap.composer]
prepend_goal = "alt-g"

[tui.keymap.chat]
resume_goal = "alt-shift-g"
```

Use a key string, an array of alternatives, or a two-stroke chord such as `"ctrl-x g"`; `[]` disables an action. New defaults yield to existing custom shortcuts and overlapping chord prefixes. Explicit conflicting bindings are rejected.

The native matcher distinguishes Alt+`g` from Alt+Shift+`g` and also accepts legacy Alt+uppercase `G` reporting. Codex retains its existing negotiated [keyboard enhancement support](https://sw.kovidgoyal.net/kitty/keyboard-protocol/). Some terminals, multiplexers or OS bindings lose this distinction; legacy Caps Lock reporting can also be ambiguous. On macOS, configure Option to send Alt/Meta or an Escape prefix. Use `/keymap debug` to inspect received keys and remap the actions to distinct keys such as `f6` and `f7` if needed.

## Fork usage-limit recovery

The public entrypoint is `codex [OPTIONS] [PROMPT]` for an interactive coding
session, or `codex <COMMAND> [OPTIONS]` for a CLI operation. Public commands are
`agents`, `exec` (`e`), `review`, `login`, `logout`, `mcp`, `plugin`, `app-server`,
`remote-control`, `completion`, `update`, `doctor`, `sandbox`, `debug`, `apply`
(`a`), `resume`, `queue`, `archive`, `delete`, `migrate-rollouts`, `unarchive`,
`fork`, `cloud` (`cloud-tasks`), `exec-server`, `features`, `migrate-nfs-db`,
and `help`. The `app` command is available on macOS and Windows. Internal commands include
`tcp-tunnel`, `execpolicy`, `responses-api-proxy`, and `stdio-to-uds`.
Use `codex <COMMAND> --help` for each command's operands and subcommands.

This fork can wait for usage-limit reset and automatically resume the turn:

```shell
codex -c auto_resume_on_usage_limit=true
```

In the interactive session, `/model` changes the model, `/status` shows the
session and usage limits, and `/usage` opens usage and reset actions. An active
usage-limit wait keeps its countdown visible while account banners update.
Dismissing an unrelated notice leaves a hidden account banner available when
switching back to its model. Reserve recovery and reset actions remain available.
Account notices keep the composer visible while an ordinary model is selected;
press a displayed action number with an empty composer to choose Upgrade or
another recovery action. Enter remains composer input, including for `/status`
and `/usage`. The focused Reserve recovery picker appears only while Reserve
is the selected model.

### Checking the local deployment

The local `main` and `reserveMerge` sources include Codex 0.166.0, usage-limit
recovery, Goal shortcuts, and the Reserve opt-out. Registered CX accounts share
the installed Codex CLI and Code Mode host. The native setup repository selects the deployed
Git ref independently and publishes both binaries together to the main, Qwen,
OpenRouter, and NVIDIA harnesses. To install the integrated `main` source:

```shell
CODEX_FORK_REF=main bash "$HOME/projects/linux_codex_claude_code_setup/scripts/sync_codex_fork.sh"
```

Existing processes keep their loaded binary; start a fresh Codex process to test
the deployed changes.

From the repository root, run the installed-runtime checks with Python 3.11 or
newer:

```shell
python3 scripts/install/verify_cx_deployment.py -v
```

The checks verify the committed revision against `main`, unchanged runtime
sources, both installed binary hashes, and the CLI/configuration for every
registered account. They preserve the selected account and start no agent turns.
The [coverage inventory](scripts/install/cx_deployment_coverage.md) describes
the saved cases. Documentation-only commits do not require rebuilding the runtime.

For focused native regressions, build test-profile fixture binaries from
`codex-rs` first, then run the tests with those binaries:

```shell
CARGO_PROFILE_CI_TEST_DEBUG=0 CARGO_BUILD_JOBS=1 \
  CARGO_TARGET_DIR="$HOME/.cache/linux_codex_claude_code_setup/codex-main/target" \
  cargo build --profile ci-test -p codex-cli --bin codex \
    -p codex-code-mode-host --bin codex-code-mode-host \
    -p codex-rmcp-client --bin test_stdio_server
umask 077
env -u NO_COLOR TERM=xterm-256color \
  CARGO_PROFILE_CI_TEST_DEBUG=0 CARGO_BUILD_JOBS=1 \
  CARGO_BIN_EXE_codex="$HOME/.cache/linux_codex_claude_code_setup/codex-main/target/ci-test/codex" \
  CARGO_BIN_EXE_test_stdio_server="$HOME/.cache/linux_codex_claude_code_setup/codex-main/target/ci-test/test_stdio_server" \
  CARGO_BIN_EXE_codex-code-mode-host="$HOME/.cache/linux_codex_claude_code_setup/codex-main/target/ci-test/codex-code-mode-host" \
  CARGO_TARGET_DIR="$HOME/.cache/linux_codex_claude_code_setup/codex-main/target" \
  just test --cargo-profile ci-test -p codex-core -p codex-tui \
    -p codex-features -p codex-app-server -p codex-app-server-protocol \
    -E 'package(codex-features) | test(usage_limit) | test(usage_wait) | test(backend_banner) | test(goal_prefix) | test(goal_resume_shortcut) | test(keymap)'
```

Test-profile builds support isolated temporary home fixtures. The terminal
variables preserve escape-sequence assertions. The 0.166.0 integration passed
355 focused feature checks, 11 schema and image-order checks, and two
upgrade-notice checks. The complete workspace
suite was not run. For direct Cargo builds, set `RUSTY_V8_ARCHIVE` and
`RUSTY_V8_SRC_BINDING_PATH` to the checksum-verified artifacts described in
[the V8 build instructions](third_party/v8/README.md); the setup installer
resolves these artifacts automatically.

### Automatic GPT Reserve switching

The local fork accepts this setting in `~/.codex/config.toml` (or the
selected `CODEX_HOME` / profile configuration):

```toml
[features]
luna_reserve_auto_switch = false
```

It prevents account usage banners from automatically selecting `gpt-reserve`,
waiting for that selection, or restricting manual model selection. Other model
fallbacks and recovery from an existing Reserve session remain available. The
upstream-compatible default is `true`; the native setup installer explicitly
sets it to `false` in every discovered Codex home and profile.

The public CLI also accepts `codex features disable luna_reserve_auto_switch`,
`codex features enable luna_reserve_auto_switch`, and `codex features list`.
Restart the CLI and resume the same conversation after changing the setting.
The saved regression runner is documented in
[codex-rs/tui/reserve_switch_coverage.md](codex-rs/tui/reserve_switch_coverage.md).

## Updating this custom fork from an official release

Preserve official release ancestry and keep it distinguishable from unpublished
fork work. Configure `upstream` as `https://github.com/openai/codex.git`; on a new
clone, add it if absent:

```shell
git remote add -t main upstream https://github.com/openai/codex.git
git config --add remote.upstream.fetch '+refs/tags/rust-v*:refs/remotes/upstream/releases/rust-v*'
```

Verify an existing remote's URL and fetch mappings before reusing them; add the
release mapping only once. In the task worktree for a requested update, fetch
the exact official tag into a retained remote-tracking ref, then merge it:

```shell
RELEASE_TAG=rust-v0.166.0
git fetch --no-tags upstream "refs/tags/${RELEASE_TAG}:refs/remotes/upstream/releases/${RELEASE_TAG}"
git merge --no-ff "refs/remotes/upstream/releases/${RELEASE_TAG}"
```

Use the requested release tag rather than blindly upgrading to the newest one.
Resolve conflicts while retaining the custom features, verify them, and deliver
the task branch into local `main`. Rebuild/install only when runtime sources
changed. Neither command pushes the fork. Do not squash official releases or
merge obsolete build branches only to retain their history; use archive tags
or bundles for that purpose.

Phrase Automation's pending-work counters subtract commits reachable from
remote-tracking refs. Retaining `upstream/releases/<tag>` excludes upstream
release commits without hiding unpublished custom changes. A tag fetched only
under `refs/tags/` does not provide that distinction. Preserve the upstream
release refs during cleanup and repeat the fetch-before-merge procedure for
each subsequent release. The project-local [agent guidance](AGENTS.md) records
the same workflow for future agents.

## Docs

- [**Codex Documentation**](https://developers.openai.com/codex)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)
- [**Open source fund**](./docs/open-source-fund.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).

## Standalone fork installer and selection menu

`bash install.sh` at this repository root opens a one-component selection menu
on a terminal. This builds the current fork source directly; it needs no
`mt_stack` or private setup repository. Noninteractive use applies the default.

```bash
bash install.sh --list-components
bash install.sh --select codex_cli
bash install.sh --home /path/to/user/home --default
```

The component builds both `codex` and `codex-code-mode-host` with locked release
Cargo inputs and this fork's existing V8 artifact resolver. It publishes both to
`~/.local/bin/` and records their digests under
`~/.local/state/codex-fork/installed.json`. Required commands are Python 3,
Git, Cargo, rustc, a C compiler, CMake and pkg-config. Rust comes from rustup;
Ubuntu system prerequisites can be installed with
`sudo apt-get install build-essential cmake pkg-config git`. Public dependency
and build-artifact downloads are needed for a clean build.

Accepted actions are `--help`, `--list-components`, `--export-selection`,
`--detect`, `--default`, `--all`, `--select codex_cli`,
`--reconfigure codex_cli`, and `--uninstall codex_cli`. `--home PATH` selects
the target home. Discovery performs no build or installation. Uninstall checks
the recorded digests before removing managed executables and retains user
configuration. Source installation preserves the selected Git branch and
requires the tracked Rust source to be unchanged. No packages are installed
through sudo automatically; a sudo invocation forwards user work to `SUDO_USER`.

Run `python3 scripts/install/check_install_fork.py` for isolated discovery,
mocked Cargo/V8 installation of both binaries, repeat installation, rejection
and uninstall checks. A complete native source build is a separate verification.
The installer is prepared locally and remains unpublished until explicitly pushed.

## Installer lifecycle

Run these commands from this checkout. The component IDs printed by discovery
are the same IDs used by the terminal menu and programmatic actions.

| Command | Behavior |
| --- | --- |
| `bash install.sh` | Open component selection on a terminal; use the default/config selection otherwise. |
| `bash install.sh --help` | Show accepted options (`-h` is equivalent). |
| `bash install.sh --list-components` | Print component IDs, labels and defaults. |
| `bash install.sh --default` | Apply default-on components. |
| `bash install.sh --all` | Apply every component. |
| `bash install.sh --select ID,...` | Apply exactly the named components. |
| `bash install.sh --reconfigure ID,...` | Reapply those components through the same install operations. |
| `bash install.sh --uninstall ID,...` | Run the selected removal operations. |
| `bash install.sh --detect` | Print `ID<TAB>installed` or `ID<TAB>absent`. |
| `bash install.sh --export-selection` | Print selection JSON without applying it. |

`--home PATH` selects the installation home. The atomic ID is `codex_cli`;
it builds both `codex` and `codex-code-mode-host`. Empty selections perform no
build or removal. Removal checks ownership before deleting either executable
and preserves `.codex` settings; modified or unowned executables are rejected.
The terminal menu delegates to the same native Python installer operations.

The licensed `.installer-support/` package is generated by
`linux_installation_scripts_functions`; `manifest.json` records provenance and
SHA-256 hashes, and `LICENSE.md` retains the MIT grant. It is a shared library
subset, not an application checkout. The default bootstrap requires no private
download or sibling repository. Explicit `ISC_COMPONENT_LIB_DIR` /
`ISC_FUNCTIONS_DIR` overrides remain available for development and the old setup
chain. Regenerate support from its owner rather than editing copies. Bash and
Python 3 are needed for this library; project build/runtime requirements above
still apply.

For example, inspect IDs first, then use the same ID with either interface:

```bash
bash install.sh --list-components
bash install.sh --select codex_cli
bash install.sh --detect
bash install.sh --uninstall codex_cli
```
