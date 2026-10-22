"""Build and install this Codex fork with its Code Mode companion executable."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
BINARIES = ('codex', 'codex-code-mode-host')


def parse_arguments(arguments):
    """Parse standalone component actions and the target home once.
    Parameters: arguments - raw command line arguments.
    Returns: typed action options and a resolved home path.
    """
    print(f'arguments={arguments!r}', file=sys.stderr)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--home', type=lambda value: Path(value).expanduser().resolve(), default=Path.home())
    mode = parser.add_mutually_exclusive_group()
    for flag in ('list-components', 'export-selection', 'detect', 'default', 'all'):
        mode.add_argument('--'+flag, action='store_true')
    for flag in ('select', 'reconfigure', 'uninstall'):
        mode.add_argument('--'+flag, choices=('codex_cli',))
    result = parser.parse_args(arguments)
    print(result, file=sys.stderr)
    return result


def require_tools():
    """Require the public fork's native build prerequisites.
    Parameters: none.
    Returns: None when every required command is available.
    """
    print('parameters=none', file=sys.stderr)
    missing = [tool for tool in ('git', 'cargo', 'rustc', 'cc', 'cmake', 'pkg-config') if not shutil.which(tool)]
    if missing:
        raise ValueError('Missing build tools: '+', '.join(missing)+'. Install Rust with rustup; system tools: sudo apt-get install build-essential cmake pkg-config git')
    print(None, file=sys.stderr)


def require_source():
    """Require an unchanged Rust source tree before building.
    Parameters: none.
    Returns: the current committed source revision.
    """
    print('parameters=none', file=sys.stderr)
    subprocess.run(['git', '-C', str(ROOT), 'diff', '--quiet', 'HEAD', '--', 'codex-rs'], check=True)
    result = subprocess.check_output(['git', '-C', str(ROOT), 'rev-parse', 'HEAD'], text=True).strip()
    print(result, file=sys.stderr)
    return result


def digest(path):
    """Read the content identity of one installed binary.
    Parameters: path - binary file path.
    Returns: its SHA-256 digest, or None when absent.
    """
    print(f'path={path!r}', file=sys.stderr)
    result = hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None
    print(result, file=sys.stderr)
    return result


def build_and_install(home):
    """Build this fork and atomically publish both native executables.
    Parameters: home - resolved target user home.
    Returns: None after the binaries and ownership receipt are written.
    """
    print(f'home={home!r}', file=sys.stderr)
    require_tools()
    revision = require_source()
    os.environ['CODEX_REPO_ROOT'] = str(ROOT)
    sys.path.insert(0, str(ROOT))
    from scripts.codex_package.targets import TARGET_SPECS
    from scripts.codex_package.v8 import resolve_codex_v8_cargo_env
    version = subprocess.check_output(['rustc', '-vV'], cwd=ROOT/'codex-rs', text=True)
    target = next(line.split(': ', 1)[1] for line in version.splitlines() if line.startswith('host: '))
    environment = dict(os.environ, STABLE_GIT_COMMIT=revision)
    environment.update(resolve_codex_v8_cargo_env(TARGET_SPECS[target], cache_root=home/'.cache/codex-fork/v8'))
    subprocess.run(['cargo', 'build', '--release', '--locked', '-p', 'codex-cli', '--bin', 'codex', '-p', 'codex-code-mode-host', '--bin', 'codex-code-mode-host'], cwd=ROOT/'codex-rs', env=environment, check=True)
    target_dir = Path(environment.get('CARGO_TARGET_DIR', str(ROOT/'codex-rs/target')))
    if not target_dir.is_absolute():
        target_dir = ROOT/'codex-rs'/target_dir
    binary_dir = home/'.local/bin'
    binary_dir.mkdir(parents=True, exist_ok=True)
    receipt = {}
    for name in BINARIES:
        with tempfile.NamedTemporaryFile(dir=binary_dir, prefix='.'+name, delete=False) as temporary:
            staged = Path(temporary.name)
        try:
            shutil.copyfile(target_dir/'release'/name, staged)
            staged.chmod(0o755)
            staged.replace(binary_dir/name)
        finally:
            staged.unlink(missing_ok=True)
        receipt[name] = digest(binary_dir/name)
    state = home/'.local/state/codex-fork'
    state.mkdir(parents=True, exist_ok=True)
    (state/'installed.json').write_text(json.dumps(receipt)+'\n')
    print(None, file=sys.stderr)


def require_owned_files(home, receipt):
    """Protect binaries changed after this installer's last publication.
    Parameters: home - target home; receipt - recorded binary digests.
    Returns: None when installed files still belong to this installer.
    """
    print(f'home={home!r}, receipt={receipt!r}', file=sys.stderr)
    for name in BINARIES:
        path = home/'.local/bin'/name
        if path.exists() and digest(path) != receipt.get(name):
            raise ValueError('Refusing to remove modified or unowned binary: '+str(path))
    print(None, file=sys.stderr)


def uninstall(home):
    """Remove only binaries matching this installer's ownership receipt.
    Parameters: home - resolved target user home.
    Returns: None after owned executables are removed; configuration is retained.
    """
    print(f'home={home!r}', file=sys.stderr)
    state = home/'.local/state/codex-fork/installed.json'
    receipt = json.loads(state.read_text()) if state.is_file() else {}
    require_owned_files(home, receipt)
    for name in BINARIES:
        (home/'.local/bin'/name).unlink(missing_ok=True)
    state.unlink(missing_ok=True)
    print(None, file=sys.stderr)


def detect_installation(home):
    """Read whether both fork executables are available in the target home.
    Parameters: home - resolved target home.
    Returns: true when both managed entrypoints are executable.
    """
    print(f'home={home!r}', file=sys.stderr)
    result = all(os.access(home/'.local/bin'/name, os.X_OK) for name in BINARIES)
    print(result, file=sys.stderr)
    return result


def main(arguments):
    """Route parsed discovery, menu and component actions.
    Parameters: arguments - raw command line arguments.
    Returns: zero for successful actions and two on validation/build failures.
    """
    print(f'arguments={arguments!r}', file=sys.stderr)
    options = parse_arguments(arguments)
    try:
        if options.list_components:
            print('codex_cli\tBuild this Codex fork and its Code Mode host\ton')
        elif options.export_selection:
            print(json.dumps({'components': {'codex_cli': {'on': 1}}}))
        elif options.detect:
            installed = detect_installation(options.home)
            print('codex_cli\t'+('1' if installed else '0'))
        elif options.uninstall:
            uninstall(options.home)
        elif any((options.select, options.reconfigure, options.default, options.all)) or not sys.stdin.isatty():
            build_and_install(options.home)
        else:
            print('1. Build and install this Codex fork and Code Mode host\nq. Cancel')
            if input('Selection [q]: ').strip() == '1':
                build_and_install(options.home)
    except (ValueError, RuntimeError, OSError, subprocess.CalledProcessError) as error:
        print(error, file=sys.stderr)
        print(2, file=sys.stderr)
        return 2
    print(0, file=sys.stderr)
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
