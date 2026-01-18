# Get Conversation



## OpenAPI

````yaml openapi/openapi.json get /api/conversations/{conversation_id}
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
  /api/conversations/{conversation_id}:
    get:
      summary: Get Conversation
      operationId: get_conversation_api_conversations__conversation_id__get
      parameters:
        - name: conversation_id
          in: path
          required: true
          schema:
            type: string
            title: Conversation Id
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                anyOf:
                  - $ref: '#/components/schemas/ConversationInfo'
                  - type: 'null'
                title: >-
                  Response Get Conversation Api Conversations  Conversation Id 
                  Get
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
    ConversationInfo:
      properties:
        conversation_id:
          type: string
          title: Conversation Id
        title:
          type: string
          title: Title
        last_updated_at:
          anyOf:
            - type: string
              format: date-time
            - type: 'null'
          title: Last Updated At
        status:
          $ref: '#/components/schemas/ConversationStatus'
          default: STOPPED
        runtime_status:
          anyOf:
            - $ref: '#/components/schemas/RuntimeStatus'
            - type: 'null'
        selected_repository:
          anyOf:
            - type: string
            - type: 'null'
          title: Selected Repository
        selected_branch:
          anyOf:
            - type: string
            - type: 'null'
          title: Selected Branch
        git_provider:
          anyOf:
            - $ref: '#/components/schemas/ProviderType'
            - type: 'null'
        trigger:
          anyOf:
            - $ref: '#/components/schemas/ConversationTrigger'
            - type: 'null'
        num_connections:
          type: integer
          title: Num Connections
          default: 0
        url:
          anyOf:
            - type: string
            - type: 'null'
          title: Url
        session_api_key:
          anyOf:
            - type: string
            - type: 'null'
          title: Session Api Key
        created_at:
          type: string
          format: date-time
          title: Created At
        pr_number:
          items:
            type: integer
          type: array
          title: Pr Number
      type: object
      required:
        - conversation_id
        - title
      title: ConversationInfo
    HTTPValidationError:
      properties:
        detail:
          items:
            $ref: '#/components/schemas/ValidationError'
          type: array
          title: Detail
      type: object
      title: HTTPValidationError
    ConversationStatus:
      type: string
      enum:
        - STARTING
        - RUNNING
        - STOPPED
      title: ConversationStatus
    RuntimeStatus:
      type: string
      enum:
        - STATUS$STOPPED
        - STATUS$BUILDING_RUNTIME
        - STATUS$STARTING_RUNTIME
        - STATUS$RUNTIME_STARTED
        - STATUS$SETTING_UP_WORKSPACE
        - STATUS$SETTING_UP_GIT_HOOKS
        - STATUS$READY
        - STATUS$ERROR
        - STATUS$ERROR_RUNTIME_DISCONNECTED
        - STATUS$ERROR_LLM_AUTHENTICATION
        - STATUS$ERROR_LLM_SERVICE_UNAVAILABLE
        - STATUS$ERROR_LLM_INTERNAL_SERVER_ERROR
        - STATUS$ERROR_LLM_OUT_OF_CREDITS
        - STATUS$ERROR_LLM_CONTENT_POLICY_VIOLATION
        - CHAT_INTERFACE$AGENT_RATE_LIMITED_STOPPED_MESSAGE
        - STATUS$GIT_PROVIDER_AUTHENTICATION_ERROR
        - STATUS$LLM_RETRY
        - STATUS$ERROR_MEMORY
      title: RuntimeStatus
    ProviderType:
      type: string
      enum:
        - github
        - gitlab
        - bitbucket
        - enterprise_sso
      title: ProviderType
    ConversationTrigger:
      type: string
      enum:
        - resolver
        - gui
        - suggested_task
        - openhands_api
        - slack
        - microagent_management
        - jira
        - jira_dc
        - linear
      title: ConversationTrigger
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
  securitySchemes:
    APIKeyHeader:
      type: apiKey
      in: header
      name: X-Session-API-Key

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt