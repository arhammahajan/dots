#!/usr/bin/env python3
"""Configure Orca to launch the agents' terminal interfaces."""
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent
HOME = Path.home()
STATE = HOME / '.config/ai/orca'
STATE.mkdir(parents=True, exist_ok=True, mode=0o700)
launcher = HOME / '.local/bin/orca-agent'
if launcher.exists() or launcher.is_symlink():
    if launcher.resolve() != ROOT / 'orca-agent':
        raise SystemExit(f'Refusing to replace unrelated {launcher}')
else:
    launcher.symlink_to(ROOT / 'orca-agent')
os.chmod(ROOT / 'orca-agent', 0o755)
# Native sign-in must override the shared CLI's router provider for this launch.
(STATE / 'antigravity-native-settings.json').write_text(
    json.dumps({'modelProvider': '', 'model': ''}) + '\n')
# Reuse the router model catalog. These MCP rows target absent plugins in the TUI.
router_patch = (ROOT.parent / 'deepseek/cordis.patch.yml').read_text()
router_patch = router_patch.split('\n- id: mcp-agent-browser')[0]
(STATE / 'dsh-terminal.patch.yml').write_text(router_patch + '''
- id: dsh-tui
  config:
    provider: model-router
    model: deepseek-v4.1-flash
    effort: low
''')
subprocess.run(['node', str(ROOT / 'sync.cjs')], check=True)
