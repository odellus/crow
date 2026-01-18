# Generate Conversation Title

> Generate a title for the conversation using LLM.



## OpenAPI

````yaml openapi/agent-sdk.json post /api/conversations/{conversation_id}/generate_title
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/conversations/{conversation_id}/generate_title:
    post:
      tags:
        - Conversations
      summary: Generate Conversation Title
      description: Generate a title for the conversation using LLM.
      operationId: >-
        generate_conversation_title_api_conversations__conversation_id__generate_title_post
      parameters:
        - name: conversation_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
            title: Conversation Id
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/GenerateTitleRequest'
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/GenerateTitleResponse'
        '404':
          description: Item not found
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
components:
  schemas:
    GenerateTitleRequest:
      properties:
        max_length:
          type: integer
          maximum: 200
          minimum: 1
          title: Max Length
          description: Maximum length of the generated title
          default: 50
        llm:
          anyOf:
            - $ref: '#/components/schemas/LLM'
            - type: 'null'
          description: Optional LLM to use for title generation
      type: object
      title: GenerateTitleRequest
      description: Payload to generate a title for a conversation.
    GenerateTitleResponse:
      properties:
        title:
          type: string
          title: Title
          description: The generated title for the conversation
      type: object
      required:
        - title
      title: GenerateTitleResponse
      description: Response containing the generated conversation title.
    HTTPValidationError:
      properties:
        detail:
          items:
            $ref: '#/components/schemas/ValidationError'
          type: array
          title: Detail
      type: object
      title: HTTPValidationError
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

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt