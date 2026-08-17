import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AppSnapshot, HistoryRecord, ReceiverStatus } from "./types";
import "./styles.css";

const receiverStatus = requiredElement<HTMLParagraphElement>("receiver-status");
const autostartToggle = requiredElement<HTMLButtonElement>("autostart-toggle");
const historyList = requiredElement<HTMLUListElement>("history-list");
const clearHistoryButton = requiredElement<HTMLButtonElement>("clear-history");
const emptyState = requiredElement<HTMLParagraphElement>("empty-state");
const warning = requiredElement<HTMLParagraphElement>("warning");
const copyFeedback = requiredElement<HTMLParagraphElement>("copy-feedback");

let autostartEnabled = false;
let feedbackTimer: number | undefined;
let stopSnapshotListener: UnlistenFn | undefined;
let stopClearListener: UnlistenFn | undefined;
let receivedSnapshotUpdate = false;
let clearInProgress = false;
let feedbackOperation = 0;
let autostartOperation = 0;
let copyQueue = Promise.resolve();
const digitFormatter = new Intl.NumberFormat(undefined, { minimumIntegerDigits: 2, useGrouping: false });

autostartToggle.disabled = true;

function requiredElement<T extends HTMLElement>(id: string): T {
  const element = document.getElementById(id);
  if (!(element instanceof HTMLElement)) {
    throw new Error(`缺少界面元素：${id}`);
  }
  return element as T;
}

function pad(value: number): string {
  return digitFormatter.format(value);
}

export function formatReceivedAt(receivedAt: number, now = new Date()): string {
  const received = new Date(receivedAt);
  const time = `${pad(received.getHours())}:${pad(received.getMinutes())}`;
  const receivedDay = new Date(received.getFullYear(), received.getMonth(), received.getDate());
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const dayDifference = Math.round((today.getTime() - receivedDay.getTime()) / 86_400_000);

  if (dayDifference === 0) {
    return time;
  }
  if (dayDifference === 1) {
    return `昨天 · ${time}`;
  }
  return `${pad(received.getMonth() + 1)}-${pad(received.getDate())} · ${time}`;
}

function statusPresentation(status: ReceiverStatus): { text: string; className: string } {
  switch (status.kind) {
    case "listening":
      return { text: `接收中 · 端口 ${status.port}`, className: "status-listening" };
    case "degraded":
      return { text: `接收异常 · 端口 ${status.port} · ${status.message}`, className: "status-degraded" };
    case "unavailable":
      return { text: `接收不可用 · 端口 ${status.port} · ${status.message}`, className: "status-unavailable" };
    case "starting":
      return { text: "正在启动接收服务", className: "status-starting" };
  }
}

function renderStatus(status: ReceiverStatus): void {
  const presentation = statusPresentation(status);
  receiverStatus.textContent = presentation.text;
  receiverStatus.className = `receiver-status ${presentation.className}`;
}

function renderAutostart(enabled: boolean): void {
  autostartEnabled = enabled;
  autostartToggle.setAttribute("aria-checked", String(enabled));
}

function createRow(record: HistoryRecord): HTMLLIElement {
  const row = document.createElement("li");
  row.className = "message-row";
  row.tabIndex = 0;
  row.dataset.copyValue = record.code ?? record.text;
  row.dataset.code = record.code ?? "";
  row.setAttribute("role", "button");
  row.setAttribute("aria-label", record.code ? `复制验证码 ${record.code}` : "复制消息");

  const meta = document.createElement("div");
  meta.className = "message-meta";
  const title = document.createElement("p");
  title.className = record.code ? "message-code" : "message-label";
  title.textContent = record.code ?? "普通消息";
  const time = document.createElement("p");
  time.className = "message-time";
  time.textContent = formatReceivedAt(record.receivedAt);
  const preview = document.createElement("p");
  preview.className = "message-preview";
  preview.textContent = record.text || "（内容为空）";

  meta.append(title, time);
  row.append(meta, preview);
  return row;
}

function renderHistory(history: HistoryRecord[]): void {
  const scrollTop = historyList.scrollTop;
  const rows = history.slice(0, 15).map(createRow);
  historyList.replaceChildren(...rows);
  emptyState.hidden = rows.length > 0;
  historyList.hidden = rows.length === 0;
  historyList.scrollTop = scrollTop;
}

function renderWarning(storageWarning: string | null): void {
  warning.hidden = !storageWarning;
  warning.textContent = storageWarning ? `历史记录保存异常：${storageWarning}` : "";
}

function renderSnapshot(snapshot: AppSnapshot): void {
  renderStatus(snapshot.receiverStatus);
  renderAutostart(snapshot.autostartEnabled);
  renderWarning(snapshot.storageWarning);
  renderHistory(snapshot.history);
}

