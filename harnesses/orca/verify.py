"""Verify terminal dispatch and native Antigravity authentication without prompts."""
import json
from pathlib import Path
import runpy
import shutil
import subprocess

ROOT = Path(__file__).resolve().parent
launcher = runpy.run_path(str(ROOT / 'orca-agent'))
results = []
for instance, provider in launcher['providers']().items():
    if not provider.get('enabled', True):
        continue
    command, env = launcher['launch_plan'](instance)
    available = shutil.which(command[0], path=env.get('PATH')) is not None
    terminal = not any(arg in ('acp', '--acp', 'vibe-acp', 'toad') for arg in command)
    results.append({'instance': instance, 'executable': available, 'terminal': terminal})
    print(instance, 'OK' if available and terminal else 'FAILED')

command, env = launcher['launch_plan']('antigravity')
models = subprocess.run([*command, 'models'], env=env, capture_output=True, text=True, timeout=30)
rows = [line for line in models.stdout.splitlines() if '\t' in line]
print('Native Antigravity:', len(rows), 'models')
report = {'terminals': results, 'antigravityModels': len(rows), 'antigravityExit': models.returncode}
(Path.home() / '.config/ai/orca/verification.json').write_text(json.dumps(report, indent=2) + '\n')
raise SystemExit(int(any(not r['executable'] or not r['terminal'] for r in results)
                    or models.returncode != 0 or len(rows) == 0))
