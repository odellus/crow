# Get Suggested Tasks

> Get suggested tasks for the authenticated user across their most recently pushed repositories.



## OpenAPI

````yaml openapi/openapi.json get /api/user/suggested-tasks
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
  /api/user/suggested-tasks:
    get:
      summary: Get Suggested Tasks
      description: >-
        Get suggested tasks for the authenticated user across their most
        recently pushed repositories.
      operationId: get_suggested_tasks_api_user_suggested_tasks_get
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                items:
                  $ref: '#/components/schemas/SuggestedTask'
                type: array
                title: Response Get Suggested Tasks Api User Suggested Tasks Get
      security:
        - APIKeyHeader: []
components:
  schemas:
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
    ProviderType:
      type: string
      enum:
        - github
        - gitlab
        - bitbucket
        - enterprise_sso
      title: ProviderType
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
  securitySchemes:
    APIKeyHeader:
      type: apiKey
      in: header
      name: X-Session-API-Key

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt