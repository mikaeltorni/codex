<p align="center"><strong>Codex CLI</strong> is a coding agent from OpenAI that runs locally on your computer.
<p align="center">
  <img src="https://github.com/openai/codex/blob/main/.github/codex-cli-splash.png" alt="Codex CLI splash" width="80%" />
</p>
</br>
If you want Codex in your code editor (VS Code, Cursor, Windsurf), <a href="https://developers.openai.com/codex/ide">install in your IDE.</a>
</br>If you want the desktop app experience, run <code>codex app</code> or visit <a href="https://chatgpt.com/codex?app-landing-page=true">the Codex App page</a>.
</br>If you are looking for the <em>cloud-based agent</em> from OpenAI, <strong>Codex Web</strong>, go to <a href="https://chatgpt.com/codex">chatgpt.com/codex</a>.</p>

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

### Goal shortcuts

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

## Docs

- [**Codex Documentation**](https://developers.openai.com/codex)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)
- [**Open source fund**](./docs/open-source-fund.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).
