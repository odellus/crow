# Reset Settings

> Resets user settings. (Deprecated)



## OpenAPI

````yaml openapi/openapi.json post /api/reset-settings
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
  /api/reset-settings:
    post:
      summary: Reset Settings
      description: Resets user settings. (Deprecated)
      operationId: reset_settings_api_reset_settings_post
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema: {}
        '410':
          description: Reset settings functionality has been removed
          content:
            application/json:
              schema:
                additionalProperties: true
                type: object
                title: Response 410 Reset Settings Api Reset Settings Post
      security:
        - APIKeyHeader: []
components:
  securitySchemes:
    APIKeyHeader:
      type: apiKey
      in: header
      name: X-Session-API-Key

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt