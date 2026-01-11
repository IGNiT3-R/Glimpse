const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// DOM 元素
let scanFullBtn, scanRegionBtn, ocrRegionBtn, clearResultsBtn, themeToggleBtn, settingsBtn;
let statusEl, statusTextEl, resultsContainer, resultsList, noResultsEl;
let resultCountEl, selectionOverlay, selectionBox, selectionInfo;

// 设置弹窗元素
let settingsModal, settingsClose, settingsSave, settingsCancel;
let shortcutFullscreenInput, shortcutRegionInput, shortcutOcrInput;

// 区域选择状态
let isSelecting = false;
let startX = 0, startY = 0;

// 快捷键录制状态
let isRecordingShortcut = false;
let currentRecordingTarget = null;

// 默认设置
const defaultSettings = {
  closeBehavior: 'exit',
  shortcuts: {
    fullscreen: 'Ctrl + Shift + S',
    region: 'Ctrl + Shift + A',
    ocr: 'Ctrl + Shift + D'
  }
};

// 初始化
window.addEventListener("DOMContentLoaded", () => {
  initElements();
  initEventListeners();
  initTauriListeners();
  initTheme();
  initSettings();
});

// 初始化 DOM 元素引用
function initElements() {
  scanFullBtn = document.querySelector("#scan-full-btn");
  scanRegionBtn = document.querySelector("#scan-region-btn");
  ocrRegionBtn = document.querySelector("#ocr-region-btn");
  clearResultsBtn = document.querySelector("#clear-results-btn");
  themeToggleBtn = document.querySelector("#theme-toggle");
  settingsBtn = document.querySelector("#settings-btn");
  statusEl = document.querySelector("#status");
  statusTextEl = document.querySelector("#status-text");
  resultsContainer = document.querySelector("#results-section");
  resultsList = document.querySelector("#results-list");
  noResultsEl = document.querySelector("#no-results");
  resultCountEl = document.querySelector("#result-count");

  // 设置弹窗元素
  settingsModal = document.querySelector("#settings-modal");
  settingsClose = document.querySelector("#settings-close");
  settingsSave = document.querySelector("#settings-save");
  settingsCancel = document.querySelector("#settings-cancel");
  shortcutFullscreenInput = document.querySelector("#shortcut-fullscreen");
  shortcutRegionInput = document.querySelector("#shortcut-region");
  shortcutOcrInput = document.querySelector("#shortcut-ocr");
}

// 初始化事件监听
function initEventListeners() {
  scanFullBtn.addEventListener("click", handleScanFull);
  scanRegionBtn.addEventListener("click", handleScanRegion);
  ocrRegionBtn.addEventListener("click", handleOcrRegion);
  clearResultsBtn.addEventListener("click", clearResults);
  themeToggleBtn.addEventListener("click", toggleTheme);

  // 设置弹窗事件
  settingsBtn.addEventListener("click", openSettings);
  settingsClose.addEventListener("click", closeSettings);
  settingsCancel.addEventListener("click", closeSettings);
  settingsSave.addEventListener("click", saveSettings);

  // 点击遮罩层关闭弹窗
  settingsModal.querySelector(".modal-backdrop").addEventListener("click", closeSettings);

  // 快捷键录制按钮
  document.querySelectorAll(".shortcut-record-btn").forEach(btn => {
    btn.addEventListener("click", () => startRecordingShortcut(btn.dataset.target));
  });

  // 监听键盘事件（用于录制快捷键）
  document.addEventListener("keydown", handleShortcutKeydown);
}

// 初始化 Tauri 事件监听（全局快捷键触发）
async function initTauriListeners() {
  await listen("trigger_scan_full", handleScanFull);
  await listen("trigger_scan_region", handleScanRegion);
  await listen("trigger_ocr_region", handleOcrRegion);

  // 监听区域扫描完成事件
  await listen("region_scan_complete", (event) => {
    console.log('[DEBUG] 收到 region_scan_complete 事件:', event.payload);
    showResults(event.payload);
  });

  // 监听 OCR 扫描完成事件
  await listen("ocr_scan_complete", (event) => {
    console.log('[DEBUG] 收到 ocr_scan_complete 事件:', event.payload);
    showOcrResult(event.payload);
  });

  // 监听区域扫描取消事件
  await listen("region_scan_cancelled", async () => {
    console.log('[DEBUG] 收到 region_scan_cancelled 事件');
    await invoke("show_window");
  });

  // 监听区域扫描错误事件
  await listen("region_scan_error", async (event) => {
    console.log('[DEBUG] 收到 region_scan_error 事件:', event.payload);
    await invoke("show_window");
    showError(`扫描失败: ${event.payload}`);
  });

  console.log('[DEBUG] Tauri 事件监听器已初始化');
}

// 全屏扫描
async function handleScanFull() {
  try {
    console.log('[DEBUG] 开始全屏扫描流程');

    // 显示加载状态
    showStatus('正在准备扫描...');

    // 最小化主窗口，准备截取屏幕
    console.log('[DEBUG] 最小化窗口');
    await invoke("minimize_window");

    // 等待窗口完全最小化
    await new Promise(resolve => setTimeout(resolve, 300));

    // 执行扫描
    console.log('[DEBUG] 调用 scan_full_screen');
    showStatus('正在识别二维码...');
    const results = await invoke("scan_full_screen");
    console.log('[DEBUG] 扫描结果:', results);

    // 显示窗口并展示结果
    console.log('[DEBUG] 恢复窗口');
    await invoke("show_window");
    showResults(results);
  } catch (error) {
    console.error('[ERROR] 扫描失败:', error);
    // 确保无论如何都恢复窗口显示
    try {
      await invoke("show_window");
    } catch (e) {
      console.error('[ERROR] 恢复窗口显示失败:', e);
    }
    showError(`扫描失败: ${error}`);
  }
}

// 区域扫描
async function handleScanRegion() {
  try {
    console.log('[DEBUG] 开始区域扫描流程');

    // 直接调用后端，后端会处理窗口隐藏和截图
    console.log('[DEBUG] 调用 start_region_selection');
    await invoke("start_region_selection");
  } catch (error) {
    console.error('[ERROR] 准备区域扫描失败:', error);
    try {
      await invoke("show_window");
    } catch (e) {
      console.error('[ERROR] 恢复窗口显示失败:', e);
    }
    showError(`准备区域扫描失败: ${error}`);
  }
}

// OCR 区域文字识别
async function handleOcrRegion() {
  try {
    console.log('[DEBUG] 开始 OCR 区域识别流程');

    // 先清空旧结果，避免显示过期数据
    const items = resultsList.querySelectorAll('.result-item');
    items.forEach(item => item.remove());
    resultCountEl.textContent = "0";
    noResultsEl.classList.add("hidden");

    // 调用后端 OCR 区域选择
    console.log('[DEBUG] 调用 start_ocr_region_selection');
    await invoke("start_ocr_region_selection");
  } catch (error) {
    console.error('[ERROR] 准备 OCR 识别失败:', error);
    try {
      await invoke("show_window");
    } catch (e) {
      console.error('[ERROR] 恢复窗口显示失败:', e);
    }
    showError(`准备 OCR 识别失败: ${error}`);
  }
}

// 显示状态
function showStatus(message) {
  statusTextEl.textContent = message;
  statusEl.classList.remove("hidden");
}

// 隐藏状态
function hideStatus() {
  statusEl.classList.add("hidden");
}

// 显示结果
function showResults(results) {
  hideStatus();

  if (!results || results.length === 0) {
    // 显示空状态（未找到二维码）
    noResultsEl.innerHTML = `
      <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/>
        <path d="M21 21l-4.35-4.35"/>
        <path d="M11 8v6M8 11h6"/>
      </svg>
      <p class="empty-title">未发现二维码</p>
      <p class="empty-desc">请尝试全屏扫描或框选包含二维码的区域</p>
    `;
    noResultsEl.classList.remove("hidden");
    resultCountEl.textContent = "0";
    return;
  }

  noResultsEl.classList.add("hidden");
  resultCountEl.textContent = results.length;

  // 清空并重新渲染结果列表（保留空状态元素）
  const items = resultsList.querySelectorAll('.result-item');
  items.forEach(item => item.remove());

  results.forEach((result, index) => {
    const item = createResultItem(result, index, results.length);
    resultsList.appendChild(item);
  });
}

