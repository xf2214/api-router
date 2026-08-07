<a name="top"></a>

<div align="center">
  <img src="docs/images/hero-banner.svg" alt="API Router Hero Banner" width="100%" style="max-width: 1280px; border-radius: 18px; box-shadow: 0 24px 60px rgba(34, 211, 238, 0.15);"/>
</div>

<br/>

<div align="center">
  <h3><samp>
    <code>http://127.0.0.1:6123</code>
  </samp></h3>
  <p>One local endpoint. All LLMs. Zero lock-in.<br/>
  <em>一个本地端点，调用所有大模型</em></p>
</div>

<br/>

<div align="center">

  [![License: MIT](https://img.shields.io/badge/License-MIT-22d3ee?style=for-the-badge&logoColor=white&labelColor=0b1020)](LICENSE)
  [![Rust 1.97+](https://img.shields.io/badge/Rust-1.97%2B-f74c00?style=for-the-badge&logo=rust&logoColor=white&labelColor=0b1020)](https://www.rust-lang.org/tools/install)
  [![Vue 3.4+](https://img.shields.io/badge/Vue-3.4%2B-42b883?style=for-the-badge&logo=vuedotjs&logoColor=white&labelColor=0b1020)](https://vuejs.org/)
  [![Tauri v2](https://img.shields.io/badge/Tauri-v2-24C8D8?style=for-the-badge&logo=tauri&logoColor=white&labelColor=0b1020)](https://tauri.app/)

  <br/>

  [![GitHub Stars](https://img.shields.io/github/stars/api-router/api-router?style=for-the-badge&logo=github&label=Stars&color=FFD166&labelColor=0b1020)](https://github.com/api-router/api-router/stargazers)
  [![GitHub Release](https://img.shields.io/github/v/release/api-router/api-router?display_name=tag&style=for-the-badge&logo=semver&label=Release&color=a78bfa&labelColor=0b1020)](https://github.com/api-router/api-router/releases)
  [![CI](https://img.shields.io/github/actions/workflow/status/api-router/api-router/ci.yml?branch=main&style=for-the-badge&logo=githubactions&label=CI&color=34d399&labelColor=0b1020)](.github/workflows/ci.yml)
  [![Platform](https://img.shields.io/badge/Platform-Win%20%7C%20macOS-f472b6?style=for-the-badge&logoColor=white&labelColor=0b1020)](#)

</div>

<br/>

<div align="center">
  <samp>
  <kbd><a href="#quick-start">🚀 Quick Start</a></kbd>
  &nbsp;·&nbsp;
  <kbd><a href="#releases">📥 Releases</a></kbd>
  &nbsp;·&nbsp;
  <kbd><a href="#showcase">🖼️ Showcase</a></kbd>
  &nbsp;·&nbsp;
  <kbd><a href="#architecture">🏗️ Architecture</a></kbd>
  &nbsp;·&nbsp;
  <kbd><a href="#faq">❓ FAQ</a></kbd>
  </samp>
  <br/><br/>
  <a href="README.md">中文</a> · <strong>English</strong>
</div>

---

## ✨ Core Features

> API Router is a **locally running**, lightweight desktop application built with Tauri v2 + Rust + Vue 3 + TypeScript. It unifies APIs from multiple LLM providers into a single local OpenAI-compatible endpoint — configure once, and every model is reachable from one address.

<br/>

<table align="center" width="100%" border="0" cellspacing="0" cellpadding="8">
  <tr>
    <td width="50%" valign="top">
      <div style="border:1px solid #1e293b;border-radius:14px;padding:20px 22px;background:#0b1020;height:100%">
        <h4 align="center" style="margin:0 0 8px 0;color:#22d3ee">🔗 Unified OpenAI-Compatible API</h4>
        <p align="center" style="margin:0;color:#94a3b8;font-size:14px;line-height:1.6">
          Standard endpoints exposed locally:<br/>
          <code>/v1/models</code> · <code>/v1/chat/completions</code><br/>
          <code>/v1/completions</code> · <code>/v1/embeddings</code>
        </p>
      </div>
    </td>
    <td width="50%" valign="top">
      <div style="border:1px solid #1e293b;border-radius:14px;padding:20px 22px;background:#0b1020;height:100%">
        <h4 align="center" style="margin:0 0 8px 0;color:#a78bfa">🗂️ Multi-Provider Management</h4>
        <p align="center" style="margin:0;color:#94a3b8;font-size:14px;line-height:1.6">
          Works with OpenAI · Anthropic · Gemini · Azure<br/>
          DeepSeek · Zhipu · SenseNova · OpenRouter<br/>
          Add / edit / delete / toggle / health check
        </p>
      </div>
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <div style="border:1px solid #1e293b;border-radius:14px;padding:20px 22px;background:#0b1020;height:100%">
        <h4 align="center" style="margin:0 0 8px 0;color:#34d399">🧭 Smart Model Routing</h4>
        <p align="center" style="margin:0;color:#94a3b8;font-size:14px;line-height:1.6">
          Map local aliases to multiple upstream backends<br/>
          Strategies: <strong>Priority</strong> · <strong>Weighted</strong> · <strong>Round-Robin</strong><br/>
          Tiered fallbacks + Group routing supported
        </p>
      </div>
    </td>
    <td width="50%" valign="top">
      <div style="border:1px solid #1e293b;border-radius:14px;padding:20px 22px;background:#0b1020;height:100%">
        <h4 align="center" style="margin:0 0 8px 0;color:#f59e0b">🛡️ High Reliability</h4>
        <p align="center" style="margin:0;color:#94a3b8;font-size:14px;line-height:1.6">
          Automatic <strong>retries</strong> on failure · <strong>Fallback</strong> handover<br/>
          <strong>Circuit breaker</strong> · QPS / concurrency <strong>throttling</strong><br/>
          Health checks with automatic back-end isolation
        </p>
      </div>
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <div style="border:1px solid #1e293b;border-radius:14px;padding:20px 22px;background:#0b1020;height:100%">
        <h4 align="center" style="margin:0 0 8px 0;color:#f472b6">📡 Streaming & Non-Streaming Passthrough</h4>
        <p align="center" style="margin:0;color:#94a3b8;font-size:14px;line-height:1.6">
          Full byte-level <strong>SSE streaming</strong> forwarding<br/>
          Client-side <code>stream</code> field respected; streaming default on<br/>
          Zero added latency · no memory ballooning
        </p>
      </div>
    </td>
    <td width="50%" valign="top">
      <div style="border:1px solid #1e293b;border-radius:14px;padding:20px 22px;background:#0b1020;height:100%">
        <div align="center">
          <img src="docs/images/feature-local-first.svg" alt="Local First" width="160" style="border-radius:10px"/>
        </div>
        <h4 align="center" style="margin:0 0 8px 0;color:#22d3ee">🔒 Local-First & Private</h4>
        <p align="center" style="margin:0;color:#94a3b8;font-size:14px;line-height:1.6">
          All data stays on your machine<br/>
          API Keys encrypted with OS native keyring<br/>
          <strong>Plaintext keys are never written to disk</strong>
        </p>
      </div>
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <div style="border:1px solid #1e293b;border-radius:14px;padding:20px 22px;background:#0b1020;height:100%">
        <h4 align="center" style="margin:0 0 8px 0;color:#fb7185">📊 Real-Time Monitoring</h4>
        <p align="center" style="margin:0;color:#94a3b8;font-size:14px;line-height:1.6">
          Service status · Configured model count<br/>
          Daily request volume · <strong>latency distribution</strong><br/>
          Token usage · Log search + CSV export
        </p>
      </div>
    </td>
    <td width="50%" valign="top">
      <div style="border:1px solid #1e293b;border-radius:14px;padding:20px 22px;background:#0b1020;height:100%">
        <h4 align="center" style="margin:0 0 8px 0;color:#60a5fa">💻 Cross-Platform Desktop</h4>
        <p align="center" style="margin:0;color:#94a3b8;font-size:14px;line-height:1.6">
          ⚪ <strong>Windows 10 / 11</strong> x64 (first-class citizen)<br/>
          🍎 macOS 12+ · Intel / Apple Silicon (close second)<br/>
          Installer &lt; 30 MB · Runtime RAM &lt; 150 MB
        </p>
      </div>
    </td>
  </tr>
</table>

---

<a name="architecture"></a>
## 🏗️ Architecture

> Four-layer data flow: Your App → Local HTTP Server → Rust Routing Core → Upstream LLM APIs. Keys and config stay on-device end-to-end.

<div align="center">
  <a href="docs/images/architecture-diagram.svg" target="_blank">
    <img src="docs/images/architecture-diagram.svg" alt="API Router Architecture Diagram" width="100%" style="max-width:960px;border-radius:16px;"/>
  </a>
  <p><em>▲ Click to open the full SVG (vector, infinitely scalable). See <a href="docs/ARCHITECTURE.md">ARCHITECTURE.md</a> for deep-dive.</em></p>
</div>

---

<a name="showcase"></a>
## 🖼️ Showcase Gallery

> All screenshots taken from the real app. Click any image to open the full-size original.

<div align="center">

| Overview Dashboard | Provider Management |
|:---:|:---:|
| [![01-Overview](docs/images/01-overview.png)](docs/images/01-overview.png) | [![02-Providers](docs/images/02-providers.png)](docs/images/02-providers.png) |
| Service health · KPIs · Provider status | CRUD · Health checks · Auto model fetch |

| Provider Form | Routing Tree |
|:---:|:---:|
| [![03-Provider-Form](docs/images/03-provider-form.png)](docs/images/03-provider-form.png) | [![05-Routing](docs/images/05-routing.png)](docs/images/05-routing.png) |
| Base URL · API Key · Model picker · Advanced | Groups · Tiers · Weights · Priority · RR |

| Monitoring Stats | Request Logs |
|:---:|:---:|
| [![06-Monitoring](docs/images/06-monitoring.png)](docs/images/06-monitoring.png) | [![07-Logs](docs/images/07-logs.png)](docs/images/07-logs.png) |
| Per-provider/model · Latency · Tokens · Success % | Live feed · Filter · Search · CSV export · Details |

| Provider Card Detail | System Settings |
|:---:|:---:|
| [![04-Provider-Detail](docs/images/04-provider-card-detail.png)](docs/images/04-provider-card-detail.png) | [![08-Settings](docs/images/08-settings.png)](docs/images/08-settings.png) |
| Latency bars · Status chips · Quick actions | Port · Token · Timeout · Breaker · Cache · Theme · About |

</div>

---

<a name="quick-start"></a>
## 🚀 Quick Start

### ① Prerequisites

| Dependency | Required | Notes |
|------------|:--------:|-------|
| 🦀 Rust | ≥ 1.97.1 | [Official installer](https://www.rust-lang.org/tools/install) |
| 🟢 Node.js + npm | LTS recommended | [Node.js website](https://nodejs.org/) |
| 🛠 WiX / NSIS | Optional | Only needed to **build installers** on Windows ([WiX](https://wixtoolset.org/) / [NSIS](https://nsis.sourceforge.io/)) |

### ② Clone & Dependencies

```bash
git clone https://github.com/api-router/api-router.git
cd api-router
npm install
```

### ③ Start Development Mode

```bash
npm run tauri:dev
```

### ④ Configure Your First Provider

1. Open the app → 🗂️ **Providers** tab → click **+ Add Provider**
2. Fill in **Name** · **Base URL** · **API Key**
3. Click **🔍 Fetch Models**, tick the models you need → **Save**
4. ✅ The app automatically creates **local model mappings** for every selected model

### ⑤ Call the Local API 🎉

```bash
curl http://127.0.0.1:6123/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4o",
    "messages": [{"role": "user", "content": "Hello, API Router!"}],
    "stream": false
  }'
```

> 📖 Full endpoint reference: [docs/API_SPEC.md](docs/API_SPEC.md). End-user walk-through: [docs/USER_GUIDE.md](docs/USER_GUIDE.md).

---

<a name="releases"></a>
## 📥 Release & Downloads

<div align="center">

  <img src="https://img.shields.io/badge/Version-v0.1.0--alpha-a78bfa?style=for-the-badge&logo=semver&labelColor=0b1020" alt="Current Version"/>
  <img src="https://img.shields.io/badge/Status-Alpha-orange?style=for-the-badge&logoColor=white&labelColor=0b1020" alt="Status"/>
  <img src="https://img.shields.io/badge/Platform-Windows%20x64%20ONLY-f472b6?style=for-the-badge&logo=windows11&logoColor=white&labelColor=0b1020" alt="Platform Windows Only"/>

</div>

<br/>

> ⚠️ **Currently Windows-only release.** macOS builds are coming in the next iteration (targeted for M2 milestone, end of Week 5). Linux / mobile are not scheduled for the near term.

### 🪟 Supported Windows Versions

| Item | Requirement |
|------|-------------|
| **OS** | Windows 10 21H2 / Windows 11 (x64, 64-bit) |
| **CPU Arch** | AMD64 / Intel x86_64 (⚠️ ARM64 packages not provided yet — build from source if needed) |
| **WebView2 Runtime** | Preinstalled on Win11; Win10 users may be prompted to install or [get it here](https://developer.microsoft.com/microsoft-edge/webview2/) |
| **RAM** | ≥ 2 GB |
| **Free disk space** | ≥ 200 MB (installer + local cache + logs) |

### 📦 Release Artifacts (v0.1.0)

| Installer format | Recommended for | Example filename | Size |
|:-----------------|:----------------|:-----------------|:----:|
| **NSIS `.exe`** ✅ | Most users: double-click wizard, supports uninstall | `API Router_0.1.0_x64-setup.exe` | ~25 MB |
| **WiX `.msi`** | Enterprise deploy / SCCM / GPO silent install | `API Router_0.1.0_x64_en-US.msi` | ~28 MB |
| **Portable `.zip`** *(optional)* | Extract-and-run green version (create manually) | `API Router_0.1.0_x64_portable.zip` | ~22 MB |

### 🔗 Download Links

Grab the latest release from GitHub Releases:

<div align="center">
  <samp>
  <kbd><a href="https://github.com/api-router/api-router/releases/latest">
    <img src="https://img.shields.io/badge/Download-Latest_Release-22d3ee?style=for-the-badge&logo=github&logoColor=white&labelColor=0b1020" alt="Download Latest Release"/>
  </a></kbd>
  &nbsp;
  <kbd><a href="https://github.com/api-router/api-router/releases/tag/v0.1.0">
    <img src="https://img.shields.io/badge/Download-v0.1.0-FFD166?style=for-the-badge&logo=github&logoColor=white&labelColor=0b1020" alt="Download v0.1.0"/>
  </a></kbd>
  </samp>
  <p><em>Use a GitHub mirror (e.g. ghproxy / gh-proxy) if raw GitHub is slow in your region.</em></p>
</div>

### ✅ Verify After Install

1. Launch **API Router** from Start Menu or desktop shortcut
2. Look for the 🟢 tray icon appearing in the system tray
3. In a browser or terminal, verify the local endpoint:

   ```bash
   curl -s http://127.0.0.1:6123/v1/models | head -c 300
   ```

4. If you get JSON with `"object": "list"`, the local service is healthy 🎉

### 🔄 Update & Uninstall

- **Update**: download the new `.exe` / `.msi` and install over the top (config + keyring keys are preserved)
- **Uninstall**: Windows Settings → Apps → API Router → Uninstall; or use the "Uninstall API Router" Start Menu shortcut
- **Config file location**: `%APPDATA%\com.api-router.app\config\config.yaml` (kept by default; use the "Remove user data" checkbox during uninstall to wipe it)
- **Key storage**: Windows Credential Manager → Generic Credentials → `api-router.provider.*`

### 🍎 macOS & 🐧 Linux Roadmap

| Platform | Status | Expected | Notes |
|----------|:------:|:--------:|-------|
| 🪟 **Windows x64** | ✅ Released | Now | MSI + NSIS installers |
| 🍎 macOS (Intel x64) | 🛠 In progress | M2 / End of Week 5 | DMG + .app, requires signing + notarize |
| 🍎 macOS (Apple Silicon aarch64) | 🛠 In progress | M2 / End of Week 5 | Universal 2 binary |
| 🐧 Linux (deb / rpm / AppImage) | 📝 Planned | Post-M3 | Native Tauri v2 support; packaging scripts TBD |
| 📱 Android / iOS | ❌ Not planned | — | Desktop only for now |

> Building macOS installers requires a macOS host (12+) and Xcode. See [docs/macos-build.md](docs/macos-build.md).

---

## 📦 Build & Release

```bash
# 1) Frontend production bundle
npm run build

# 2) Rust release build (executable only, no installer packaging)
cd src-tauri
cargo build --release
  # Output: src-tauri/target/release/api-router(.exe)

# 3) Build installers (requires WiX/NSIS; downloads WebView runtime from GitHub)
cd ..
npm run tauri:build
  # Windows MSI: src-tauri/target/release/bundle/msi/API Router_<ver>_x64_en-US.msi
  # Windows NSIS: src-tauri/target/release/bundle/nsis/API Router_<ver>_x64-setup.exe
  # macOS DMG:   src-tauri/target/release/bundle/dmg/API Router_<ver>_aarch64.dmg (macOS host)
```

> 🍎 For macOS universal binary & notarization tips, see [docs/macos-build.md](docs/macos-build.md).

---

## 🧱 Project Structure

```
api-router/
├── docs/                       # Documentation
│   ├── images/                 # All image assets referenced by this README
│   ├── PRD.md                  # Product requirements
│   ├── ARCHITECTURE.md         # Technical architecture & design decisions
│   ├── API_SPEC.md             # Local OpenAI-compatible API spec
│   ├── USER_GUIDE.md           # End-user walk-through
│   ├── ROADMAP.md              # Release milestones & roadmap
│   └── UI_DESIGN_SPEC.md       # UI layout & design system notes
├── scripts/                    # Build / release helper scripts
├── src/                        # 🟢 Vue 3 frontend source
│   ├── components/             # Shared components / forms / widgets
│   ├── pages/                  # 6 main pages: Overview, Providers, ..., Settings
│   ├── services/tauri/         # Wrapped Tauri IPC callers
│   ├── stores/                 # Lightweight reactive stores
│   ├── i18n/                   # zh-CN / en-US locales
│   └── composables/            # useTheme / useToast / useCsvExport etc.
├── src-tauri/                  # 🦀 Rust + Tauri backend core
│   ├── src/
│   │   ├── config/             # Config validation + YAML persistence
│   │   ├── core/               # Routing strategies · Tiers · Groups
│   │   ├── infra/              # HTTP client · Breaker · Cache · Metrics · Transform
│   │   ├── server/             # Axum HTTP service · SSE streaming forwarder
│   │   └── tauri_impl/         # All #[tauri::command] front-end bridges
│   ├── Cargo.toml
│   ├── tauri.conf.json         # Desktop app metadata (window, icons, bundle)
│   └── build.rs
├── .github/workflows/ci.yml    # 3-platform CI: Rust checks · Frontend checks · Tauri smoke build
├── package.json
├── vite.config.ts
└── tsconfig.json
```

---

## 🛠️ Tech Stack

<div align="center" style="display:flex;flex-wrap:wrap;gap:12px;justify-content:center;align-items:center;">
  <img src="https://img.shields.io/badge/Tauri_v2-%23FFC131?style=for-the-badge&logo=tauri&logoColor=000" alt="Tauri"/>
  <img src="https://img.shields.io/badge/Rust_%2B_Tokio-dea584?style=for-the-badge&logo=rust&logoColor=000" alt="Rust + Tokio"/>
  <img src="https://img.shields.io/badge/Axum_0.7-black?style=for-the-badge&logoColor=white" alt="Axum"/>
  <img src="https://img.shields.io/badge/Vue_3_%2B_TypeScript-42b883?style=for-the-badge&logo=vuedotjs&logoColor=white" alt="Vue 3 + TypeScript"/>
  <img src="https://img.shields.io/badge/reqwest_0.12-339933?style=for-the-badge" alt="reqwest"/>
  <img src="https://img.shields.io/badge/YAML_Config-%23CB171E?style=for-the-badge&logo=yaml&logoColor=white" alt="YAML"/>
  <img src="https://img.shields.io/badge/Keyring_Encrypted-%230ea5e9?style=for-the-badge&logo=1password&logoColor=white" alt="Keyring"/>
  <img src="https://img.shields.io/badge/Vite_5-646CFF?style=for-the-badge&logo=vite&logoColor=white" alt="Vite"/>
  <img src="https://img.shields.io/badge/GitHub_Actions-%232671E5?style=for-the-badge&logo=githubactions&logoColor=white" alt="GitHub Actions"/>
</div>

---

<a name="faq"></a>
## ❓ Frequently Asked Questions

<details>
<summary><strong>Q: Why API Router instead of LiteLLM / OpenRouter / Portkey?</strong></summary>
<br/>
<p>They solve problems at different layers:</p>
<ul>
  <li>☁️ <strong>OpenRouter / Portkey</strong> are <strong>cloud-hosted</strong> aggregators; requests pass through third-party servers. Great if you want a drop-in managed service and don't have data-residency concerns. API Router runs <strong>100% locally</strong> — every request goes straight from your machine to the upstream provider.</li>
  <li>🐍 <strong>LiteLLM</strong> is a <em>Python library</em> you embed into your own app as a dependency. API Router is a <strong>standalone desktop app</strong> with a full GUI for provider management, health checks, monitoring dashboard, log browser, and one-click config import/export.</li>
  <li>🧩 <strong>They combine perfectly</strong>: add OpenRouter <em>as one provider</em> inside API Router for the broadest coverage, while wiring your direct-vendor accounts to API Router for zero-middleman latency & privacy.</li>
</ul>
</details>

<details>
<summary><strong>Q: Where are my API Keys stored? Is it safe? Is anything uploaded?</strong></summary>
<br/>
<ul>
  <li>🔐 <strong>Encrypted with your OS-native credential store</strong>: Windows uses <em>Windows Credential Manager</em>; macOS uses <em>Keychain Access</em>; Linux uses <em>DBus Secret Service</em> — all through the battle-tested <a href="https://crates.io/crates/keyring">keyring</a> crate.</li>
  <li>🚫 Your <code>config.yaml</code> stores <strong>only provider IDs, never plaintext keys</strong>. Even if the file leaks, nothing can be used.</li>
  <li>✈️ API Router <strong>sends zero telemetry and zero credentials anywhere</strong> besides the upstream provider Base URLs you explicitly configure.</li>
</ul>
</details>

<details>
<summary><strong>Q: Which platforms are supported? Any plans for Linux / Android / iOS?</strong></summary>
<br/>
<ul>
  <li>✅ <strong>Windows 10 / 11 x64</strong>: first-class citizen; CI validates every commit.</li>
  <li>✅ <strong>macOS 12+ (Intel · Apple Silicon)</strong>: close second, with a universal-binary build script in <a href="docs/macos-build.md">docs/macos-build.md</a>.</li>
  <li>🛠 <strong>Linux (deb/rpm/AppImage)</strong>: fully supported by the underlying Tauri 2 stack. Only missing the installer-packaging scripts and tray icons. <em>PRs welcome!</em></li>
  <li>📱 <strong>Android / iOS</strong>: not on the roadmap right now; feel free to open a discussion issue if that changes.</li>
</ul>
</details>

<details>
<summary><strong>Q: Which client apps does this work with out of the box?</strong></summary>
<br/>
<p>Any client that supports <em>“custom OpenAI-compatible Base URL”</em>:</p>
<ul>
  <li>🤖 ChatBox / NextChat / LobeChat / OpenWebUI</li>
  <li>📝 Obsidian Smart Connections / Logseq Copilot</li>
  <li>💻 VS Code Continue / Cursor (point it at an OpenAI-compatible endpoint)</li>
  <li>🎨 Anyquery / Dify / FastGPT / … and dozens more</li>
</ul>
<p>Just set <code>http://127.0.0.1:6123/v1</code> as the Base URL and use the local alias model names you set up inside API Router.</p>
</details>

<details>
<summary><strong>Q: What happens when one of my upstreams goes down?</strong></summary>
<br/>
<ul>
  <li>Automatic <strong>Fallback</strong>: moves on to the next configured backend target for that model (if any).</li>
  <li>Repeated failures on a single backend trigger the <strong>circuit breaker</strong>, which skips it for a cool-down period so your requests don't pile up against a dead endpoint.</li>
  <li>After the failure threshold a backend is marked unhealthy, and the periodic health-check thread probes it in the background until it recovers.</li>
</ul>
</details>

---

## 📚 Documentation Index

| Document | Contents |
|----------|----------|
| 📋 [Product Requirements](docs/PRD.md) | Positioning · Functional & non-functional reqs · Success metrics |
| 🏗️ [Technical Architecture](docs/ARCHITECTURE.md) | Modules · Data flow · Key design decisions |
| 📡 [Local API Spec](docs/API_SPEC.md) | Endpoints · Request / response formats · Streaming |
| 🗺️ [Roadmap](docs/ROADMAP.md) | Milestones · Completed / In-progress · Backlog |
| 🎨 [UI Design Spec](docs/UI_DESIGN_SPEC.md) | Page layout · Components · Density · Theming |
| 🤖 [AI Agent Guidelines](docs/AGENTS.md) | Human + AI collaboration rules · Code conventions · Quality gates |
| 👤 [User Guide](docs/USER_GUIDE.md) | End-user walk-through of every screen |
| 🍎 [macOS Build Notes](docs/macos-build.md) | Universal binaries · Signing · Notarization tips |
| 📝 [Changelog](CHANGELOG.md) | Per-version Added / Fixed / Known issues + download checksum matrix |

---

## 🤝 Contributing

We welcome **Issues** and **Pull Requests**! Please read [CONTRIBUTING.en-US.md](CONTRIBUTING.en-US.md) ([中文版](CONTRIBUTING.md)) for details on:
- Setting up a dev environment
- Conventional Commits commit format
- Rust / TypeScript code conventions
- Quality gates & CI checklist

Also please follow the community standards in [CODE_OF_CONDUCT.en-US.md](CODE_OF_CONDUCT.en-US.md) ([中文版](CODE_OF_CONDUCT.md)): be kind, respectful, and inclusive.

---

## 📄 License

This project is open-sourced under the [MIT License](LICENSE).

---

## 🙏 Acknowledgements

| Project | Used for |
|---------|----------|
| [Tauri](https://tauri.app/) | Cross-platform desktop framework — key to the &lt; 30 MB installer |
| [Rust](https://www.rust-lang.org/) | Foundation of the routing core, HTTP server, and key handling |
| [Tokio](https://tokio.rs/) | Async runtime |
| [Axum](https://github.com/tokio-rs/axum) | Local HTTP server (OpenAI-compatible endpoints) |
| [reqwest](https://github.com/seanmonstar/reqwest) | Upstream API client (rustls-tls) |
| [Vue 3](https://vuejs.org/) | Frontend UI |
| [keyring](https://crates.io/crates/keyring) | Cross-platform encrypted credential storage |
| [shields.io](https://shields.io/) | Every badge you see in this README |

<br/>

<div align="center">
  <samp>
  <a href="#top">⬆ Back to top</a>
  &nbsp;·&nbsp;
  <a href="https://github.com/api-router/api-router/issues/new/choose">🐛 Report a bug / 💡 Suggest a feature</a>
  </samp>
  <br/><br/>
  <sub>Made with 💙 using <code>Rust + Tauri + Vue 3</code> · Local-first. Your data stays yours.</sub>
</div>
