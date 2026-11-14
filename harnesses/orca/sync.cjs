// Use Orca's running settings service; never edit its live SQLite database.
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const home = os.homedir();
const state = path.join(home, '.config/ai/orca');
const shim = fs.readFileSync(path.join(home, '.config/orca/linux-orca-cli-shim/orca'), 'utf8');
const runtime = shim.match(/^runtime_root='([^']+)'/m)?.[1];
if (!runtime || !fs.existsSync(runtime)) throw Error('Open Orca to refresh its CLI runtime');
const { RuntimeClient } = require(path.join(runtime, 'resources/app.asar.unpacked/out/cli/runtime-client.js'));
const client = new RuntimeClient();
const launch = path.join(home, '.local/bin/orca-agent');
const harness = path.join(home, '.local/bin/harness');
async function call(method, params) {
  const reply = await client.call(method, params);
  if (!reply.ok) throw Error(`${method} failed`);
  return reply.result;
}
async function main() {
  const before = await call('settings.get', {});
  const quick = await call('settings.getTerminalQuickCommands', {});
  const backup = path.join(state, `orca-settings-before-${Date.now()}.json`);
  fs.writeFileSync(backup, JSON.stringify({ ...before, ...quick }, null, 2), { mode: 0o600 });
  const inventory = JSON.parse(fs.readFileSync(path.join(state, 'inventory.json')));
  const native = {claude: 'claude', gemini: 'gemini', muse: 'muse',
    cline: 'cline', kilo: 'kilo', kimi: 'kimi', 'mistral-vibe': 'vibe',
    'qwen-code': 'qwen', copilot: 'copilot', pi: 'pi', opencode: 'opencode'};
  for (const [agent, selector] of Object.entries(native)) {
    await call('settings.mutateAgentLaunch', {type: 'command', agent, value: `${harness} ${selector}`});
  }
  const clineEnv = JSON.parse(fs.readFileSync(path.join(home, '.t3/userdata/settings.json')))
    .providerInstances.acpRegistry_cline?.environment ?? [];
  for (const entry of clineEnv) {
    if (entry.name === 'CLINE_API_KEY') continue;
    const value = entry.name === 'PATH' ? `${home}/.local/share/fnm:${entry.value}` : entry.value;
    await call('settings.mutateAgentLaunch', {type: 'environment-set', agent: 'cline', name: entry.name, value});
  }
  for (const [agent, binary] of Object.entries({codex: path.join(home, '.local/bin/codex'),
                                               grok: path.join(home, '.local/bin/grok'),
                                               omp: path.join(home, '.local/bin/omp')})) {
    await call('settings.mutateAgentLaunch', {type: 'command', agent, value: binary});
  }
  await call('settings.mutateAgentLaunch', {type: 'availability', agent: 'cursor', enabled: false});
  const instances = {codex: {label: 'Codex', enabled: true}, ...inventory};
  for (const saved of quick.terminalQuickCommands) {
    if (saved.id.startsWith('t3-') && !instances[saved.id.slice(3)]?.enabled) {
      await call('settings.updateTerminalQuickCommands', {mutation: {type: 'delete', id: saved.id}});
    }
  }
  for (const [id, info] of Object.entries(instances)) {
    if (!info.enabled) continue;
    await call('settings.updateTerminalQuickCommands', {mutation: {type: 'upsert', command: {
      id: `t3-${id}`, label: `T3 · ${info.label}`, scope: {type: 'global'},
      action: 'terminal-command', command: `${launch} ${id}`, appendEnter: true
    }}});
  }
  const after = await call('settings.getAgentLaunch', {});
  fs.writeFileSync(path.join(state, 'applied-launch-settings.json'), JSON.stringify(after, null, 2), {mode: 0o600});
  console.log(`Installed ${Object.values(instances).filter(p => p.enabled).length} named launches. Backup: ${backup}`);
}
main().catch(error => { console.error(error.message); process.exitCode = 1; });
