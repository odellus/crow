# Zed IDE

> Configure OpenHands with the Zed code editor through the Agent Client Protocol

[Zed](https://zed.dev/) is a high-performance code editor with built-in support for the Agent Client Protocol.

<video controls className="w-full aspect-video" src="https://github.com/user-attachments/assets/5b921c1d-7543-4d59-b7dd-a6cb51321fd5" />

## Prerequisites

Before configuring Zed, ensure you have:

1. **OpenHands CLI installed** - See [Installation](/openhands/usage/cli/installation)
2. **LLM settings configured** - Run `openhands` and use `/settings`
3. **Zed editor** - Download from [zed.dev](https://zed.dev/)

## Configuration

### Step 1: Open Agent Settings

1. Open Zed
2. Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux) to open the command palette
3. Search for `agent: open settings`

<img src="https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-settings.png?fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=b4f3b30bd4db616477bbea6d683c7612" alt="Zed Command Palette" data-og-width="1832" width="1832" data-og-height="1048" height="1048" data-path="openhands/static/img/acp-zed-settings.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-settings.png?w=280&fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=cedf1f760484fd9d164ce5884b978bf2 280w, https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-settings.png?w=560&fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=bd7cd43fa868c5694f01838731f32d3e 560w, https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-settings.png?w=840&fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=69ece997759c323bd672509ec471a18e 840w, https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-settings.png?w=1100&fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=b287d2c65aafda220d238835c81fd512 1100w, https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-settings.png?w=1650&fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=0346a210ef6d2f6ee7117efcf6eb28d3 1650w, https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-settings.png?w=2500&fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=f4107ee00913b98dfcffeacdae3a0de6 2500w" />

### Step 2: Add OpenHands as an Agent

1. On the right side, click `+ Add Agent`
2. Select `Add Custom Agent`

<img src="https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-add-agent.png?fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=222ab514849bab7fe288006fb69727db" alt="Zed Add Custom Agent" data-og-width="1414" width="1414" data-og-height="678" height="678" data-path="openhands/static/img/acp-zed-add-agent.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-add-agent.png?w=280&fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=10b2b4154abc397793bc7b558b689b68 280w, https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-add-agent.png?w=560&fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=58fe00d88aeaf6dd3f0f44b239d4d5a5 560w, https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-add-agent.png?w=840&fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=d7c43fb756a5a409e74bae5c9ba8365e 840w, https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-add-agent.png?w=1100&fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=4f6f7247432b319a1e57b56f786cff3e 1100w, https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-add-agent.png?w=1650&fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=3ce261d111b4ba83b47822e2d76b1e71 1650w, https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-add-agent.png?w=2500&fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=39dcc45d2cc22672f40dc875da3dc1c8 2500w" />

### Step 3: Configure the Agent

Add the following configuration to the `agent_servers` field:

```json  theme={null}
{
  "agent_servers": {
    "OpenHands": {
      "command": "uvx",
      "args": [
        "openhands",
        "acp"
      ],
      "env": {}
    }
  }
}
```

### Step 4: Save and Use

1. Save the settings file
2. You can now use OpenHands within Zed!

<img src="https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-use-openhands.png?fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=59f60bacc1d0130ec2f2299e111a08f8" alt="Zed Use OpenHands Agent" data-og-width="1286" width="1286" data-og-height="902" height="902" data-path="openhands/static/img/acp-zed-use-openhands.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-use-openhands.png?w=280&fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=0904b3b3f0f607b7a593f98ac71a92f0 280w, https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-use-openhands.png?w=560&fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=fc82fcd2c07473c815253946c4af3af8 560w, https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-use-openhands.png?w=840&fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=a1f56807302259c5983935e3d39a31d5 840w, https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-use-openhands.png?w=1100&fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=22d3876478c41c3a3372f29799ec8aec 1100w, https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-use-openhands.png?w=1650&fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=6efdb1beef64294108b312f31e9d2886 1650w, https://mintcdn.com/allhandsai/6GIpIENz-WIWOCZh/openhands/static/img/acp-zed-use-openhands.png?w=2500&fit=max&auto=format&n=6GIpIENz-WIWOCZh&q=85&s=46fe5cb7759958d4db021945fcc91597 2500w" />

## Advanced Configuration

### LLM-Approve Mode

For automatic LLM-based approval of actions:

```json  theme={null}
{
  "agent_servers": {
    "OpenHands (LLM Approve)": {
      "command": "uvx",
      "args": [
        "openhands",
        "acp",
        "--llm-approve"
      ],
      "env": {}
    }
  }
}
```

### Resume a Specific Conversation

To resume a previous conversation:

```json  theme={null}
{
  "agent_servers": {
    "OpenHands (Resume)": {
      "command": "uvx",
      "args": [
        "openhands",
        "acp",
        "--resume",
        "abc123def456"
      ],
      "env": {}
    }
  }
}
```

Replace `abc123def456` with your actual conversation ID. Find conversation IDs by running `openhands --resume` in your terminal.

### Resume Latest Conversation

```json  theme={null}
{
  "agent_servers": {
    "OpenHands (Latest)": {
      "command": "uvx",
      "args": [
        "openhands",
        "acp",
        "--resume",
        "--last"
      ],
      "env": {}
    }
  }
}
```

### Multiple Configurations

You can add multiple OpenHands configurations for different use cases:

```json  theme={null}
{
  "agent_servers": {
    "OpenHands": {
      "command": "uvx",
      "args": ["openhands", "acp"],
      "env": {}
    },
    "OpenHands (Auto-Approve)": {
      "command": "uvx",
      "args": ["openhands", "acp", "--always-approve"],
      "env": {}
    },
    "OpenHands (Resume Latest)": {
      "command": "uvx",
      "args": ["openhands", "acp", "--resume", "--last"],
      "env": {}
    }
  }
}
```

## Troubleshooting

### Accessing Debug Logs

If you encounter issues:

1. Open the command palette (`Cmd+Shift+P` or `Ctrl+Shift+P`)
2. Type and select `acp debug log`
3. Review the logs for errors or warnings
4. Restart the conversation to reload connections after configuration changes

### Common Issues

**"openhands" command not found**

Ensure OpenHands is installed and in your PATH:

```bash  theme={null}
which openhands
# Should return a path like /Users/you/.local/bin/openhands
```

If using `uvx`, ensure uv is installed:

```bash  theme={null}
uv --version
```

**Agent doesn't start**

1. Check that your LLM settings are configured: run `openhands` and verify `/settings`
2. Verify the configuration JSON syntax is valid
3. Check the ACP debug logs for detailed errors

**Conversation doesn't persist**

Conversations are stored in `~/.openhands/conversations`. Ensure this directory is writable.

<Note>
  After making configuration changes, restart the conversation in Zed to apply them.
</Note>

## See Also

* [IDE Integration Overview](/openhands/usage/cli/ide/overview) - ACP concepts and other IDEs
* [Zed Documentation](https://zed.dev/docs) - Official Zed documentation
* [Resume Conversations](/openhands/usage/cli/resume) - Find conversation IDs


---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt