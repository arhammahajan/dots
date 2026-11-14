"""Verify terminal dispatch and native Antigravity authentication without prompts."""
import json
from pathlib import Path
import shlex
import shutil
import subprocess

ROOT = Path(__file__).resolve().parent
harness = ROOT.parent / 'harness'
providers = {'codex': {'enabled': True}, **json.loads(
    (Path.home() / '.t3/userdata/settings.json').read_text())['providerInstances']}
results = []
for instance, provider in providers.items():
    if not provider.get('enabled', True):
        continue
    described = subprocess.run([str(harness), 'profile', '--describe', instance],
                               check=True, capture_output=True, text=True, timeout=10)
    command = shlex.split(described.stdout)
    available = shutil.which(command[0]) is not None
    terminal = not any(arg in ('acp', '--acp', 'vibe-acp', 'toad') for arg in command)
    results.append({'instance': instance, 'executable': available, 'terminal': terminal})
    print(instance, 'OK' if available and terminal else 'FAILED')

models = subprocess.run([str(harness), 'profile', 'antigravity', 'models'],
                        capture_output=True, text=True, timeout=30)
rows = [line for line in models.stdout.splitlines() if '\t' in line]
print('Native Antigravity:', len(rows), 'models')
report = {'terminals': results, 'antigravityModels': len(rows), 'antigravityExit': models.returncode}
(Path.home() / '.config/ai/orca/verification.json').write_text(json.dumps(report, indent=2) + '\n')
raise SystemExit(int(any(not r['executable'] or not r['terminal'] for r in results)
                    or models.returncode != 0 or len(rows) == 0))
