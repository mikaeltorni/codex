"""Exercise the public fork installer using fake builds and isolated owned files."""
import json
import os
from pathlib import Path
import pty
import select
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]


def run_menu(environment, keys):
    """Drive install/removal/cancel in the fork's real terminal component menu.

    Parameters: environment - isolated subprocess settings; keys - terminal input bytes.
    Returns: the process exit status and full terminal output.
    """
    print(f'environment={environment!r} keys={keys!r}')
    master, slave = pty.openpty()
    process = subprocess.Popen(['bash', str(ROOT/'install.sh')], env=environment,
                               stdin=slave, stdout=slave, stderr=slave, start_new_session=True)
    os.close(slave)
    output = bytearray()
    sent = False
    discarded = False
    deadline = time.monotonic()+20
    try:
        while time.monotonic() < deadline:
            if select.select([master], [], [], 0.1)[0]:
                try:
                    chunk = os.read(master, 65536)
                except OSError:
                    break
                if not chunk:
                    break
                output.extend(chunk)
                if not sent and b'Codex CLI and Code Mode host' in output:
                    os.write(master, keys)
                    sent = True
                if not discarded and b'Choice [1/2/3]' in output:
                    os.write(master, b'3\r')
                    discarded = True
            elif process.poll() is not None:
                break
        if process.poll() is None:
            process.wait(timeout=2)
        assert sent, output.decode(errors='replace')
        result = process.returncode, output.decode(errors='replace')
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()
        os.close(master)
    print(result)
    return result


with tempfile.TemporaryDirectory(prefix='codex-fork-installer-') as directory:
    temporary = Path(directory)
    home = temporary/'home'
    home.mkdir()
    tools = temporary/'bin'
    tools.mkdir()
    calls = temporary/'build-calls'
    (tools/'rustc').write_text('#!/bin/sh\nprintf "host: x86_64-unknown-linux-gnu\\n"\n')
    (tools/'cargo').write_text('''#!/bin/sh
printf '%s\\n' "$*" >> "$BUILD_CALLS"
mkdir -p "$CARGO_TARGET_DIR/release"
for binary in codex codex-code-mode-host; do
 printf '#!/bin/sh\\nexit 0\\n' > "$CARGO_TARGET_DIR/release/$binary"
 chmod +x "$CARGO_TARGET_DIR/release/$binary"
done
''')
    for tool in tools.iterdir():
        tool.chmod(0o755)
    env = dict(PATH=str(tools)+':'+os.environ['PATH'], HOME=str(home),
               CARGO_TARGET_DIR=str(temporary/'build'), BUILD_CALLS=str(calls),
               RUSTY_V8_ARCHIVE=str(temporary/'v8'), RUSTY_V8_SRC_BINDING_PATH=str(temporary/'binding'),
               XDG_STATE_HOME=str(temporary/'state'), XDG_CACHE_HOME=str(temporary/'cache'), TERM='xterm')
    public = ['bash',str(ROOT/'install.sh'),'--home',str(home)]
    settings = home/'.codex/config.toml'
    settings.parent.mkdir()
    settings.write_text('personal configuration remains\n')
    for arguments in (['--list-components'], ['--export-selection'], ['--detect'], ['--help']):
        result = subprocess.run([*public,*arguments],env=env,text=True,capture_output=True)
        assert result.returncode == 0, (arguments,result.stdout,result.stderr)
        if arguments == ['--detect']:
            assert result.stdout == 'codex_cli\tabsent\n', result.stdout
        assert not calls.exists(), arguments
    for arguments in (['--select','codex_cli'], ['--reconfigure','codex_cli'], ['--default'], ['--all'], []):
        result = subprocess.run([*public,*arguments],env=env,text=True,capture_output=True,stdin=subprocess.DEVNULL)
        assert result.returncode == 0, (arguments,result.stdout,result.stderr)
        for name in ('codex','codex-code-mode-host'):
            assert (home/'.local/bin'/name).read_bytes() == b'#!/bin/sh\nexit 0\n', (arguments,name)
        assert settings.read_text() == 'personal configuration remains\n'
    assert len(calls.read_text().splitlines()) == 5 and all('--bin codex-code-mode-host' in line for line in calls.read_text().splitlines()), calls.read_text()
    before = calls.read_bytes()
    receipt = home/'.local/state/codex-fork/installed.json'
    receipt_before = receipt.read_bytes()
    for arguments in (['--select','unknown'], ['--uninstall','unknown'], ['--select'], ['--uninstall'],
                      ['--reconfigure'], ['--select','codex_cli,codex_cli'], ['--uninstall','codex_cli,codex_cli'],
                      ['--default','--all'], ['--detect','--select','codex_cli'], ['--help','unexpected']):
        result = subprocess.run([*public,*arguments],env=env,text=True,capture_output=True)
        assert result.returncode == 2, (arguments,result.stdout,result.stderr)
        assert calls.read_bytes() == before and receipt.read_bytes() == receipt_before, arguments
        assert (home/'.local/bin/codex-code-mode-host').read_bytes() == b'#!/bin/sh\nexit 0\n', arguments
    for mode in ('--select','--reconfigure','--uninstall'):
        result = subprocess.run([*public,mode,''],env=env,text=True,capture_output=True)
        assert result.returncode == 0, (mode,result.stdout,result.stderr)
        assert calls.read_bytes() == before and receipt.read_bytes() == receipt_before, mode
    result = subprocess.run([*public,'--detect'],env=env,text=True,capture_output=True)
    assert result.returncode == 0 and result.stdout == 'codex_cli\tinstalled\n', (result.stdout,result.stderr)
    (home/'.local/bin/codex').write_text('user modified binary\n')
    result = subprocess.run([*public,'--uninstall','codex_cli'],env=env,text=True,capture_output=True)
    assert result.returncode == 2, (result.stdout,result.stderr)
    assert (home/'.local/bin/codex').read_text() == 'user modified binary\n'
    assert (home/'.local/bin/codex-code-mode-host').read_bytes() == b'#!/bin/sh\nexit 0\n'
    (home/'.local/bin/codex').write_bytes(b'#!/bin/sh\nexit 0\n')
    result = subprocess.run([*public,'--uninstall','codex_cli'],env=env,text=True,capture_output=True)
    assert result.returncode == 0, result.stderr
    assert not (home/'.local/bin/codex').exists() and not (home/'.local/bin/codex-code-mode-host').exists()
    assert settings.read_text() == 'personal configuration remains\n'
    before = calls.read_bytes()
    status, output = run_menu(env, b'q')
    assert status == 0 and calls.read_bytes() == before, output
    status, output = run_menu(env, b'\r')
    assert status == 0 and (home/'.local/bin/codex-code-mode-host').is_file(), output
    status, output = run_menu(env, b'u\r')
    assert status == 0 and not (home/'.local/bin/codex').exists(), output
    assert not (home/'.local/bin/codex-code-mode-host').exists()
    assert settings.read_text() == 'personal configuration remains\n'
print('PASS: fork lifecycle, five native build forms, empty selections, owned removal, private configuration and real menu parity')
