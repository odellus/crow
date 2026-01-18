# Store Provider Tokens



## OpenAPI

````yaml openapi/openapi.json post /api/add-git-providers
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
  /api/add-git-providers:
    post:
      summary: Store Provider Tokens
      operationId: store_provider_tokens_api_add_git_providers_post
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/POSTProviderModel'
        required: true
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema: {}
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
    POSTProviderModel:
      properties:
        mcp_config:
          anyOf:
            - $ref: '#/components/schemas/MCPConfig'
            - type: 'null'
        provider_tokens:
          additionalProperties:
            $ref: '#/components/schemas/ProviderToken'
          propertyNames:
            $ref: '#/components/schemas/ProviderType'
          type: object
          title: Provider Tokens
          default: {}
      type: object
      title: POSTProviderModel
      description: Settings for POST requests
    HTTPValidationError:
      properties:
        detail:
          items:
            $ref: '#/components/schemas/ValidationError'
          type: array
          title: Detail
      type: object
      title: HTTPValidationError
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
    ProviderToken:
      properties:
        token:
          anyOf:
            - type: string
              format: password
              writeOnly: true
            - type: 'null'
          title: Token
        user_id:
          anyOf:
            - type: string
            - type: 'null'
          title: User Id
        host:
          anyOf:
            - type: string
            - type: 'null'
          title: Host
      type: object
      title: ProviderToken
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