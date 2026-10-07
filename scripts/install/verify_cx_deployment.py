#!/usr/bin/env python3
"""Read-only checks for the locally installed main CX runtime."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tomllib
import unittest


REPO = Path(__file__).resolve().parents[2]
REF = "refs/heads/main"


class CxDeploymentTests(unittest.TestCase):
    def setUp(self):
        self.home = Path.home()
        self.prefix = self.home / ".local/share/linux_codex_claude_code_setup/npm"
        self.cli = self.prefix / "bin/codex"
        self.registry = (
            self.home / ".local/state/codex-agent-tracker/codex-instances.json"
        )
        self.registry_before = self.registry.read_bytes()
        self.accounts = json.loads(self.registry_before)["instances"]
        self.assertGreater(len(self.accounts), 0)
        manifest = subprocess.check_output(
            ["git", "show", f"{REF}:codex-rs/Cargo.toml"], cwd=REPO, text=True
        )
        self.version = tomllib.loads(manifest)["workspace"]["package"]["version"]
        self.env = dict(
            os.environ,
            CODEX_AGENT_SKIP_TRACKER="1",
            ATTS_SUPPRESS_ANNOUNCEMENTS="1",
            AGENT_COMMAND_CENTER_RAW_LAUNCH="1",
        )

    def tearDown(self):
        self.assertEqual(self.registry.read_bytes(), self.registry_before)

    def test_committed_runtime(self):
        revision = (self.prefix / "codex-fork-revision").read_text().strip()
        self.assertRegex(revision, r"^[0-9a-f]{40}$")
        subprocess.run(
            ["git", "merge-base", "--is-ancestor", revision, REF], cwd=REPO, check=True
        )
        subprocess.run(
            [
                "git",
                "diff",
                "--exit-code",
                revision,
                REF,
                "--",
                ".cargo",
                "codex-rs",
                "patches",
                "third_party/v8",
                "scripts/codex_package",
            ],
            cwd=REPO,
            check=True,
        )
        build = (
            self.home / ".cache/linux_codex_claude_code_setup/codex-main/target/release"
        )
        for name in ("codex", "codex-code-mode-host"):
            installed = self.prefix / "bin" / name
            self.assertTrue(installed.is_file() and os.access(installed, os.X_OK))
            with installed.open("rb") as deployed, (build / name).open("rb") as source:
                self.assertEqual(
                    hashlib.file_digest(deployed, "sha256").hexdigest(),
                    hashlib.file_digest(source, "sha256").hexdigest(),
                )

    def test_registered_account_launchers(self):
        launcher_dir = self.home / ".local/bin"
        shared_wrapper = (launcher_dir / "codex").read_text()
        self.assertIn(f'"{self.cli}" "$@"', shared_wrapper)
        for alias in ("cx", *(f"cx{account['id']}" for account in self.accounts)):
            wrapper = launcher_dir / alias
            self.assertTrue(os.access(wrapper, os.X_OK))
            self.assertIn(f"{launcher_dir}/agent-command-center", wrapper.read_text())
        for account in self.accounts:
            with self.subTest(instance=account["id"]):
                account_home = Path(account["home"])
                self.assertTrue(account_home.is_dir())
                env = dict(self.env, CODEX_HOME=str(account_home))
                expected = f"codex-cli {self.version}"
                for executable in (self.cli, launcher_dir / f"codex{account['id']}"):
                    result = subprocess.run(
                        [str(executable), "--version"],
                        env=env,
                        check=True,
                        capture_output=True,
                        text=True,
                        timeout=30,
                    )
                    self.assertEqual(result.stdout.strip(), expected)
                    self.assertEqual(self.registry.read_bytes(), self.registry_before)
                result = subprocess.run(
                    [
                        str(self.cli),
                        "-c",
                        "auto_resume_on_usage_limit=true",
                        "--enable",
                        "step_model_switching",
                        "features",
                        "list",
                    ],
                    env=env,
                    check=True,
                    capture_output=True,
                    text=True,
                    timeout=30,
                )
                self.assertRegex(
                    result.stdout, r"(?m)^step_model_switching\s+.*\btrue$"
                )
                self.assertEqual(self.registry.read_bytes(), self.registry_before)

    def test_cli_rejections(self):
        for arguments in (["-c"], ["--cx-deployment-unknown-option"]):
            with self.subTest(arguments=arguments):
                result = subprocess.run(
                    [str(self.cli), *arguments],
                    env=self.env,
                    capture_output=True,
                    timeout=30,
                )
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(self.registry.read_bytes(), self.registry_before)
                version = subprocess.run(
                    [str(self.cli), "--version"],
                    env=self.env,
                    check=True,
                    capture_output=True,
                    text=True,
                    timeout=30,
                )
                self.assertEqual(version.stdout.strip(), f"codex-cli {self.version}")
                self.assertEqual(self.registry.read_bytes(), self.registry_before)


if __name__ == "__main__":
    unittest.main()
