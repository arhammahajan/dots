const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { call } = require('./rpc.cjs');
const state = path.join(os.homedir(), '.config/ai/orca');
async function main() {
  const config = JSON.parse(fs.readFileSync(path.join(state, 'settings.json'), 'utf8'));
  const quick = await call('settings.getTerminalQuickCommands', {});
  await call('settings.mutateAgentLaunch', {
    type: 'default', agent: config.defaultAgent, setupOnly: config.setupOnly,
  });
  await call('settings.mutateAgentLaunch', {type: 'permission', choice: config.permission});
  for (const [agent, value] of Object.entries(config.commands)) {
    await call('settings.mutateAgentLaunch', {type: 'command', agent, value});
    await call('settings.mutateAgentLaunch', {type: 'availability', agent, enabled: true});
  }
  for (const [agent, environment] of Object.entries(config.environment)) {
    for (const [name, value] of Object.entries(environment)) {
      await call('settings.mutateAgentLaunch', {type: 'environment-set', agent, name, value});
    }
  }
  for (const agent of config.disabledAgents) {
    await call('settings.mutateAgentLaunch', {type: 'availability', agent, enabled: false});
  }
  const shortcutIds = new Set(config.shortcuts.map(command => command.id));
  for (const saved of quick.terminalQuickCommands) {
    if (saved.id.startsWith('t3-') ||
        (saved.id.startsWith('orca-profile-') && !shortcutIds.has(saved.id))) {
      await call('settings.updateTerminalQuickCommands', {mutation: {type: 'delete', id: saved.id}});
    }
  }
  for (const command of config.shortcuts) {
    await call('settings.updateTerminalQuickCommands', {mutation: {type: 'upsert', command}});
  }
  console.log('Applied tracked Orca agent settings and Quick Commands.');
}
main().catch(error => { console.error(error.message); process.exitCode = 1; });
