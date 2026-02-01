# Agent Client Protocol

Ideally this is where the agent client protocol for the crow agent is implemented but right now it all lives in `src/crow/agent`, which is a mistake.


So I'm going to go through and put some README.md files in the appropriate directories telling the agent what I want to put in here in the hopes that all the `grep` and `file` will lead it to read this and know that this directory is for the agent client protocol and the agent client protocol only.

```python
from crow.agent import crow_agent 
from agent_client_protocol import (
# A bunch of ACP stuff to implement the protocol around the agent, give it the MCP tools defined in the ACP client, which is like half the point of agent client protocol
```
