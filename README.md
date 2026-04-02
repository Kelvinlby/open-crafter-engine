<h1 align="center">Open Crafter Engine</h1>
<h3 align="center">Inference engine backend.</h3>

<div align="center">

[![paper](https://img.shields.io/static/v1?label=Github&message=open-crafter&color=white&logo=github&style=for-the-badge)](https://github.com/Kelvinlby/open-crafter)

</div>

# HTTP Server API Endpoints

The HTTP server runs on the configured host and port (default: serves web UI + API). All API endpoints are prefixed with `/api`.

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

## Discord Configuration

### GET `/api/discord`

Retrieve the current Discord bot configuration.

**Response:** `200 OK`
```json
{
  "botToken": "your-bot-token",
  "adminRoleId": "123456789012345678",
  "channelIds": ["111111111111111111", "222222222222222222"]
}
```

---

### POST `/api/discord/save`

Save Discord bot configuration to the configuration file.

**Request:**
```json
{
  "botToken": "your-bot-token",
  "adminRoleId": "123456789012345678",
  "channelIds": ["111111111111111111", "222222222222222222"]
}
```

**Response:** `200 OK`
```
"ok"
```

---

## Example Usage

### cURL Examples

```bash
# Get current model configuration
curl http://localhost:8080/api/model

# Scan for models
curl -X POST http://localhost:8080/api/model/scan \
  -H "Content-Type: application/json" \
  -d '{"modelPath": "/path/to/models"}'

# Save model selection
curl -X POST http://localhost:8080/api/model/save \
  -H "Content-Type: application/json" \
  -d '{"modelPath": "/path/to/models", "selectedModel": "/path/to/models/my-model"}'

# Save a hyperparameter
curl -X POST http://localhost:8080/api/model/hyperparam \
  -H "Content-Type: application/json" \
  -d '{"paramId": "temperature", "value": 0.8}'

# Get runtime info
curl http://localhost:8080/api/runtime

# Save inference device
curl -X POST http://localhost:8080/api/runtime/save \
  -H "Content-Type: application/json" \
  -d '{"inferenceDevice": "CUDA:0 (NVIDIA GeForce RTX 3080)"}'

# Get Discord config
curl http://localhost:8080/api/discord

# Save Discord config
curl -X POST http://localhost:8080/api/discord/save \
  -H "Content-Type: application/json" \
  -d '{"botToken": "token", "adminRoleId": "123", "channelIds": ["456"]}'
```

### JavaScript (fetch) Example

```javascript
// Save model configuration
const response = await fetch('http://localhost:8080/api/model/save', {
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
- All successful POST operations return the string `"ok"`.
- Configuration changes are persisted to the engine's config file.
- Hyperparameter changes are written to the selected model's `metadata.json`.

