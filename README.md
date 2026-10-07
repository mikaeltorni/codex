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

### Automatic GPT Reserve switching

The local 0.160.1 fork accepts this setting in `~/.codex/config.toml` (or the
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

The change is isolated on `feat/codex_reserve-auto-switch-config`, based on
`rust-v0.160.1`, for upstream review. That release automatically selects Reserve
in its [account-banner handler](https://github.com/openai/codex/blob/rust-v0.160.1/codex-rs/tui/src/chatwidget/backend_banners.rs)
and has no corresponding opt-out in its
[configuration schema](https://github.com/openai/codex/blob/rust-v0.160.1/codex-rs/core/config.schema.json).
The new flag defaults to `true` to preserve that behavior unless explicitly
configured. Local installer policy is maintained separately by native setup
and Agent Command Center.

Validation on the release-based branch executed 45 focused feature, Reserve
and backend-banner checks, all passing. The saved coverage inventory names the
runner and cases; the full workspace suite was not run. `just fmt`,
`just write-config-schema` and `just bazel-lock-update` also completed. The lockfile
change only aligns existing workspace package versions with the release's
`0.160.1` manifest; it introduces no dependency upgrade.

Review the focused change with:

```shell
git diff rust-v0.160.1...feat/codex_reserve-auto-switch-config
```

## Docs

- [**Codex Documentation**](https://developers.openai.com/codex)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)
- [**Open source fund**](./docs/open-source-fund.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).
