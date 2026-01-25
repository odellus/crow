# Agent Client Protocol (ACP) Book

A comprehensive local collection of the Agent Client Protocol documentation, fetched from [agentclientprotocol.com](https://agentclientprotocol.com).

## About ACP

The Agent Client Protocol standardizes communication between code editors/IDEs and coding agents, suitable for both local and remote scenarios. ACP solves integration overhead by providing a standardized protocol similar to how the Language Server Protocol (LSP) standardized language server integration.

## Documentation Structure

This repository contains the complete ACP documentation organized into the following sections:

### 📚 Overview
- [Introduction](./overview/introduction.md) - Get started with ACP
- [Architecture](./overview/architecture.md) - Overview of ACP architecture
- [Agents](./overview/agents.md) - Agents implementing ACP
- [Clients](./overview/clients.md) - Clients implementing ACP

### 🔧 Protocol Reference
- [Overview](./protocol/overview.md) - How ACP works
- [Initialization](./protocol/initialization.md) - How all ACP connections begin
- [Prompt Turn](./protocol/prompt-turn.md) - Understanding the core conversation flow
- [Session Setup](./protocol/session-setup.md) - Creating and loading sessions
- [Session Modes](./protocol/session-modes.md) - Switch between different agent operating modes
- [Content](./protocol/content.md) - Understanding content blocks in ACP
- [Tool Calls](./protocol/tool-calls.md) - How agents report tool call execution
- [Terminals](./protocol/terminals.md) - Executing and managing terminal commands
- [File System](./protocol/file-system.md) - Client filesystem access methods
- [Agent Plan](./protocol/agent-plan.md) - How agents communicate their execution plans
- [Slash Commands](./protocol/slash-commands.md) - Advertise available slash commands to clients
- [Transports](./protocol/transports.md) - Mechanisms for agents and clients to communicate
- [Extensibility](./protocol/extensibility.md) - Adding custom data and capabilities
- [Schema](./protocol/schema.md) - Schema definitions for ACP

### 📖 Libraries
- [Python](./libraries/python.md) - Python library for ACP
- [TypeScript](./libraries/typescript.md) - TypeScript library for ACP
- [Rust](./libraries/rust.md) - Rust library for ACP
- [Kotlin](./libraries/kotlin.md) - Kotlin library for ACP
- [Community](./libraries/community.md) - Community managed libraries

### 🤝 Community
- [Contributing](./community/contributing.md) - How to participate in ACP development
- [Code of Conduct](./community/code-of-conduct.md) - Community guidelines
- [Communication](./community/communication.md) - Communication methods for contributors
- [Governance](./community/governance.md) - How the ACP project is governed
- [Working & Interest Groups](./community/working-interest-groups.md) - Collaborative groups within ACP

### 📋 Requests for Dialog (RFDs)
RFDs are the process for introducing changes to the protocol:

- [About RFDs](./rfds/about.md) - Our process for introducing changes
- [Introduce RFD Process](./rfds/introduce-rfd-process.md) - Initial RFD process proposal
- [MCP-over-ACP](./rfds/mcp-over-acp.md) - MCP Transport via ACP Channels
- [Agent Telemetry Export](./rfds/agent-telemetry-export.md) - Telemetry export mechanism
- [Authentication Methods](./rfds/auth-methods.md) - Authentication standards
- [Meta Field Propagation](./rfds/meta-propagation.md) - Metadata propagation conventions
- [Agent Extensions via Proxies](./rfds/proxy-chains.md) - ACP Proxy chains
- [Request Cancellation](./rfds/request-cancellation.md) - Cancellation mechanism
- [Rust SDK based on SACP](./rfds/rust-sdk-v1.md) - Rust SDK proposal
- [Session Config Options](./rfds/session-config-options.md) - Configuration options
- [Session Fork](./rfds/session-fork.md) - Forking existing sessions
- [Session Info Update](./rfds/session-info-update.md) - Session information updates
- [Session List](./rfds/session-list.md) - Listing sessions
- [Session Resume](./rfds/session-resume.md) - Resuming existing sessions
- [Session Usage](./rfds/session-usage.md) - Session usage and context status
- [ACP Agent Registry](./rfds/acp-agent-registry.md) - Agent registry proposal

### 🎨 Brand
- [Brand Assets](./brand.md) - Assets for the ACP brand

### 📢 Updates
- [Updates & Announcements](./updates.md) - Latest news about ACP

## Source

All documentation is sourced from the official [Agent Client Protocol website](https://agentclientprotocol.com). For the most up-to-date information, please visit the official site.

## File Index

See [llms.txt](./llms.txt) for a complete index of all documentation files.
