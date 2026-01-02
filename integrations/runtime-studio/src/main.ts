import './style.css';

type ModelSpec = {
  name: string;
  version: string;
  format?: string;
  backend?: string;
};

type StreamChunk = {
  kind: 'start' | 'delta' | 'event' | 'end';
  request_id: string;
  model?: string;
  metadata?: Record<string, string>;
  delta_text?: string;
  name?: string;
  data?: unknown;
  usage?: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
};

const baseUrlInput = document.getElementById('baseUrl') as HTMLInputElement;
const saveBaseButton = document.getElementById('saveBase') as HTMLButtonElement;
const pingButton = document.getElementById('ping') as HTMLButtonElement;
const gatewayStatus = document.getElementById('gatewayStatus') as HTMLSpanElement;
const runtimeStatus = document.getElementById('runtimeStatus') as HTMLSpanElement;
const metricsLink = document.getElementById('metricsLink') as HTMLAnchorElement;

const loadForm = document.getElementById('loadForm') as HTMLFormElement;
const modelName = document.getElementById('modelName') as HTMLInputElement;
const modelVersion = document.getElementById('modelVersion') as HTMLInputElement;
const modelPath = document.getElementById('modelPath') as HTMLInputElement;
const refreshModels = document.getElementById('refreshModels') as HTMLButtonElement;
const modelList = document.getElementById('modelList') as HTMLUListElement;
const modelCount = document.getElementById('modelCount') as HTMLSpanElement;

const inferForm = document.getElementById('inferForm') as HTMLFormElement;
const inferModel = document.getElementById('inferModel') as HTMLInputElement;
const inferMode = document.getElementById('inferMode') as HTMLSelectElement;
const inferPrompt = document.getElementById('inferPrompt') as HTMLTextAreaElement;
const inferMaxTokens = document.getElementById('inferMaxTokens') as HTMLInputElement;
const inferTemperature = document.getElementById('inferTemperature') as HTMLInputElement;
const inferStream = document.getElementById('inferStream') as HTMLInputElement;
const inferOutput = document.getElementById('inferOutput') as HTMLPreElement;
const inferStatus = document.getElementById('inferStatus') as HTMLSpanElement;

const eventLog = document.getElementById('eventLog') as HTMLUListElement;

const state = {
  baseUrl: localStorage.getItem('runtimeBaseUrl') || 'http://localhost:8080',
};

function setPillStatus(el: HTMLSpanElement, ok: boolean | null, text: string) {
  el.textContent = text;
  el.classList.remove('ok', 'bad', 'neutral');
  if (ok === null) {
    el.classList.add('neutral');
    return;
  }
  el.classList.add(ok ? 'ok' : 'bad');
}

function logEvent(message: string) {
  const li = document.createElement('li');
  const time = new Date().toLocaleTimeString();
  li.textContent = `[${time}] ${message}`;
  eventLog.prepend(li);
}

function updateBaseUrl() {
  const url = baseUrlInput.value.trim();
  state.baseUrl = url || 'http://localhost:8080';
  localStorage.setItem('runtimeBaseUrl', state.baseUrl);
  metricsLink.href = `${state.baseUrl}/metrics`;
  logEvent(`Base URL set to ${state.baseUrl}`);
}

async function fetchJson(path: string, init?: RequestInit) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 15000);
  try {
    const res = await fetch(`${state.baseUrl}${path}`, {
      ...init,
      headers: {
        'content-type': 'application/json',
        ...(init?.headers || {}),
      },
      signal: controller.signal,
    });
    const data = await res.json().catch(() => ({}));
    if (!res.ok) {
      throw new Error(data?.error?.message || `Request failed: ${res.status}`);
    }
    return data;
  } finally {
    clearTimeout(timer);
  }
}

async function pingHealth() {
  try {
    const health = await fetch(`${state.baseUrl}/healthz`);
    setPillStatus(gatewayStatus, health.ok, health.ok ? 'ok' : 'down');
  } catch (err) {
    setPillStatus(gatewayStatus, false, 'down');
  }

  try {
    const ready = await fetch(`${state.baseUrl}/readyz`);
    setPillStatus(runtimeStatus, ready.ok, ready.ok ? 'ready' : 'not ready');
  } catch (err) {
    setPillStatus(runtimeStatus, false, 'not ready');
  }
}

