"""Exercise the launcher with fake installations; never launch real agents."""

import json
import os
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[1]
LAUNCHER = ROOT / "harness"
PROVIDER_VARS = (
    "MODEL_ROUTER_URL", "MODEL_ROUTER_API_KEY", "ANTHROPIC_BASE_URL",
    "ANTHROPIC_AUTH_TOKEN", "OPENAI_BASE_URL", "OPENAI_API_KEY", "META_API_KEY",
    "GROK_HOME", "GROK_MODELS_BASE_URL", "GROK_MODELS_LIST_URL", "XAI_API_KEY",
    "GROK_MODEL_ROUTER_SESSION", "FX_MODEL_ROUTER_SESSION", "VIBE_ROUTER_SESSION",
    "HARNESS_AGENT",
    "GOOGLE_GEMINI_BASE_URL", "GEMINI_API_KEY", "GEMINI_HOME",
    "GEMINI_MODEL_ROUTER", "GEMINI_CLI_REPO",
    "AGY_ACP_ENABLE_GATEWAY_AUTH", "AGY_ACP_DEFAULT_MODEL", "AGY_LLM_GATEWAY_URL",
    "AGY_LLM_GATEWAY_API_KEY", "AGY_LLM_GATEWAY_MODELS",
)
AGENTS = (
    "claude", "codex", "grok", "cline", "kilo", "kimi-cli", "muse", "fx",
    "deepseek", "vibe", "vibe-acp", "kimi", "mcode", "opencode", "pi", "qwen", "copilot",
    "antigravity", "gemini",
)


class HarnessTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name)
        self.bin = self.home / "bin"
        self.bin.mkdir()
        self.env = {
            "HOME": str(self.home),
            "PATH": f"{self.bin}:/usr/bin:/bin",
            "XDG_CONFIG_HOME": str(self.home / "config"),
            "XDG_DATA_HOME": str(self.home / "data"),
        }
        self.recorder = (
            f"#!{sys.executable}\n"
            "import json, os, sys\n"
            f"keys = {PROVIDER_VARS!r}\n"
            "print(json.dumps({'binary': sys.argv[0], 'args': sys.argv[1:], "
            "'env': {k: os.environ[k] for k in keys if k in os.environ}}))\n"
            "sys.exit(int(os.environ.get('TEST_EXIT_CODE', '0')))\n"
        )
        # Session tokens and Antigravity startup configuration use real Python.
        (self.bin / "python3").symlink_to(sys.executable)
        for name in ("claude", "kimi", "mcode", "opencode", "pi", "qwen", "copilot", "gemini", "node"):
            self.write_executable(self.bin / name, self.recorder)
        for relative in (
            ".local/bin/codex", ".local/bin/grok", ".local/bin/muse",
            ".kilo/bin/kilo", ".local/share/kimi-cli/bin/kimi-cli",
            ".local/libexec/fx/fx-router", ".local/share/deepseek-harness/bin/dsh",
            ".local/share/uv/tools/mistral-vibe/bin/vibe",
            ".local/share/uv/tools/mistral-vibe/bin/vibe-acp", "node/bin/cline",
            ".local/libexec/antigravity/agy",
            ".t3/tools/antigravity-acp/linux-x64/versions/test-release/agy_acp_server.par",
        ):
            self.write_executable(self.home / relative, self.recorder)
        settings = self.home / ".gemini/antigravity-cli/settings.json"
        settings.parent.mkdir(parents=True)
        settings.write_text(json.dumps({"model": "Saved model"}))
        entry = self.home / "dev/gemini-cli/packages/cli/dist/index.js"
        entry.parent.mkdir(parents=True)
        entry.write_text("// Fake Gemini entry; the Node recorder captures dispatch.\n")
        active = self.home / ".t3/tools/antigravity-acp/linux-x64/active.json"
        active.write_text(json.dumps({"releaseId": "test-release"}))
        self.write_executable(
            self.bin / "fnm",
            '#!/bin/sh\n[ "$*" = "exec --using=default which node" ] || exit 1\n'
            'printf "%s/node/bin/node\\n" "$HOME"\n',
        )

    def write_executable(self, path, content):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)
        path.chmod(0o755)

    def start_router_inventory(self):
        paths = []

        class Inventory(BaseHTTPRequestHandler):
            def log_message(self, *_):
                pass

            def do_GET(self):
                paths.append(self.path)
                body = json.dumps({"data": [{"id": "other-model"}, {"id": "deepseek-v4.1-flash"}]}).encode()
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)

        server = ThreadingHTTPServer(("127.0.0.1", 0), Inventory)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()

        def close():
            server.shutdown()
            server.server_close()
            thread.join()

        self.addCleanup(close)
        self.env["MODEL_ROUTER_URL"] = f"http://127.0.0.1:{server.server_port}"
        return paths

    def run_launcher(self, *args, executable=LAUNCHER):
        return subprocess.run(
            [str(executable), *args], env=self.env, text=True,
            capture_output=True, timeout=10,
        )

    def launch(self, *args, executable=LAUNCHER):
        result = self.run_launcher(*args, executable=executable)
        self.assertEqual(result.returncode, 0, result.stderr)
        return json.loads(result.stdout)

    def assert_session(self, value):
        self.assertRegex(value, r"^router-session-[0-9a-f-]{36}$")

    def test_help_and_invalid_agent_do_not_load_environment(self):
        config = self.home / "config/ai/model-router.env"
        config.parent.mkdir(parents=True)
        config.write_text("exit 99\n")
        for flag in ("--help", "-h"):
            self.assertEqual(self.run_launcher(flag).returncode, 0)
        for args in ((), ("unknown",)):
            result = self.run_launcher(*args)
            self.assertEqual(result.returncode, 2)
            self.assertIn("Usage: harness", result.stderr)

    def test_argument_forwarding_for_every_agent(self):
        args = ["--help", "two words", "", "*", "$(touch forbidden)", "--", "-x"]
        for agent in AGENTS:
            with self.subTest(agent=agent):
                result = self.launch(agent, *args)
                self.assertEqual(result["args"][-len(args):], args)

    def test_environment_loading_and_provider_isolation(self):
        config = self.home / "config/ai/model-router.env"
        config.parent.mkdir(parents=True)
        # The master exports settings even if the file uses plain assignments.
        config.write_text("MODEL_ROUTER_URL=http://router.example:4100/\nMODEL_ROUTER_API_KEY=test-client\n")
        claude = self.launch("claude")["env"]
        self.assertEqual(claude["ANTHROPIC_BASE_URL"], "http://router.example:4100/")
        self.assertEqual(claude["ANTHROPIC_AUTH_TOKEN"], "test-client")
        self.assertEqual(self.launch("muse")["env"]["META_API_KEY"], "test-client")
        grok = self.launch("grok")["env"]
        self.assertEqual(grok["GROK_HOME"], str(self.home / ".grok-t3/model-router"))
        self.assertEqual(grok["GROK_MODELS_BASE_URL"], "http://router.example:4100/v1")
        self.assertEqual(grok["GROK_MODELS_LIST_URL"], "http://router.example:4100/v1/models")
        self.assertEqual(grok["XAI_API_KEY"], "test-client")
        for agent in ("opencode", "pi", "qwen", "copilot", "kimi", "mcode", "kilo"):
            with self.subTest(agent=agent):
                self.assertEqual(self.launch(agent)["env"], {
                    "MODEL_ROUTER_URL": "http://router.example:4100/",
                    "MODEL_ROUTER_API_KEY": "test-client",
                })

    def test_router_defaults_and_inherited_overrides(self):
        default = self.launch("claude")["env"]
        self.assertEqual(default["MODEL_ROUTER_URL"], "http://127.0.0.1:4000")
        self.assertEqual(default["MODEL_ROUTER_API_KEY"], "unused")
        self.env.update(MODEL_ROUTER_URL="http://override:5000", MODEL_ROUTER_API_KEY="override")
        self.assertEqual(self.launch("claude")["env"]["ANTHROPIC_AUTH_TOKEN"], "override")
        self.env.pop("XDG_CONFIG_HOME")
        config = self.home / ".config/ai/model-router.env"
        config.parent.mkdir(parents=True)
        config.write_text("MODEL_ROUTER_API_KEY=home-config\n")
        self.assertEqual(self.launch("claude")["env"]["ANTHROPIC_AUTH_TOKEN"], "home-config")

    def test_codex_overrides_precede_user_arguments(self):
        args = self.launch("codex", "-c", 'model="local-llama"')["args"]
        self.assertEqual(args[:6], ["-c", 'model_provider="model_router"', "-c", 'model="deepseek-v4.1-flash"', "-c", 'service_tier="default"'])
        self.assertIn('model_providers.model_router.base_url="http://127.0.0.1:4000/v1"', args)
        self.assertIn('model_providers.model_router.env_key="MODEL_ROUTER_API_KEY"', args)
        self.assertIn('model_providers.model_router.wire_api="responses"', args)
        self.assertEqual(args[-2:], ["-c", 'model="local-llama"'])

    def test_sessions_are_fresh_for_every_invocation(self):
        for agent, key in (
            ("grok", "GROK_MODEL_ROUTER_SESSION"), ("fx", "FX_MODEL_ROUTER_SESSION"),
            ("vibe", "VIBE_ROUTER_SESSION"), ("vibe-acp", "VIBE_ROUTER_SESSION"),
            ("kimi-cli", "OPENAI_API_KEY"),
            ("antigravity", "GEMINI_API_KEY"),
            ("gemini", "GEMINI_API_KEY"),
        ):
            with self.subTest(agent=agent):
                first = self.launch(agent)["env"][key]
                second = self.launch(agent)["env"][key]
                self.assert_session(first)
                self.assertNotEqual(first, second)
        self.assertEqual(self.launch("kimi-cli")["env"]["OPENAI_BASE_URL"], "http://127.0.0.1:4000/v1")
        first, second = self.launch("cline"), self.launch("cline")
        self.assertEqual(first["args"][0], "--key")
        self.assert_session(first["args"][1])
        self.assertNotEqual(first["args"][1], second["args"][1])

    def test_deepseek_default_and_aliases(self):
        self.assertEqual(self.launch("deepseek")["args"], ["web"])
        self.assertEqual(self.launch("dsh")["args"], ["web"])
        self.assertEqual(self.launch("deepseek", "acp")["args"], ["acp"])
        self.assertEqual(self.launch("kimi-code", "doctor")["args"], ["doctor"])

    def test_t3_environment_selection_preserves_all_arguments(self):
        for agent in AGENTS:
            with self.subTest(agent=agent):
                explicit = self.launch(agent, "--version", "two words", "")
                self.env["HARNESS_AGENT"] = agent
                selected = self.launch("--version", "two words", "")
                self.env.pop("HARNESS_AGENT")
                self.assertEqual(selected["binary"], explicit["binary"])
                self.assertEqual(selected["args"][-3:], ["--version", "two words", ""])
                self.assertNotIn("HARNESS_AGENT", selected["env"])
                self.assertNotIn("HARNESS_AGENT", explicit["env"])
        self.env["HARNESS_AGENT"] = "claude"
        # T3's first argument belongs to the selected agent even if it is an agent name.
        self.assertEqual(self.launch("codex")["args"], ["codex"])
        self.assertEqual(self.launch()["args"], [])
        self.env["HARNESS_AGENT"] = "unknown"
        self.assertEqual(self.run_launcher("--version").returncode, 2)
        self.env["HARNESS_AGENT"] = ""
        self.assertEqual(self.launch("claude", "--help")["args"], ["--help"])

    def test_antigravity_cli_launches_directly_with_saved_or_explicit_model(self):
        master_link = self.bin / "harness"
        master_link.symlink_to(LAUNCHER)
        self.env["MODEL_ROUTER_URL"] = "http://router.example:4100/"
        for agent in ("antigravity", "agy"):
            with self.subTest(agent=agent):
                expected = ["--model", "Saved model", "--help", "two words"]
                result = self.launch(agent, "--help", "two words", executable=master_link)
                self.assertEqual(result["binary"], str(self.home / ".local/libexec/antigravity/agy"))
                self.assertEqual(result["args"], expected)
                self.assertEqual(result["env"]["GOOGLE_GEMINI_BASE_URL"], "http://router.example:4100")
                self.assert_session(result["env"]["GEMINI_API_KEY"])
                self.env["HARNESS_AGENT"] = agent
                self.assertEqual(self.launch("--help", "two words", executable=master_link)["args"], expected)
                self.env.pop("HARNESS_AGENT")
        for args in (["--model", "explicit", "--help"], ["--model=explicit", "--help"]):
            self.assertEqual(self.launch("antigravity", *args)["args"], args)
        (self.home / ".gemini/antigravity-cli/settings.json").write_text("{}")
        self.assertEqual(self.launch("antigravity")["args"], ["--model", "deepseek-v4.1-flash"])

    def test_antigravity_acp_launches_installed_release_directly(self):
        paths = self.start_router_inventory()
        result = self.launch("antigravity-acp", "two words", "")
        self.assertEqual(result["binary"], str(self.home / ".t3/tools/antigravity-acp/linux-x64/versions/test-release/agy_acp_server.par"))
        self.assertEqual(result["args"], ["two words", ""])
        self.assertEqual(result["env"]["AGY_LLM_GATEWAY_URL"], self.env["MODEL_ROUTER_URL"])
        self.assertEqual(result["env"]["GEMINI_HOME"], str(self.home / ".config/ai/antigravity-router-acp"))
        self.assertEqual(result["env"]["AGY_ACP_ENABLE_GATEWAY_AUTH"], "1")
        self.assertEqual(result["env"]["AGY_ACP_DEFAULT_MODEL"], "deepseek-v4.1-flash")
        self.assertEqual(result["env"]["AGY_LLM_GATEWAY_MODELS"], "deepseek-v4.1-flash,other-model")
        self.assert_session(result["env"]["AGY_LLM_GATEWAY_API_KEY"])
        self.env["HARNESS_AGENT"] = "antigravity-acp"
        second = self.launch("--help")
        self.assertEqual(second["args"], ["--help"])
        self.assertNotEqual(result["env"]["AGY_LLM_GATEWAY_API_KEY"], second["env"]["AGY_LLM_GATEWAY_API_KEY"])
        self.assertNotIn("HARNESS_AGENT", second["env"])
        self.assertEqual(paths, ["/v1/models", "/v1/models"])

    def test_gemini_uses_default_node_installation_and_router_endpoint(self):
        self.env["MODEL_ROUTER_URL"] = "http://127.0.0.1:4100/"
        args = ["--model", "other-model", "--prompt", "two words"]
        fallback = self.launch("gemini", *args)
        self.assertEqual(fallback["binary"], str(self.bin / "node"))
        entry = self.home / "dev/gemini-cli/packages/cli/dist/index.js"
        self.assertEqual(fallback["args"], [str(entry), *args])
        installed = self.home / "data/fnm/aliases/default/bin/node"
        self.write_executable(installed, self.recorder)
        result = self.launch("gemini", *args)
        self.assertEqual(result["binary"], str(installed))
        self.assertEqual(result["args"], [str(entry), *args])
        self.assertEqual(result["env"]["GOOGLE_GEMINI_BASE_URL"], "http://127.0.0.1:4100")
        self.assert_session(result["env"]["GEMINI_API_KEY"])
        self.assertEqual(result["env"]["GEMINI_MODEL_ROUTER"], "1")
        self.env["GEMINI_CLI_REPO"] = str(self.home / "not-built")
        missing = self.run_launcher("gemini", "--acp")
        self.assertEqual(missing.returncode, 1)
        self.assertIn("build Gemini CLI", missing.stderr)

    def test_provider_templates_invoke_the_master(self):
        templates = (
            "claude/t3-provider.json", "codex/t3-router-provider.json",
            "grok/t3-router-provider.json", "muse/t3-provider.json", "fx/t3-provider.json",
            "antigravity/acp/t3-provider.json", "deepseek/t3-provider.json",
            "vibe/t3-provider.json", "kimi-cli/t3-provider.json",
            "kilo/t3-provider.json",
            "gemini/t3-provider.json",
        )
        args = ["--version", "two words", ""]
        self.start_router_inventory()
        for template in templates:
            with self.subTest(template=template):
                instance, = json.loads((ROOT / template).read_text()).values()
                config = instance["config"]
                self.assertEqual(Path(config.get("binaryPath", config.get("commandPath"))).name, "harness")
                selection = [v["value"] for v in instance.get("environment", []) if v["name"] == "HARNESS_AGENT"]
                local = config.get("source") == "local"
                if local:
                    self.assertFalse(selection)
                    launch_args = config["commandArgs"] + args
                    selected_agent = launch_args[0]
                else:
                    self.assertEqual(len(selection), 1)
                    self.env["HARNESS_AGENT"] = selection[0]
                    selected_agent = selection[0]
                    launch_args = args
                    # Registry overrides retain the registry distribution's arguments.
                    if config.get("source") == "registry":
                        registry_args = {
                            "kimi-cli": ["acp"], "kilo": ["acp"], "gemini": ["--acp"],
                        }.get(selected_agent, [])
                        launch_args = registry_args + args
                result = self.launch(*launch_args)
                self.env.pop("HARNESS_AGENT", None)
                self.assertEqual(result["args"][-len(args):], args)
                self.assertNotIn("HARNESS_AGENT", result["env"])

    def test_kilo_t3_uses_shared_router_settings_and_registry_arguments(self):
        provider, = json.loads((ROOT / "kilo/t3-provider.json").read_text()).values()
        self.assertEqual([entry["name"] for entry in provider["environment"]], ["HARNESS_AGENT"])
        config = self.home / "config/ai/model-router.env"
        config.parent.mkdir(parents=True)
        config.write_text("MODEL_ROUTER_URL=http://shared-router:4100\nMODEL_ROUTER_API_KEY=shared-client\n")
        self.env.update({entry["name"]: entry["value"] for entry in provider["environment"]})
        result = self.launch("acp", "--port", "0")
        self.assertEqual(result["binary"], str(self.home / ".kilo/bin/kilo"))
        self.assertEqual(result["args"], ["acp", "--port", "0"])
        self.assertEqual(result["env"]["MODEL_ROUTER_URL"], "http://shared-router:4100")
        self.assertEqual(result["env"]["MODEL_ROUTER_API_KEY"], "shared-client")
        self.assertNotIn("HARNESS_AGENT", result["env"])

    def test_exit_status_is_preserved(self):
        self.env["TEST_EXIT_CODE"] = "37"
        self.assertEqual(self.run_launcher("claude").returncode, 37)


