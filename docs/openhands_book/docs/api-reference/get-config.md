# Get Config

> Get current config.



## OpenAPI

````yaml openapi/openapi.json get /api/options/config
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
  /api/options/config:
    get:
      summary: Get Config
      description: Get current config.
      operationId: get_config_api_options_config_get
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                additionalProperties: true
                type: object
                title: Response Get Config Api Options Config Get
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