function beginFeedbackOperation(): number {
  const operation = ++feedbackOperation;
  window.clearTimeout(feedbackTimer);
  copyFeedback.textContent = "";
  copyFeedback.classList.remove("is-error");
  return operation;
}

function showFeedback(operation: number, message: string, isError = false): void {
  if (operation !== feedbackOperation) {
    return;
  }
  window.clearTimeout(feedbackTimer);
  copyFeedback.textContent = message;
  copyFeedback.classList.toggle("is-error", isError);
  feedbackTimer = window.setTimeout(() => {
    if (operation !== feedbackOperation) {
      return;
    }
    copyFeedback.textContent = "";
    copyFeedback.classList.remove("is-error");
  }, 1500);
}

async function copyRow(copyValue: string, code: string, feedbackToken: number): Promise<void> {
  try {
    await invoke("copy_text", { text: copyValue });
    showFeedback(feedbackToken, code ? `已复制验证码 ${code}` : "已复制消息");
  } catch {
    showFeedback(feedbackToken, "复制失败，请重试", true);
  }
}

function enqueueCopy(row: HTMLLIElement): void {
  const copyValue = row.dataset.copyValue;
  if (copyValue === undefined) {
    return;
  }
  const code = row.dataset.code;
  const feedbackToken = beginFeedbackOperation();
  copyQueue = copyQueue.then(() => copyRow(copyValue, code ?? "", feedbackToken));
}

async function confirmAndClearHistory(): Promise<void> {
  if (clearInProgress) {
    return;
  }
  if (!window.confirm("确定清空全部历史记录吗？")) {
    return;
  }
  const feedbackToken = beginFeedbackOperation();
  clearInProgress = true;
  clearHistoryButton.disabled = true;
  try {
    const snapshot = await invoke<AppSnapshot>("clear_history");
    renderSnapshot(snapshot);
    showFeedback(feedbackToken, "历史记录已清空");
  } catch {
    showFeedback(feedbackToken, "清空历史失败，请重试", true);
  } finally {
    clearInProgress = false;
    clearHistoryButton.disabled = false;
  }
}

async function toggleAutostart(): Promise<void> {
  const desiredState = !autostartEnabled;
  const feedbackToken = beginFeedbackOperation();
  const operation = ++autostartOperation;
  autostartToggle.disabled = true;
  try {
    const verifiedState = await invoke<boolean>("set_autostart", { enabled: desiredState });
    if (operation === autostartOperation) {
      renderAutostart(verifiedState);
    }
  } catch {
    showFeedback(feedbackToken, "自启动设置失败，请重试", true);
  } finally {
    if (operation === autostartOperation) {
      autostartToggle.disabled = false;
    }
  }
}

historyList.addEventListener("click", (event) => {
  const target = event.target;
  if (!(target instanceof Element)) {
    return;
  }
  const row = target.closest<HTMLLIElement>(".message-row");
  if (row && historyList.contains(row)) {
    enqueueCopy(row);
  }
});

historyList.addEventListener("keydown", (event) => {
  if (event.key !== "Enter" && event.key !== " ") {
    return;
  }
  const target = event.target;
  if (target instanceof HTMLLIElement && target.classList.contains("message-row")) {
    event.preventDefault();
    enqueueCopy(target);
  }
});

autostartToggle.addEventListener("click", () => {
  void toggleAutostart();
});

clearHistoryButton.addEventListener("click", () => {
  void confirmAndClearHistory();
});

async function initialize(): Promise<void> {
  const initializationFeedbackToken = beginFeedbackOperation();
  try {
    stopSnapshotListener = await listen<AppSnapshot>("snapshot-updated", (event) => {
      receivedSnapshotUpdate = true;
      renderSnapshot(event.payload);
    });
  } catch {
    showFeedback(initializationFeedbackToken, "无法监听状态更新", true);
  }

  try {
    stopClearListener = await listen("request-clear-history", () => {
      void confirmAndClearHistory();
    });
  } catch {
    showFeedback(initializationFeedbackToken, "无法接收清空请求", true);
  }

  try {
    const snapshot = await invoke<AppSnapshot>("get_snapshot");
    if (!receivedSnapshotUpdate) {
      renderSnapshot(snapshot);
    }
  } catch {
    showFeedback(initializationFeedbackToken, "无法加载接收状态", true);
  }

  const operation = ++autostartOperation;
  try {
    const verifiedState = await invoke<boolean>("get_autostart");
    if (operation === autostartOperation) {
      renderAutostart(verifiedState);
    }
  } catch {
    showFeedback(initializationFeedbackToken, "无法验证自启动状态", true);
  } finally {
    if (operation === autostartOperation) {
      autostartToggle.disabled = false;
    }
  }
}

window.addEventListener("beforeunload", () => {
  window.clearTimeout(feedbackTimer);
  if (stopSnapshotListener) {
    void stopSnapshotListener();
  }
  if (stopClearListener) {
    void stopClearListener();
  }
});

void initialize();
