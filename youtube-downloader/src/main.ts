import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open as openDialog, ask } from "@tauri-apps/plugin-dialog";
import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";

// DOM Elements
let urlInput: HTMLInputElement;
let fetchInfoBtn: HTMLButtonElement;
let appTitleEl: HTMLElement | null;
let videoInfo: HTMLElement;
let videoThumbnail: HTMLImageElement;
let videoTitle: HTMLElement;
let videoUploader: HTMLElement;
let videoDuration: HTMLElement;
let downloadOptions: HTMLElement;
let toolSelect: HTMLSelectElement;
let qualitySelect: HTMLSelectElement;
let outputPath: HTMLInputElement;
let selectPathBtn: HTMLButtonElement;
let downloadSection: HTMLElement;
let downloadBtn: HTMLButtonElement;
let progressSection: HTMLElement;
let progressStatus: HTMLElement;
let progressPercent: HTMLElement;
let progressBar: HTMLElement;
let statusMessage: HTMLElement;
let toggleTerminalBtn: HTMLButtonElement;
let terminalContent: HTMLElement;
let terminalLog: HTMLElement;
let autoFallbackToggle: HTMLInputElement | null;

let selectedPath = "";

type ServiceKey = "generic" | "youtube" | "instagram" | "tiktok" | "x" | "facebook" | "vimeo";

const USER_PROXY_KEY = "downloader.userProxy";
const USER_AUTO_FALLBACK_KEY = "downloader.autoFallback";
const USER_COOKIES_MODE_KEY = "downloader.cookiesMode"; // chrome | file | none
const USER_COOKIES_FILE_KEY = "downloader.cookiesFile";

function getUserProxy(): string | null {
  try {
    const v = localStorage.getItem(USER_PROXY_KEY);
    const trimmed = (v ?? "").trim();
    return trimmed ? trimmed : null;
  } catch {
    return null;
  }
}

function setUserProxy(value: string | null) {
  try {
    if (!value) localStorage.removeItem(USER_PROXY_KEY);
    else localStorage.setItem(USER_PROXY_KEY, value);
  } catch {
    // ignore
  }
}

type CookiesMode = "chrome" | "file" | "none";

function getCookiesMode(): CookiesMode {
  try {
    const v = (localStorage.getItem(USER_COOKIES_MODE_KEY) ?? "").trim();
    if (v === "none" || v === "file" || v === "chrome") return v;
    return "chrome";
  } catch {
    return "chrome";
  }
}

function setCookiesMode(mode: CookiesMode) {
  try {
    localStorage.setItem(USER_COOKIES_MODE_KEY, mode);
  } catch {
    // ignore
  }
}

function getCookiesFile(): string | null {
  try {
    const v = (localStorage.getItem(USER_COOKIES_FILE_KEY) ?? "").trim();
    return v ? v : null;
  } catch {
    return null;
  }
}

function setCookiesFile(path: string | null) {
  try {
    if (!path) localStorage.removeItem(USER_COOKIES_FILE_KEY);
    else localStorage.setItem(USER_COOKIES_FILE_KEY, path);
  } catch {
    // ignore
  }
}

function getCookiesConfig(): { cookiesFromBrowser: boolean; cookiesPath: string | null } {
  const mode = getCookiesMode();
  if (mode === "chrome") return { cookiesFromBrowser: true, cookiesPath: null };
  if (mode === "file") return { cookiesFromBrowser: false, cookiesPath: getCookiesFile() };
  return { cookiesFromBrowser: false, cookiesPath: null };
}

// Initialize app
window.addEventListener("DOMContentLoaded", () => {
  initializeElements();
  attachEventListeners();
  setupProgressListener();

  // Set default download path
  // Set default download path
  setDefaultDownloadPath();
  loadVersion();
  setupTools();
  loadNetworkStatus(); // Load network status on init

  // Initial title
  updateAppTitle("");

  // Restore auto-fallback toggle state
  if (autoFallbackToggle) {
    autoFallbackToggle.checked = getAutoFallback();
    autoFallbackToggle.addEventListener("change", () => {
      setAutoFallback(autoFallbackToggle!.checked);
      showStatus(`Auto fallback: ${autoFallbackToggle!.checked ? "on" : "off"}`, "success");
    });
  }

  // Setup network status refresh button
  const refreshBtn = document.getElementById("refresh-network");
  if (refreshBtn) {
    refreshBtn.addEventListener("click", () => loadNetworkStatus());
  }
});

function initializeElements() {
  urlInput = document.querySelector("#url-input")!;
  fetchInfoBtn = document.querySelector("#fetch-info-btn")!;
  appTitleEl = document.querySelector("#app-title");
  videoInfo = document.querySelector("#video-info")!;
  videoThumbnail = document.querySelector("#video-thumbnail")!;
  videoTitle = document.querySelector("#video-title")!;
  videoUploader = document.querySelector("#video-uploader")!;
  videoDuration = document.querySelector("#video-duration")!;
  downloadOptions = document.querySelector("#download-options")!;
  toolSelect = document.querySelector("#tool-select")!;
  qualitySelect = document.querySelector("#quality-select")!;
  outputPath = document.querySelector("#output-path")!;
  selectPathBtn = document.querySelector("#select-path-btn")!;
  downloadSection = document.querySelector("#download-section")!;
  downloadBtn = document.querySelector("#download-btn")!;
  progressSection = document.querySelector("#progress-section")!;
  progressStatus = document.querySelector("#progress-status")!;
  progressPercent = document.querySelector("#progress-percent")!;
  progressBar = document.querySelector("#progress-bar")!;
  statusMessage = document.querySelector("#status-message")!;
  toggleTerminalBtn = document.querySelector("#toggle-terminal")!;
  terminalContent = document.querySelector("#terminal-content")!;
  terminalLog = document.querySelector("#terminal-log")!;
  autoFallbackToggle = document.querySelector("#auto-fallback");
}

function attachEventListeners() {
  fetchInfoBtn.addEventListener("click", handleFetchInfo);
  selectPathBtn.addEventListener("click", handleSelectPath);
  downloadBtn.addEventListener("click", handleDownload);

  // Allow Enter key in URL input
  urlInput.addEventListener("keypress", (e) => {
    if (e.key === "Enter") {
      handleFetchInfo();
    }
  });

  // Update title based on entered URL (service detection)
  urlInput.addEventListener("input", () => updateAppTitle(urlInput.value));

  // Terminal toggle
  const terminalHeader = document.querySelector(".terminal-header");
  if (terminalHeader) {
    terminalHeader.addEventListener("click", toggleTerminal);
  }
  toggleTerminalBtn.addEventListener("click", (e) => {
    // Prevent double-toggle when clicking the arrow (bubble -> header)
    e.stopPropagation();
    toggleTerminal();
  });
}

function detectService(url: string): ServiceKey {
  const value = url.trim().toLowerCase();
  if (!value) return "generic";

  // Quick checks without requiring URL parsing to succeed
  if (value.includes("youtube.com") || value.includes("youtu.be")) return "youtube";
  if (value.includes("instagram.com")) return "instagram";
  if (value.includes("tiktok.com")) return "tiktok";
  if (value.includes("x.com") || value.includes("twitter.com")) return "x";
  if (value.includes("facebook.com") || value.includes("fb.watch")) return "facebook";
  if (value.includes("vimeo.com")) return "vimeo";
  return "generic";
}

function serviceLabel(service: ServiceKey): string {
  switch (service) {
    case "youtube":
      return "YouTube Downloader";
    case "instagram":
      return "Instagram Downloader";
    case "tiktok":
      return "TikTok Downloader";
    case "x":
      return "X Downloader";
    case "facebook":
      return "Facebook Downloader";
    case "vimeo":
      return "Vimeo Downloader";
    case "generic":
    default:
      return "Downloader";
  }
}

async function setNativeWindowTitle(title: string) {
  // Tauri only; keep safe for web contexts / tests
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().setTitle(title);
  } catch {
    // ignore
  }
}

function updateAppTitle(url: string) {
  const label = serviceLabel(detectService(url));
  if (appTitleEl) appTitleEl.textContent = label;
  document.title = label;
  void setNativeWindowTitle(label);
}

async function setDefaultDownloadPath() {
  selectedPath = `${await getHomeDir()}/Downloads`;
  outputPath.value = selectedPath;
}

async function getHomeDir(): Promise<string> {
  // Return user's home directory
  return "/Users/olgazaharova";
}

function getAutoFallback(): boolean {
  try {
    const raw = localStorage.getItem(USER_AUTO_FALLBACK_KEY);
    if (raw === null) return true; // default on
    return raw === "1";
  } catch {
    return true;
  }
}

function setAutoFallback(value: boolean) {
  try {
    localStorage.setItem(USER_AUTO_FALLBACK_KEY, value ? "1" : "0");
  } catch {
    // ignore
  }
}

async function handleFetchInfo() {
  const url = urlInput.value.trim();
  updateAppTitle(url);

  if (!url) {
    showStatus("Please enter a video URL", "error");
    return;
  }

  // Basic URL validation (we support multiple services)
  if (!/^https?:\/\//i.test(url)) {
    showStatus("Please enter a valid URL (must start with http:// or https://)", "error");
    return;
  }

  // Show loading state
  fetchInfoBtn.disabled = true;
  fetchInfoBtn.textContent = "Loading...";
  hideStatus();

  // Log action
  addLog(`Fetching video info: ${url}`, "info");

  const startedAt = Date.now();
  const heartbeatTimers: number[] = [];
  const addHeartbeat = (afterMs: number, level: "info" | "warning", message: (elapsedSec: number) => string) => {
    heartbeatTimers.push(
      window.setTimeout(() => {
        const elapsedSec = Math.round((Date.now() - startedAt) / 1000);
        // If it takes long enough to show a heartbeat, auto-expand the log
        // so the user doesn't have to scroll to verify it's working.
        ensureTerminalExpanded();
        addLog(message(elapsedSec), level);
      }, afterMs)
    );
  };

  // If yt-dlp is slow/hanging, keep the user informed.
  addHeartbeat(8000, "info", (s) => `Still working… (${s}s)`);
  addHeartbeat(20000, "warning", (s) => `Still working… (${s}s). This can take longer depending on network/proxy.`);
  addHeartbeat(35000, "warning", (s) => `This is taking unusually long… (${s}s). It may be stuck.`);

  try {
    addLog("Running yt-dlp...", "info");
    const cookies = getCookiesConfig();
    const info = await invoke<any>("get_video_info", {
      url,
      proxy: getUserProxy(),
      cookiesFromBrowser: cookies.cookiesFromBrowser,
      cookiesPath: cookies.cookiesPath,
    });
    addLog(`Video info received: ${info.title}`, "success");
    displayVideoInfo(info);
    showDownloadOptions();
  } catch (error) {
    addLog(`Error: ${error}`, "error");
    showStatus(`Error: ${error}`, "error");
    hideVideoInfo();
  } finally {
    heartbeatTimers.forEach((t) => window.clearTimeout(t));
    fetchInfoBtn.disabled = false;
    fetchInfoBtn.innerHTML = `
      <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
      Fetch info
    `;
  }
}

function displayVideoInfo(info: any) {
  videoThumbnail.src = info.thumbnail;
  videoTitle.textContent = info.title;
  videoUploader.textContent = info.uploader;
  videoDuration.textContent = info.duration;

  // Update quality options dynamically
  if (info.formats && info.formats.length > 0) {
    qualitySelect.innerHTML = "";

    info.formats.forEach((fmt: any) => {
      const option = document.createElement("option");
      option.value = fmt.value;
      const sizeStr = fmt.size ? ` (${fmt.size})` : "";

      let icon = "📺";
      if (fmt.value === "audio") icon = "🎵";
      else if (fmt.value === "best") icon = "🏆";

      option.textContent = `${icon} ${fmt.label}${sizeStr}`;
      qualitySelect.appendChild(option);
    });

    // Default to the first option (Best Quality) as per user request for max quality
    if (qualitySelect.options.length > 0) {
      qualitySelect.selectedIndex = 0;
    }
  }

  videoInfo.classList.remove("hidden");
  document.body.classList.add("has-video");
}

function hideVideoInfo() {
  videoInfo.classList.add("hidden");
  downloadOptions.classList.add("hidden");
  downloadSection.classList.add("hidden");
  progressSection.classList.add("hidden");
  document.body.classList.remove("has-video");
}

function showDownloadOptions() {
  downloadOptions.classList.remove("hidden");
  downloadSection.classList.remove("hidden");
}

async function handleSelectPath() {
  const selected = await openDialog({
    directory: true,
    multiple: false,
    title: "Select Download Folder",
  });

  if (selected && typeof selected === "string") {
    selectedPath = selected;
    outputPath.value = selected;
  }
}

