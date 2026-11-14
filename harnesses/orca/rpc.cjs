// Reuse the installed Orca CLI client so settings writes go through its runtime.
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const shim = fs.readFileSync(path.join(os.homedir(), '.config/orca/linux-orca-cli-shim/orca'), 'utf8');
const runtime = shim.match(/^runtime_root='([^']+)'/m)?.[1];
if (!runtime || !fs.existsSync(runtime)) throw Error('Open Orca to refresh its CLI runtime');
const { RuntimeClient } = require(path.join(runtime, 'resources/app.asar.unpacked/out/cli/runtime-client.js'));
const client = new RuntimeClient();
async function call(method, params) {
  const reply = await client.call(method, params);
  if (!reply.ok) throw Error(`${method} failed`);
  return reply.result;
}
module.exports = { call };
if (require.main === module) {
  call(process.argv[2], JSON.parse(process.argv[3] ?? '{}'))
    .then(result => console.log(JSON.stringify(result)))
    .catch(error => { console.error(error.message); process.exitCode = 1; });
}
