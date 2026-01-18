# List Agents

> List available agent types supported by this server.



## OpenAPI

````yaml openapi/openapi.json get /api/options/agents
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
  /api/options/agents:
    get:
      summary: List Agents
      description: List available agent types supported by this server.
      operationId: get_agents_api_options_agents_get
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                items:
                  type: string
                type: array
                title: Response Get Agents Api Options Agents Get
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