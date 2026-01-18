# New Conversation

> Initialize a new session or join an existing one.

After successful initialization, the client should connect to the WebSocket
using the returned conversation ID.



## OpenAPI

````yaml openapi/openapi.json post /api/conversations
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
  /api/conversations:
    post:
      summary: New Conversation
      description: >-
        Initialize a new session or join an existing one.


        After successful initialization, the client should connect to the
        WebSocket

        using the returned conversation ID.
      operationId: new_conversation_api_conversations_post
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/InitSessionRequest'
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ConversationResponse'
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
      security:
        - APIKeyHeader: []
components:
  schemas:
    InitSessionRequest:
      properties:
        repository:
          anyOf:
            - type: string
            - type: 'null'
          title: Repository
        git_provider:
          anyOf:
            - $ref: '#/components/schemas/ProviderType'
            - type: 'null'
        selected_branch:
          anyOf:
            - type: string
            - type: 'null'
          title: Selected Branch
        initial_user_msg:
          anyOf:
            - type: string
            - type: 'null'
          title: Initial User Msg
        image_urls:
          anyOf:
            - items:
                type: string
              type: array
            - type: 'null'
          title: Image Urls
        replay_json:
          anyOf:
            - type: string
            - type: 'null'
          title: Replay Json
        suggested_task:
          anyOf:
            - $ref: '#/components/schemas/SuggestedTask'
            - type: 'null'
        create_microagent:
          anyOf:
            - $ref: '#/components/schemas/CreateMicroagent'
            - type: 'null'
        conversation_instructions:
          anyOf:
            - type: string
            - type: 'null'
          title: Conversation Instructions
        mcp_config:
          anyOf:
            - $ref: '#/components/schemas/MCPConfig'
            - type: 'null'
        conversation_id:
          type: string
          title: Conversation Id
      additionalProperties: false
      type: object
      title: InitSessionRequest
    ConversationResponse:
      properties:
        status:
          type: string
          title: Status
        conversation_id:
          type: string
          title: Conversation Id
        message:
          anyOf:
            - type: string
            - type: 'null'
          title: Message
        conversation_status:
          anyOf:
            - $ref: '#/components/schemas/ConversationStatus'
            - type: 'null'
      type: object
      required:
        - status
        - conversation_id
      title: ConversationResponse
    HTTPValidationError:
      properties:
        detail:
          items:
            $ref: '#/components/schemas/ValidationError'
          type: array
          title: Detail
      type: object
      title: HTTPValidationError
    ProviderType:
      type: string
      enum:
        - github
        - gitlab
        - bitbucket
        - enterprise_sso
      title: ProviderType
    SuggestedTask:
      properties:
        git_provider:
          $ref: '#/components/schemas/ProviderType'
        task_type:
          $ref: '#/components/schemas/TaskType'
        repo:
          type: string
          title: Repo
        issue_number:
          type: integer
          title: Issue Number
        title:
          type: string
          title: Title
      type: object
      required:
        - git_provider
        - task_type
        - repo
        - issue_number
        - title
      title: SuggestedTask
    CreateMicroagent:
      properties:
        repo:
          type: string
          title: Repo
        git_provider:
          anyOf:
            - $ref: '#/components/schemas/ProviderType'
            - type: 'null'
        title:
          anyOf:
            - type: string
            - type: 'null'
          title: Title
      type: object
      required:
        - repo
      title: CreateMicroagent
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
    ConversationStatus:
      type: string
      enum:
        - STARTING
        - RUNNING
        - STOPPED
      title: ConversationStatus
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
    TaskType:
      type: string
      enum:
        - MERGE_CONFLICTS
        - FAILING_CHECKS
        - UNRESOLVED_COMMENTS
        - OPEN_ISSUE
        - OPEN_PR
        - CREATE_MICROAGENT
      title: TaskType
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