# Search Conversations

> Search / List conversations



## OpenAPI

````yaml openapi/agent-sdk.json get /api/conversations/search
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/conversations/search:
    get:
      tags:
        - Conversations
      summary: Search Conversations
      description: Search / List conversations
      operationId: search_conversations_api_conversations_search_get
      parameters:
        - name: page_id
          in: query
          required: false
          schema:
            anyOf:
              - type: string
              - type: 'null'
            title: Optional next_page_id from the previously returned page
        - name: limit
          in: query
          required: false
          schema:
            type: integer
            exclusiveMinimum: 0
            title: The max number of results in the page
            lte: 100
            default: 100
        - name: status
          in: query
          required: false
          schema:
            anyOf:
              - $ref: '#/components/schemas/ConversationExecutionStatus'
              - type: 'null'
            title: Optional filter by conversation execution status
        - name: sort_order
          in: query
          required: false
          schema:
            $ref: '#/components/schemas/ConversationSortOrder'
            title: Sort order for conversations
            default: CREATED_AT_DESC
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ConversationPage'
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
components:
  schemas:
    ConversationExecutionStatus:
      type: string
      enum:
        - idle
        - running
        - paused
        - waiting_for_confirmation
        - finished
        - error
        - stuck
        - deleting
      title: ConversationExecutionStatus
      description: Enum representing the current execution state of the conversation.
    ConversationSortOrder:
      type: string
      enum:
        - CREATED_AT
        - UPDATED_AT
        - CREATED_AT_DESC
        - UPDATED_AT_DESC
      title: ConversationSortOrder
      description: Enum for conversation sorting options.
    ConversationPage:
      properties:
        items:
          items:
            $ref: '#/components/schemas/ConversationInfo'
          type: array
          title: Items
        next_page_id:
          anyOf:
            - type: string
            - type: 'null'
          title: Next Page Id
      type: object
      required:
        - items
      title: ConversationPage
    HTTPValidationError:
      properties:
        detail:
          items:
            $ref: '#/components/schemas/ValidationError'
          type: array
          title: Detail
      type: object
      title: HTTPValidationError
    ConversationInfo:
      properties:
        id:
          type: string
          format: uuid
          title: Id
          description: Unique conversation ID
        agent:
          $ref: '#/components/schemas/AgentBase-Output'
          description: >-
            The agent running in the conversation. This is persisted to allow
            resuming conversations and check agent configuration to handle e.g.,
            tool changes, LLM changes, etc.
        workspace:
          $ref: '#/components/schemas/BaseWorkspace-Output'
          description: >-
            Workspace used by the agent to execute commands and read/write
            files. Not the process working directory.
        persistence_dir:
          anyOf:
            - type: string
            - type: 'null'
          title: Persistence Dir
          description: >-
            Directory for persisting conversation state and events. If None,
            conversation will not be persisted.
          default: workspace/conversations
        max_iterations:
          type: integer
          exclusiveMinimum: 0
          title: Max Iterations
          description: Maximum number of iterations the agent can perform in a single run.
          default: 500
        stuck_detection:
          type: boolean
          title: Stuck Detection
          description: Whether to enable stuck detection for the agent.
          default: true
        execution_status:
          $ref: '#/components/schemas/ConversationExecutionStatus'
          default: idle
        confirmation_policy:
          $ref: '#/components/schemas/ConfirmationPolicyBase-Output'
          default:
            kind: NeverConfirm
        security_analyzer:
          anyOf:
            - $ref: '#/components/schemas/SecurityAnalyzerBase-Output'
            - type: 'null'
          description: Optional security analyzer to evaluate action risks.
        activated_knowledge_skills:
          items:
            type: string
          type: array
          title: Activated Knowledge Skills
          description: List of activated knowledge skills name
        blocked_actions:
          additionalProperties:
            type: string
          type: object
          title: Blocked Actions
          description: Actions blocked by PreToolUse hooks, keyed by action ID
        blocked_messages:
          additionalProperties:
            type: string
          type: object
          title: Blocked Messages
          description: Messages blocked by UserPromptSubmit hooks, keyed by message ID
        stats:
          $ref: '#/components/schemas/ConversationStats-Output'
          description: Conversation statistics for tracking LLM metrics
        secret_registry:
          $ref: '#/components/schemas/SecretRegistry-Output'
          description: Registry for handling secrets and sensitive data
        title:
          anyOf:
            - type: string
            - type: 'null'
          title: Title
          description: User-defined title for the conversation
        metrics:
          anyOf:
            - $ref: '#/components/schemas/MetricsSnapshot'
            - type: 'null'
        created_at:
          type: string
          format: date-time
          title: Created At
        updated_at:
          type: string
          format: date-time
          title: Updated At
      type: object
      required:
        - id
        - agent
        - workspace
      title: ConversationInfo
      description: >-
        Information about a conversation running locally without a Runtime
        sandbox.
    ValidationError:
      properties:
        loc:
          items:
            anyOf:
              - type: string
              - type: integer
          type: array
          title: Location
        msg:
          type: string
          title: Message
        type:
          type: string
          title: Error Type
      type: object
      required:
        - loc
        - msg
        - type
      title: ValidationError
    AgentBase-Output:
      $ref: '#/components/schemas/Agent-Output'
    BaseWorkspace-Output:
      oneOf:
        - $ref: '#/components/schemas/LocalWorkspace-Output'
        - $ref: '#/components/schemas/RemoteWorkspace-Output'
      discriminator:
        propertyName: kind
        mapping:
          openhands__sdk__workspace__local__LocalWorkspace-Output__1: '#/components/schemas/LocalWorkspace-Output'
          openhands__sdk__workspace__remote__base__RemoteWorkspace-Output__1: '#/components/schemas/RemoteWorkspace-Output'
    ConfirmationPolicyBase-Output:
      oneOf:
        - $ref: '#/components/schemas/AlwaysConfirm-Output'
        - $ref: '#/components/schemas/ConfirmRisky-Output'
        - $ref: '#/components/schemas/NeverConfirm-Output'
      discriminator:
        propertyName: kind
        mapping:
          openhands__sdk__security__confirmation_policy__AlwaysConfirm-Output__1: '#/components/schemas/AlwaysConfirm-Output'
          openhands__sdk__security__confirmation_policy__ConfirmRisky-Output__1: '#/components/schemas/ConfirmRisky-Output'
          openhands__sdk__security__confirmation_policy__NeverConfirm-Output__1: '#/components/schemas/NeverConfirm-Output'
    SecurityAnalyzerBase-Output:
      $ref: '#/components/schemas/LLMSecurityAnalyzer-Output'
    ConversationStats-Output:
      additionalProperties: true
      type: object
    SecretRegistry-Output:
      properties:
        secret_sources:
          additionalProperties:
            $ref: '#/components/schemas/SecretSource-Output'
          type: object
          title: Secret Sources
      type: object
      title: SecretRegistry
      description: >-
        Manages secrets and injects them into bash commands when needed.


        The secret registry stores a mapping of secret keys to SecretSources

        that retrieve the actual secret values. When a bash command is about to
        be

        executed, it scans the command for any secret keys and injects the
        corresponding

        environment variables.


        Secret sources will redact / encrypt their sensitive values as
        appropriate when

        serializing, depending on the content of the context. If a context is
        present

        and contains a 'cipher' object, this is used for encryption. If it
        contains a

        boolean 'expose_secrets' flag set to True, secrets are dunped in plain
        text.

        Otherwise secrets are redacted.


        Additionally, it tracks the latest exported values to enable consistent
        masking

        even when callable secrets fail on subsequent calls.
    MetricsSnapshot:
      properties:
        model_name:
          type: string
          title: Model Name
          description: Name of the model
          default: default
        accumulated_cost:
          type: number
          minimum: 0
          title: Accumulated Cost
          description: Total accumulated cost, must be non-negative
          default: 0
        max_budget_per_task:
          anyOf:
            - type: number
            - type: 'null'
          title: Max Budget Per Task
          description: Maximum budget per task
        accumulated_token_usage:
          anyOf:
            - $ref: '#/components/schemas/TokenUsage'
            - type: 'null'
          description: Accumulated token usage across all calls
      type: object
      title: MetricsSnapshot
      description: |-
        A snapshot of metrics at a point in time.

        Does not include lists of individual costs, latencies, or token usages.
    Agent-Output:
      properties:
        llm:
          $ref: '#/components/schemas/LLM'
          description: LLM configuration for the agent.
          examples:
            - api_key: your_api_key_here
              base_url: https://llm-proxy.eval.all-hands.dev
              model: litellm_proxy/anthropic/claude-sonnet-4-5-20250929
        tools:
          items:
            $ref: '#/components/schemas/openhands__sdk__tool__spec__Tool'
          type: array
          title: Tools
          description: List of tools to initialize for the agent.
          examples:
            - name: TerminalTool
              params: {}
            - name: FileEditorTool
              params: {}
            - name: TaskTrackerTool
              params: {}
        mcp_config:
          additionalProperties: true
          type: object
          title: Mcp Config
          description: Optional MCP configuration dictionary to create MCP tools.
          examples:
            - mcpServers:
                fetch:
                  args:
                    - mcp-server-fetch
                  command: uvx
        filter_tools_regex:
          anyOf:
            - type: string
            - type: 'null'
          title: Filter Tools Regex
          description: >-
            Optional regex to filter the tools available to the agent by name.
            This is applied after any tools provided in `tools` and any MCP
            tools are added.
          examples:
            - ^(?!repomix)(.*)|^repomix.*pack_codebase.*$
        include_default_tools:
          items:
            type: string
          type: array
          title: Include Default Tools
          description: >-
            List of default tool class names to include. By default, the agent
            includes 'FinishTool' and 'ThinkTool'. Set to an empty list to
            disable all default tools, or provide a subset to include only
            specific ones. Example: include_default_tools=['FinishTool'] to only
            include FinishTool, or include_default_tools=[] to disable all
            default tools.
          examples:
            - - FinishTool
              - ThinkTool
            - - FinishTool
            - []
        agent_context:
          anyOf:
            - $ref: '#/components/schemas/AgentContext-Output'
            - type: 'null'
          description: Optional AgentContext to initialize the agent with specific context.
          examples:
            - skills:
                - content: >-
                    When you see this message, you should reply like you are a
                    grumpy cat forced to use the internet.
                  name: AGENTS.md
                  type: repo
                - content: >-
                    IMPORTANT! The user has said the magic word "flarglebargle".
                    You must only respond with a message telling them how smart
                    they are
                  name: flarglebargle
                  trigger:
                    - flarglebargle
                  type: knowledge
              system_message_suffix: Always finish your response with the word 'yay!'
              user_message_prefix: The first character of your response should be 'I'
        system_prompt_filename:
          type: string
          title: System Prompt Filename
          description: >-
            System prompt template filename. Can be either:

            - A relative filename (e.g., 'system_prompt.j2') loaded from the
            agent's prompts directory

            - An absolute path (e.g., '/path/to/custom_prompt.j2')
          default: system_prompt.j2
        security_policy_filename:
          type: string
          title: Security Policy Filename
          description: >-
            Security policy template filename. Can be either:

            - A relative filename (e.g., 'security_policy.j2') loaded from the
            agent's prompts directory

            - An absolute path (e.g., '/path/to/custom_security_policy.j2')
          default: security_policy.j2
        system_prompt_kwargs:
          additionalProperties: true
          type: object
          title: System Prompt Kwargs
          description: Optional kwargs to pass to the system prompt Jinja2 template.
          examples:
            - cli_mode: true
        condenser:
          anyOf:
            - $ref: '#/components/schemas/CondenserBase-Output'
            - type: 'null'
          description: Optional condenser to use for condensing conversation history.
          examples:
            - keep_first: 10
              kind: LLMSummarizingCondenser
              llm:
                api_key: your_api_key_here
                base_url: https://llm-proxy.eval.all-hands.dev
                model: litellm_proxy/anthropic/claude-sonnet-4-5-20250929
              max_size: 80
        kind:
          type: string
          const: Agent
          title: Kind
      type: object
      required:
        - llm
        - kind
      title: Agent
      description: >-
        Main agent implementation for OpenHands.


        The Agent class provides the core functionality for running AI agents
        that can

        interact with tools, process messages, and execute actions. It inherits
        from

        AgentBase and implements the agent execution logic.


        Example:
            >>> from openhands.sdk import LLM, Agent, Tool
            >>> llm = LLM(model="claude-sonnet-4-20250514", api_key=SecretStr("key"))
            >>> tools = [Tool(name="TerminalTool"), Tool(name="FileEditorTool")]
            >>> agent = Agent(llm=llm, tools=tools)
    LocalWorkspace-Output:
      properties:
        working_dir:
          type: string
          title: Working Dir
          description: >-
            The working directory for agent operations and tool execution.
            Accepts both string paths and Path objects. Path objects are
            automatically converted to strings.
        kind:
          type: string
          const: LocalWorkspace
          title: Kind
      type: object
      required:
        - working_dir
        - kind
      title: LocalWorkspace
      description: >-
        Local workspace implementation that operates on the host filesystem.


        LocalWorkspace provides direct access to the local filesystem and
        command execution

        environment. It's suitable for development and testing scenarios where
        the agent

        should operate directly on the host system.


        Example:
            >>> workspace = LocalWorkspace(working_dir="/path/to/project")
            >>> with workspace:
            ...     result = workspace.execute_command("ls -la")
            ...     content = workspace.read_file("README.md")
    RemoteWorkspace-Output:
      properties:
        working_dir:
          type: string
          title: Working Dir
          description: The working directory for agent operations and tool execution.
        host:
          type: string
          title: Host
          description: The remote host URL for the workspace.
        api_key:
          anyOf:
            - type: string
            - type: 'null'
          title: Api Key
          description: API key for authenticating with the remote host.
        kind:
          type: string
          const: RemoteWorkspace
          title: Kind
      type: object
      required:
        - working_dir
        - host
        - kind
      title: RemoteWorkspace
      description: >-
        Remote workspace implementation that connects to an OpenHands agent
        server.


        RemoteWorkspace provides access to a sandboxed environment running on a
        remote

        OpenHands agent server. This is the recommended approach for production
        deployments

        as it provides better isolation and security.


        Example:
            >>> workspace = RemoteWorkspace(
            ...     host="https://agent-server.example.com",
            ...     working_dir="/workspace"
            ... )
            >>> with workspace:
            ...     result = workspace.execute_command("ls -la")
            ...     content = workspace.read_file("README.md")
    AlwaysConfirm-Output:
      properties:
        kind:
          type: string
          const: AlwaysConfirm
          title: Kind
      type: object
      required:
        - kind
      title: AlwaysConfirm
    ConfirmRisky-Output:
      properties:
        threshold:
          $ref: '#/components/schemas/SecurityRisk'
          default: HIGH
        confirm_unknown:
          type: boolean
          title: Confirm Unknown
          default: true
        kind:
          type: string
          const: ConfirmRisky
          title: Kind
      type: object
      required:
        - kind
      title: ConfirmRisky
    NeverConfirm-Output:
      properties:
        kind:
          type: string
          const: NeverConfirm
          title: Kind
      type: object
      required:
        - kind
      title: NeverConfirm
    LLMSecurityAnalyzer-Output:
      properties:
        kind:
          type: string
          const: LLMSecurityAnalyzer
          title: Kind
      type: object
      required:
        - kind
      title: LLMSecurityAnalyzer
      description: >-
        LLM-based security analyzer.


        This analyzer respects the security_risk attribute that can be set by
        the LLM

        when generating actions, similar to OpenHands' LLMRiskAnalyzer.


        It provides a lightweight security analysis approach that leverages the
        LLM's

        understanding of action context and potential risks.
    SecretSource-Output:
      oneOf:
        - $ref: '#/components/schemas/LookupSecret-Output'
        - $ref: '#/components/schemas/StaticSecret-Output'
      discriminator:
        propertyName: kind
        mapping:
          openhands__sdk__secret__secrets__LookupSecret-Output__1: '#/components/schemas/LookupSecret-Output'
          openhands__sdk__secret__secrets__StaticSecret-Output__1: '#/components/schemas/StaticSecret-Output'
    TokenUsage:
      properties:
        model:
          type: string
          title: Model
          default: ''
        prompt_tokens:
          type: integer
          minimum: 0
          title: Prompt Tokens
          description: Prompt tokens must be non-negative
          default: 0
        completion_tokens:
          type: integer
          minimum: 0
          title: Completion Tokens
          description: Completion tokens must be non-negative
          default: 0
        cache_read_tokens:
          type: integer
          minimum: 0
          title: Cache Read Tokens
          description: Cache read tokens must be non-negative
          default: 0
        cache_write_tokens:
          type: integer
          minimum: 0
          title: Cache Write Tokens
          description: Cache write tokens must be non-negative
          default: 0
        reasoning_tokens:
          type: integer
          minimum: 0
          title: Reasoning Tokens
          description: Reasoning tokens must be non-negative
          default: 0
        context_window:
          type: integer
          minimum: 0
          title: Context Window
          description: Context window must be non-negative
          default: 0
        per_turn_token:
          type: integer
          minimum: 0
          title: Per Turn Token
          description: Per turn tokens must be non-negative
          default: 0
        response_id:
          type: string
          title: Response Id
          default: ''
      type: object
      title: TokenUsage
      description: Metric tracking detailed token usage per completion call.
    LLM:
      properties:
        model:
          type: string
          title: Model
          description: Model name.
          default: claude-sonnet-4-20250514
        api_key:
          anyOf:
            - type: string
            - type: string
              format: password
              writeOnly: true
            - type: 'null'
          title: Api Key
          description: API key.
        base_url:
          anyOf:
            - type: string
            - type: 'null'
          title: Base Url
          description: Custom base URL.
        api_version:
          anyOf:
            - type: string
            - type: 'null'
          title: Api Version
          description: API version (e.g., Azure).
        aws_access_key_id:
          anyOf:
            - type: string
            - type: string
              format: password
              writeOnly: true
            - type: 'null'
          title: Aws Access Key Id
        aws_secret_access_key:
          anyOf:
            - type: string
            - type: string
              format: password
              writeOnly: true
            - type: 'null'
          title: Aws Secret Access Key
        aws_region_name:
          anyOf:
            - type: string
            - type: 'null'
          title: Aws Region Name
        openrouter_site_url:
          type: string
          title: Openrouter Site Url
          default: https://docs.all-hands.dev/
        openrouter_app_name:
          type: string
          title: Openrouter App Name
          default: OpenHands
        num_retries:
          type: integer
          minimum: 0
          title: Num Retries
          default: 5
        retry_multiplier:
          type: number
          minimum: 0
          title: Retry Multiplier
          default: 8
        retry_min_wait:
          type: integer
          minimum: 0
          title: Retry Min Wait
          default: 8
        retry_max_wait:
          type: integer
          minimum: 0
          title: Retry Max Wait
          default: 64
        timeout:
          anyOf:
            - type: integer
              minimum: 0
            - type: 'null'
          title: Timeout
          description: >-
            HTTP timeout in seconds. Default is 300s (5 minutes). Set to None to
            disable timeout (not recommended for production).
          default: 300
        max_message_chars:
          type: integer
          minimum: 1
          title: Max Message Chars
          description: Approx max chars in each event/content sent to the LLM.
          default: 30000
        temperature:
          anyOf:
            - type: number
              minimum: 0
            - type: 'null'
          title: Temperature
          description: >-
            Sampling temperature for response generation. Defaults to 0 for most
            models and provider default for reasoning models.
        top_p:
          anyOf:
            - type: number
              maximum: 1
              minimum: 0
            - type: 'null'
          title: Top P
          default: 1
        top_k:
          anyOf:
            - type: number
              minimum: 0
            - type: 'null'
          title: Top K
        max_input_tokens:
          anyOf:
            - type: integer
              minimum: 1
            - type: 'null'
          title: Max Input Tokens
          description: >-
            The maximum number of input tokens. Note that this is currently
            unused, and the value at runtime is actually the total tokens in
            OpenAI (e.g. 128,000 tokens for GPT-4).
        max_output_tokens:
          anyOf:
            - type: integer
              minimum: 1
            - type: 'null'
          title: Max Output Tokens
          description: The maximum number of output tokens. This is sent to the LLM.
        model_canonical_name:
          anyOf:
            - type: string
            - type: 'null'
          title: Model Canonical Name
          description: >-
            Optional canonical model name for feature registry lookups. The
            OpenHands SDK maintains a model feature registry that maps model
            names to capabilities (e.g., vision support, prompt caching,
            responses API support). When using proxied or aliased model
            identifiers, set this field to the canonical model name (e.g.,
            'openai/gpt-4o') to ensure correct capability detection. If not
            provided, the 'model' field will be used for capability lookups.
        extra_headers:
          anyOf:
            - additionalProperties:
                type: string
              type: object
            - type: 'null'
          title: Extra Headers
          description: Optional HTTP headers to forward to LiteLLM requests.
        input_cost_per_token:
          anyOf:
            - type: number
              minimum: 0
            - type: 'null'
          title: Input Cost Per Token
          description: The cost per input token. This will available in logs for user.
        output_cost_per_token:
          anyOf:
            - type: number
              minimum: 0
            - type: 'null'
          title: Output Cost Per Token
          description: The cost per output token. This will available in logs for user.
        ollama_base_url:
          anyOf:
            - type: string
            - type: 'null'
          title: Ollama Base Url
        stream:
          type: boolean
          title: Stream
          description: >-
            Enable streaming responses from the LLM. When enabled, the provided
            `on_token` callback in .completions and .responses will be invoked
            for each chunk of tokens.
          default: false
        drop_params:
          type: boolean
          title: Drop Params
          default: true
        modify_params:
          type: boolean
          title: Modify Params
          description: >-
            Modify params allows litellm to do transformations like adding a
            default message, when a message is empty.
          default: true
        disable_vision:
          anyOf:
            - type: boolean
            - type: 'null'
          title: Disable Vision
          description: >-
            If model is vision capable, this option allows to disable image
            processing (useful for cost reduction).
        disable_stop_word:
          anyOf:
            - type: boolean
            - type: 'null'
          title: Disable Stop Word
          description: Disable using of stop word.
          default: false
        caching_prompt:
          type: boolean
          title: Caching Prompt
          description: Enable caching of prompts.
          default: true
        log_completions:
          type: boolean
          title: Log Completions
          description: Enable logging of completions.
          default: false
        log_completions_folder:
          type: string
          title: Log Completions Folder
          description: >-
            The folder to log LLM completions to. Required if log_completions is
            True.
          default: logs/completions
        custom_tokenizer:
          anyOf:
            - type: string
            - type: 'null'
          title: Custom Tokenizer
          description: A custom tokenizer to use for token counting.
        native_tool_calling:
          type: boolean
          title: Native Tool Calling
          description: Whether to use native tool calling.
          default: true
        force_string_serializer:
          anyOf:
            - type: boolean
            - type: 'null'
          title: Force String Serializer
          description: >-
            Force using string content serializer when sending to LLM API. If
            None (default), auto-detect based on model. Useful for providers
            that do not support list content, like HuggingFace and Groq.
        reasoning_effort:
          anyOf:
            - type: string
              enum:
                - low
                - medium
                - high
                - xhigh
                - none
            - type: 'null'
          title: Reasoning Effort
          description: >-
            The effort to put into reasoning. This is a string that can be one
            of 'low', 'medium', 'high', 'xhigh', or 'none'. Can apply to all
            reasoning models.
          default: high
        reasoning_summary:
          anyOf:
            - type: string
              enum:
                - auto
                - concise
                - detailed
            - type: 'null'
          title: Reasoning Summary
          description: >-
            The level of detail for reasoning summaries. This is a string that
            can be one of 'auto', 'concise', or 'detailed'. Requires verified
            OpenAI organization. Only sent when explicitly set.
        enable_encrypted_reasoning:
          type: boolean
          title: Enable Encrypted Reasoning
          description: >-
            If True, ask for ['reasoning.encrypted_content'] in Responses API
            include.
          default: true
        prompt_cache_retention:
          anyOf:
            - type: string
            - type: 'null'
          title: Prompt Cache Retention
          description: >-
            Retention policy for prompt cache. Only sent for GPT-5+ models;
            explicitly stripped for all other models.
          default: 24h
        extended_thinking_budget:
          anyOf:
            - type: integer
            - type: 'null'
          title: Extended Thinking Budget
          description: >-
            The budget tokens for extended thinking, supported by Anthropic
            models.
          default: 200000
        seed:
          anyOf:
            - type: integer
            - type: 'null'
          title: Seed
          description: The seed to use for random number generation.
        safety_settings:
          anyOf:
            - items:
                additionalProperties:
                  type: string
                type: object
              type: array
            - type: 'null'
          title: Safety Settings
          description: >-
            Safety settings for models that support them (like Mistral AI and
            Gemini)
        usage_id:
          type: string
          title: Usage Id
          description: >-
            Unique usage identifier for the LLM. Used for registry lookups,
            telemetry, and spend tracking.
          default: default
        litellm_extra_body:
          additionalProperties: true
          type: object
          title: Litellm Extra Body
          description: >-
            Additional key-value pairs to pass to litellm's extra_body
            parameter. This is useful for custom inference endpoints that need
            additional parameters for configuration, routing, or advanced
            features. NOTE: Not all LLM providers support extra_body parameters.
            Some providers (e.g., OpenAI) may reject requests with unrecognized
            options. This is commonly supported by: - LiteLLM proxy servers
            (routing metadata, tracing) - vLLM endpoints (return_token_ids,
            etc.) - Custom inference clusters Examples: - Proxy routing:
            {'trace_version': '1.0.0', 'tags': ['agent:my-agent']} - vLLM
            features: {'return_token_ids': True}
      type: object
      title: LLM
      description: >-
        Language model interface for OpenHands agents.


        The LLM class provides a unified interface for interacting with various

        language models through the litellm library. It handles model
        configuration,

        API authentication,

        retry logic, and tool calling capabilities.


        Example:
            >>> from openhands.sdk import LLM
            >>> from pydantic import SecretStr
            >>> llm = LLM(
            ...     model="claude-sonnet-4-20250514",
            ...     api_key=SecretStr("your-api-key"),
            ...     usage_id="my-agent"
            ... )
            >>> # Use with agent or conversation
    openhands__sdk__tool__spec__Tool:
      properties:
        name:
          type: string
          title: Name
          description: >-
            Name of the tool class, e.g., 'TerminalTool'. Import it from an
            `openhands.tools.<module>` subpackage.
          examples:
            - TerminalTool
            - FileEditorTool
            - TaskTrackerTool
        params:
          additionalProperties: true
          type: object
          title: Params
          description: >-
            Parameters for the tool's .create() method, e.g., {'working_dir':
            '/app'}
          examples:
            - working_dir: /workspace
      type: object
      required:
        - name
      title: Tool
      description: |-
        Defines a tool to be initialized for the agent.

        This is only used in agent-sdk for type schema for server use.
    AgentContext-Output:
      properties:
        skills:
          items:
            $ref: '#/components/schemas/Skill'
          type: array
          title: Skills
          description: List of available skills that can extend the user's input.
        system_message_suffix:
          anyOf:
            - type: string
            - type: 'null'
          title: System Message Suffix
          description: Optional suffix to append to the system prompt.
        user_message_suffix:
          anyOf:
            - type: string
            - type: 'null'
          title: User Message Suffix
          description: Optional suffix to append to the user's message.
        load_user_skills:
          type: boolean
          title: Load User Skills
          description: >-
            Whether to automatically load user skills from ~/.openhands/skills/
            and ~/.openhands/microagents/ (for backward compatibility). 
          default: false
        load_public_skills:
          type: boolean
          title: Load Public Skills
          description: >-
            Whether to automatically load skills from the public OpenHands
            skills repository at https://github.com/OpenHands/skills. This
            allows you to get the latest skills without SDK updates.
          default: false
        secrets:
          anyOf:
            - additionalProperties:
                anyOf:
                  - type: string
                  - $ref: '#/components/schemas/SecretSource-Output'
              type: object
            - type: 'null'
          title: Secrets
          description: >-
            Dictionary mapping secret keys to values or secret sources. Secrets
            are used for authentication and sensitive data handling. Values can
            be either strings or SecretSource instances (str | SecretSource).
      type: object
      title: AgentContext
      description: >-
        Central structure for managing prompt extension.


        AgentContext unifies all the contextual inputs that shape how the system

        extends and interprets user prompts. It combines both static environment

        details and dynamic, user-activated extensions from skills.


        Specifically, it provides:

        - **Repository context / Repo Skills**: Information about the active
        codebase,
          branches, and repo-specific instructions contributed by repo skills.
        - **Runtime context**: Current execution environment (hosts, working
          directory, secrets, date, etc.).
        - **Conversation instructions**: Optional task- or channel-specific
        rules
          that constrain or guide the agent’s behavior across the session.
        - **Knowledge Skills**: Extensible components that can be triggered by
        user input
          to inject knowledge or domain-specific guidance.

        Together, these elements make AgentContext the primary container
        responsible

        for assembling, formatting, and injecting all prompt-relevant context
        into

        LLM interactions.
    CondenserBase-Output:
      oneOf:
        - $ref: '#/components/schemas/LLMSummarizingCondenser-Output'
        - $ref: '#/components/schemas/NoOpCondenser-Output'
        - $ref: '#/components/schemas/PipelineCondenser-Output'
      discriminator:
        propertyName: kind
        mapping:
          openhands__sdk__context__condenser__llm_summarizing_condenser__LLMSummarizingCondenser-Output__1: '#/components/schemas/LLMSummarizingCondenser-Output'
          openhands__sdk__context__condenser__no_op_condenser__NoOpCondenser-Output__1: '#/components/schemas/NoOpCondenser-Output'
          openhands__sdk__context__condenser__pipeline_condenser__PipelineCondenser-Output__1: '#/components/schemas/PipelineCondenser-Output'
    SecurityRisk:
      type: string
      enum:
        - UNKNOWN
        - LOW
        - MEDIUM
        - HIGH
      title: SecurityRisk
      description: |-
        Security risk levels for actions.

        Based on OpenHands security risk levels but adapted for agent-sdk.
        Integer values allow for easy comparison and ordering.
    LookupSecret-Output:
      properties:
        description:
          anyOf:
            - type: string
            - type: 'null'
          title: Description
          description: Optional description for this secret
        url:
          type: string
          title: Url
        headers:
          additionalProperties:
            type: string
          type: object
          title: Headers
        kind:
          type: string
          const: LookupSecret
          title: Kind
      type: object
      required:
        - url
        - kind
      title: LookupSecret
      description: A secret looked up from some external url
    StaticSecret-Output:
      properties:
        description:
          anyOf:
            - type: string
            - type: 'null'
          title: Description
          description: Optional description for this secret
        value:
          anyOf:
            - type: string
              format: password
              writeOnly: true
            - type: 'null'
          title: Value
        kind:
          type: string
          const: StaticSecret
          title: Kind
      type: object
      required:
        - kind
      title: StaticSecret
      description: A secret stored locally
    Skill:
      properties:
        name:
          type: string
          title: Name
        content:
          type: string
          title: Content
        trigger:
          anyOf:
            - oneOf:
                - $ref: '#/components/schemas/KeywordTrigger'
                - $ref: '#/components/schemas/TaskTrigger'
              discriminator:
                propertyName: type
                mapping:
                  keyword: '#/components/schemas/KeywordTrigger'
                  task: '#/components/schemas/TaskTrigger'
            - type: 'null'
          title: Trigger
          description: >-
            Trigger determines when skill content is auto-injected. None = no
            auto-injection (for AgentSkills: agent reads on demand; for legacy:
            full content always in system prompt). KeywordTrigger = auto-inject
            when keywords appear in user messages. TaskTrigger = auto-inject for
            specific tasks, may require user input.
        source:
          anyOf:
            - type: string
            - type: 'null'
          title: Source
          description: >-
            The source path or identifier of the skill. When it is None, it is
            treated as a programmatically defined skill.
        mcp_tools:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Mcp Tools
          description: >-
            MCP tools configuration for the skill (repo skills only). It should
            conform to the MCPConfig schema:
            https://gofastmcp.com/clients/client#configuration-format
        inputs:
          items:
            $ref: '#/components/schemas/InputMetadata'
          type: array
          title: Inputs
          description: Input metadata for the skill (task skills only)
        is_agentskills_format:
          type: boolean
          title: Is Agentskills Format
          description: >-
            Whether this skill was loaded from a SKILL.md file following the
            AgentSkills standard. AgentSkills-format skills use progressive
            disclosure: always listed in <available_skills> with name,
            description, and location. If the skill also has triggers, content
            is auto-injected when triggered AND agent can read file anytime.
          default: false
        description:
          anyOf:
            - type: string
            - type: 'null'
          title: Description
          description: >-
            A brief description of what the skill does and when to use it.
            AgentSkills standard field (max 1024 characters).
        license:
          anyOf:
            - type: string
            - type: 'null'
          title: License
          description: >-
            The license under which the skill is distributed. AgentSkills
            standard field (e.g., 'Apache-2.0', 'MIT').
        compatibility:
          anyOf:
            - type: string
            - type: 'null'
          title: Compatibility
          description: >-
            Environment requirements or compatibility notes for the skill.
            AgentSkills standard field (e.g., 'Requires git and docker').
        metadata:
          anyOf:
            - additionalProperties:
                type: string
              type: object
            - type: 'null'
          title: Metadata
          description: >-
            Arbitrary key-value metadata for the skill. AgentSkills standard
            field for extensibility.
        allowed_tools:
          anyOf:
            - items:
                type: string
              type: array
            - type: 'null'
          title: Allowed Tools
          description: >-
            List of pre-approved tools for this skill. AgentSkills standard
            field (parsed from space-delimited string).
        resources:
          anyOf:
            - $ref: '#/components/schemas/SkillResources'
            - type: 'null'
          description: >-
            Resource directories for the skill (scripts/, references/, assets/).
            AgentSkills standard field. Only populated for SKILL.md directory
            format.
      type: object
      required:
        - name
        - content
      title: Skill
      description: >-
        A skill provides specialized knowledge or functionality.


        Skill behavior depends on format (is_agentskills_format) and trigger:


        AgentSkills format (SKILL.md files):

        - Always listed in <available_skills> with name, description, location

        - Agent reads full content on demand (progressive disclosure)

        - If has triggers: content is ALSO auto-injected when triggered


        Legacy OpenHands format:

        - With triggers: Listed in <available_skills>, content injected on
        trigger

        - Without triggers (None): Full content in <REPO_CONTEXT>, always active


        This model supports both OpenHands-specific fields and AgentSkills
        standard

        fields (https://agentskills.io/specification) for cross-platform
        compatibility.
    LLMSummarizingCondenser-Output:
      properties:
        llm:
          $ref: '#/components/schemas/LLM'
        max_size:
          type: integer
          exclusiveMinimum: 0
          title: Max Size
          default: 240
        max_tokens:
          anyOf:
            - type: integer
            - type: 'null'
          title: Max Tokens
        keep_first:
          type: integer
          minimum: 0
          title: Keep First
          default: 2
        kind:
          type: string
          const: LLMSummarizingCondenser
          title: Kind
      type: object
      required:
        - llm
        - kind
      title: LLMSummarizingCondenser
      description: >-
        LLM-based condenser that summarizes forgotten events.


        Uses an independent LLM (stored in the `llm` attribute) for generating
        summaries

        of forgotten events. The optional `agent_llm` parameter passed to
        condense() is

        the LLM used by the agent for token counting purposes, and you should
        not assume

        it is the same as the one defined in this condenser.
    NoOpCondenser-Output:
      properties:
        kind:
          type: string
          const: NoOpCondenser
          title: Kind
      type: object
      required:
        - kind
      title: NoOpCondenser
      description: |-
        Simple condenser that returns a view un-manipulated.

        Primarily intended for testing purposes.
    PipelineCondenser-Output:
      properties:
        condensers:
          items:
            $ref: '#/components/schemas/CondenserBase-Output'
          type: array
          title: Condensers
        kind:
          type: string
          const: PipelineCondenser
          title: Kind
      type: object
      required:
        - condensers
        - kind
      title: PipelineCondenser
      description: >-
        A condenser that applies a sequence of condensers in order.


        All condensers are defined primarily by their `condense` method, which
        takes a

        `View` and an optional `agent_llm` parameter, returning either a new
        `View` or a

        `Condensation` event. That means we can chain multiple condensers
        together by

        passing `View`s along and exiting early if any condenser returns a
        `Condensation`.


        For example:

            # Use the pipeline condenser to chain multiple other condensers together
            condenser = PipelineCondenser(condensers=[
                CondenserA(...),
                CondenserB(...),
                CondenserC(...),
            ])

            result = condenser.condense(view, agent_llm=agent_llm)

            # Doing the same thing without the pipeline condenser requires more boilerplate
            # for the monadic chaining
            other_result = view

            if isinstance(other_result, View):
                other_result = CondenserA(...).condense(other_result, agent_llm=agent_llm)

            if isinstance(other_result, View):
                other_result = CondenserB(...).condense(other_result, agent_llm=agent_llm)

            if isinstance(other_result, View):
                other_result = CondenserC(...).condense(other_result, agent_llm=agent_llm)

            assert result == other_result
    KeywordTrigger:
      properties:
        type:
          type: string
          const: keyword
          title: Type
          default: keyword
        keywords:
          items:
            type: string
          type: array
          title: Keywords
      type: object
      required:
        - keywords
      title: KeywordTrigger
      description: >-
        Trigger for keyword-based skills.


        These skills are activated when specific keywords appear in the user's
        query.
    TaskTrigger:
      properties:
        type:
          type: string
          const: task
          title: Type
          default: task
        triggers:
          items:
            type: string
          type: array
          title: Triggers
      type: object
      required:
        - triggers
      title: TaskTrigger
      description: >-
        Trigger for task-specific skills.


        These skills are activated for specific task types and can modify
        prompts.
    InputMetadata:
      properties:
        name:
          type: string
          title: Name
          description: Name of the input parameter
        description:
          type: string
          title: Description
          description: Description of the input parameter
      type: object
      required:
        - name
        - description
      title: InputMetadata
      description: Metadata for task skill inputs.
    SkillResources:
      properties:
        skill_root:
          type: string
          title: Skill Root
          description: Root directory of the skill (absolute path)
        scripts:
          items:
            type: string
          type: array
          title: Scripts
          description: List of script files in scripts/ directory (relative paths)
        references:
          items:
            type: string
          type: array
          title: References
          description: List of reference files in references/ directory (relative paths)
        assets:
          items:
            type: string
          type: array
          title: Assets
          description: List of asset files in assets/ directory (relative paths)
      type: object
      required:
        - skill_root
      title: SkillResources
      description: |-
        Resource directories for a skill (AgentSkills standard).

        Per the AgentSkills specification, skills can include:
        - scripts/: Executable scripts the agent can run
        - references/: Reference documentation and examples
        - assets/: Static assets (images, data files, etc.)

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt