import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

// DOM Elements
let urlInput: HTMLInputElement;
let fetchInfoBtn: HTMLButtonElement;
let videoInfo: HTMLElement;
let videoThumbnail: HTMLImageElement;
let videoTitle: HTMLElement;
let videoUploader: HTMLElement;
let videoDuration: HTMLElement;
let downloadOptions: HTMLElement;
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

let selectedPath = "";

// Initialize app
window.addEventListener("DOMContentLoaded", () => {
  initializeElements();
  attachEventListeners();
  setupProgressListener();
  
  // Set default download path
  setDefaultDownloadPath();
});

function initializeElements() {
  urlInput = document.querySelector("#url-input")!;
  fetchInfoBtn = document.querySelector("#fetch-info-btn")!;
  videoInfo = document.querySelector("#video-info")!;
  videoThumbnail = document.querySelector("#video-thumbnail")!;
  videoTitle = document.querySelector("#video-title")!;
  videoUploader = document.querySelector("#video-uploader")!;
  videoDuration = document.querySelector("#video-duration")!;
  downloadOptions = document.querySelector("#download-options")!;
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
}

async function setDefaultDownloadPath() {
  selectedPath = `${await getHomeDir()}/Downloads`;
  outputPath.value = selectedPath;
}

async function getHomeDir(): Promise<string> {
  // Return user's home directory
  return "/Users/olgazaharova";
}

async function handleFetchInfo() {
  const url = urlInput.value.trim();
  
  if (!url) {
    showStatus("Пожалуйста, введите URL видео", "error");
    return;
  }
  
  // Validate YouTube URL
  if (!url.includes("youtube.com") && !url.includes("youtu.be")) {
    showStatus("Пожалуйста, введите корректную ссылку на YouTube", "error");
    return;
  }
  
  // Show loading state
  fetchInfoBtn.disabled = true;
  fetchInfoBtn.textContent = "Загрузка...";
  hideStatus();
  
  try {
    const info = await invoke("get_video_info", { url });
    displayVideoInfo(info);
    showDownloadOptions();
  } catch (error) {
    showStatus(`Ошибка: ${error}`, "error");
    hideVideoInfo();
  } finally {
    fetchInfoBtn.disabled = false;
    fetchInfoBtn.innerHTML = `
      <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
      Получить информацию
    `;
  }
}

function displayVideoInfo(info: any) {
  videoThumbnail.src = info.thumbnail;
  videoTitle.textContent = info.title;
  videoUploader.textContent = info.uploader;
  videoDuration.textContent = info.duration;
  
  videoInfo.classList.remove("hidden");
}

function hideVideoInfo() {
  videoInfo.classList.add("hidden");
  downloadOptions.classList.add("hidden");
  downloadSection.classList.add("hidden");
  progressSection.classList.add("hidden");
}

function showDownloadOptions() {
  downloadOptions.classList.remove("hidden");
  downloadSection.classList.remove("hidden");
}

async function handleSelectPath() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: "Выберите папку для сохранения",
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
    showStatus("Пожалуйста, выберите папку для сохранения", "error");
    return;
  }
  
  // Disable download button
  downloadBtn.disabled = true;
  downloadBtn.textContent = "Скачивание...";
  
  // Show progress section
  progressSection.classList.remove("hidden");
  hideStatus();
  
  try {
    const result = await invoke("download_video", {
      url,
      quality,
      outputPath: selectedPath,
    });
    
    showStatus(String(result), "success");
    
    // Reset progress after a delay
    setTimeout(() => {
      progressSection.classList.add("hidden");
      resetProgress();
    }, 3000);
    
  } catch (error) {
    showStatus(`Ошибка скачивания: ${error}`, "error");
    progressSection.classList.add("hidden");
  } finally {
    downloadBtn.disabled = false;
    downloadBtn.innerHTML = `
      <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4m4-5l5 5 5-5m-5 5V3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
      Скачать видео
    `;
  }
}

function setupProgressListener() {
  listen("download-progress", (event: any) => {
    const progress = event.payload;
    updateProgress(progress.percent, progress.status);
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
  progressStatus.textContent = "Подготовка...";
}

function showStatus(message: string, type: "success" | "error") {
  statusMessage.textContent = message;
  statusMessage.className = `status-message ${type}`;
  statusMessage.classList.remove("hidden");
}

function hideStatus() {
  statusMessage.classList.add("hidden");
}
