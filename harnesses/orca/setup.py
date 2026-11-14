#!/usr/bin/env python3
"""Prepare isolated Antigravity accounts and install Orca launch choices."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

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

# Retain the ACP executable and its adjacent harness independently of T3 updates.
runtime = STATE / 'antigravity-runtime'
if not runtime.exists():
    candidates = list((HOME / '.t3/tools/antigravity-acp/linux-x64/versions').glob('*/agy_acp_server.par'))
    if not candidates:
        raise SystemExit('No compatible installed Antigravity ACP executable was found')
    source = max(candidates, key=lambda p: p.stat().st_mtime)
    with tempfile.TemporaryDirectory(dir=STATE, prefix='.runtime-') as staging:
        prepared = Path(staging) / 'runtime'
        prepared.mkdir(mode=0o700)
        for name in ('agy_acp_server.par', 'localharness_external'):
            shutil.copy2(source.parent / name, prepared / name)
        prepared.rename(runtime)

for instance in ('antigravity', 'antigravity_mahajanarham'):
    source = HOME / '.t3/userdata/providers/antigravity' / hashlib.sha256(instance.encode()).hexdigest()
    target = STATE / 'accounts' / instance
    if not (target / 'antigravity-acp/acp_token.json').exists():
        # Keep credentials in private machine state, outside the dotfiles checkout.
        if not (source / 'antigravity-acp/acp_token.json').is_file():
            raise SystemExit(f'Missing T3 credentials for {instance}')
        target.mkdir(parents=True, exist_ok=True, mode=0o700)
        for folder, names in {'antigravity-acp': ['settings.json', 'acp_token.json'],
                              'config': ['mcp_config.json', 'config.json']}.items():
            (target / folder).mkdir(exist_ok=True, mode=0o700)
            for name in names:
                original = source / folder / name
                if original.exists():
                    shutil.copy2(original, target / folder / name)
                    os.chmod(target / folder / name, 0o600)
        for folder in ('skills', 'antigravity-cli/skills'):
            link = target / folder
            link.parent.mkdir(parents=True, exist_ok=True)
            if not link.exists() and not link.is_symlink():
                link.symlink_to(ROOT.parent / 'shared/skills', target_is_directory=True)

settings = json.loads((HOME / '.t3/userdata/settings.json').read_text())
# Save only UI/provider metadata; environment entries may contain credentials.
inventory = {k: {'driver': p['driver'], 'label': p.get('displayName') or k,
                 'enabled': p.get('enabled', True),
                 'models': p.get('config', {}).get('customModels', [])}
             for k, p in settings['providerInstances'].items()}
(STATE / 'inventory.json').write_text(json.dumps(inventory, indent=2) + '\n')
subprocess.run(['node', str(ROOT / 'sync.cjs')], check=True)
