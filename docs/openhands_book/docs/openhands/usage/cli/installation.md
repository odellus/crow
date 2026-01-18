# Installation

> Install the OpenHands CLI on your system

## Installation Methods

<Tabs>
  <Tab title="Using uv (recommended)">
    Requires Python 3.12+ and [uv](https://docs.astral.sh/uv/) installed.

    **Install OpenHands:**

    ```bash  theme={null}
    uv tool install openhands --python 3.12
    ```

    **Run OpenHands:**

    ```bash  theme={null}
    openhands
    ```

    **Upgrade OpenHands:**

    ```bash  theme={null}
    uv tool upgrade openhands --python 3.12
    ```
  </Tab>

  <Tab title="Executable Binary">
    Install the OpenHands CLI binary with the install script:

    ```bash  theme={null}
    curl -fsSL https://install.openhands.dev/install.sh | sh
    ```

    Then run:

    ```bash  theme={null}
    openhands
    ```

    <Note>
      Your system may require you to allow permissions to run the executable.

      <Accordion title="MacOS">
        When running the OpenHands CLI on Mac, you may get a warning that says "openhands can't be opened because Apple
        cannot check it for malicious software."

        1. Open `System Settings`.
        2. Go to `Privacy & Security`.
        3. Scroll down to `Security` and click `Allow Anyway`.
        4. Rerun the OpenHands CLI.

                <img src="https://mintcdn.com/allhandsai/iD2ZOFx_0cyRss23/openhands/static/img/cli-security-mac.png?fit=max&auto=format&n=iD2ZOFx_0cyRss23&q=85&s=18533d8e124036fbb1ceb2faffcbc3b7" alt="mac-security" data-og-width="702" width="702" data-og-height="236" height="236" data-path="openhands/static/img/cli-security-mac.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/iD2ZOFx_0cyRss23/openhands/static/img/cli-security-mac.png?w=280&fit=max&auto=format&n=iD2ZOFx_0cyRss23&q=85&s=31086474efeadd0b87707233ad15e892 280w, https://mintcdn.com/allhandsai/iD2ZOFx_0cyRss23/openhands/static/img/cli-security-mac.png?w=560&fit=max&auto=format&n=iD2ZOFx_0cyRss23&q=85&s=ef1adbbfae89001c3e8e96fe78a7c9c0 560w, https://mintcdn.com/allhandsai/iD2ZOFx_0cyRss23/openhands/static/img/cli-security-mac.png?w=840&fit=max&auto=format&n=iD2ZOFx_0cyRss23&q=85&s=c3c80769c17fa4e668b567150980cde5 840w, https://mintcdn.com/allhandsai/iD2ZOFx_0cyRss23/openhands/static/img/cli-security-mac.png?w=1100&fit=max&auto=format&n=iD2ZOFx_0cyRss23&q=85&s=43b10c209d1e4f9f5e45d4c74d68dad3 1100w, https://mintcdn.com/allhandsai/iD2ZOFx_0cyRss23/openhands/static/img/cli-security-mac.png?w=1650&fit=max&auto=format&n=iD2ZOFx_0cyRss23&q=85&s=d8f1563ecc6050056988c2064ad1233b 1650w, https://mintcdn.com/allhandsai/iD2ZOFx_0cyRss23/openhands/static/img/cli-security-mac.png?w=2500&fit=max&auto=format&n=iD2ZOFx_0cyRss23&q=85&s=05c4dcf971b8e7f2427adec42f1bd9a7 2500w" />
      </Accordion>
    </Note>
  </Tab>

  <Tab title="Using Docker">
    1. Set the following environment variable in your terminal:
       * `SANDBOX_VOLUMES` to specify the directory you want OpenHands to access ([See using SANDBOX\_VOLUMES for more info](/openhands/usage/runtimes/docker#using-sandbox_volumes))

    2. Ensure you have configured your settings before starting:
       * Set up `~/.openhands/settings.json` with your LLM configuration

    3. Run the following command:

    ```bash  theme={null}
    docker run -it \
        --pull=always \
        -e SANDBOX_RUNTIME_CONTAINER_IMAGE=docker.openhands.dev/openhands/runtime:1.2-nikolaik \
        -e SANDBOX_USER_ID=$(id -u) \
        -e SANDBOX_VOLUMES=$SANDBOX_VOLUMES \
        -v /var/run/docker.sock:/var/run/docker.sock \
        -v ~/.openhands:/root/.openhands \
        --add-host host.docker.internal:host-gateway \
        --name openhands-cli-$(date +%Y%m%d%H%M%S) \
        python:3.12-slim \
        bash -c "pip install uv && uv tool install openhands --python 3.12 && openhands"
    ```

    The `-e SANDBOX_USER_ID=$(id -u)` is passed to the Docker command to ensure the sandbox user matches the host user's
    permissions. This prevents the agent from creating root-owned files in the mounted workspace.
  </Tab>
</Tabs>

## First Run

The first time you run the CLI, it will take you through configuring the required LLM settings. These will be saved
for future sessions in `~/.openhands/settings.json`.

The conversation history will be saved in `~/.openhands/conversations`.

<Note>
  If you're upgrading from a CLI version before release 1.0.0, you'll need to redo your settings setup as the
  configuration format has changed.
</Note>

## Next Steps

* [Quick Start](/openhands/usage/cli/quick-start) - Learn the basics of using the CLI
* [MCP Servers](/openhands/usage/cli/mcp-servers) - Configure MCP servers


---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt