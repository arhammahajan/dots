"""Import existing T3 Google grants into Orca's native Antigravity account store."""
import base64
from datetime import datetime, timedelta, timezone
import hashlib
import json
from pathlib import Path
import subprocess
from urllib.parse import urlencode
from urllib.request import Request, urlopen

HOME = Path.home()
ROOT = Path(__file__).resolve().parent
STATE = HOME / '.config/ai/orca'


def rpc(method, params):
    result = subprocess.run(['node', str(ROOT / 'rpc.cjs'), method, json.dumps(params)],
                            capture_output=True, text=True, timeout=30)
    if result.returncode:
        raise RuntimeError(result.stderr.strip())
    return json.loads(result.stdout)


def secret(args, value=None):
    result = subprocess.run(['secret-tool', *args, 'service', 'gemini',
                             'username', 'antigravity'], input=value,
                            capture_output=True, text=True, timeout=12)
    return result


def main():
    mapping_file = STATE / 'antigravity-native-accounts.json'
    if mapping_file.exists():
        mapping = json.loads(mapping_file.read_text())
        saved = rpc('accounts.antigravityList', {'runtime': 'host'})
        ids = {account['id'] for account in saved['accounts']}
        if set(mapping) == {'antigravity', 'antigravity_mahajanarham'} and all(
                account in ids for account in mapping.values()):
            print('Both native Antigravity accounts are already saved in Orca')
            return
    old = secret(['lookup'])
    if old.returncode not in (0, 1):
        raise RuntimeError('Cannot read the native Antigravity keyring')
    previous = old.stdout.rstrip('\n') if old.returncode == 0 else None
    mapping = {}
    try:
        for instance in ('antigravity', 'antigravity_mahajanarham'):
            source = (HOME / '.t3/userdata/providers/antigravity' /
                      hashlib.sha256(instance.encode()).hexdigest() /
                      'antigravity-acp/acp_token.json')
            grant = json.loads(source.read_text())
            payload = {key: grant[key] for key in ('client_id', 'client_secret', 'refresh_token')}
            payload['grant_type'] = 'refresh_token'
            try:
                with urlopen(Request(grant['token_uri'], data=urlencode(payload).encode()),
                             timeout=25) as response:
                    token = json.load(response)
            except Exception as error:
                raise RuntimeError(f'{instance}: OAuth refresh failed ({type(error).__name__})') from None
            token['refresh_token'] = token.get('refresh_token', grant['refresh_token'])
            token['expiry'] = (datetime.now(timezone.utc) + timedelta(
                seconds=token.pop('expires_in', 3600))).isoformat().replace('+00:00', 'Z')
            credential = json.dumps({'auth_method': 'consumer',
                                     'id_token': token.pop('id_token', None), 'token': token})
            encoded = 'go-keyring-base64:' + base64.b64encode(credential.encode()).decode()
            if secret(['store', '--label=Antigravity'], encoded).returncode:
                raise RuntimeError('Cannot write the native Antigravity keyring')
            current = rpc('accounts.antigravityAddCurrent', {'runtime': 'host'})
            if not current.get('activeAccountId'):
                raise RuntimeError('Orca did not recognize the imported native account')
            mapping[instance] = current['activeAccountId']
            print(f'{instance}: saved in Orca native account selector')
        mapping_file.write_text(json.dumps(mapping, indent=2) + '\n')
        mapping_file.chmod(0o600)
    finally:
        restored = secret(['clear']) if previous is None else secret(
            ['store', '--label=Antigravity'], previous)
        if restored.returncode not in (0, 1):
            raise RuntimeError('Could not restore the previous native Antigravity credential')
    if previous is None:
        rpc('accounts.antigravitySelect', {'target': {'runtime': 'host'},
                                         'accountId': mapping['antigravity']})


if __name__ == '__main__':
    main()
