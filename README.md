<h1 align="center">OpenAI Codex CLI</h1>

<p align="center"><strong>Install</strong>: download the right binary for your OS/arch</p>

<p align="center"><strong>Codex CLI</strong> is a coding agent from OpenAI that runs locally on your computer.</br>If you are looking for the <em>cloud-based agent</em> from OpenAI, <strong>Codex Web</strong>, see <a href="https://chatgpt.com/codex">chatgpt.com/codex</a>.</p>

<p align="center">
  <img src="./.github/codex-cli-splash.png" alt="Codex CLI splash" width="80%" />
  </p>

---

## Quickstart

### Installing and running Codex CLI

Install directly from this fork’s GitHub Release binary. No credentials required.

Option A — one‑liner install script (recommended):

```shell
REPO="buihongduc132/codex" bash -c "$(curl -fsSL https://raw.githubusercontent.com/buihongduc132/codex/main/scripts/install.sh)"
```

- To pin a version (tag), set `VERSION`, e.g.:

```shell
REPO="buihongduc132/codex" VERSION="v0.1.0" bash -c "$(curl -fsSL https://raw.githubusercontent.com/buihongduc132/codex/main/scripts/install.sh)"
```

Option B — manual download:

1) Go to this fork’s Releases page https://github.com/buihongduc132/codex/releases and download the appropriate archive for your platform, named like:

- macOS
  - Apple Silicon/arm64: `codex-aarch64-apple-darwin.tar.gz`
  - x86_64 (older Mac hardware): `codex-x86_64-apple-darwin.tar.gz`
- Linux
  - x86_64: `codex-x86_64-unknown-linux-gnu.tar.gz` (or `-musl` if you built musl)
  - arm64: `codex-aarch64-unknown-linux-gnu.tar.gz` (or `-musl` if you built musl)

2) Extract and move the `codex` binary somewhere on your `PATH`, e.g. `~/.local/bin`.

Then run `codex` to get started:

```shell
codex
```

<details>
<summary>Maintainers: building the release archive locally</summary>

Use the helper script to build and package an archive for your host platform:

```shell
scripts/build_release.sh
```

This produces `dist/codex-<target-triple>.tar.gz` and a `.sha256` checksum. Upload the tarball to this fork’s Release as an asset. The install script will fetch from `releases/latest` by default, or a specific tag if `VERSION` is set.

</details>

### Using Codex with your ChatGPT plan

<p align="center">
  <img src="./.github/codex-cli-login.png" alt="Codex CLI login" width="80%" />
  </p>

Run `codex` and select **Sign in with ChatGPT**. We recommend signing into your ChatGPT account to use Codex as part of your Plus, Pro, Team, Edu, or Enterprise plan. [Learn more about what's included in your ChatGPT plan](https://help.openai.com/en/articles/11369540-codex-in-chatgpt).

You can also use Codex with an API key, but this requires [additional setup](./docs/authentication.md#usage-based-billing-alternative-use-an-openai-api-key). If you previously used an API key for usage-based billing, see the [migration steps](./docs/authentication.md#migrating-from-usage-based-billing-api-key). If you're having trouble with login, please comment on [this issue](https://github.com/openai/codex/issues/1243).

### Model Context Protocol (MCP)

Codex CLI supports [MCP servers](./docs/advanced.md#model-context-protocol-mcp). Enable by adding an `mcp_servers` section to your `~/.codex/config.toml`.


### Configuration

Codex CLI supports a rich set of configuration options, with preferences stored in `~/.codex/config.toml`. For full configuration options, see [Configuration](./docs/config.md).

---

### Aliases & Profiles

- `codex` – The original, official CLI binary and defaults.
- `qox` – Our custom Codex profiles/settings (e.g., under `prompts/config/codex`).
- `qoo` – Our dev build used from this repo (or a previous build artifact).
- `qol` / `qoo-lan` – Client profile that talks to the LAN bridge; requires the bridge running on a host in your network and pairing with its components.

LAN usage (client profile example):

```
[model_providers.lan]
name = "LAN ChatGPT Proxy"
base_url = "http://<BRIDGE_HOST>:4050/v1"
wire_api = "responses"
requires_openai_auth = false

[profiles.qoo-lan]
model = "local_md"        # bridge maps to gpt-5
model_provider = "lan"
approval_policy = "never"
```

Run: `qoo --profile qoo-lan e "..."` to target the LAN bridge.

Note: The LAN bridge (qoo-bridge) lives in a separate repository to avoid ambiguity here. Clone and run it separately (manage with PM2 if desired) and point `qoo-lan` to it.

---

### Docs & FAQ

- [**Getting started**](./docs/getting-started.md)
  - [CLI usage](./docs/getting-started.md#cli-usage)
  - [Running with a prompt as input](./docs/getting-started.md#running-with-a-prompt-as-input)
  - [Example prompts](./docs/getting-started.md#example-prompts)
  - [Memory with AGENTS.md](./docs/getting-started.md#memory-with-agentsmd)
  - [Configuration](./docs/config.md)
- [**Sandbox & approvals**](./docs/sandbox.md)
- [**Authentication**](./docs/authentication.md)
  - [Auth methods](./docs/authentication.md#forcing-a-specific-auth-method-advanced)
  - [Login on a "Headless" machine](./docs/authentication.md#connecting-on-a-headless-machine)
- [**Advanced**](./docs/advanced.md)
  - [Non-interactive / CI mode](./docs/advanced.md#non-interactive--ci-mode)
  - [Tracing / verbose logging](./docs/advanced.md#tracing--verbose-logging)
  - [Model Context Protocol (MCP)](./docs/advanced.md#model-context-protocol-mcp)
- [**Zero data retention (ZDR)**](./docs/zdr.md)
- [**Contributing**](./docs/contributing.md)
- [**Install & build**](./docs/install.md)
  - [System Requirements](./docs/install.md#system-requirements)
  - [DotSlash](./docs/install.md#dotslash)
  - [Build from source](./docs/install.md#build-from-source)
- [**FAQ**](./docs/faq.md)
- [**Open source fund**](./docs/open-source-fund.md)

---

## License

This repository is licensed under the [Apache-2.0 License](LICENSE).
