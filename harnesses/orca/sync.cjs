const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { call } = require('./rpc.cjs');
const home = os.homedir();
const state = path.join(home, '.config/ai/orca');
const launch = path.join(home, '.local/bin/orca-agent');
const harness = path.join(home, '.local/bin/harness');
async function main() {
  const before = await call('settings.get', {});
  const quick = await call('settings.getTerminalQuickCommands', {});
  const backup = path.join(state, `orca-settings-before-${Date.now()}.json`);
  fs.writeFileSync(backup, JSON.stringify({ ...before, ...quick }, null, 2), { mode: 0o600 });
  // Commands which already store router settings in the CLI need no launch wrapper.
  const commands = {
    claude: `${harness} claude`, gemini: `${harness} gemini`,
    muse: `${harness} muse`, cline: `${harness} cline --tui`,
    kilo: `${harness} kilo`, 'mistral-vibe': `${harness} vibe`,
    kimi: path.join(home, '.kimi-code/bin/kimi'),
    codex: '', grok: '', omp: '', opencode: '', pi: '', 'qwen-code': '', copilot: '',
    antigravity: `${home}/.local/libexec/antigravity/agy --settings ${state}/antigravity-native-settings.json`,
    dsh: `${launch} acpRegistry_dsh`,
  };
  for (const [agent, value] of Object.entries(commands)) {
    await call('settings.mutateAgentLaunch', {type: 'command', agent, value});
    await call('settings.mutateAgentLaunch', {type: 'availability', agent, enabled: true});
  }
  const t3 = JSON.parse(fs.readFileSync(path.join(home, '.t3/userdata/settings.json')));
  const clineEnv = t3.providerInstances.acpRegistry_cline?.environment ?? [];
  for (const entry of clineEnv) {
    if (entry.name === 'CLINE_API_KEY') continue;
    const value = entry.name === 'PATH' ? `${home}/.local/share/fnm:${entry.value}` : entry.value;
    await call('settings.mutateAgentLaunch', {type: 'environment-set', agent: 'cline', name: entry.name, value});
  }
  // Keep new native Antigravity launches on the keyring account, not inherited API auth.
  for (const name of ['GEMINI_API_KEY', 'GOOGLE_API_KEY', 'GOOGLE_GEMINI_BASE_URL']) {
    await call('settings.mutateAgentLaunch', {type: 'environment-set', agent: 'antigravity', name, value: ''});
  }
  for (const agent of ['cursor', 'kiro']) {
    await call('settings.mutateAgentLaunch', {type: 'availability', agent, enabled: false});
  }
  // Extra configurations and CLIs absent from Orca's built-in picker only.
  const shortcuts = {
    codex_router: 'Codex · Model Router', grok_router: 'Grok · Model Router',
    antigravity: 'Antigravity · mahajanarham0',
    antigravity_mahajanarham: 'Antigravity · mahajanarham',
    acpRegistry_antigravity_router: 'Antigravity · Model Router',
    acpRegistry_kimi_legacy: 'Kimi CLI · Legacy',
    acpRegistry_mcode: 'MiniMax Code', fx: 'fx',
  };
  for (const saved of quick.terminalQuickCommands) {
    if (saved.id.startsWith('t3-') ||
        (saved.id.startsWith('orca-profile-') && !shortcuts[saved.id.slice('orca-profile-'.length)])) {
      await call('settings.updateTerminalQuickCommands', {mutation: {type: 'delete', id: saved.id}});
    }
  }
  for (const [id, label] of Object.entries(shortcuts)) {
    if (!t3.providerInstances[id]?.enabled) continue;
    await call('settings.updateTerminalQuickCommands', {mutation: {type: 'upsert', command: {
      id: `orca-profile-${id}`, label, scope: {type: 'global'},
      action: 'terminal-command', command: `${launch} ${id}`, appendEnter: true,
    }}});
  }
  const after = await call('settings.getAgentLaunch', {});
  fs.writeFileSync(path.join(state, 'applied-launch-settings.json'), JSON.stringify(after, null, 2), {mode: 0o600});
  console.log(`Configured terminal picker and extra profiles. Backup: ${backup}`);
}
main().catch(error => { console.error(error.message); process.exitCode = 1; });
