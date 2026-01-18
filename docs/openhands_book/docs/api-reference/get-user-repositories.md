# Get User Repositories



## OpenAPI

````yaml openapi/openapi.json get /api/user/repositories
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
  /api/user/repositories:
    get:
      summary: Get User Repositories
      operationId: get_user_repositories_api_user_repositories_get
      parameters:
        - name: sort
          in: query
          required: false
          schema:
            type: string
            default: pushed
            title: Sort
        - name: selected_provider
          in: query
          required: false
          schema:
            anyOf:
              - $ref: '#/components/schemas/ProviderType'
              - type: 'null'
            title: Selected Provider
        - name: page
          in: query
          required: false
          schema:
            anyOf:
              - type: integer
              - type: 'null'
            title: Page
        - name: per_page
          in: query
          required: false
          schema:
            anyOf:
              - type: integer
              - type: 'null'
            title: Per Page
        - name: installation_id
          in: query
          required: false
          schema:
            anyOf:
              - type: string
              - type: 'null'
            title: Installation Id
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/Repository'
                title: Response Get User Repositories Api User Repositories Get
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
    ProviderType:
      type: string
      enum:
        - github
        - gitlab
        - bitbucket
        - enterprise_sso
      title: ProviderType
    Repository:
      properties:
        id:
          type: string
          title: Id
        full_name:
          type: string
          title: Full Name
        git_provider:
          $ref: '#/components/schemas/ProviderType'
        is_public:
          type: boolean
          title: Is Public
        stargazers_count:
          anyOf:
            - type: integer
            - type: 'null'
          title: Stargazers Count
        link_header:
          anyOf:
            - type: string
            - type: 'null'
          title: Link Header
        pushed_at:
          anyOf:
            - type: string
            - type: 'null'
          title: Pushed At
        owner_type:
          anyOf:
            - $ref: '#/components/schemas/OwnerType'
            - type: 'null'
        main_branch:
          anyOf:
            - type: string
            - type: 'null'
          title: Main Branch
      type: object
      required:
        - id
        - full_name
        - git_provider
        - is_public
      title: Repository
    HTTPValidationError:
      properties:
        detail:
          items:
            $ref: '#/components/schemas/ValidationError'
          type: array
          title: Detail
      type: object
      title: HTTPValidationError
    OwnerType:
      type: string
      enum:
        - user
        - organization
      title: OwnerType
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