// 显示 OCR 识别结果
function showOcrResult(result) {
  console.log('[DEBUG] showOcrResult 被调用，参数:', result);
  hideStatus();

  // 强制清空所有旧结果
  const items = resultsList.querySelectorAll('.result-item');
  console.log('[DEBUG] 清空旧结果数量:', items.length);
  items.forEach(item => item.remove());

  // 重置结果计数
  resultCountEl.textContent = "0";

  if (!result || !result.text || result.text.trim() === "") {
    console.log('[DEBUG] 未识别到文字，显示空状态');
    // 显示空状态（未识别到文字）
    noResultsEl.innerHTML = `
      <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <line x1="16" y1="13" x2="8" y2="13"/>
        <line x1="16" y1="17" x2="8" y2="17"/>
      </svg>
      <p class="empty-title">未识别到文字</p>
      <p class="empty-desc">请尝试选择包含清晰文字的区域</p>
    `;
    noResultsEl.classList.remove("hidden");
    return;
  }

  console.log('[DEBUG] 识别到文字，长度:', result.text.length, '引擎:', result.language);
  noResultsEl.classList.add("hidden");
  resultCountEl.textContent = "1";

  // 创建 OCR 结果项
  const item = createOcrResultItem(result);
  resultsList.appendChild(item);
}

// 创建 OCR 结果项
function createOcrResultItem(result) {
  const item = document.createElement("div");
  item.className = "result-item";

  // 类型标签
  const typeLabel = document.createElement("span");
  typeLabel.className = "result-type result-type-ocr";
  typeLabel.textContent = `📄 文字识别 (${result.language})`;

  // 内容
  const content = document.createElement("div");
  content.className = "result-content result-content-ocr";
  content.textContent = result.text;

  // 操作按钮
  const actions = document.createElement("div");
  actions.className = "result-actions";

  const copyBtn = document.createElement("button");
  copyBtn.className = "btn-copy";
  copyBtn.textContent = "复制";
  copyBtn.onclick = () => copyToClipboard(result.text, copyBtn);

  actions.appendChild(copyBtn);

  // 组装
  item.appendChild(typeLabel);
  item.appendChild(content);
  item.appendChild(actions);

  return item;
}

// 创建结果项
function createResultItem(result, index, total) {
  const item = document.createElement("div");
  item.className = "result-item";

  // 类型标签（多个二维码时显示序号）
  const typeLabel = document.createElement("span");
  typeLabel.className = "result-type";
  const prefix = total > 1 ? `二维码${index + 1} - ` : "";
  typeLabel.textContent = prefix + getQrTypeLabel(result.qr_type);

  // 内容
  const content = document.createElement("div");
  content.className = "result-content";
  content.textContent = result.content;

  // 操作按钮
  const actions = document.createElement("div");
  actions.className = "result-actions";

  const copyBtn = document.createElement("button");
  copyBtn.className = "btn-copy";
  copyBtn.textContent = "复制";
  copyBtn.onclick = () => copyToClipboard(result.content, copyBtn);

  actions.appendChild(copyBtn);

  // 组装
  item.appendChild(typeLabel);
  item.appendChild(content);
  item.appendChild(actions);

  return item;
}

// 获取二维码类型标签
function getQrTypeLabel(qrType) {
  const labels = {
    "Url": "🔗 链接",
    "Text": "📝 文本",
    "Email": "📧 邮箱",
    "Phone": "📞 电话",
    "Other": "🔖 其他"
  };
  return labels[qrType] || "🔖 其他";
}

// 复制到剪贴板
async function copyToClipboard(text, button) {
  try {
    await navigator.clipboard.writeText(text);

    // 更新按钮状态
    const originalText = button.textContent;
    button.textContent = "✓ 已复制";
    button.classList.add("copied");

    setTimeout(() => {
      button.textContent = originalText;
      button.classList.remove("copied");
    }, 2000);
  } catch (error) {
    showError(`复制失败: ${error}`);
  }
}

// 清空结果
function clearResults() {
  // 移除所有结果项
  const items = resultsList.querySelectorAll('.result-item');
  items.forEach(item => item.remove());

  // 恢复空状态
  noResultsEl.innerHTML = `
    <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="11" cy="11" r="8"/>
      <path d="M21 21l-4.35-4.35"/>
      <path d="M11 8v6M8 11h6"/>
    </svg>
    <p class="empty-title">等待扫描</p>
    <p class="empty-desc">点击左侧按钮或使用快捷键开始扫描</p>
  `;
  noResultsEl.classList.remove("hidden");
  resultCountEl.textContent = "0";
}