async function handleDownload() {
  const url = urlInput.value.trim();
  const quality = qualitySelect.value;

  if (!selectedPath) {
    showStatus("Please choose an output folder", "error");
    return;
  }

  // Disable download button
  downloadBtn.disabled = true;
  downloadBtn.textContent = "Downloading...";

  // Show progress section
  progressSection.classList.remove("hidden");
  hideStatus();

  // Log action
  addLog(`Starting download: quality=${quality}`, "info");
  addLog(`Tool: ${toolSelect.value}`, "info");
  addLog(`Auto fallback: ${getAutoFallback() ? "on" : "off"}`, "info");
  addLog(`Output: ${selectedPath}`, "info");

  try {
    const cookies = getCookiesConfig();
    const result = await invoke("download_video", {
      url,
      quality,
      outputPath: selectedPath,
      tool: toolSelect.value,
      proxy: getUserProxy(),
      allowFallback: getAutoFallback(),
      cookiesFromBrowser: cookies.cookiesFromBrowser,
      cookiesPath: cookies.cookiesPath,
    });

    addLog(String(result), "success");
    showStatus(String(result), "success");

    // Ask user to open folder
    try {
      const shouldOpen = await ask("Download completed successfully! Open folder?", {
        title: "Download Complete",
        kind: "info",
      });

      if (shouldOpen) {
        // Use revealItemInDir to open Finder at the folder location
        try {
          await revealItemInDir(selectedPath);
        } catch {
          // Fallback to openPath if revealItemInDir fails
          await openPath(selectedPath);
        }
      }
    } catch (e) {
      console.error("Failed to show dialog:", e);
    }

    // Reset progress after a delay
    setTimeout(() => {
      progressSection.classList.add("hidden");
      resetProgress();
    }, 3000);

  } catch (error) {
    addLog(`Download error: ${error}`, "error");
    showStatus(`Download error: ${error}`, "error");
    progressSection.classList.add("hidden");
  } finally {
    downloadBtn.disabled = false;
    downloadBtn.innerHTML = `
      <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4m4-5l5 5 5-5m-5 5V3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
      Download Video
    `;
  }
}

function setupProgressListener() {
  let lastStatus = "";
  let lastLogAt = 0;

  const maybeLogProgress = (status: string) => {
    const trimmed = (status ?? "").trim();
    if (!trimmed || trimmed === lastStatus) return;

    const now = Date.now();
    // Throttle: avoid spamming the log if backend sends many updates
    if (now - lastLogAt < 900) return;

    lastStatus = trimmed;
    lastLogAt = now;

    const lower = trimmed.toLowerCase();
    const level =
      lower.includes("failed") || lower.includes("error") || lower.includes("forbidden") || lower.includes("403")
        ? "warning"
        : "info";

    addLog(trimmed, level as any);
  };

  listen("download-progress", (event: any) => {
    const progress = event.payload;
    updateProgress(progress.percent, progress.status);
    maybeLogProgress(progress.status);
  });
}

function updateProgress(percent: number, status: string) {
  progressBar.style.width = `${percent}%`;
  progressPercent.textContent = `${Math.round(percent)}%`;
  progressStatus.textContent = status;
}

function resetProgress() {
  progressBar.style.width = "0%";
  progressPercent.textContent = "0%";
  progressStatus.textContent = "Preparing...";
}

function showStatus(message: string, type: "success" | "error") {
  statusMessage.textContent = message;
  statusMessage.className = `status-message ${type}`;
  statusMessage.classList.remove("hidden");
}

function hideStatus() {
  statusMessage.classList.add("hidden");
}

//Terminal Log Functions
function toggleTerminal() {
  terminalContent.classList.toggle("collapsed");
  toggleTerminalBtn.classList.toggle("collapsed");
}

function ensureTerminalExpanded() {
  if (terminalContent.classList.contains("collapsed")) {
    terminalContent.classList.remove("collapsed");
    toggleTerminalBtn.classList.remove("collapsed");
  }
}

function addLog(message: string, type: "info" | "success" | "error" | "warning" = "info") {
  const line = document.createElement("div");
  line.className = `log-line log-${type}`;
  line.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
  terminalLog.appendChild(line);

  // Auto-scroll to bottom
  terminalLog.scrollTop = terminalLog.scrollHeight;

  // Keep UI compact: auto-expand only on warning/error.
  if (type === "error" || type === "warning") {
    ensureTerminalExpanded();
  }
}

// Load version from Tauri
async function loadVersion() {
  try {
    const { getVersion } = await import("@tauri-apps/api/app");
    const version = await getVersion();
    const versionEl = document.getElementById("footer-version");
    if (versionEl) {
      versionEl.textContent = version;
    }
  } catch (error) {
    console.error("Failed to load version:", error);
  }
}

// Network Status
interface NetworkStatus {
  proxy: string | null;
  mode: string;
  external_ip: string | null;
}

async function loadNetworkStatus() {
  const proxyEl = document.getElementById("status-proxy");
  const modeEl = document.getElementById("status-mode");
  const ipEl = document.getElementById("status-ip");
  const refreshBtn = document.getElementById("refresh-network");

  // Show loading state
  if (refreshBtn) refreshBtn.classList.add("loading");
  if (proxyEl) proxyEl.textContent = "detecting...";
  if (ipEl) ipEl.textContent = "...";

  try {
    const status = await invoke<NetworkStatus>("get_network_status", {
      userProxy: getUserProxy(),
    });

    // Update proxy display
    if (proxyEl) {
      proxyEl.textContent = status.proxy || "none";
      if (status.proxy) {
        // Truncate long proxy URLs
        const maxLen = 40;
        proxyEl.textContent = status.proxy.length > maxLen
          ? status.proxy.slice(0, maxLen) + "…"
          : status.proxy;
      }
    }

    // Update mode display
    if (modeEl) {
      modeEl.textContent = status.mode;
      modeEl.className = "status-value mode-" + status.mode;
    }

    // Update IP display
    if (ipEl) {
      ipEl.textContent = status.external_ip || "unavailable";
    }

    addLog(`Network status: mode=${status.mode}, IP=${status.external_ip || "N/A"}`, "info");
  } catch (error) {
    console.error("Failed to load network status:", error);
    if (proxyEl) proxyEl.textContent = "error";
    if (modeEl) modeEl.textContent = "unknown";
    if (ipEl) ipEl.textContent = "error";
  } finally {
    if (refreshBtn) refreshBtn.classList.remove("loading");
  }
}

// Tools Management
interface ToolInfo {
  name: string;
  tool_type: string;
  version: string | null;
  path: string | null;
  is_available: boolean;
  last_updated: string | null;
}

const toolsList = document.getElementById("tools-list");

function setupTools() {
  const toggleToolsBtn = document.getElementById("toggle-tools");
  const toolsContent = document.getElementById("tools-content");
  const toolsHeader = document.getElementById("tools-header");
  if (!toggleToolsBtn || !toolsContent) return;

  const toggle = () => {
    toolsContent.classList.toggle("collapsed");
    toggleToolsBtn.classList.toggle("collapsed");
  };

  // Click on the arrow OR anywhere on the header
  toggleToolsBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    toggle();
  });
  if (toolsHeader) toolsHeader.addEventListener("click", toggle);

  // Proxy settings UI
  const proxyInput = document.getElementById("proxy-input") as HTMLInputElement | null;
  const proxySave = document.getElementById("proxy-save") as HTMLButtonElement | null;
  const proxyClear = document.getElementById("proxy-clear") as HTMLButtonElement | null;
  if (proxyInput) proxyInput.value = getUserProxy() ?? "";
  if (proxySave && proxyInput) {
    proxySave.addEventListener("click", () => {
      const value = proxyInput.value.trim();
      setUserProxy(value || null);
      showStatus(value ? `Proxy saved: ${value}` : "Proxy cleared", "success");
      addLog(value ? `Proxy saved: ${value}` : "Proxy cleared", "success");
    });
  }
  if (proxyClear && proxyInput) {
    proxyClear.addEventListener("click", () => {
      proxyInput.value = "";
      setUserProxy(null);
      showStatus("Proxy cleared", "success");
      addLog("Proxy cleared", "success");
    });
  }

  // Cookies settings UI
  const cookiesMode = document.getElementById("cookies-mode") as HTMLSelectElement | null;
  const cookiesFile = document.getElementById("cookies-file") as HTMLInputElement | null;
  const cookiesPick = document.getElementById("cookies-pick") as HTMLButtonElement | null;
  const cookiesClear = document.getElementById("cookies-clear") as HTMLButtonElement | null;

  const refreshCookiesUi = () => {
    const mode = getCookiesMode();
    if (cookiesMode) cookiesMode.value = mode;
    const file = getCookiesFile();
    if (cookiesFile) cookiesFile.value = file ?? "";
    const fileEnabled = mode === "file";
    if (cookiesFile) cookiesFile.disabled = !fileEnabled;
    if (cookiesPick) cookiesPick.disabled = !fileEnabled;
    if (cookiesClear) cookiesClear.disabled = !fileEnabled;
  };

  refreshCookiesUi();

  if (cookiesMode) {
    cookiesMode.addEventListener("change", () => {
      const v = cookiesMode.value as CookiesMode;
      setCookiesMode(v);
      if (v !== "file") setCookiesFile(null);
      refreshCookiesUi();
      showStatus(`Cookies mode: ${v}`, "success");
      addLog(`Cookies mode: ${v}`, "info");
    });
  }

  if (cookiesPick && cookiesFile) {
    cookiesPick.addEventListener("click", async () => {
      const selected = await openDialog({
        multiple: false,
        title: "Select cookies.txt",
        filters: [{ name: "Text", extensions: ["txt"] }],
      });
      if (selected && typeof selected === "string") {
        setCookiesMode("file");
        setCookiesFile(selected);
        refreshCookiesUi();
        showStatus("Cookies file selected", "success");
        addLog(`Cookies file selected: ${selected}`, "success");
      }
    });
  }


  if (cookiesClear) {
    cookiesClear.addEventListener("click", () => {
      setCookiesFile(null);
      setCookiesMode("chrome");
      refreshCookiesUi();
      showStatus("Cookies reset to Chrome", "success");
      addLog("Cookies reset to Chrome", "success");
    });
  }

  loadTools();
}

async function loadTools() {
  try {
    const tools = await invoke<ToolInfo[]>("get_tools_status");
    renderTools(tools);
  } catch (error) {
    console.error("Failed to load tools:", error);
    if (toolsList) toolsList.textContent = "Error loading tools";
  }
}

function renderTools(tools: ToolInfo[]) {
  if (!toolsList) return;
  toolsList.innerHTML = "";

  tools.forEach(tool => {
    const item = document.createElement("div");
    item.className = "tool-item";

    // Status indicator
    const statusClass = tool.is_available ? "status-ok" : "status-missing";
    const statusText = tool.is_available ? "Available" : "Not found";
    const versionText = tool.version ? `v${tool.version}` : "";

    item.innerHTML = `
      <div class="tool-info">
        <span class="tool-name">${tool.name}</span>
        <span class="tool-version">${versionText}</span>
        <span class="tool-status ${statusClass}">${statusText}</span>
      </div>
      <div class="tool-actions">
        ${tool.is_available
        ? `<button class="update-btn" data-tool="${tool.name}" title="Update this tool">↻</button>`
        : `<button class="install-btn" data-tool="${tool.name}" title="Install this tool">Install</button>`}
      </div>
    `;

    // Add update listener
    const updateBtn = item.querySelector(".update-btn");
    if (updateBtn) {
      updateBtn.addEventListener("click", () => handleUpdateTool(tool.name));
    }

    const installBtn = item.querySelector(".install-btn");
    if (installBtn) {
      installBtn.addEventListener("click", () => handleInstallTool(tool.name));
    }

    toolsList.appendChild(item);
  });
}

async function handleUpdateTool(toolName: string) {
  addLog(`Starting update for ${toolName}...`, "info");
  try {
    const result = await invoke<string>("update_tool", { toolType: toolName });
    addLog(`Update result for ${toolName}: ${result}`, "success");
    loadTools(); // Reload to show new version
  } catch (error) {
    addLog(`Update error for ${toolName}: ${error}`, "error");
  }
}

async function handleInstallTool(toolName: string) {
  addLog(`Starting install for ${toolName}...`, "info");
  try {
    const result = await invoke<string>("install_tool", { toolType: toolName });
    addLog(result, "success");
    addLog("Re-checking tool status...", "info");
    await loadTools();
  } catch (error) {
    addLog(`Install error for ${toolName}: ${error}`, "error");
    addLog(`Tip: open Tools panel and follow the Homebrew/pipx instructions.`, "warning");
  }
}
