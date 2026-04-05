<h1 align="center">Open Crafter Engine</h1>
<h3 align="center">Inference engine backend.</h3>

<div align="center">

[![mod](https://img.shields.io/static/v1?label=Github&message=open-crafter&color=white&logo=github&style=for-the-badge)](https://github.com/Kelvinlby/open-crafter)
[![discord](https://img.shields.io/static/v1?label=Discord&message=Chat&color=7289DA&logo=discord&style=for-the-badge)](https://discord.gg/FjRpnp3S8z)

</div>

# Architecture overview

Open Crafter has three components:

- **Mod** — Fabric client mod. Runs inside Minecraft, hosts a Unix domain socket server, and exposes game state/actions to the engine via JSON-RPC.
- **Engine** (this repo) — Rust backend that drives the AI model and issues commands over the socket.
- **Web UI** — Frontend panel, opened in-game via the mod's embedded browser.

# Configuration

The engine stores its configuration in `engine-config.json`, located one directory above the binary (`../engine-config.json` relative to the executable). The file is created with defaults on first run and can be updated at runtime via `POST /api/config/save` on the Control Panel (see below), which also restarts the OpenAI API server to apply the new settings.

## `acceptedIpRange`

CIDR notation string controlling which client IPs are allowed to reach the OpenAI API.

| Value | Effect |
|---|---|
| `0.0.0.0/0` | Allow all IPv4 addresses (open access) |
| `::/0` | Allow all IPv6 addresses |
| `192.168.1.0/24` | Allow only the `192.168.1.x` subnet |
| `10.0.0.0/8` | Allow the entire RFC-1918 private range |

IPv4-mapped IPv6 addresses (e.g. `::ffff:192.168.1.5`) are automatically unwrapped before matching, so an IPv4 CIDR correctly matches clients connecting over a dual-stack socket.

## `port`

Integer string between `"1"` and `"65535"`. The OpenAI-compatible API server listens on this port. Default: `"8080"`. The Control Panel port is set separately via the `--port` CLI flag (default `6121`) and is not affected by this field.

## `apiKeys`

Array of `{ "name": string, "key": string }` objects. The `key` field is used as the Bearer token. Keys should be random hex strings — 64 hex characters (32 bytes) is recommended. Example entry:

```json
{ "name": "my-integration", "key": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2" }
```

API keys can be added and removed via the Control Panel without restarting the server.

---

# OpenAI Compatible API Endpoints

The OpenAI Compatible API server runs on a separate port (default: `8080`). This server provides endpoints compatible with OpenAI's API format, allowing integration with existing OpenAI SDKs and tools.

All endpoints require authentication via Bearer token (configured API keys) and are subject to IP filtering based on the configured IP range.

**Base URL:** `http://localhost:<port>/v1`

## Authentication

Include your API key in the Authorization header:
```
Authorization: Bearer <your-api-key>
```

---

## Model Endpoints

### GET `/v1/models`

List all available models in the configured model directory.

**Response:** `200 OK`
```json
{
  "object": "list",
  "data": [
    {
      "id": "my-model",
      "object": "model",
      "created": 1234567890,
      "owned_by": "user"
    }
  ]
}
```

---

### GET `/v1/models/{model_id}`

Retrieve detailed information about a specific model.

**URL parameter:** `model_id` — The model identifier (folder name or metadata model name).

**Response:** `200 OK`
```json
{
  "id": "my-model",
  "object": "model",
  "created": 1234567890,
  "owned_by": "user",
  "model_name": "My Custom Model",
  "model_version": "1.0.0"
}
```

**Error:** `404 Not Found` if the model doesn't exist.
```json
{
  "error": {
    "message": "Model 'nonexistent' not found",
    "type": "invalid_request_error",
    "param": "model",
    "code": "model_not_found"
  }
}
```

---

## Chat Completion

### POST `/v1/chat/completions`

Generate a response for the given conversation. Context is managed server-side, so only the current turn needs to be provided.

**Request:**
```json
{
  "model": "my-model",
  "messages": [
    {
      "role": "user",
      "content": "Hello, how are you?"
    }
  ],
  "stream": false,
  "temperature": 0.7,
  "max_tokens": 256
}
```

**Response (non-streaming):** `200 OK`
```json
{
  "id": "chatcmpl-uuid-here",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "my-model",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "I'm doing well, thank you!"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 0,
    "completion_tokens": 0,
    "total_tokens": 0
  }
}
```

**Response (streaming):** `200 OK` with Server-Sent Events (SSE)
```
data: {"id":"chatcmpl-uuid","object":"chat.completion.chunk","created":1234567890,"model":"my-model","choices":[{"index":0,"delta":{"role":"assistant"},"finish_reason":null}]}

data: {"id":"chatcmpl-uuid","object":"chat.completion.chunk","created":1234567890,"model":"my-model","choices":[{"index":0,"delta":{"content":"I'm doing well"},"finish_reason":null}]}

data: {"id":"chatcmpl-uuid","object":"chat.completion.chunk","created":1234567890,"model":"my-model","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]
```

---

# Control Panel API Endpoints

The HTTP server for Web control panel runs on the configured host and port (default: serves web UI + API). All API endpoints are prefixed with `/api`.

**Base URL:** `http://localhost:<port>/api`

---

## Model Management

### GET `/api/model`

Retrieve the current model configuration, available models, and hyperparameters.

**Response:** `200 OK`
```json
{
  "modelPath": "/path/to/models",
  "selectedModel": "/path/to/models/selected-model",
  "availableModels": [
    { "folder": "/path/to/models/model-a", "name": "Model A" },
    { "folder": "/path/to/models/model-b", "name": "Model B" }
  ],
  "hyperparams": [
    {
      "id": "temperature",
      "title": "Temperature",
      "value": 0.7,
      "min": 0.0,
      "max": 1.0,
      "step": 0.01,
      "defaultValue": 0.7
    }
  ]
}
```

---

### POST `/api/model/scan`

Scan a directory for available models.

**Request:**
```json
{
  "modelPath": "/path/to/models"
}
```

**Response:** `200 OK`
```json
[
  { "folder": "/path/to/models/model-a", "name": "Model A" },
  { "folder": "/path/to/models/model-b", "name": "Model B" }
]
```

---

### POST `/api/model/save`

Save the model path and selected model to the configuration file.

**Request:**
```json
{
  "modelPath": "/path/to/models",
  "selectedModel": "/path/to/models/selected-model"
}
```

**Response:** `200 OK`
```
"ok"
```

---

### POST `/api/model/hyperparam`

Save a hyperparameter value to the selected model's `metadata.json`.

**Prerequisite:** A model must be selected via `/api/model/save` first.

**Request:**
```json
{
  "paramId": "temperature",
  "value": 0.8
}
```

**Response:** `200 OK`
```
"ok"
```

**Error:** `400 Bad Request` if no model is selected.

---

## Runtime Configuration

### GET `/api/runtime`

Retrieve system resource usage (RAM, VRAM, GPU) and available inference devices.

**Response:** `200 OK`
```json
{
  "ram": { "label": "RAM", "value": 45.2, "detail": "7.2 / 16.0 GB" },
  "vram": { "label": "VRAM", "value": 62.0, "detail": "8.0 / 12.0 GB" },
  "gpu": { "label": "GPU", "value": 30, "detail": "30% utilization" },
  "selectedDevice": "CUDA:0 (NVIDIA GeForce RTX 3080)",
  "availableDevices": [
    "CUDA:0 (NVIDIA GeForce RTX 3080)",
    "CPU"
  ]
}
```

---

### POST `/api/runtime/save`

Save the inference device selection to the configuration file.

**Request:**
```json
{
  "inferenceDevice": "CUDA:0 (NVIDIA GeForce RTX 3080)"
}
```

**Response:** `200 OK`
```
"ok"
```

---

## Skills & Tools

### GET `/api/skills`

Retrieve the list of available skills.

**Response:** `200 OK`
```json
[
  {
    "id": "pathfinding",
    "title": "Pathfinding",
    "version": "1.2.0",
    "description": "A* pathfinding with dynamic obstacle avoidance..."
  },
  {
    "id": "building",
    "title": "Building",
    "version": "0.8.1",
    "description": "Schematic-based building with automatic material gathering..."
  }
]
```

---

### GET `/api/tools`

Retrieve the list of available tools.

**Response:** `200 OK`
```json
[
  {
    "id": "chat",
    "title": "Chat",
    "version": "1.0.0",
    "description": "Send and receive in-game chat messages..."
  },
  {
    "id": "inventory",
    "title": "Inventory",
    "version": "1.3.2",
    "description": "Inspect and manage player inventory..."
  }
]
```

---

## API Configuration

### GET `/api/config`

Retrieve the current API configuration.

**Response:** `200 OK`
```json
{
  "acceptedIpRange": "0.0.0.0/0",
  "port": "8080",
  "apiKeys": [
    { "name": "dev-key", "key": "a1b2c3d4..." }
  ]
}
```

---

### POST `/api/config/save`

Save the accepted IP range and port to the configuration file.

**Request:**
```json
{
  "acceptedIpRange": "192.168.1.0/24",
  "port": "8080"
}
```

**Response:** `200 OK`
```
"ok"
```

---

### POST `/api/config/api-key`

Add a new API key. The key value should be generated client-side (e.g. a 32-byte random hex string).

**Request:**
```json
{
  "name": "my-integration",
  "key": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
}
```

**Response:** `200 OK`
```
"ok"
```

---

### DELETE `/api/config/api-key/{index}`

Remove the API key at the given zero-based index in the stored keys array.

**URL parameter:** `index` — integer, zero-based position in the `apiKeys` array.

**Response:** `200 OK`
```
"ok"
```

**Error:** `400 Bad Request` if the index is out of range.

---

## Example Usage

### cURL Examples

```bash
# Get current model configuration
curl http://localhost:6121/api/model

# Scan for models
curl -X POST http://localhost:6121/api/model/scan \
  -H "Content-Type: application/json" \
  -d '{"modelPath": "/path/to/models"}'

# Save model selection
curl -X POST http://localhost:6121/api/model/save \
  -H "Content-Type: application/json" \
  -d '{"modelPath": "/path/to/models", "selectedModel": "/path/to/models/my-model"}'

# Save a hyperparameter
curl -X POST http://localhost:6121/api/model/hyperparam \
  -H "Content-Type: application/json" \
  -d '{"paramId": "temperature", "value": 0.8}'

# Get runtime info
curl http://localhost:6121/api/runtime

# Save inference device
curl -X POST http://localhost:6121/api/runtime/save \
  -H "Content-Type: application/json" \
  -d '{"inferenceDevice": "CUDA:0 (NVIDIA GeForce RTX 3080)"}'

# Get API configuration
curl http://localhost:6121/api/config

# Save IP range and port
curl -X POST http://localhost:6121/api/config/save \
  -H "Content-Type: application/json" \
  -d '{"acceptedIpRange": "192.168.1.0/24", "port": "8080"}'

# Add an API key
curl -X POST http://localhost:6121/api/config/api-key \
  -H "Content-Type: application/json" \
  -d '{"name": "my-integration", "key": "a1b2c3d4e5f6..."}'

# Delete the first API key
curl -X DELETE http://localhost:6121/api/config/api-key/0
```

### JavaScript (fetch) Example

```javascript
// Save model configuration
const response = await fetch('http://localhost:6121/api/model/save', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    modelPath: '/path/to/models',
    selectedModel: '/path/to/models/my-model'
  })
});

if (response.ok) {
  console.log('Model saved successfully');
}
```

---

## Notes

- All POST endpoints expect `Content-Type: application/json` header.
- All successful POST and DELETE operations return the string `"ok"`.
- Configuration changes are persisted to the engine's config file.
- Hyperparameter changes are written to the selected model's `metadata.json`.