class SkillsTest(unittest.TestCase):
    """Keep the shared catalog and every harness's skill path in sync."""

    SHARED = ROOT / "shared" / "skills"
    SKIP = {"shared", "tests", "__pycache__", "deepseek"}

    def shared_skills(self):
        return {path.name for path in self.SHARED.iterdir() if path.is_dir()}

    def harness_dirs(self):
        return {
            path.name: path
            for path in ROOT.iterdir()
            if path.is_dir() and path.name not in self.SKIP
        }

    def test_harnesses_do_not_duplicate_the_shared_catalog(self):
        self.assertIn("technical-blog-editor", self.shared_skills())
        for name, path in self.harness_dirs().items():
            with self.subTest(harness=name):
                self.assertFalse(
                    (path / "skills").exists() or (path / "skills").is_symlink(),
                    f"{name} should point at shared/skills, not keep its own copy",
                )

    def run_setup(self, home, *args):
        env = {
            **os.environ,
            "HOME": str(home),
            "XDG_CONFIG_HOME": str(home / "config"),
            "XDG_DATA_HOME": str(home / "data"),
            "COPILOT_HOME": str(home / "copilot"),
            "CLAUDE_CONFIG_DIR": str(home / "claude"),
            "CODEX_HOME": str(home / "codex"),
        }
        return subprocess.run(
            [str(ROOT / "setup.sh"), *args],
            env=env,
            check=True,
            capture_output=True,
            text=True,
        )

    def test_setup_writes_only_symlinks(self):
        with tempfile.TemporaryDirectory() as temp:
            home = Path(temp)
            self.run_setup(home)

            file_links = {
                ".local/bin/harness": ROOT / "harness",
                "config/ai/model-router.env": ROOT / "shared/model-router.env",
                "config/systemd/user/model-router.service": (
                    ROOT / "shared/model-router.service"
                ),
                "claude/settings.json": ROOT / "claude/settings.json",
                "copilot/settings.json": ROOT / "copilot/settings.json",
                ".grok/config.toml": ROOT / "grok/config.toml",
                ".grok-t3/model-router/config.toml": ROOT / "grok/router-config.toml",
                "config/opencode/opencode.json": ROOT / "opencode/opencode.json",
                ".pi/agent/settings.json": ROOT / "pi/settings.json",
                ".qwen/settings.json": ROOT / "qwen/settings.json",
                ".gemini/settings.json": ROOT / "gemini/settings.json",
                ".kimi-code/config.toml": ROOT / "kimi-code/config.toml",
                ".kimi/config.toml": ROOT / "kimi-cli/config.toml",
                ".minimax/config.yaml": ROOT / "mcode/config.yaml",
                ".gemini/antigravity-cli/settings.json": (
                    ROOT / "antigravity/cli/settings.json"
                ),
                "config/muse/settings.json": ROOT / "muse/settings.json",
                ".fx/settings.json": ROOT / "fx/settings.json",
                "config/kilo/kilo.json": ROOT / "kilo/kilo.json",
                ".vibe/config.toml": ROOT / "vibe/config.toml",
                ".cline/data/settings/cline_mcp_settings.json": (
                    ROOT / "cline/cline_mcp_settings.json"
                ),
            }
            for destination, source in file_links.items():
                with self.subTest(destination=destination):
                    link = home / destination
                    self.assertTrue(link.is_symlink())
                    self.assertEqual(link.resolve(), source.resolve())

            directory_links = {
                "claude/skills": self.SHARED,
                ".cline/skills": self.SHARED,
                ".minimax/skills": self.SHARED,
                ".gemini/config/skills": self.SHARED,
                ".gemini/skills": self.SHARED,
                ".fx/skills": self.SHARED,
                ".kimi-code/skills": self.SHARED,
                ".kimi/skills": self.SHARED,
                "codex/skills/shared": self.SHARED,
            }
            for destination, source in directory_links.items():
                with self.subTest(destination=destination):
                    link = home / destination
                    self.assertTrue(link.is_symlink())
                    self.assertEqual(link.resolve(), source.resolve())

            for source in self.SHARED.iterdir():
                if not source.is_dir():
                    continue
                with self.subTest(destination="config/muse/skills", skill=source.name):
                    link = home / "config/muse/skills" / source.name
                    self.assertTrue(link.is_symlink())
                    self.assertEqual(link.resolve(), source.resolve())

            self.run_setup(home, "--check")

    def test_setup_keeps_muse_store_real(self):
        with tempfile.TemporaryDirectory() as temp:
            home = Path(temp)
            muse = home / "config/muse"
            muse.mkdir(parents=True)
            (muse / "skills").symlink_to(self.SHARED)

            self.run_setup(home)

            skills = muse / "skills"
            self.assertTrue(skills.is_dir())
            self.assertFalse(skills.is_symlink())
            for source in self.SHARED.iterdir():
                if not source.is_dir():
                    continue
                link = skills / source.name
                self.assertTrue(link.is_symlink())
                self.assertEqual(link.resolve(), source.resolve())
            self.assertTrue((self.SHARED / "hyperframes/SKILL.md").is_file())

    def test_setup_backs_up_replaced_paths(self):
        with tempfile.TemporaryDirectory() as temp:
            home = Path(temp)
            settings = home / ".fx/settings.json"
            settings.parent.mkdir()
            settings.write_text("{}\n")

            self.run_setup(home)

            self.assertTrue(settings.is_symlink())
            backups = list(
                (home / "data/dots-backups").glob("harnesses-*/.fx/settings.json")
            )
            self.assertEqual(len(backups), 1)
            self.assertEqual(backups[0].read_text(), "{}\n")

    def test_explicit_skill_settings_point_at_the_shared_catalog(self):
        expected = "~/dev/dots/harnesses/shared/skills"

        json_cases = {
            "copilot": ("settings.json", lambda data: data["skillDirectories"]),
            "pi": ("settings.json", lambda data: data["skills"]),
            "qwen": ("settings.json", lambda data: data["skills"]["directories"]),
            "opencode": ("opencode.json", lambda data: data["skills"]),
            "kilo": ("kilo.json", lambda data: data["skills"]),
        }
        for harness, (filename, getter) in json_cases.items():
            with self.subTest(harness=harness):
                data = json.loads((ROOT / harness / filename).read_text())
                self.assertEqual(getter(data), [expected])

        toml_cases = {
            "grok": ("config.toml", lambda data: data["skills"]["paths"]),
            "vibe": ("config.toml", lambda data: data["skill_paths"]),
            "kimi-code": ("config.toml", lambda data: data["extra_skill_dirs"]),
            "kimi-cli": ("config.toml", lambda data: data["extra_skill_dirs"]),
        }
        for harness, (filename, getter) in toml_cases.items():
            with self.subTest(harness=harness):
                with (ROOT / harness / filename).open("rb") as handle:
                    data = tomllib.load(handle)
                self.assertEqual(getter(data), [expected])

        with (ROOT / "grok/router-config.toml").open("rb") as handle:
            self.assertEqual(
                tomllib.load(handle)["skills"]["paths"], [expected]
            )


if __name__ == "__main__":
    unittest.main()
