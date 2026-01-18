# List Security Analyzers

> List supported security analyzers.



## OpenAPI

````yaml openapi/openapi.json get /api/options/security-analyzers
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
  /api/options/security-analyzers:
    get:
      summary: List Security Analyzers
      description: List supported security analyzers.
      operationId: get_security_analyzers_api_options_security_analyzers_get
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                items:
                  type: string
                type: array
                title: >-
                  Response Get Security Analyzers Api Options Security Analyzers
                  Get
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