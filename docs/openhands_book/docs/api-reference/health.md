# Health



## OpenAPI

````yaml openapi/openapi.json get /health
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
  /health:
    get:
      summary: Health
      operationId: health_health_get
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                type: string
                title: Response Health Health Get

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt