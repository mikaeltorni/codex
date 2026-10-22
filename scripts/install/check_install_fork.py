"""Exercise the fork installer with fake build commands and an isolated home."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

root=Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory() as directory:
    temp=Path(directory)
    home=temp/'home'
    home.mkdir()
    tools=temp/'bin'
    tools.mkdir()
    (tools/'rustc').write_text('#!/bin/sh\nprintf "host: x86_64-unknown-linux-gnu\\n"\n')
    (tools/'cargo').write_text('#!/bin/sh\nprintf "%s\\n" "$*" >> "$BUILD_CALLS"\nmkdir -p "$CARGO_TARGET_DIR/release"\nfor binary in codex codex-code-mode-host; do printf "#!/bin/sh\\nexit 0\\n" > "$CARGO_TARGET_DIR/release/$binary"; chmod +x "$CARGO_TARGET_DIR/release/$binary"; done\n')
    for tool in tools.iterdir():
        tool.chmod(0o755)
    env=dict(os.environ,HOME=str(home),PATH=str(tools)+':'+os.environ['PATH'],CARGO_TARGET_DIR=str(temp/'build'),BUILD_CALLS=str(temp/'calls'),RUSTY_V8_ARCHIVE=str(temp/'v8'),RUSTY_V8_SRC_BINDING_PATH=str(temp/'binding'))
    for args in (['--list-components'],['--export-selection'],['--detect']):
        result=subprocess.run(['bash',str(root/'install.sh'),'--home',str(home),*args],env=env,text=True,capture_output=True)
        assert result.returncode==0,(args,result.stdout,result.stderr)
    for attempt in range(2):
        result=subprocess.run(['bash',str(root/'install.sh'),'--home',str(home),'--select','codex_cli'],env=env,text=True,capture_output=True)
        assert result.returncode==0,(result.stdout,result.stderr)
    assert (home/'.local/bin/codex').is_file() and (home/'.local/bin/codex-code-mode-host').is_file()
    assert 'codex-code-mode-host' in (temp/'calls').read_text()
    result=subprocess.run(['bash',str(root/'install.sh'),'--home',str(home),'--select','unknown'],env=env,text=True,capture_output=True)
    assert result.returncode!=0
    assert (home/'.local/bin/codex').is_file()
    result=subprocess.run(['bash',str(root/'install.sh'),'--home',str(home),'--uninstall','codex_cli'],env=env,text=True,capture_output=True)
    assert result.returncode==0,result.stderr
    assert not (home/'.local/bin/codex').exists()
print('PASS: independent fork discovery, repeatable native build with host, rejection and owned uninstall')