async function loadModels() {
  modelList.innerHTML = '';
  modelCount.textContent = '0';
  try {
    const data = await fetchJson('/esnode/v1/models', { method: 'GET' });
    const models: ModelSpec[] = data.models || [];
    models.forEach((spec) => {
      const li = document.createElement('li');
      li.innerHTML = `<strong>${spec.name}</strong><span>${spec.version || 'v0'}</span>`;
      modelList.appendChild(li);
    });
    modelCount.textContent = String(models.length);
  } catch (err) {
    logEvent(`List models failed: ${(err as Error).message}`);
  }
}

async function loadBundle(event: Event) {
  event.preventDefault();
  const body = {
    name: modelName.value.trim(),
    version: modelVersion.value.trim() || 'v0',
    source: { kind: 'local_path', path: modelPath.value.trim() || 'models/fixture.gguf' },
    format: 'auto',
    backend: 'auto',
    backend_settings: {},
    labels: {},
  };
  try {
    const data = await fetchJson('/esnode/v1/models/load', {
      method: 'POST',
      body: JSON.stringify(body),
    });
    logEvent(`Loaded bundle: ${data.model_handle || body.name}`);
    await loadModels();
  } catch (err) {
    logEvent(`Load failed: ${(err as Error).message}`);
  }
}

function buildInferRequest() {
  const requestId = `req-${Date.now()}`;
  const model = inferModel.value.trim();
  const prompt = inferPrompt.value.trim();
  const mode = inferMode.value;
  const params = {
    stream: inferStream.checked,
    max_tokens: inferMaxTokens.value ? Number(inferMaxTokens.value) : undefined,
    temperature: inferTemperature.value ? Number(inferTemperature.value) : undefined,
  };

  const input =
    mode === 'completion'
      ? { type: 'completion', prompt }
      : { type: 'chat', messages: [{ role: 'user', content: prompt }] };

  return {
    request_id: requestId,
    model,
    input,
    params,
    metadata: {},
  };
}

async function runInference(event: Event) {
  event.preventDefault();
  inferOutput.textContent = '';
  setPillStatus(inferStatus, null, 'working');
  const body = buildInferRequest();

  if (!body.model) {
    setPillStatus(inferStatus, false, 'missing model');
    return;
  }

  if (inferStream.checked) {
    await runStream(body);
    return;
  }

  try {
    const data = await fetchJson('/esnode/v1/infer', {
      method: 'POST',
      body: JSON.stringify(body),
    });
    const content = data.output?.message?.content || data.output?.text || JSON.stringify(data, null, 2);
    inferOutput.textContent = content;
    setPillStatus(inferStatus, true, 'done');
    logEvent(`Inference complete for ${body.model}`);
  } catch (err) {
    setPillStatus(inferStatus, false, 'error');
    inferOutput.textContent = (err as Error).message;
  }
}

async function runStream(body: ReturnType<typeof buildInferRequest>) {
  try {
    const res = await fetch(`${state.baseUrl}/esnode/v1/infer/stream`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(body),
    });

    if (!res.ok || !res.body) {
      throw new Error(`Stream failed: ${res.status}`);
    }

    const reader = res.body.getReader();
    const decoder = new TextDecoder();
    let buffer = '';

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      buffer += decoder.decode(value, { stream: true });

      const parts = buffer.split('\n\n');
      buffer = parts.pop() || '';
      for (const part of parts) {
        const line = part.split('\n').find((l) => l.startsWith('data:'));
        if (!line) continue;
        const payload = line.replace('data:', '').trim();
        const chunk = JSON.parse(payload) as StreamChunk;
        if (chunk.kind === 'delta' && chunk.delta_text) {
          inferOutput.textContent += chunk.delta_text;
        }
      }
    }

    setPillStatus(inferStatus, true, 'stream complete');
    logEvent(`Stream finished for ${body.model}`);
  } catch (err) {
    setPillStatus(inferStatus, false, 'stream error');
    inferOutput.textContent = (err as Error).message;
  }
}

function init() {
  baseUrlInput.value = state.baseUrl;
  metricsLink.href = `${state.baseUrl}/metrics`;
  saveBaseButton.addEventListener('click', () => {
    updateBaseUrl();
    pingHealth();
    loadModels();
  });
  pingButton.addEventListener('click', pingHealth);
  loadForm.addEventListener('submit', loadBundle);
  refreshModels.addEventListener('click', loadModels);
  inferForm.addEventListener('submit', runInference);
  updateBaseUrl();
  pingHealth();
  loadModels();
}

init();
