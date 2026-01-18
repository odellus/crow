# Local LLMs

> When using a Local LLM, OpenHands may have limited functionality. It is highly recommended that you use GPUs to serve local models for optimal experience.

## News

* 2025/12/12: We now recommend two powerful local models for OpenHands: [Qwen3-Coder-30B-A3B-Instruct](https://huggingface.co/Qwen/Qwen3-Coder-30B-A3B-Instruct) and [Devstral Small 2 (24B)](https://huggingface.co/mistralai/Devstral-Small-2-24B-Instruct-2512). Both models deliver excellent performance on coding tasks and work great with OpenHands!

## Quickstart: Running OpenHands with a Local LLM using LM Studio

This guide explains how to serve a local LLM using [LM Studio](https://lmstudio.ai/) and have OpenHands connect to it.

We recommend:

* **LM Studio** as the local model server, which handles metadata downloads automatically and offers a simple, user-friendly interface for configuration.
* **Qwen3-Coder-30B-A3B-Instruct** as the LLM for software development. This model is optimized for coding tasks and works excellently with agent-style workflows like OpenHands.

### Hardware Requirements

Running Qwen3-Coder-30B-A3B-Instruct requires:

* A recent GPU with at least 12GB of VRAM (tested on RTX 3060 with 12GB VRAM + 64GB RAM), or
* A Mac with Apple Silicon with at least 32GB of RAM

### 1. Install LM Studio

Download and install the LM Studio desktop app from [lmstudio.ai](https://lmstudio.ai/).

### 2. Download the Model

1. Make sure to set the User Interface Complexity Level to "Power User", by clicking on the appropriate label at the bottom of the window.
2. Click the "Discover" button (Magnifying Glass icon) on the left navigation bar to open the Models download page.

<img src="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/01_lm_studio_open_model_hub.png?fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=27c3b8464b07bb18aa7cfdde82829e9d" alt="image" data-og-width="1920" width="1920" data-og-height="1032" height="1032" data-path="openhands/usage/llms/screenshots/01_lm_studio_open_model_hub.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/01_lm_studio_open_model_hub.png?w=280&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=ca82265eb25d8343a93f3b5a07d5df31 280w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/01_lm_studio_open_model_hub.png?w=560&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=380695e8f6eb09c8c12a26cdd16faf5f 560w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/01_lm_studio_open_model_hub.png?w=840&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=96b8e971166c97543ccbe62abf4b0193 840w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/01_lm_studio_open_model_hub.png?w=1100&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=a2eea21cca9c2951c04f5bc9fbb0d4eb 1100w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/01_lm_studio_open_model_hub.png?w=1650&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=827925ab773e2ff41db846b2f09a514d 1650w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/01_lm_studio_open_model_hub.png?w=2500&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=a19215f7a73a4a573c89a21f86fe71de 2500w" />

3. Search for **"Qwen3-Coder-30B-A3B-Instruct"**, confirm you're downloading from the official Qwen publisher, then proceed to download.

<img src="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/02_lm_studio_download_devstral.png?fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=983b16c4094239746d9be254efbc12ef" alt="image" data-og-width="1920" width="1920" data-og-height="1032" height="1032" data-path="openhands/usage/llms/screenshots/02_lm_studio_download_devstral.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/02_lm_studio_download_devstral.png?w=280&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=315f4d1fca74cde6819d2a37f7bf5fb0 280w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/02_lm_studio_download_devstral.png?w=560&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=8cbee8309367e6d872359b22c1ca29a3 560w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/02_lm_studio_download_devstral.png?w=840&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=4bb5d1c33eb13c0d1454ca4e4a71c85a 840w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/02_lm_studio_download_devstral.png?w=1100&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=4037c16b9920256e66624f2ade660bd3 1100w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/02_lm_studio_download_devstral.png?w=1650&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=a550a98cad22a1b75f2953836ace02e4 1650w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/02_lm_studio_download_devstral.png?w=2500&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=610aee0303a1201b13af2c94c0f1ffea 2500w" />

4. Wait for the download to finish.

### 3. Load the Model

1. Click the "Developer" button (Console icon) on the left navigation bar to open the Developer Console.
2. Click the "Select a model to load" dropdown at the top of the application window.

<img src="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/03_lm_studio_open_load_model.png?fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=c1b1709cb9b82f35435ebf06faa8fb3c" alt="image" data-og-width="1920" width="1920" data-og-height="1032" height="1032" data-path="openhands/usage/llms/screenshots/03_lm_studio_open_load_model.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/03_lm_studio_open_load_model.png?w=280&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=3db609a83aa34bb9902ca6dd282ad576 280w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/03_lm_studio_open_load_model.png?w=560&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=4ed8bc0b304044a6e9480d5e203d0296 560w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/03_lm_studio_open_load_model.png?w=840&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=38a091ece0841924075a56c16210705a 840w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/03_lm_studio_open_load_model.png?w=1100&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=9258141621ed9ab454c875080626a8a2 1100w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/03_lm_studio_open_load_model.png?w=1650&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=f3dcf974e166cc29ca6230cfe2d99895 1650w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/03_lm_studio_open_load_model.png?w=2500&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=fe347aee1902210083abf3aa513efc54 2500w" />

3. Enable the "Manually choose model load parameters" switch.
4. Select **Qwen3-Coder-30B-A3B-Instruct** from the model list.

<img src="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/04_lm_studio_setup_devstral_part_1.png?fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=9135278733ad9082eb1c9ecebfc9a023" alt="image" data-og-width="1920" width="1920" data-og-height="1032" height="1032" data-path="openhands/usage/llms/screenshots/04_lm_studio_setup_devstral_part_1.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/04_lm_studio_setup_devstral_part_1.png?w=280&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=a848af0169c442c9aedb211a2fe3f25c 280w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/04_lm_studio_setup_devstral_part_1.png?w=560&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=c5a5a99778c06208e2ccb2f3e4fc4e39 560w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/04_lm_studio_setup_devstral_part_1.png?w=840&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=2c885b2aa5a6c514a2a30ee90db39616 840w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/04_lm_studio_setup_devstral_part_1.png?w=1100&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=cfb9b135fa0193347e38f8b467f1c487 1100w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/04_lm_studio_setup_devstral_part_1.png?w=1650&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=99153cfd92e97de6093be8e3c5b9e58d 1650w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/04_lm_studio_setup_devstral_part_1.png?w=2500&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=15604122752816d8b751fa442a52bb82 2500w" />

5. Enable the "Show advanced settings" switch at the bottom of the Model settings flyout to show all the available settings.
6. Set "Context Length" to at least 22000 (for lower VRAM systems) or 32768 (recommended for better performance) and enable Flash Attention.
7. Click "Load Model" to start loading the model.

<img src="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/05_lm_studio_setup_devstral_part_2.png?fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=c95786e14fe21c1dfc58a99b7e2eaab7" alt="image" data-og-width="1920" width="1920" data-og-height="1032" height="1032" data-path="openhands/usage/llms/screenshots/05_lm_studio_setup_devstral_part_2.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/05_lm_studio_setup_devstral_part_2.png?w=280&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=adef6122bfb07bff27f383d118d45d1a 280w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/05_lm_studio_setup_devstral_part_2.png?w=560&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=3b2b48a475e40cd54a16c4b0f19ead0c 560w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/05_lm_studio_setup_devstral_part_2.png?w=840&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=506f23971fe84d7073b7a8998e270fb1 840w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/05_lm_studio_setup_devstral_part_2.png?w=1100&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=fa0de32e07175fda751ac1352fba825e 1100w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/05_lm_studio_setup_devstral_part_2.png?w=1650&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=d49e3457879240d1a3d4698cefb2ecc5 1650w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/05_lm_studio_setup_devstral_part_2.png?w=2500&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=6ba950f965a85d8b2bd114f4ff271fc7 2500w" />

### 4. Start the LLM server

1. Enable the switch next to "Status" at the top-left of the Window.
2. Take note of the Model API Identifier shown on the sidebar on the right.

<img src="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/06_lm_studio_start_server.png?fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=e9b870812d2e6c119a40f5e3fdcbb753" alt="image" data-og-width="1920" width="1920" data-og-height="1032" height="1032" data-path="openhands/usage/llms/screenshots/06_lm_studio_start_server.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/06_lm_studio_start_server.png?w=280&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=bc97f71a56f0e5dce4c30adb0bfa4325 280w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/06_lm_studio_start_server.png?w=560&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=1ac68a95457e6ccbee418b0acbf36f2a 560w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/06_lm_studio_start_server.png?w=840&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=64cbbdc17152993cbae0deee51fdd205 840w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/06_lm_studio_start_server.png?w=1100&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=22f2a2546ddb9d70d29510a59116dc31 1100w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/06_lm_studio_start_server.png?w=1650&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=6ea8f1d5853629ea3e517a35b4756601 1650w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/06_lm_studio_start_server.png?w=2500&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=5fae95515aed07a590da0266fc7d7089 2500w" />

### 5. Start OpenHands

1. Check [the installation guide](/openhands/usage/run-openhands/local-setup) and ensure all prerequisites are met before running OpenHands, then run:

```bash  theme={null}
docker pull docker.openhands.dev/openhands/runtime:1.2-nikolaik

docker run -it --rm --pull=always \
    -e SANDBOX_RUNTIME_CONTAINER_IMAGE=docker.openhands.dev/openhands/runtime:1.2-nikolaik \
    -e LOG_ALL_EVENTS=true \
    -v /var/run/docker.sock:/var/run/docker.sock \
    -v ~/.openhands:/.openhands \
    -p 3000:3000 \
    --add-host host.docker.internal:host-gateway \
    --name openhands-app \
    docker.openhands.dev/openhands/openhands:1.2
```

2. Wait until the server is running (see log below):

```
Digest: sha256:e72f9baecb458aedb9afc2cd5bc935118d1868719e55d50da73190d3a85c674f
Status: Image is up to date for docker.openhands.dev/openhands/openhands:1.2
Starting OpenHands...
Running OpenHands as root
14:22:13 - openhands:INFO: server_config.py:50 - Using config class None
INFO:     Started server process [8]
INFO:     Waiting for application startup.
INFO:     Application startup complete.
INFO:     Uvicorn running on http://0.0.0.0:3000 (Press CTRL+C to quit)
```

3. Visit `http://localhost:3000` in your browser.

### 6. Configure OpenHands to use the LLM server

Once you open OpenHands in your browser, you'll need to configure it to use the local LLM server you just started.

When started for the first time, OpenHands will prompt you to set up the LLM provider.

1. Click "see advanced settings" to open the LLM Settings page.

<img src="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/07_openhands_open_advanced_settings.png?fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=e75089f5ae410c8b57187a450d097760" alt="image" data-og-width="731" width="731" data-og-height="354" height="354" data-path="openhands/usage/llms/screenshots/07_openhands_open_advanced_settings.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/07_openhands_open_advanced_settings.png?w=280&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=86fb7c39456f3d866d2a46b8694a3353 280w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/07_openhands_open_advanced_settings.png?w=560&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=d73d37b0c85a1e9695712541aa59e5bb 560w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/07_openhands_open_advanced_settings.png?w=840&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=8c4ed75ab022052737ac5a09b33c8069 840w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/07_openhands_open_advanced_settings.png?w=1100&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=9252a59be1dbed710d371fd403f7f160 1100w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/07_openhands_open_advanced_settings.png?w=1650&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=9e4a68dd6ca4316f6369ad84ee6e0c69 1650w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/07_openhands_open_advanced_settings.png?w=2500&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=0ce883da7df58dc59ee86cb9079fee5c 2500w" />

2. Enable the "Advanced" switch at the top of the page to show all the available settings.

3. Set the following values:
   * **Custom Model**: `openai/qwen/qwen3-coder-30b-a3b-instruct` (the Model API identifier from LM Studio, prefixed with "openai/")
   * **Base URL**: `http://host.docker.internal:1234/v1`
   * **API Key**: `local-llm`

4. Click "Save Settings" to save the configuration.

<img src="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/08_openhands_configure_local_llm_parameters.png?fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=d72545e9bf771f96627aebc5568b8c04" alt="image" data-og-width="1920" width="1920" data-og-height="1032" height="1032" data-path="openhands/usage/llms/screenshots/08_openhands_configure_local_llm_parameters.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/08_openhands_configure_local_llm_parameters.png?w=280&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=d3ac987da15c9dea153aa881a3d8d1e4 280w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/08_openhands_configure_local_llm_parameters.png?w=560&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=48767ec810d91c7207fe5971f46661a0 560w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/08_openhands_configure_local_llm_parameters.png?w=840&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=d1fe17c58c99b661b9460bc374c53032 840w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/08_openhands_configure_local_llm_parameters.png?w=1100&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=308c1402414f3b00b0c60ce0d24a1c18 1100w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/08_openhands_configure_local_llm_parameters.png?w=1650&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=ef65d07c6f10d9656ad4472872ab0a2d 1650w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/usage/llms/screenshots/08_openhands_configure_local_llm_parameters.png?w=2500&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=7866a67ccf9eb806e5ed5a7259017ad8 2500w" />

That's it! You can now start using OpenHands with the local LLM server.

If you encounter any issues, let us know on [Slack](https://openhands.dev/joinslack).

## Advanced: Alternative LLM Backends

This section describes how to run local LLMs with OpenHands using alternative backends like Ollama, SGLang, or vLLM — without relying on LM Studio.

### Create an OpenAI-Compatible Endpoint with Ollama

* Install Ollama following [the official documentation](https://ollama.com/download).
* Example launch command for Qwen3-Coder-30B-A3B-Instruct:

```bash  theme={null}
# ⚠️ WARNING: OpenHands requires a large context size to work properly.
# When using Ollama, set OLLAMA_CONTEXT_LENGTH to at least 22000.
# The default (4096) is way too small — not even the system prompt will fit, and the agent will not behave correctly.
OLLAMA_CONTEXT_LENGTH=32768 OLLAMA_HOST=0.0.0.0:11434 OLLAMA_KEEP_ALIVE=-1 nohup ollama serve &
ollama pull qwen3-coder:30b
```

### Create an OpenAI-Compatible Endpoint with vLLM or SGLang

First, download the model checkpoint:

```bash  theme={null}
huggingface-cli download Qwen/Qwen3-Coder-30B-A3B-Instruct --local-dir Qwen/Qwen3-Coder-30B-A3B-Instruct
```

#### Serving the model using SGLang

* Install SGLang following [the official documentation](https://docs.sglang.ai/start/install.html).
* Example launch command (with at least 2 GPUs):

```bash  theme={null}
SGLANG_ALLOW_OVERWRITE_LONGER_CONTEXT_LEN=1 python3 -m sglang.launch_server \
    --model Qwen/Qwen3-Coder-30B-A3B-Instruct \
    --served-model-name Qwen3-Coder-30B-A3B-Instruct \
    --port 8000 \
    --tp 2 --dp 1 \
    --host 0.0.0.0 \
    --api-key mykey --context-length 131072
```

#### Serving the model using vLLM

* Install vLLM following [the official documentation](https://docs.vllm.ai/en/latest/getting_started/installation.html).
* Example launch command (with at least 2 GPUs):

```bash  theme={null}
vllm serve Qwen/Qwen3-Coder-30B-A3B-Instruct \
    --host 0.0.0.0 --port 8000 \
    --api-key mykey \
    --tensor-parallel-size 2 \
    --served-model-name Qwen3-Coder-30B-A3B-Instruct \
    --enable-prefix-caching
```

If you are interested in further improved inference speed, you can also try Snowflake's version
of vLLM, [ArcticInference](https://www.snowflake.com/en/engineering-blog/fast-speculative-decoding-vllm-arctic/),
which can achieve up to 2x speedup in some cases.

1. Install the Arctic Inference library that automatically patches vLLM:

```bash  theme={null}
pip install git+https://github.com/snowflakedb/ArcticInference.git
```

2. Run the launch command with speculative decoding enabled:

```bash  theme={null}
vllm serve Qwen/Qwen3-Coder-30B-A3B-Instruct \
    --host 0.0.0.0 --port 8000 \
    --api-key mykey \
    --tensor-parallel-size 2 \
    --served-model-name Qwen3-Coder-30B-A3B-Instruct \
    --speculative-config '{"method": "suffix"}'
```

### Run OpenHands (Alternative Backends)

#### Using Docker

Run OpenHands using [the official docker run command](/openhands/usage/run-openhands/local-setup).

#### Using Development Mode

Use the instructions in [Development.md](https://github.com/OpenHands/OpenHands/blob/main/Development.md) to build OpenHands.

Start OpenHands using `make run`.

### Configure OpenHands (Alternative Backends)

Once OpenHands is running, open the Settings page in the UI and go to the `LLM` tab.

1. Click **"see advanced settings"** to access the full configuration panel.
2. Enable the **Advanced** toggle at the top of the page.
3. Set the following parameters, if you followed the examples above:
   * **Custom Model**: `openai/<served-model-name>`
     * For **Ollama**: `openai/qwen3-coder:30b`
     * For **SGLang/vLLM**: `openai/Qwen3-Coder-30B-A3B-Instruct`
   * **Base URL**: `http://host.docker.internal:<port>/v1`
     Use port `11434` for Ollama, or `8000` for SGLang and vLLM.
   * **API Key**:
     * For **Ollama**: any placeholder value (e.g. `dummy`, `local-llm`)
     * For **SGLang** or **vLLM**: use the same key provided when starting the server (e.g. `mykey`)


---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt