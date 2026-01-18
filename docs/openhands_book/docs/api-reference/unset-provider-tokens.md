# Unset Provider Tokens



## OpenAPI

````yaml openapi/openapi.json post /api/unset-provider-tokens
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
  /api/unset-provider-tokens:
    post:
      summary: Unset Provider Tokens
      operationId: unset_provider_tokens_api_unset_provider_tokens_post
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                additionalProperties:
                  type: string
                type: object
                title: Response Unset Provider Tokens Api Unset Provider Tokens Post
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