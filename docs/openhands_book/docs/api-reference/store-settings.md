# Store Settings



## OpenAPI

````yaml openapi/openapi.json post /api/settings
openapi: 3.1.0
info:
  title: OpenHands
  description: 'OpenHands: Code Less, Make More'
  version: 0.53.0
servers:
  - url: https://app.all-hands.dev
    description: Production server
  - url: http://localhost:3000
    description: Local development server
security: []
paths:
  /api/settings:
    post:
      summary: Store Settings
      operationId: store_settings_api_settings_post
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/Settings'
        required: true
      responses:
        '200':
          description: Settings stored successfully
          content:
            application/json:
              schema:
                additionalProperties: true
                type: object
                title: Response 200 Store Settings Api Settings Post
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
        '500':
          description: Error storing settings
          content:
            application/json:
              schema:
                additionalProperties: true
                type: object
                title: Response 500 Store Settings Api Settings Post
      security:
        - APIKeyHeader: []
components:
  schemas:
    Settings:
      properties:
        language:
          anyOf:
            - type: string
            - type: 'null'
          title: Language
        agent:
          anyOf:
            - type: string
            - type: 'null'
          title: Agent
        max_iterations:
          anyOf:
            - type: integer
            - type: 'null'
          title: Max Iterations
        security_analyzer:
          anyOf:
            - type: string
            - type: 'null'
          title: Security Analyzer
        confirmation_mode:
          anyOf:
            - type: boolean
            - type: 'null'
          title: Confirmation Mode
        llm_model:
          anyOf:
            - type: string
            - type: 'null'
          title: Llm Model
        llm_api_key:
          anyOf:
            - type: string
              format: password
              writeOnly: true
            - type: 'null'
          title: Llm Api Key
        llm_base_url:
          anyOf:
            - type: string
            - type: 'null'
          title: Llm Base Url
        remote_runtime_resource_factor:
          anyOf:
            - type: integer
            - type: 'null'
          title: Remote Runtime Resource Factor
        secrets_store:
          $ref: '#/components/schemas/UserSecrets-Input'
        enable_default_condenser:
          type: boolean
          title: Enable Default Condenser
          default: true
        enable_sound_notifications:
          type: boolean
          title: Enable Sound Notifications
          default: false
        enable_proactive_conversation_starters:
          type: boolean
          title: Enable Proactive Conversation Starters
          default: true
        enable_solvability_analysis:
          type: boolean
          title: Enable Solvability Analysis
          default: true
        user_consents_to_analytics:
          anyOf:
            - type: boolean
            - type: 'null'
          title: User Consents To Analytics
        sandbox_base_container_image:
          anyOf:
            - type: string
            - type: 'null'
          title: Sandbox Base Container Image
        sandbox_runtime_container_image:
          anyOf:
            - type: string
            - type: 'null'
          title: Sandbox Runtime Container Image
        mcp_config:
          anyOf:
            - $ref: '#/components/schemas/MCPConfig'
            - type: 'null'
        search_api_key:
          anyOf:
            - type: string
              format: password
              writeOnly: true
            - type: 'null'
          title: Search Api Key
        sandbox_api_key:
          anyOf:
            - type: string
              format: password
              writeOnly: true
            - type: 'null'
          title: Sandbox Api Key
        max_budget_per_task:
          anyOf:
            - type: number
            - type: 'null'
          title: Max Budget Per Task
        email:
          anyOf:
            - type: string
            - type: 'null'
          title: Email
        email_verified:
          anyOf:
            - type: boolean
            - type: 'null'
          title: Email Verified
        git_user_name:
          anyOf:
            - type: string
            - type: 'null'
          title: Git User Name
        git_user_email:
          anyOf:
            - type: string
            - type: 'null'
          title: Git User Email
      type: object
      title: Settings
      description: Persisted settings for OpenHands sessions
    HTTPValidationError:
      properties:
        detail:
          items:
            $ref: '#/components/schemas/ValidationError'
          type: array
          title: Detail
      type: object
      title: HTTPValidationError
    UserSecrets-Input:
      properties:
        provider_tokens:
          additionalProperties:
            type: string
          type: object
          title: Provider Tokens
        custom_secrets:
          additionalProperties:
            type: string
          type: object
          title: Custom Secrets
      type: object
      title: UserSecrets
    MCPConfig:
      properties:
        sse_servers:
          items:
            $ref: '#/components/schemas/MCPSSEServerConfig'
          type: array
          title: Sse Servers
        stdio_servers:
          items:
            $ref: '#/components/schemas/MCPStdioServerConfig'
          type: array
          title: Stdio Servers
        shttp_servers:
          items:
            $ref: '#/components/schemas/MCPSHTTPServerConfig'
          type: array
          title: Shttp Servers
      additionalProperties: false
      type: object
      title: MCPConfig
      description: |-
        Configuration for MCP (Message Control Protocol) settings.

        Attributes:
            sse_servers: List of MCP SSE server configs
            stdio_servers: List of MCP stdio server configs. These servers will be added to the MCP Router running inside runtime container.
            shttp_servers: List of MCP HTTP server configs.
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
    MCPSSEServerConfig:
      properties:
        url:
          type: string
          title: Url
        api_key:
          anyOf:
            - type: string
            - type: 'null'
          title: Api Key
      type: object
      required:
        - url
      title: MCPSSEServerConfig
      description: |-
        Configuration for a single MCP server.

        Attributes:
            url: The server URL
            api_key: Optional API key for authentication
    MCPStdioServerConfig:
      properties:
        name:
          type: string
          title: Name
        command:
          type: string
          title: Command
        args:
          items:
            type: string
          type: array
          title: Args
        env:
          additionalProperties:
            type: string
          type: object
          title: Env
      type: object
      required:
        - name
        - command
      title: MCPStdioServerConfig
      description: |-
        Configuration for a MCP server that uses stdio.

        Attributes:
            name: The name of the server
            command: The command to run the server
            args: The arguments to pass to the server
            env: The environment variables to set for the server
    MCPSHTTPServerConfig:
      properties:
        url:
          type: string
          title: Url
        api_key:
          anyOf:
            - type: string
            - type: 'null'
          title: Api Key
      type: object
      required:
        - url
      title: MCPSHTTPServerConfig
  securitySchemes:
    APIKeyHeader:
      type: apiKey
      in: header
      name: X-Session-API-Key

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt