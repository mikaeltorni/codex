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

## Fork usage-limit recovery

The public entrypoint is `codex [OPTIONS] [PROMPT]` for an interactive coding
session, or `codex <COMMAND> [OPTIONS]` for a CLI operation. Public commands are
`agents`, `exec` (`e`), `review`, `login`, `logout`, `mcp`, `plugin`, `app-server`,
`remote-control`, `completion`, `update`, `doctor`, `sandbox`, `debug`, `apply`
(`a`), `resume`, `queue`, `archive`, `delete`, `migrate-rollouts`, `unarchive`,
`fork`, `cloud` (`cloud-tasks`), `exec-server`, `features`, and `help`. The `app`
command is available on macOS and Windows. Internal commands include
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

### Checking the CX test deployment

On this machine, registered CX accounts share the installed Codex CLI and Code
Mode host. The `newTest1601` test deployment publishes both binaries together.
Existing processes keep their loaded binary; start a fresh Codex process to test
the deployed changes.

From the repository root, run the installed-runtime checks with Python 3.11 or
newer:

```shell
python3 scripts/install/verify_cx_deployment.py -v
```

The checks verify the committed revision against `newTest1601`, unchanged runtime
sources, both installed binary hashes, and the CLI/configuration for every
registered account. They preserve the selected account and start no agent turns.
The [coverage inventory](scripts/install/cx_deployment_coverage.md) describes
the saved cases. Documentation-only commits do not require rebuilding the runtime.

For the TUI regression suite, run from `codex-rs` using the installed CLI:

```shell
umask 077
env -u NO_COLOR TERM=xterm-256color \
  CARGO_BIN_EXE_codex="$HOME/.local/share/linux_codex_claude_code_setup/npm/bin/codex" \
  CARGO_TARGET_DIR="$HOME/.cache/linux_codex_claude_code_setup/codex-main/target" \
  just test -p codex-tui
```

Private fixture directories satisfy existing IPC trust checks. The normal
terminal color environment preserves escape-sequence assertions. The complete
workspace suite requires separate approval under this repository's `AGENTS.md`.

## Docs

- [**Codex Documentation**](https://developers.openai.com/codex)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)
- [**Open source fund**](./docs/open-source-fund.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).
