# Get User



## OpenAPI

````yaml openapi/openapi.json get /api/user/info
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
  /api/user/info:
    get:
      summary: Get User
      operationId: get_user_api_user_info_get
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/User'
      security:
        - APIKeyHeader: []
components:
  schemas:
    User:
      properties:
        id:
          type: string
          title: Id
        login:
          type: string
          title: Login
        avatar_url:
          type: string
          title: Avatar Url
        company:
          anyOf:
            - type: string
            - type: 'null'
          title: Company
        name:
          anyOf:
            - type: string
            - type: 'null'
          title: Name
        email:
          anyOf:
            - type: string
            - type: 'null'
          title: Email
      type: object
      required:
        - id
        - login
        - avatar_url
      title: User
  securitySchemes:
    APIKeyHeader:
      type: apiKey
      in: header
      name: X-Session-API-Key

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt