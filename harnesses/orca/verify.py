"""Check real ACP handshakes without sending model prompts."""
import concurrent.futures
import json
import os
from pathlib import Path
import runpy
import selectors
import signal
import subprocess
import time

launcher = runpy.run_path(str(Path(__file__).with_name('orca-agent')))


def probe(instance):
    command, env = launcher['launch_plan'](instance, True)
    process = subprocess.Popen(command, env=env, stdin=subprocess.PIPE,
                               stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                               text=True, start_new_session=True)
    selector = selectors.DefaultSelector()
    selector.register(process.stdout, selectors.EVENT_READ)

    def rpc(request_id, method, params):
        request = dict(jsonrpc='2.0', id=request_id, method=method, params=params)
        process.stdin.write(json.dumps(request) + '\n')
        process.stdin.flush()
        deadline = time.monotonic() + 35
        while time.monotonic() < deadline:
            if not selector.select(max(0, deadline - time.monotonic())):
                break
            line = process.stdout.readline()
            if not line:
                raise RuntimeError('process exited')
            try:
                response = json.loads(line)
            except ValueError:
                continue
            if response.get('id') == request_id:
                return response
        raise RuntimeError('timeout')

    try:
        response = rpc(1, 'initialize', {
            'protocolVersion': 1,
            'clientCapabilities': {'fs': {'readTextFile': True, 'writeTextFile': True},
                                   'terminal': True},
            'clientInfo': {'name': 'orca-config-check', 'version': '1'},
        })
        info = response.get('result', {}).get('agentInfo', {})
        result = {'instance': instance, 'initialize': info or response.get('error')}
        if instance.startswith('antigravity') or 'antigravity_router' in instance:
            response = rpc(2, 'session/new', {'cwd': '/tmp', 'mcpServers': []})
            result['session'] = 'ok' if 'result' in response else response.get('error')
            result['models'] = len(response.get('result', {}).get('models', {}).get('availableModels', []))
        return result
    except Exception as error:
        return {'instance': instance, 'error': str(error)}
    finally:
        try:
            os.killpg(process.pid, signal.SIGTERM)
            process.wait(5)
        except (ProcessLookupError, subprocess.TimeoutExpired):
            try:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(5)
            except ProcessLookupError:
                pass
        selector.close()
        for stream in (process.stdin, process.stdout):
            try:
                stream.close()
            except BrokenPipeError:
                pass


if __name__ == '__main__':
    instances = [key for key, provider in launcher['providers']().items()
                 if provider.get('enabled', True)
                 and provider['driver'] in ('acpRegistry', 'antigravity')]
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        results = list(pool.map(probe, instances))
    for result in results:
        print(json.dumps(result))
    report = Path.home() / '.config/ai/orca/verification.json'
    report.write_text(json.dumps(results, indent=2) + '\n')
    raise SystemExit(int(any('error' in row or not row.get('initialize')
                            or ('session' in row and row['session'] != 'ok')
                            for row in results)))
