# HandyGemini

**HandyGemini** is a fork of [Handy](https://github.com/cjpais/Handy) that extends the base speech-to-text application with Google Gemini AI integration. It enables users to get AI-powered answers quickly from Gemini, together with user-selected context (e.g., highlighted text, screenshots, active window images).

## What's New in HandyGemini

HandyGemini adds comprehensive Gemini AI integration on top of Handy's core speech-to-text functionality:

### 🚀 Core Gemini Features

- **Google OAuth Authentication**: Secure login to Google accounts for authenticated Gemini API access
- **Direct Audio to Gemini**: Option to send audio directly to Gemini for transcription and AI processing (bypassing local transcription)
- **Gemini Model Selection**: Choose from multiple Gemini models:
  - `gemini-3-flash-preview` - Fast, efficient responses
  - `gemini-1.5-pro` - High-quality, detailed responses
  - `gemini-1.5-flash` - Balanced speed and quality
- **Conversation History**: Maintains context across multiple Gemini interactions for more natural conversations
- **Reset Button**: Clear conversation history to start fresh interactions

### 📸 Context Capture

- **Screenshot Support**: Capture and send screenshots with your questions
  - **Full Screen Capture**: Capture entire screen (all platforms)
  - **Active Window Capture**: macOS-specific active window capture (falls back to full screen on other platforms)
  - **Smart Instructions**: Automatically focuses Gemini on the main canvas area, ignoring UI elements
- **IP-based Location Context**: Automatically includes user's approximate location (derived from IP) for personalized responses

### 🎨 User Experience Enhancements

- **Gemini Popup Window**: Beautiful popup interface to display Gemini responses with:
  - Full markdown rendering support
  - LaTeX math rendering (with proper handling of currency symbols)
  - Auto-scrolling for long responses
  - Conversation history display
- **Status Overlays**: Real-time status updates during Gemini operations:
  - "Sending to Gemini..." - When audio/context is being sent
  - "Answer is ready" - When response is received
  - "No audio detected" - When insufficient audio is captured
- **Empty Audio Detection**: Prevents sending empty or insufficient audio to Gemini API
- **Visual Distinction**: Added "G" badge overlay to app icons to distinguish from base Handy

### 🔧 Technical Improvements

- **Robust Error Handling**: Better handling of edge cases and API errors
- **Platform-Specific Optimizations**: macOS-specific active window capture with fallback mechanisms
- **Improved State Management**: Consistent UI state management for overlay and transcription flows

## Current Limitations & Missing Features

### ⚠️ Build Status

**Currently Available:**
- ✅ **macOS (Apple Silicon)**: DMG installer available in [releases](https://github.com/billylo1/HandyGemini/releases)
- ✅ **macOS Notarization**: Builds are signed and notarized with Developer ID Application certificate

**Not Yet Available:**
- ❌ **Windows Builds**: Windows installers (MSI/Setup.exe) are not currently built
- ❌ **Linux Builds**: Linux packages (AppImage/DEB/RPM) are not currently built
- ❌ **macOS Intel (x86_64)**: Only Apple Silicon builds are currently available

### 🔄 Updater Status

- ⚠️ **Update Checker**: Configured to check for updates from HandyGemini fork
- ⚠️ **latest.json**: Needs to be generated during build and uploaded to releases for updater to work fully

### 📝 Development Notes

To build for other platforms, you'll need to:
1. Set up platform-specific build environments (see [BUILD.md](BUILD.md))
2. Configure platform-specific signing certificates (Windows code signing, Linux GPG)
3. Run `bun run tauri build` for the target platform
4. Upload the generated `latest.json` file to releases for updater functionality

## Google OAuth Setup

To enable Gemini AI features, you need to set up Google OAuth credentials. See [GOOGLE_OAUTH_SETUP.md](GOOGLE_OAUTH_SETUP.md) for detailed instructions.

Quick start:
1. Create a Google Cloud project and enable Generative Language API
2. Create OAuth 2.0 credentials (Desktop app type)
3. Set environment variables:
   ```bash
   export GOOGLE_OAUTH_CLIENT_ID="your-client-id.apps.googleusercontent.com"
   export GOOGLE_OAUTH_CLIENT_SECRET="your-client-secret"
   ```
4. Run the app and sign in via Settings > Post Process > Google Login


# Handy

[![Discord](https://img.shields.io/badge/Discord-%235865F2.svg?style=for-the-badge&logo=discord&logoColor=white)](https://discord.com/invite/WVBeWsNXK4)

**A free, open source, and extensible speech-to-text application that works completely offline.**

Handy is a cross-platform desktop application built with Tauri (Rust + React/TypeScript) that provides simple, privacy-focused speech transcription. Press a shortcut, speak, and have your words appear in any text field—all without sending your voice to the cloud.

## Why Handy?

Handy was created to fill the gap for a truly open source, extensible speech-to-text tool. As stated on [handy.computer](https://handy.computer):

- **Free**: Accessibility tooling belongs in everyone's hands, not behind a paywall
- **Open Source**: Together we can build further. Extend Handy for yourself and contribute to something bigger
- **Private**: Your voice stays on your computer. Get transcriptions without sending audio to the cloud
- **Simple**: One tool, one job. Transcribe what you say and put it into a text box

Handy isn't trying to be the best speech-to-text app—it's trying to be the most forkable one.

## How It Works

1. **Press** a configurable keyboard shortcut to start/stop recording (or use push-to-talk mode)
2. **Speak** your words while the shortcut is active
3. **Release** and Handy processes your speech using Whisper
4. **Get** your transcribed text pasted directly into whatever app you're using

The process is entirely local:

- Silence is filtered using VAD (Voice Activity Detection) with Silero
- Transcription uses your choice of models:
  - **Whisper models** (Small/Medium/Turbo/Large) with GPU acceleration when available
  - **Parakeet V3** - CPU-optimized model with excellent performance and automatic language detection
- Works on Windows, macOS, and Linux

## Quick Start

### Installation

**For HandyGemini:**
1. Download the latest release from the [HandyGemini releases page](https://github.com/billylo1/HandyGemini/releases)
2. Currently only macOS (Apple Silicon) builds are available
3. See [Current Limitations](#current-limitations--missing-features) for build status

**For Base Handy (all platforms):**
1. Download the latest release from the [Handy releases page](https://github.com/cjpais/Handy/releases) or the [website](https://handy.computer)
2. Install the application following platform-specific instructions
3. Launch Handy and grant necessary system permissions (microphone, accessibility)
4. Configure your preferred keyboard shortcuts in Settings
5. Start transcribing!

### Development Setup

For detailed build instructions including platform-specific requirements, see [BUILD.md](BUILD.md).

## Architecture

Handy is built as a Tauri application combining:

- **Frontend**: React + TypeScript with Tailwind CSS for the settings UI
- **Backend**: Rust for system integration, audio processing, and ML inference
- **Core Libraries**:
  - `whisper-rs`: Local speech recognition with Whisper models
  - `transcription-rs`: CPU-optimized speech recognition with Parakeet models
  - `cpal`: Cross-platform audio I/O
  - `vad-rs`: Voice Activity Detection
  - `rdev`: Global keyboard shortcuts and system events
  - `rubato`: Audio resampling

### Debug Mode

Handy includes an advanced debug mode for development and troubleshooting. Access it by pressing:

- **macOS**: `Cmd+Shift+D`
- **Windows/Linux**: `Ctrl+Shift+D`

## Known Issues & Current Limitations

This project is actively being developed and has some [known issues](https://github.com/cjpais/Handy/issues). We believe in transparency about the current state:

### Major Issues (Help Wanted)

**Whisper Model Crashes:**

- Whisper models crash on certain system configurations (Windows and Linux)
- Does not affect all systems - issue is configuration-dependent
  - If you experience crashes and are a developer, please help to fix and provide debug logs!

**Wayland Support (Linux):**

- Limited support for Wayland display server
- Requires [`wtype`](https://github.com/atx/wtype) or [`dotool`](https://sr.ht/~geb/dotool/) for text input to work correctly (see [Linux Notes](#linux-notes) below for installation)

### Linux Notes

**Text Input Tools:**

For reliable text input on Linux, install the appropriate tool for your display server:

| Display Server | Recommended Tool | Install Command                                    |
| -------------- | ---------------- | -------------------------------------------------- |
| X11            | `xdotool`        | `sudo apt install xdotool`                         |
| Wayland        | `wtype`          | `sudo apt install wtype`                           |
| Both           | `dotool`         | `sudo apt install dotool` (requires `input` group) |

- **X11**: Install `xdotool` for both direct typing and clipboard paste shortcuts
- **Wayland**: Install `wtype` (preferred) or `dotool` for text input to work correctly
- **dotool setup**: Requires adding your user to the `input` group: `sudo usermod -aG input $USER` (then log out and back in)

Without these tools, Handy falls back to enigo which may have limited compatibility, especially on Wayland.

**Other Notes:**

- The recording overlay is disabled by default on Linux (`Overlay Position: None`) because certain compositors treat it as the active window. When the overlay is visible it can steal focus, which prevents Handy from pasting back into the application that triggered transcription. If you enable the overlay anyway, be aware that clipboard-based pasting might fail or end up in the wrong window.
- If you are having trouble with the app, running with the environment variable `WEBKIT_DISABLE_DMABUF_RENDERER=1` may help
- You can manage global shortcuts outside of Handy and still control the app via signals. Sending `SIGUSR2` to the Handy process toggles recording on/off, which lets Wayland window managers or other hotkey daemons keep ownership of keybindings. Example (Sway):

  ```ini
  bindsym $mod+o exec pkill -USR2 -n handy
  ```

  `pkill` here simply delivers the signal—it does not terminate the process.

### Platform Support

- **macOS (both Intel and Apple Silicon)**
- **x64 Windows**
- **x64 Linux**

### System Requirements/Recommendations

The following are recommendations for running Handy on your own machine. If you don't meet the system requirements, the performance of the application may be degraded. We are working on improving the performance across all kinds of computers and hardware.

**For Whisper Models:**

- **macOS**: M series Mac, Intel Mac
- **Windows**: Intel, AMD, or NVIDIA GPU
- **Linux**: Intel, AMD, or NVIDIA GPU
  - Ubuntu 22.04, 24.04

**For Parakeet V3 Model:**

- **CPU-only operation** - runs on a wide variety of hardware
- **Minimum**: Intel Skylake (6th gen) or equivalent AMD processors
- **Performance**: ~5x real-time speed on mid-range hardware (tested on i5)
- **Automatic language detection** - no manual language selection required

## Roadmap & Active Development

We're actively working on several features and improvements. Contributions and feedback are welcome!

### In Progress

**Debug Logging:**

- Adding debug logging to a file to help diagnose issues

**macOS Keyboard Improvements:**

- Support for Globe key as transcription trigger
- A rewrite of global shortcut handling for MacOS, and potentially other OS's too.

**Opt-in Analytics:**

- Collect anonymous usage data to help improve Handy
- Privacy-first approach with clear opt-in

**Settings Refactoring:**

- Cleanup and refactor settings system which is becoming bloated and messy
- Implement better abstractions for settings management

**Tauri Commands Cleanup:**

- Abstract and organize Tauri command patterns
- Investigate tauri-specta for improved type safety and organization

## Troubleshooting

### Manual Model Installation (For Proxy Users or Network Restrictions)

If you're behind a proxy, firewall, or in a restricted network environment where Handy cannot download models automatically, you can manually download and install them. The URLs are publicly accessible from any browser.

#### Step 1: Find Your App Data Directory

1. Open Handy settings
2. Navigate to the **About** section
3. Copy the "App Data Directory" path shown there, or use the shortcuts:
   - **macOS**: `Cmd+Shift+D` to open debug menu
   - **Windows/Linux**: `Ctrl+Shift+D` to open debug menu

The typical paths are:

- **macOS**: `~/Library/Application Support/com.pais.handy/`
- **Windows**: `C:\Users\{username}\AppData\Roaming\com.pais.handy\`
- **Linux**: `~/.config/com.pais.handy/`

#### Step 2: Create Models Directory

Inside your app data directory, create a `models` folder if it doesn't already exist:

```bash
# macOS/Linux
mkdir -p ~/Library/Application\ Support/com.pais.handy/models

# Windows (PowerShell)
New-Item -ItemType Directory -Force -Path "$env:APPDATA\com.pais.handy\models"
```

#### Step 3: Download Model Files

Download the models you want from below

**Whisper Models (single .bin files):**

- Small (487 MB): `https://blob.handy.computer/ggml-small.bin`
- Medium (492 MB): `https://blob.handy.computer/whisper-medium-q4_1.bin`
- Turbo (1600 MB): `https://blob.handy.computer/ggml-large-v3-turbo.bin`
- Large (1100 MB): `https://blob.handy.computer/ggml-large-v3-q5_0.bin`

**Parakeet Models (compressed archives):**

- V2 (473 MB): `https://blob.handy.computer/parakeet-v2-int8.tar.gz`
- V3 (478 MB): `https://blob.handy.computer/parakeet-v3-int8.tar.gz`

#### Step 4: Install Models

**For Whisper Models (.bin files):**

Simply place the `.bin` file directly into the `models` directory:

```
{app_data_dir}/models/
├── ggml-small.bin
├── whisper-medium-q4_1.bin
├── ggml-large-v3-turbo.bin
└── ggml-large-v3-q5_0.bin
```

**For Parakeet Models (.tar.gz archives):**

1. Extract the `.tar.gz` file
2. Place the **extracted directory** into the `models` folder
3. The directory must be named exactly as follows:
   - **Parakeet V2**: `parakeet-tdt-0.6b-v2-int8`
   - **Parakeet V3**: `parakeet-tdt-0.6b-v3-int8`

Final structure should look like:

```
{app_data_dir}/models/
├── parakeet-tdt-0.6b-v2-int8/     (directory with model files inside)
│   ├── (model files)
│   └── (config files)
└── parakeet-tdt-0.6b-v3-int8/     (directory with model files inside)
    ├── (model files)
    └── (config files)
```

**Important Notes:**

- For Parakeet models, the extracted directory name **must** match exactly as shown above
- Do not rename the `.bin` files for Whisper models—use the exact filenames from the download URLs
- After placing the files, restart Handy to detect the new models

#### Step 5: Verify Installation

1. Restart Handy
2. Open Settings → Models
3. Your manually installed models should now appear as "Downloaded"
4. Select the model you want to use and test transcription

### How to Contribute

**For HandyGemini:**
1. **Check existing issues** at [github.com/billylo1/HandyGemini/issues](https://github.com/billylo1/HandyGemini/issues)
2. **Fork the repository** and create a feature branch
3. **Test thoroughly** on your target platform
4. **Submit a pull request** with clear description of changes

**For Base Handy:**
1. **Check existing issues** at [github.com/cjpais/Handy/issues](https://github.com/cjpais/Handy/issues)
2. **Fork the repository** and create a feature branch
3. **Test thoroughly** on your target platform
4. **Submit a pull request** with clear description of changes
5. **Join the discussion** - reach out at [contact@handy.computer](mailto:contact@handy.computer)

The goal is to create both a useful tool and a foundation for others to build upon—a well-patterned, simple codebase that serves the community.

## Sponsors

<div align="center">
  We're grateful for the support of our sponsors who help make Handy possible:
  <br><br>
  <a href="https://wordcab.com">
    <img src="sponsor-images/wordcab.png" alt="Wordcab" width="120" height="120">
  </a>
  &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
  <a href="https://github.com/epicenter-so/epicenter">
    <img src="sponsor-images/epicenter.png" alt="Epicenter" width="120" height="120">
  </a>
</div>

## Related Projects

- **[Handy (Base Project)](https://github.com/cjpais/Handy)** - The original Handy speech-to-text application
- **[Handy CLI](https://github.com/cjpais/handy-cli)** - The original Python command-line version
- **[handy.computer](https://handy.computer)** - Project website with demos and documentation

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Acknowledgments

- **Whisper** by OpenAI for the speech recognition model
- **whisper.cpp and ggml** for amazing cross-platform whisper inference/acceleration
- **Silero** for great lightweight VAD
- **Tauri** team for the excellent Rust-based app framework
- **Community contributors** helping make Handy better

---

_"Your search for the right speech-to-text tool can end here—not because Handy is perfect, but because you can make it perfect for you."_
