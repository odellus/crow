# List Supported Models

> List model identifiers available on this server based on configured providers.



## OpenAPI

````yaml openapi/openapi.json get /api/options/models
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
  /api/options/models:
    get:
      summary: List Supported Models
      description: >-
        List model identifiers available on this server based on configured
        providers.
      operationId: get_litellm_models_api_options_models_get
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                items:
                  type: string
                type: array
                title: Response Get Litellm Models Api Options Models Get
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