// 隐藏结果
function hideResults() {
  const items = resultsList.querySelectorAll('.result-item');
  items.forEach(item => item.remove());
  noResultsEl.classList.remove("hidden");
}

// 显示错误
function showError(message) {
  hideStatus();
  alert(message);
}

// 初始化主题
function initTheme() {
  // 从 localStorage 读取保存的主题，默认为亮色
  const savedTheme = localStorage.getItem('theme') || 'light';
  document.documentElement.setAttribute('data-theme', savedTheme);
}

// 切换主题
function toggleTheme() {
  const currentTheme = document.documentElement.getAttribute('data-theme');
  const newTheme = currentTheme === 'dark' ? 'light' : 'dark';

  document.documentElement.setAttribute('data-theme', newTheme);
  localStorage.setItem('theme', newTheme);
}

// ==================== 设置功能 ====================

// 初始化设置
function initSettings() {
  const settings = loadSettings();

  // 应用退出行为设置
  const closeBehaviorRadio = document.querySelector(`input[name="close-behavior"][value="${settings.closeBehavior}"]`);
  if (closeBehaviorRadio) {
    closeBehaviorRadio.checked = true;
  }

  // 应用快捷键设置
  shortcutFullscreenInput.value = settings.shortcuts.fullscreen;
  shortcutRegionInput.value = settings.shortcuts.region;
  shortcutOcrInput.value = settings.shortcuts.ocr;

  // 更新按钮上显示的快捷键
  updateShortcutDisplay();

  // 通知后端当前设置
  applySettingsToBackend(settings);
}

// 加载设置
function loadSettings() {
  const saved = localStorage.getItem('appSettings');
  if (saved) {
    try {
      const parsed = JSON.parse(saved);
      return {
        ...defaultSettings,
        ...parsed,
        shortcuts: { ...defaultSettings.shortcuts, ...parsed.shortcuts }
      };
    } catch (e) {
      console.error('加载设置失败:', e);
    }
  }
  return { ...defaultSettings };
}

// 保存设置到 localStorage
function saveSettingsToStorage(settings) {
  localStorage.setItem('appSettings', JSON.stringify(settings));
}

// 打开设置弹窗
function openSettings() {
  const settings = loadSettings();

  // 填充当前设置值
  const closeBehaviorRadio = document.querySelector(`input[name="close-behavior"][value="${settings.closeBehavior}"]`);
  if (closeBehaviorRadio) {
    closeBehaviorRadio.checked = true;
  }

  shortcutFullscreenInput.value = settings.shortcuts.fullscreen;
  shortcutRegionInput.value = settings.shortcuts.region;

  settingsModal.classList.remove("hidden");
}

// 关闭设置弹窗
function closeSettings() {
  settingsModal.classList.add("hidden");
  stopRecordingShortcut();
}

// 保存设置
async function saveSettings() {
  const closeBehavior = document.querySelector('input[name="close-behavior"]:checked').value;
  const shortcuts = {
    fullscreen: shortcutFullscreenInput.value,
    region: shortcutRegionInput.value,
    ocr: shortcutOcrInput.value
  };

  const settings = {
    closeBehavior,
    shortcuts
  };

  try {
    // 应用到后端
    await applySettingsToBackend(settings);

    // 保存到 localStorage
    saveSettingsToStorage(settings);

    // 更新按钮上显示的快捷键
    updateShortcutDisplay();

    // 关闭弹窗
    closeSettings();

    // 显示成功提示
    showToast('设置保存成功', 'success');

    console.log('[DEBUG] 设置已保存:', settings);
  } catch (error) {
    console.error('[ERROR] 保存设置失败:', error);
    showToast('保存失败: ' + error, 'error');
  }
}

// 应用设置到后端
async function applySettingsToBackend(settings) {
  // 更新退出行为
  await invoke("set_close_behavior", { behavior: settings.closeBehavior });

  // 更新快捷键
  await invoke("update_shortcuts", {
    fullscreenShortcut: convertShortcutToBackend(settings.shortcuts.fullscreen),
    regionShortcut: convertShortcutToBackend(settings.shortcuts.region),
    ocrShortcut: convertShortcutToBackend(settings.shortcuts.ocr)
  });
}

// 显示提示消息
function showToast(message, type = 'info') {
  // 移除已有的 toast
  const existingToast = document.querySelector('.toast');
  if (existingToast) {
    existingToast.remove();
  }

  const toast = document.createElement('div');
  toast.className = `toast toast-${type}`;
  toast.textContent = message;
  document.body.appendChild(toast);

  // 触发动画
  requestAnimationFrame(() => {
    toast.classList.add('show');
  });

  // 3秒后自动消失
  setTimeout(() => {
    toast.classList.remove('show');
    setTimeout(() => toast.remove(), 300);
  }, 3000);
}

// 更新按钮上显示的快捷键
function updateShortcutDisplay() {
  const settings = loadSettings();

  const fullscreenKbd = document.querySelector("#scan-full-btn .scan-btn-kbd");
  const regionKbd = document.querySelector("#scan-region-btn .scan-btn-kbd");

  if (fullscreenKbd) {
    fullscreenKbd.textContent = settings.shortcuts.fullscreen;
  }
  if (regionKbd) {
    regionKbd.textContent = settings.shortcuts.region;
  }
  
  const ocrKbd = document.querySelector("#ocr-region-btn .scan-btn-kbd");
  if (ocrKbd) {
    ocrKbd.textContent = settings.shortcuts.ocr;
  }
}

// 将前端快捷键格式转换为后端格式
function convertShortcutToBackend(shortcut) {
  // 前端格式: "Ctrl + Shift + S"
  // 后端格式: "ctrl+shift+s"
  return shortcut
    .toLowerCase()
    .replace(/\s*\+\s*/g, '+')
    .replace('control', 'ctrl');
}

// ==================== 快捷键录制 ====================

// 开始录制快捷键
function startRecordingShortcut(target) {
  isRecordingShortcut = true;
  currentRecordingTarget = target;

  const input = target === 'fullscreen' ? shortcutFullscreenInput : (target === 'region' ? shortcutRegionInput : shortcutOcrInput);
  const btn = document.querySelector(`.shortcut-record-btn[data-target="${target}"]`);

  input.classList.add("recording");
  input.value = "请按下快捷键...";
  btn.classList.add("recording");
  btn.textContent = "停止";

  // 点击停止按钮时停止录制
  btn.onclick = () => stopRecordingShortcut();
}

// 停止录制快捷键
function stopRecordingShortcut() {
  if (!isRecordingShortcut) return;

  const target = currentRecordingTarget;
  const input = target === 'fullscreen' ? shortcutFullscreenInput : (target === 'region' ? shortcutRegionInput : shortcutOcrInput);
  const btn = document.querySelector(`.shortcut-record-btn[data-target="${target}"]`);

  input.classList.remove("recording");
  btn.classList.remove("recording");
  btn.textContent = "录制";

  // 恢复原来的点击事件
  btn.onclick = () => startRecordingShortcut(target);

  // 如果没有录制到有效快捷键，恢复原值
  if (input.value === "请按下快捷键...") {
    const settings = loadSettings();
    input.value = target === 'fullscreen' ? settings.shortcuts.fullscreen : (target === 'region' ? settings.shortcuts.region : settings.shortcuts.ocr);
  }

  isRecordingShortcut = false;
  currentRecordingTarget = null;
}

// 处理快捷键录制的键盘事件
function handleShortcutKeydown(event) {
  if (!isRecordingShortcut) return;

  event.preventDefault();
  event.stopPropagation();

  // 忽略单独的修饰键
  if (['Control', 'Shift', 'Alt', 'Meta'].includes(event.key)) {
    return;
  }

  // 构建快捷键字符串
  const parts = [];
  if (event.ctrlKey) parts.push('Ctrl');
  if (event.shiftKey) parts.push('Shift');
  if (event.altKey) parts.push('Alt');

  // 获取主键
  let key = event.key.toUpperCase();
  if (key === ' ') key = 'Space';
  if (key.length === 1) key = key.toUpperCase();

  parts.push(key);

  const shortcut = parts.join(' + ');

  // 至少需要一个修饰键 + 一个普通键
  if (parts.length < 2) {
    return;
  }

  // 更新输入框
  const input = currentRecordingTarget === 'fullscreen' ? shortcutFullscreenInput : (currentRecordingTarget === 'region' ? shortcutRegionInput : shortcutOcrInput);
  input.value = shortcut;

  // 停止录制
  stopRecordingShortcut();
}

