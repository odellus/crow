# Search Conversation Events

> Search / List local events



## OpenAPI

````yaml openapi/agent-sdk.json get /api/conversations/{conversation_id}/events/search
openapi: 3.1.0
info:
  title: OpenHands Agent Server
  description: OpenHands Agent Server - REST/WebSocket interface for OpenHands AI Agent
  version: 0.1.0
servers: []
security: []
paths:
  /api/conversations/{conversation_id}/events/search:
    get:
      tags:
        - Events
      summary: Search Conversation Events
      description: Search / List local events
      operationId: >-
        search_conversation_events_api_conversations__conversation_id__events_search_get
      parameters:
        - name: conversation_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
            title: Conversation Id
        - name: page_id
          in: query
          required: false
          schema:
            anyOf:
              - type: string
              - type: 'null'
            title: Optional next_page_id from the previously returned page
        - name: limit
          in: query
          required: false
          schema:
            type: integer
            exclusiveMinimum: 0
            title: The max number of results in the page
            lte: 100
            default: 100
        - name: kind
          in: query
          required: false
          schema:
            anyOf:
              - type: string
              - type: 'null'
            title: >-
              Optional filter by event kind/type (e.g., ActionEvent,
              MessageEvent)
        - name: source
          in: query
          required: false
          schema:
            anyOf:
              - type: string
              - type: 'null'
            title: Optional filter by event source (e.g., agent, user, environment)
        - name: body
          in: query
          required: false
          schema:
            anyOf:
              - type: string
              - type: 'null'
            title: Optional filter by message content (case-insensitive)
        - name: sort_order
          in: query
          required: false
          schema:
            $ref: '#/components/schemas/EventSortOrder'
            title: Sort order for events
            default: TIMESTAMP
        - name: timestamp__gte
          in: query
          required: false
          schema:
            anyOf:
              - type: string
                format: date-time
              - type: 'null'
            title: 'Filter: event timestamp >= this datetime'
        - name: timestamp__lt
          in: query
          required: false
          schema:
            anyOf:
              - type: string
                format: date-time
              - type: 'null'
            title: 'Filter: event timestamp < this datetime'
      responses:
        '200':
          description: Successful Response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/EventPage'
        '404':
          description: Conversation not found
        '422':
          description: Validation Error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HTTPValidationError'
components:
  schemas:
    EventSortOrder:
      type: string
      enum:
        - TIMESTAMP
        - TIMESTAMP_DESC
      title: EventSortOrder
      description: Enum for event sorting options.
    EventPage:
      properties:
        items:
          items:
            $ref: '#/components/schemas/Event'
          type: array
          title: Items
        next_page_id:
          anyOf:
            - type: string
            - type: 'null'
          title: Next Page Id
      type: object
      required:
        - items
      title: EventPage
    HTTPValidationError:
      properties:
        detail:
          items:
            $ref: '#/components/schemas/ValidationError'
          type: array
          title: Detail
      type: object
      title: HTTPValidationError
    Event:
      oneOf:
        - $ref: '#/components/schemas/Condensation'
        - $ref: '#/components/schemas/CondensationRequest'
        - $ref: '#/components/schemas/CondensationSummaryEvent'
        - $ref: '#/components/schemas/ConversationErrorEvent'
        - $ref: '#/components/schemas/ConversationStateUpdateEvent'
        - $ref: '#/components/schemas/LLMCompletionLogEvent'
        - $ref: '#/components/schemas/ActionEvent'
        - $ref: '#/components/schemas/MessageEvent'
        - $ref: '#/components/schemas/AgentErrorEvent'
        - $ref: '#/components/schemas/ObservationEvent'
        - $ref: '#/components/schemas/UserRejectObservation'
        - $ref: '#/components/schemas/SystemPromptEvent'
        - $ref: '#/components/schemas/TokenEvent'
        - $ref: '#/components/schemas/PauseEvent'
      discriminator:
        propertyName: kind
        mapping:
          openhands__sdk__event__condenser__Condensation-Output__1: '#/components/schemas/Condensation'
          openhands__sdk__event__condenser__CondensationRequest-Output__1: '#/components/schemas/CondensationRequest'
          openhands__sdk__event__condenser__CondensationSummaryEvent-Output__1: '#/components/schemas/CondensationSummaryEvent'
          openhands__sdk__event__conversation_error__ConversationErrorEvent-Output__1: '#/components/schemas/ConversationErrorEvent'
          openhands__sdk__event__conversation_state__ConversationStateUpdateEvent-Output__1: '#/components/schemas/ConversationStateUpdateEvent'
          openhands__sdk__event__llm_completion_log__LLMCompletionLogEvent-Output__1: '#/components/schemas/LLMCompletionLogEvent'
          openhands__sdk__event__llm_convertible__action__ActionEvent-Output__1: '#/components/schemas/ActionEvent'
          openhands__sdk__event__llm_convertible__message__MessageEvent-Output__1: '#/components/schemas/MessageEvent'
          openhands__sdk__event__llm_convertible__observation__AgentErrorEvent-Output__1: '#/components/schemas/AgentErrorEvent'
          openhands__sdk__event__llm_convertible__observation__ObservationEvent-Output__1: '#/components/schemas/ObservationEvent'
          openhands__sdk__event__llm_convertible__observation__UserRejectObservation-Output__1: '#/components/schemas/UserRejectObservation'
          openhands__sdk__event__llm_convertible__system__SystemPromptEvent-Output__1: '#/components/schemas/SystemPromptEvent'
          openhands__sdk__event__token__TokenEvent-Output__1: '#/components/schemas/TokenEvent'
          openhands__sdk__event__user_action__PauseEvent-Output__1: '#/components/schemas/PauseEvent'
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
    Condensation:
      properties:
        id:
          type: string
          title: Id
          description: Unique event id (ULID/UUID)
        timestamp:
          type: string
          title: Timestamp
          description: Event timestamp
        source:
          type: string
          enum:
            - agent
            - user
            - environment
          title: Source
          default: environment
        forgotten_event_ids:
          items:
            type: string
          type: array
          title: Forgotten Event Ids
          description: >-
            The IDs of the events that are being forgotten (removed from the
            `View` given to the LLM).
        summary:
          anyOf:
            - type: string
            - type: 'null'
          title: Summary
          description: An optional summary of the events being forgotten.
        summary_offset:
          anyOf:
            - type: integer
              minimum: 0
            - type: 'null'
          title: Summary Offset
          description: >-
            An optional offset to the start of the resulting view indicating
            where the summary should be inserted.
        llm_response_id:
          type: string
          title: Llm Response Id
          description: >-
            Completion or Response ID of the LLM response that generated this
            event
        kind:
          type: string
          const: Condensation
          title: Kind
      additionalProperties: false
      type: object
      required:
        - llm_response_id
        - kind
      title: Condensation
      description: >-
        This action indicates a condensation of the conversation history is
        happening.
    CondensationRequest:
      properties:
        id:
          type: string
          title: Id
          description: Unique event id (ULID/UUID)
        timestamp:
          type: string
          title: Timestamp
          description: Event timestamp
        source:
          type: string
          enum:
            - agent
            - user
            - environment
          title: Source
          default: environment
        kind:
          type: string
          const: CondensationRequest
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: CondensationRequest
      description: >-
        This action is used to request a condensation of the conversation
        history.


        Attributes:
            action (str): The action type, namely ActionType.CONDENSATION_REQUEST.
    CondensationSummaryEvent:
      properties:
        id:
          type: string
          title: Id
          description: Unique event id (ULID/UUID)
        timestamp:
          type: string
          title: Timestamp
          description: Event timestamp
        source:
          type: string
          enum:
            - agent
            - user
            - environment
          title: Source
          default: environment
        summary:
          type: string
          title: Summary
        kind:
          type: string
          const: CondensationSummaryEvent
          title: Kind
      additionalProperties: false
      type: object
      required:
        - summary
        - kind
      title: CondensationSummaryEvent
      description: This event represents a summary generated by a condenser.
    ConversationErrorEvent:
      properties:
        id:
          type: string
          title: Id
          description: Unique event id (ULID/UUID)
        timestamp:
          type: string
          title: Timestamp
          description: Event timestamp
        source:
          type: string
          enum:
            - agent
            - user
            - environment
          title: Source
          description: The source of this event
        code:
          type: string
          title: Code
          description: Code for the error - typically a type
        detail:
          type: string
          title: Detail
          description: Details about the error
        kind:
          type: string
          const: ConversationErrorEvent
          title: Kind
      additionalProperties: false
      type: object
      required:
        - source
        - code
        - detail
        - kind
      title: ConversationErrorEvent
      description: >-
        Conversation-level failure that is NOT sent back to the LLM.


        This event is emitted by the conversation runtime when an unexpected

        exception bubbles up and prevents the run loop from continuing. It is

        intended for client applications (e.g., UIs) to present a top-level
        error

        state, and for orchestration to react. It is not an observation and it
        is

        not LLM-convertible.


        Differences from AgentErrorEvent:

        - Not tied to any tool_name/tool_call_id (AgentErrorEvent is a tool
          observation).
        - Typically source='environment' and the run loop moves to an ERROR
        state,
          while AgentErrorEvent has source='agent' and the conversation can
          continue.
    ConversationStateUpdateEvent:
      properties:
        id:
          type: string
          title: Id
          description: Unique event id (ULID/UUID)
        timestamp:
          type: string
          title: Timestamp
          description: Event timestamp
        source:
          type: string
          enum:
            - agent
            - user
            - environment
          title: Source
          default: environment
        key:
          type: string
          title: Key
          description: Unique key for this state update event
        value:
          title: Value
          description: Serialized conversation state updates
        kind:
          type: string
          const: ConversationStateUpdateEvent
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: ConversationStateUpdateEvent
      description: >-
        Event that contains conversation state updates.


        This event is sent via websocket whenever the conversation state
        changes,

        allowing remote clients to stay in sync without making REST API calls.


        All fields are serialized versions of the corresponding
        ConversationState fields

        to ensure compatibility with websocket transmission.
    LLMCompletionLogEvent:
      properties:
        id:
          type: string
          title: Id
          description: Unique event id (ULID/UUID)
        timestamp:
          type: string
          title: Timestamp
          description: Event timestamp
        source:
          type: string
          enum:
            - agent
            - user
            - environment
          title: Source
          default: environment
        filename:
          type: string
          title: Filename
          description: The intended filename for this log (relative to log directory)
        log_data:
          type: string
          title: Log Data
          description: The JSON-encoded log data to be written to the file
        model_name:
          type: string
          title: Model Name
          description: The model name for context
          default: unknown
        usage_id:
          type: string
          title: Usage Id
          description: The LLM usage_id that produced this log
          default: default
        kind:
          type: string
          const: LLMCompletionLogEvent
          title: Kind
      additionalProperties: false
      type: object
      required:
        - filename
        - log_data
        - kind
      title: LLMCompletionLogEvent
      description: >-
        Event containing LLM completion log data.


        When an LLM is configured with log_completions=True in a remote
        conversation,

        this event streams the completion log data back to the client through
        WebSocket

        instead of writing it to a file inside the Docker container.
    ActionEvent:
      properties:
        id:
          type: string
          title: Id
          description: Unique event id (ULID/UUID)
        timestamp:
          type: string
          title: Timestamp
          description: Event timestamp
        source:
          type: string
          enum:
            - agent
            - user
            - environment
          title: Source
          default: agent
        thought:
          items:
            $ref: '#/components/schemas/TextContent'
          type: array
          title: Thought
          description: The thought process of the agent before taking this action
        reasoning_content:
          anyOf:
            - type: string
            - type: 'null'
          title: Reasoning Content
          description: Intermediate reasoning/thinking content from reasoning models
        thinking_blocks:
          items:
            anyOf:
              - $ref: '#/components/schemas/ThinkingBlock'
              - $ref: '#/components/schemas/RedactedThinkingBlock'
          type: array
          title: Thinking Blocks
          description: Anthropic thinking blocks from the LLM response
        responses_reasoning_item:
          anyOf:
            - $ref: '#/components/schemas/ReasoningItemModel'
            - type: 'null'
          description: OpenAI Responses reasoning item from model output
        action:
          anyOf:
            - $ref: '#/components/schemas/Action'
            - type: 'null'
          description: Single tool call returned by LLM (None when non-executable)
        tool_name:
          type: string
          title: Tool Name
          description: The name of the tool being called
        tool_call_id:
          type: string
          title: Tool Call Id
          description: The unique id returned by LLM API for this tool call
        tool_call:
          $ref: '#/components/schemas/MessageToolCall'
          description: >-
            The tool call received from the LLM response. We keep a copy of it
            so it is easier to construct it into LLM messageThis could be
            different from `action`: e.g., `tool_call` may contain
            `security_risk` field predicted by LLM when LLM risk analyzer is
            enabled, while `action` does not.
        llm_response_id:
          type: string
          title: Llm Response Id
          description: >-
            Completion or Response ID of the LLM response that generated this
            eventE.g., Can be used to group related actions from same LLM
            response. This helps in tracking and managing results of parallel
            function calling from the same LLM response.
        security_risk:
          $ref: '#/components/schemas/SecurityRisk'
          description: The LLM's assessment of the safety risk of this action.
          default: UNKNOWN
        summary:
          anyOf:
            - type: string
            - type: 'null'
          title: Summary
          description: >-
            A concise summary (approximately 10 words) of what this action does,
            provided by the LLM for explainability and debugging. Examples of
            good summaries: 'editing configuration file for deployment settings'
            | 'searching codebase for authentication function definitions' |
            'installing required dependencies from package manifest' | 'running
            tests to verify bug fix' | 'viewing directory structure to locate
            source files'
        kind:
          type: string
          const: ActionEvent
          title: Kind
      additionalProperties: false
      type: object
      required:
        - thought
        - tool_name
        - tool_call_id
        - tool_call
        - llm_response_id
        - kind
      title: ActionEvent
    MessageEvent:
      properties:
        id:
          type: string
          title: Id
          description: Unique event id (ULID/UUID)
        timestamp:
          type: string
          title: Timestamp
          description: Event timestamp
        source:
          type: string
          enum:
            - agent
            - user
            - environment
          title: Source
        llm_message:
          $ref: '#/components/schemas/Message'
          description: The exact LLM message for this message event
        llm_response_id:
          anyOf:
            - type: string
            - type: 'null'
          title: Llm Response Id
          description: >-
            Completion or Response ID of the LLM response that generated this
            eventIf the source != 'agent', this field is None
        activated_skills:
          items:
            type: string
          type: array
          title: Activated Skills
          description: List of activated skill name
        extended_content:
          items:
            $ref: '#/components/schemas/TextContent'
          type: array
          title: Extended Content
          description: List of content added by agent context
        sender:
          anyOf:
            - type: string
            - type: 'null'
          title: Sender
          description: >-
            Optional identifier of the sender. Can be used to track message
            origin in multi-agent scenarios.
        kind:
          type: string
          const: MessageEvent
          title: Kind
      additionalProperties: false
      type: object
      required:
        - source
        - llm_message
        - kind
      title: MessageEvent
      description: >-
        Message from either agent or user.


        This is originally the "MessageAction", but it suppose not to be tool
        call.
    AgentErrorEvent:
      properties:
        id:
          type: string
          title: Id
          description: Unique event id (ULID/UUID)
        timestamp:
          type: string
          title: Timestamp
          description: Event timestamp
        source:
          type: string
          enum:
            - agent
            - user
            - environment
          title: Source
          default: agent
        tool_name:
          type: string
          title: Tool Name
          description: The tool name that this observation is responding to
        tool_call_id:
          type: string
          title: Tool Call Id
          description: The tool call id that this observation is responding to
        error:
          type: string
          title: Error
          description: The error message from the scaffold
        kind:
          type: string
          const: AgentErrorEvent
          title: Kind
      additionalProperties: false
      type: object
      required:
        - tool_name
        - tool_call_id
        - error
        - kind
      title: AgentErrorEvent
      description: >-
        Error triggered by the agent.


        Note: This event should not contain model "thought" or
        "reasoning_content". It

        represents an error produced by the agent/scaffold, not model output.
    ObservationEvent:
      properties:
        id:
          type: string
          title: Id
          description: Unique event id (ULID/UUID)
        timestamp:
          type: string
          title: Timestamp
          description: Event timestamp
        source:
          type: string
          enum:
            - agent
            - user
            - environment
          title: Source
          default: environment
        tool_name:
          type: string
          title: Tool Name
          description: The tool name that this observation is responding to
        tool_call_id:
          type: string
          title: Tool Call Id
          description: The tool call id that this observation is responding to
        observation:
          $ref: '#/components/schemas/Observation'
          description: The observation (tool call) sent to LLM
        action_id:
          type: string
          title: Action Id
          description: The action id that this observation is responding to
        kind:
          type: string
          const: ObservationEvent
          title: Kind
      additionalProperties: false
      type: object
      required:
        - tool_name
        - tool_call_id
        - observation
        - action_id
        - kind
      title: ObservationEvent
    UserRejectObservation:
      properties:
        id:
          type: string
          title: Id
          description: Unique event id (ULID/UUID)
        timestamp:
          type: string
          title: Timestamp
          description: Event timestamp
        source:
          type: string
          enum:
            - agent
            - user
            - environment
          title: Source
          default: environment
        tool_name:
          type: string
          title: Tool Name
          description: The tool name that this observation is responding to
        tool_call_id:
          type: string
          title: Tool Call Id
          description: The tool call id that this observation is responding to
        rejection_reason:
          type: string
          title: Rejection Reason
          description: Reason for rejecting the action
          default: User rejected the action
        action_id:
          type: string
          title: Action Id
          description: The action id that this observation is responding to
        kind:
          type: string
          const: UserRejectObservation
          title: Kind
      additionalProperties: false
      type: object
      required:
        - tool_name
        - tool_call_id
        - action_id
        - kind
      title: UserRejectObservation
      description: Observation when user rejects an action in confirmation mode.
    SystemPromptEvent:
      properties:
        id:
          type: string
          title: Id
          description: Unique event id (ULID/UUID)
        timestamp:
          type: string
          title: Timestamp
          description: Event timestamp
        source:
          type: string
          enum:
            - agent
            - user
            - environment
          title: Source
          default: agent
        system_prompt:
          $ref: '#/components/schemas/TextContent'
          description: The system prompt text
        tools:
          items:
            $ref: '#/components/schemas/ToolDefinition'
          type: array
          title: Tools
          description: List of tools as ToolDefinition objects
        kind:
          type: string
          const: SystemPromptEvent
          title: Kind
      additionalProperties: false
      type: object
      required:
        - system_prompt
        - tools
        - kind
      title: SystemPromptEvent
      description: System prompt added by the agent.
    TokenEvent:
      properties:
        id:
          type: string
          title: Id
          description: Unique event id (ULID/UUID)
        timestamp:
          type: string
          title: Timestamp
          description: Event timestamp
        source:
          type: string
          enum:
            - agent
            - user
            - environment
          title: Source
        prompt_token_ids:
          items:
            type: integer
          type: array
          title: Prompt Token Ids
          description: The exact prompt token IDs for this message event
        response_token_ids:
          items:
            type: integer
          type: array
          title: Response Token Ids
          description: The exact response token IDs for this message event
        kind:
          type: string
          const: TokenEvent
          title: Kind
      additionalProperties: false
      type: object
      required:
        - source
        - prompt_token_ids
        - response_token_ids
        - kind
      title: TokenEvent
      description: Event from VLLM representing token IDs used in LLM interaction.
    PauseEvent:
      properties:
        id:
          type: string
          title: Id
          description: Unique event id (ULID/UUID)
        timestamp:
          type: string
          title: Timestamp
          description: Event timestamp
        source:
          type: string
          enum:
            - agent
            - user
            - environment
          title: Source
          default: user
        kind:
          type: string
          const: PauseEvent
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: PauseEvent
      description: Event indicating that the agent execution was paused by user request.
    TextContent:
      properties:
        cache_prompt:
          type: boolean
          title: Cache Prompt
          default: false
        type:
          type: string
          const: text
          title: Type
          default: text
        text:
          type: string
          title: Text
        enable_truncation:
          type: boolean
          title: Enable Truncation
          default: true
      additionalProperties: false
      type: object
      required:
        - text
      title: TextContent
    ThinkingBlock:
      properties:
        type:
          type: string
          const: thinking
          title: Type
          default: thinking
        thinking:
          type: string
          title: Thinking
          description: The thinking content
        signature:
          anyOf:
            - type: string
            - type: 'null'
          title: Signature
          description: Cryptographic signature for the thinking block
      type: object
      required:
        - thinking
      title: ThinkingBlock
      description: |-
        Anthropic thinking block for extended thinking feature.

        This represents the raw thinking blocks returned by Anthropic models
        when extended thinking is enabled. These blocks must be preserved
        and passed back to the API for tool use scenarios.
    RedactedThinkingBlock:
      properties:
        type:
          type: string
          const: redacted_thinking
          title: Type
          default: redacted_thinking
        data:
          type: string
          title: Data
          description: The redacted thinking content
      type: object
      required:
        - data
      title: RedactedThinkingBlock
      description: >-
        Redacted thinking block for previous responses without extended
        thinking.


        This is used as a placeholder for assistant messages that were generated

        before extended thinking was enabled.
    ReasoningItemModel:
      properties:
        id:
          anyOf:
            - type: string
            - type: 'null'
          title: Id
        summary:
          items:
            type: string
          type: array
          title: Summary
        content:
          anyOf:
            - items:
                type: string
              type: array
            - type: 'null'
          title: Content
        encrypted_content:
          anyOf:
            - type: string
            - type: 'null'
          title: Encrypted Content
        status:
          anyOf:
            - type: string
            - type: 'null'
          title: Status
      type: object
      title: ReasoningItemModel
      description: |-
        OpenAI Responses reasoning item (non-stream, subset we consume).

        Do not log or render encrypted_content.
    Action:
      oneOf:
        - $ref: '#/components/schemas/MCPToolAction'
        - $ref: '#/components/schemas/FinishAction'
        - $ref: '#/components/schemas/ThinkAction'
        - $ref: '#/components/schemas/BrowserAction'
        - $ref: '#/components/schemas/BrowserClickAction'
        - $ref: '#/components/schemas/BrowserCloseTabAction'
        - $ref: '#/components/schemas/BrowserGetContentAction'
        - $ref: '#/components/schemas/BrowserGetStateAction'
        - $ref: '#/components/schemas/BrowserGetStorageAction'
        - $ref: '#/components/schemas/BrowserGoBackAction'
        - $ref: '#/components/schemas/BrowserListTabsAction'
        - $ref: '#/components/schemas/BrowserNavigateAction'
        - $ref: '#/components/schemas/BrowserScrollAction'
        - $ref: '#/components/schemas/BrowserSetStorageAction'
        - $ref: '#/components/schemas/BrowserSwitchTabAction'
        - $ref: '#/components/schemas/BrowserTypeAction'
        - $ref: '#/components/schemas/FileEditorAction'
        - $ref: '#/components/schemas/EditAction'
        - $ref: '#/components/schemas/ListDirectoryAction'
        - $ref: '#/components/schemas/ReadFileAction'
        - $ref: '#/components/schemas/WriteFileAction'
        - $ref: '#/components/schemas/GlobAction'
        - $ref: '#/components/schemas/GrepAction'
        - $ref: '#/components/schemas/PlanningFileEditorAction'
        - $ref: '#/components/schemas/TaskTrackerAction'
        - $ref: '#/components/schemas/TerminalAction'
      discriminator:
        propertyName: kind
        mapping:
          openhands__sdk__mcp__definition__MCPToolAction-Output__1: '#/components/schemas/MCPToolAction'
          openhands__sdk__tool__builtins__finish__FinishAction-Output__1: '#/components/schemas/FinishAction'
          openhands__sdk__tool__builtins__think__ThinkAction-Output__1: '#/components/schemas/ThinkAction'
          openhands__tools__browser_use__definition__BrowserAction-Output__1: '#/components/schemas/BrowserAction'
          openhands__tools__browser_use__definition__BrowserClickAction-Output__1: '#/components/schemas/BrowserClickAction'
          openhands__tools__browser_use__definition__BrowserCloseTabAction-Output__1: '#/components/schemas/BrowserCloseTabAction'
          openhands__tools__browser_use__definition__BrowserGetContentAction-Output__1: '#/components/schemas/BrowserGetContentAction'
          openhands__tools__browser_use__definition__BrowserGetStateAction-Output__1: '#/components/schemas/BrowserGetStateAction'
          openhands__tools__browser_use__definition__BrowserGetStorageAction-Output__1: '#/components/schemas/BrowserGetStorageAction'
          openhands__tools__browser_use__definition__BrowserGoBackAction-Output__1: '#/components/schemas/BrowserGoBackAction'
          openhands__tools__browser_use__definition__BrowserListTabsAction-Output__1: '#/components/schemas/BrowserListTabsAction'
          openhands__tools__browser_use__definition__BrowserNavigateAction-Output__1: '#/components/schemas/BrowserNavigateAction'
          openhands__tools__browser_use__definition__BrowserScrollAction-Output__1: '#/components/schemas/BrowserScrollAction'
          openhands__tools__browser_use__definition__BrowserSetStorageAction-Output__1: '#/components/schemas/BrowserSetStorageAction'
          openhands__tools__browser_use__definition__BrowserSwitchTabAction-Output__1: '#/components/schemas/BrowserSwitchTabAction'
          openhands__tools__browser_use__definition__BrowserTypeAction-Output__1: '#/components/schemas/BrowserTypeAction'
          openhands__tools__file_editor__definition__FileEditorAction-Output__1: '#/components/schemas/FileEditorAction'
          openhands__tools__gemini__edit__definition__EditAction-Output__1: '#/components/schemas/EditAction'
          openhands__tools__gemini__list_directory__definition__ListDirectoryAction-Output__1: '#/components/schemas/ListDirectoryAction'
          openhands__tools__gemini__read_file__definition__ReadFileAction-Output__1: '#/components/schemas/ReadFileAction'
          openhands__tools__gemini__write_file__definition__WriteFileAction-Output__1: '#/components/schemas/WriteFileAction'
          openhands__tools__glob__definition__GlobAction-Output__1: '#/components/schemas/GlobAction'
          openhands__tools__grep__definition__GrepAction-Output__1: '#/components/schemas/GrepAction'
          openhands__tools__planning_file_editor__definition__PlanningFileEditorAction-Output__1: '#/components/schemas/PlanningFileEditorAction'
          openhands__tools__task_tracker__definition__TaskTrackerAction-Output__1: '#/components/schemas/TaskTrackerAction'
          openhands__tools__terminal__definition__TerminalAction-Output__1: '#/components/schemas/TerminalAction'
    MessageToolCall:
      properties:
        id:
          type: string
          title: Id
          description: Canonical tool call id
        name:
          type: string
          title: Name
          description: Tool/function name
        arguments:
          type: string
          title: Arguments
          description: JSON string of arguments
        origin:
          type: string
          enum:
            - completion
            - responses
          title: Origin
          description: Originating API family
      type: object
      required:
        - id
        - name
        - arguments
        - origin
      title: MessageToolCall
      description: |-
        Transport-agnostic tool call representation.

        One canonical id is used for linking across actions/observations and
        for Responses function_call_output call_id.
    SecurityRisk:
      type: string
      enum:
        - UNKNOWN
        - LOW
        - MEDIUM
        - HIGH
      title: SecurityRisk
      description: |-
        Security risk levels for actions.

        Based on OpenHands security risk levels but adapted for agent-sdk.
        Integer values allow for easy comparison and ordering.
    Message:
      properties:
        role:
          type: string
          enum:
            - user
            - system
            - assistant
            - tool
          title: Role
        content:
          items:
            anyOf:
              - $ref: '#/components/schemas/TextContent'
              - $ref: '#/components/schemas/ImageContent'
          type: array
          title: Content
        cache_enabled:
          type: boolean
          title: Cache Enabled
          default: false
        vision_enabled:
          type: boolean
          title: Vision Enabled
          default: false
        function_calling_enabled:
          type: boolean
          title: Function Calling Enabled
          default: false
        tool_calls:
          anyOf:
            - items:
                $ref: '#/components/schemas/MessageToolCall'
              type: array
            - type: 'null'
          title: Tool Calls
        tool_call_id:
          anyOf:
            - type: string
            - type: 'null'
          title: Tool Call Id
        name:
          anyOf:
            - type: string
            - type: 'null'
          title: Name
        force_string_serializer:
          type: boolean
          title: Force String Serializer
          description: >-
            Force using string content serializer when sending to LLM API.
            Useful for providers that do not support list content, like
            HuggingFace and Groq.
          default: false
        send_reasoning_content:
          type: boolean
          title: Send Reasoning Content
          description: >-
            Whether to include the full reasoning content when sending to the
            LLM. Useful for models that support extended reasoning, like
            Kimi-K2-thinking.
          default: false
        reasoning_content:
          anyOf:
            - type: string
            - type: 'null'
          title: Reasoning Content
          description: Intermediate reasoning/thinking content from reasoning models
        thinking_blocks:
          items:
            anyOf:
              - $ref: '#/components/schemas/ThinkingBlock'
              - $ref: '#/components/schemas/RedactedThinkingBlock'
          type: array
          title: Thinking Blocks
          description: Raw Anthropic thinking blocks for extended thinking feature
        responses_reasoning_item:
          anyOf:
            - $ref: '#/components/schemas/ReasoningItemModel'
            - type: 'null'
          description: OpenAI Responses reasoning item from model output
      type: object
      required:
        - role
      title: Message
    Observation:
      oneOf:
        - $ref: '#/components/schemas/MCPToolObservation'
        - $ref: '#/components/schemas/FinishObservation'
        - $ref: '#/components/schemas/ThinkObservation'
        - $ref: '#/components/schemas/BrowserObservation'
        - $ref: '#/components/schemas/FileEditorObservation'
        - $ref: '#/components/schemas/EditObservation'
        - $ref: '#/components/schemas/ListDirectoryObservation'
        - $ref: '#/components/schemas/ReadFileObservation'
        - $ref: '#/components/schemas/WriteFileObservation'
        - $ref: '#/components/schemas/GlobObservation'
        - $ref: '#/components/schemas/GrepObservation'
        - $ref: '#/components/schemas/PlanningFileEditorObservation'
        - $ref: '#/components/schemas/TaskTrackerObservation'
        - $ref: '#/components/schemas/TerminalObservation'
      discriminator:
        propertyName: kind
        mapping:
          openhands__sdk__mcp__definition__MCPToolObservation-Output__1: '#/components/schemas/MCPToolObservation'
          openhands__sdk__tool__builtins__finish__FinishObservation-Output__1: '#/components/schemas/FinishObservation'
          openhands__sdk__tool__builtins__think__ThinkObservation-Output__1: '#/components/schemas/ThinkObservation'
          openhands__tools__browser_use__definition__BrowserObservation-Output__1: '#/components/schemas/BrowserObservation'
          openhands__tools__file_editor__definition__FileEditorObservation-Output__1: '#/components/schemas/FileEditorObservation'
          openhands__tools__gemini__edit__definition__EditObservation-Output__1: '#/components/schemas/EditObservation'
          openhands__tools__gemini__list_directory__definition__ListDirectoryObservation-Output__1: '#/components/schemas/ListDirectoryObservation'
          openhands__tools__gemini__read_file__definition__ReadFileObservation-Output__1: '#/components/schemas/ReadFileObservation'
          openhands__tools__gemini__write_file__definition__WriteFileObservation-Output__1: '#/components/schemas/WriteFileObservation'
          openhands__tools__glob__definition__GlobObservation-Output__1: '#/components/schemas/GlobObservation'
          openhands__tools__grep__definition__GrepObservation-Output__1: '#/components/schemas/GrepObservation'
          openhands__tools__planning_file_editor__definition__PlanningFileEditorObservation-Output__1: '#/components/schemas/PlanningFileEditorObservation'
          openhands__tools__task_tracker__definition__TaskTrackerObservation-Output__1: '#/components/schemas/TaskTrackerObservation'
          openhands__tools__terminal__definition__TerminalObservation-Output__1: '#/components/schemas/TerminalObservation'
    ToolDefinition:
      oneOf:
        - $ref: '#/components/schemas/MCPToolDefinition'
        - $ref: '#/components/schemas/FinishTool'
        - $ref: '#/components/schemas/ThinkTool'
        - $ref: '#/components/schemas/BrowserClickTool'
        - $ref: '#/components/schemas/BrowserCloseTabTool'
        - $ref: '#/components/schemas/BrowserGetContentTool'
        - $ref: '#/components/schemas/BrowserGetStateTool'
        - $ref: '#/components/schemas/BrowserGetStorageTool'
        - $ref: '#/components/schemas/BrowserGoBackTool'
        - $ref: '#/components/schemas/BrowserListTabsTool'
        - $ref: '#/components/schemas/BrowserNavigateTool'
        - $ref: '#/components/schemas/BrowserScrollTool'
        - $ref: '#/components/schemas/BrowserSetStorageTool'
        - $ref: '#/components/schemas/BrowserSwitchTabTool'
        - $ref: '#/components/schemas/BrowserToolSet'
        - $ref: '#/components/schemas/BrowserTypeTool'
        - $ref: '#/components/schemas/FileEditorTool'
        - $ref: '#/components/schemas/EditTool'
        - $ref: '#/components/schemas/ListDirectoryTool'
        - $ref: '#/components/schemas/ReadFileTool'
        - $ref: '#/components/schemas/WriteFileTool'
        - $ref: '#/components/schemas/GlobTool'
        - $ref: '#/components/schemas/GrepTool'
        - $ref: '#/components/schemas/PlanningFileEditorTool'
        - $ref: '#/components/schemas/TaskTrackerTool'
        - $ref: '#/components/schemas/TerminalTool'
      discriminator:
        propertyName: kind
        mapping:
          openhands__sdk__mcp__tool__MCPToolDefinition-Output__1: '#/components/schemas/MCPToolDefinition'
          openhands__sdk__tool__builtins__finish__FinishTool-Output__1: '#/components/schemas/FinishTool'
          openhands__sdk__tool__builtins__think__ThinkTool-Output__1: '#/components/schemas/ThinkTool'
          openhands__tools__browser_use__definition__BrowserClickTool-Output__1: '#/components/schemas/BrowserClickTool'
          openhands__tools__browser_use__definition__BrowserCloseTabTool-Output__1: '#/components/schemas/BrowserCloseTabTool'
          openhands__tools__browser_use__definition__BrowserGetContentTool-Output__1: '#/components/schemas/BrowserGetContentTool'
          openhands__tools__browser_use__definition__BrowserGetStateTool-Output__1: '#/components/schemas/BrowserGetStateTool'
          openhands__tools__browser_use__definition__BrowserGetStorageTool-Output__1: '#/components/schemas/BrowserGetStorageTool'
          openhands__tools__browser_use__definition__BrowserGoBackTool-Output__1: '#/components/schemas/BrowserGoBackTool'
          openhands__tools__browser_use__definition__BrowserListTabsTool-Output__1: '#/components/schemas/BrowserListTabsTool'
          openhands__tools__browser_use__definition__BrowserNavigateTool-Output__1: '#/components/schemas/BrowserNavigateTool'
          openhands__tools__browser_use__definition__BrowserScrollTool-Output__1: '#/components/schemas/BrowserScrollTool'
          openhands__tools__browser_use__definition__BrowserSetStorageTool-Output__1: '#/components/schemas/BrowserSetStorageTool'
          openhands__tools__browser_use__definition__BrowserSwitchTabTool-Output__1: '#/components/schemas/BrowserSwitchTabTool'
          openhands__tools__browser_use__definition__BrowserToolSet-Output__1: '#/components/schemas/BrowserToolSet'
          openhands__tools__browser_use__definition__BrowserTypeTool-Output__1: '#/components/schemas/BrowserTypeTool'
          openhands__tools__file_editor__definition__FileEditorTool-Output__1: '#/components/schemas/FileEditorTool'
          openhands__tools__gemini__edit__definition__EditTool-Output__1: '#/components/schemas/EditTool'
          openhands__tools__gemini__list_directory__definition__ListDirectoryTool-Output__1: '#/components/schemas/ListDirectoryTool'
          openhands__tools__gemini__read_file__definition__ReadFileTool-Output__1: '#/components/schemas/ReadFileTool'
          openhands__tools__gemini__write_file__definition__WriteFileTool-Output__1: '#/components/schemas/WriteFileTool'
          openhands__tools__glob__definition__GlobTool-Output__1: '#/components/schemas/GlobTool'
          openhands__tools__grep__definition__GrepTool-Output__1: '#/components/schemas/GrepTool'
          openhands__tools__planning_file_editor__definition__PlanningFileEditorTool-Output__1: '#/components/schemas/PlanningFileEditorTool'
          openhands__tools__task_tracker__definition__TaskTrackerTool-Output__1: '#/components/schemas/TaskTrackerTool'
          openhands__tools__terminal__definition__TerminalTool-Output__1: '#/components/schemas/TerminalTool'
    MCPToolAction:
      properties:
        data:
          additionalProperties: true
          type: object
          title: Data
          description: Dynamic data fields from the tool call
        kind:
          type: string
          const: MCPToolAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: MCPToolAction
      description: |-
        Schema for MCP input action.

        It is just a thin wrapper around raw JSON and does
        not do any validation.

        Validation will be performed by MCPTool.__call__
        by constructing dynamically created Pydantic model
        from the MCP tool input schema.
    FinishAction:
      properties:
        message:
          type: string
          title: Message
          description: Final message to send to the user.
        kind:
          type: string
          const: FinishAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - message
        - kind
      title: FinishAction
    ThinkAction:
      properties:
        thought:
          type: string
          title: Thought
          description: The thought to log.
        kind:
          type: string
          const: ThinkAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - thought
        - kind
      title: ThinkAction
      description: Action for logging a thought without making any changes.
    BrowserAction:
      properties:
        kind:
          type: string
          const: BrowserAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: BrowserAction
      description: |-
        Base class for all browser actions.

        This base class serves as the parent for all browser-related actions,
        enabling proper type hierarchy and eliminating the need for union types.
    BrowserClickAction:
      properties:
        index:
          type: integer
          minimum: 0
          title: Index
          description: The index of the element to click (from browser_get_state)
        new_tab:
          type: boolean
          title: New Tab
          description: >-
            Whether to open any resulting navigation in a new tab. Default:
            False
          default: false
        kind:
          type: string
          const: BrowserClickAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - index
        - kind
      title: BrowserClickAction
      description: Schema for clicking elements.
    BrowserCloseTabAction:
      properties:
        tab_id:
          type: string
          title: Tab Id
          description: 4 Character Tab ID of the tab to close (from browser_list_tabs)
        kind:
          type: string
          const: BrowserCloseTabAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - tab_id
        - kind
      title: BrowserCloseTabAction
      description: Schema for closing browser tabs.
    BrowserGetContentAction:
      properties:
        extract_links:
          type: boolean
          title: Extract Links
          description: 'Whether to include links in the content (default: False)'
          default: false
        start_from_char:
          type: integer
          minimum: 0
          title: Start From Char
          description: 'Character index to start from in the page content (default: 0)'
          default: 0
        kind:
          type: string
          const: BrowserGetContentAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: BrowserGetContentAction
      description: Schema for getting page content in markdown.
    BrowserGetStateAction:
      properties:
        include_screenshot:
          type: boolean
          title: Include Screenshot
          description: 'Whether to include a screenshot of the current page. Default: False'
          default: false
        kind:
          type: string
          const: BrowserGetStateAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: BrowserGetStateAction
      description: Schema for getting browser state.
    BrowserGetStorageAction:
      properties:
        kind:
          type: string
          const: BrowserGetStorageAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: BrowserGetStorageAction
      description: >-
        Schema for getting browser storage (cookies, local storage, session
        storage).
    BrowserGoBackAction:
      properties:
        kind:
          type: string
          const: BrowserGoBackAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: BrowserGoBackAction
      description: Schema for going back in browser history.
    BrowserListTabsAction:
      properties:
        kind:
          type: string
          const: BrowserListTabsAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: BrowserListTabsAction
      description: Schema for listing browser tabs.
    BrowserNavigateAction:
      properties:
        url:
          type: string
          title: Url
          description: The URL to navigate to
        new_tab:
          type: boolean
          title: New Tab
          description: 'Whether to open in a new tab. Default: False'
          default: false
        kind:
          type: string
          const: BrowserNavigateAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - url
        - kind
      title: BrowserNavigateAction
      description: Schema for browser navigation.
    BrowserScrollAction:
      properties:
        direction:
          type: string
          enum:
            - up
            - down
          title: Direction
          description: 'Direction to scroll. Options: ''up'', ''down''. Default: ''down'''
          default: down
        kind:
          type: string
          const: BrowserScrollAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: BrowserScrollAction
      description: Schema for scrolling the page.
    BrowserSetStorageAction:
      properties:
        storage_state:
          additionalProperties: true
          type: object
          title: Storage State
          description: >-
            Storage state dictionary containing 'cookies' and 'origins' (from
            browser_get_storage)
        kind:
          type: string
          const: BrowserSetStorageAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - storage_state
        - kind
      title: BrowserSetStorageAction
      description: >-
        Schema for setting browser storage (cookies, local storage, session
        storage).
    BrowserSwitchTabAction:
      properties:
        tab_id:
          type: string
          title: Tab Id
          description: 4 Character Tab ID of the tab to switch to (from browser_list_tabs)
        kind:
          type: string
          const: BrowserSwitchTabAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - tab_id
        - kind
      title: BrowserSwitchTabAction
      description: Schema for switching browser tabs.
    BrowserTypeAction:
      properties:
        index:
          type: integer
          minimum: 0
          title: Index
          description: The index of the input element (from browser_get_state)
        text:
          type: string
          title: Text
          description: The text to type
        kind:
          type: string
          const: BrowserTypeAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - index
        - text
        - kind
      title: BrowserTypeAction
      description: Schema for typing text into elements.
    FileEditorAction:
      properties:
        command:
          type: string
          enum:
            - view
            - create
            - str_replace
            - insert
            - undo_edit
          title: Command
          description: >-
            The commands to run. Allowed options are: `view`, `create`,
            `str_replace`, `insert`, `undo_edit`.
        path:
          type: string
          title: Path
          description: Absolute path to file or directory.
        file_text:
          anyOf:
            - type: string
            - type: 'null'
          title: File Text
          description: >-
            Required parameter of `create` command, with the content of the file
            to be created.
        old_str:
          anyOf:
            - type: string
            - type: 'null'
          title: Old Str
          description: >-
            Required parameter of `str_replace` command containing the string in
            `path` to replace.
        new_str:
          anyOf:
            - type: string
            - type: 'null'
          title: New Str
          description: >-
            Optional parameter of `str_replace` command containing the new
            string (if not given, no string will be added). Required parameter
            of `insert` command containing the string to insert.
        insert_line:
          anyOf:
            - type: integer
              minimum: 0
            - type: 'null'
          title: Insert Line
          description: >-
            Required parameter of `insert` command. The `new_str` will be
            inserted AFTER the line `insert_line` of `path`.
        view_range:
          anyOf:
            - items:
                type: integer
              type: array
            - type: 'null'
          title: View Range
          description: >-
            Optional parameter of `view` command when `path` points to a file.
            If none is given, the full file is shown. If provided, the file will
            be shown in the indicated line number range, e.g. [11, 12] will show
            lines 11 and 12. Indexing at 1 to start. Setting `[start_line, -1]`
            shows all lines from `start_line` to the end of the file.
        kind:
          type: string
          const: FileEditorAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - command
        - path
        - kind
      title: FileEditorAction
      description: Schema for file editor operations.
    EditAction:
      properties:
        file_path:
          type: string
          title: File Path
          description: The path to the file to modify.
        old_string:
          type: string
          title: Old String
          description: >-
            The text to replace. To create a new file, use an empty string. Must
            match the exact text in the file including whitespace.
        new_string:
          type: string
          title: New String
          description: The text to replace it with.
        expected_replacements:
          type: integer
          minimum: 0
          title: Expected Replacements
          description: >-
            Number of replacements expected. Defaults to 1. Use when you want to
            replace multiple occurrences. The edit will fail if the actual count
            doesn't match.
          default: 1
        kind:
          type: string
          const: EditAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - file_path
        - old_string
        - new_string
        - kind
      title: EditAction
      description: Schema for edit operation.
    ListDirectoryAction:
      properties:
        dir_path:
          type: string
          title: Dir Path
          description: The path to the directory to list. Defaults to current directory.
          default: .
        recursive:
          type: boolean
          title: Recursive
          description: Whether to list subdirectories recursively (up to 2 levels).
          default: false
        kind:
          type: string
          const: ListDirectoryAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: ListDirectoryAction
      description: Schema for list directory operation.
    ReadFileAction:
      properties:
        file_path:
          type: string
          title: File Path
          description: The path to the file to read.
        offset:
          anyOf:
            - type: integer
              minimum: 0
            - type: 'null'
          title: Offset
          description: >-
            Optional: The 0-based line number to start reading from. Use for
            paginating through large files.
        limit:
          anyOf:
            - type: integer
              minimum: 1
            - type: 'null'
          title: Limit
          description: >-
            Optional: Maximum number of lines to read. Use with 'offset' to
            paginate through large files.
        kind:
          type: string
          const: ReadFileAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - file_path
        - kind
      title: ReadFileAction
      description: Schema for read file operation.
    WriteFileAction:
      properties:
        file_path:
          type: string
          title: File Path
          description: The path to the file to write to.
        content:
          type: string
          title: Content
          description: The content to write to the file.
        kind:
          type: string
          const: WriteFileAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - file_path
        - content
        - kind
      title: WriteFileAction
      description: Schema for write file operation.
    GlobAction:
      properties:
        pattern:
          type: string
          title: Pattern
          description: The glob pattern to match files (e.g., "**/*.js", "src/**/*.ts")
        path:
          anyOf:
            - type: string
            - type: 'null'
          title: Path
          description: >-
            The directory (absolute path) to search in. Defaults to the current
            working directory.
        kind:
          type: string
          const: GlobAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - pattern
        - kind
      title: GlobAction
      description: Schema for glob pattern matching operations.
    GrepAction:
      properties:
        pattern:
          type: string
          title: Pattern
          description: The regex pattern to search for in file contents
        path:
          anyOf:
            - type: string
            - type: 'null'
          title: Path
          description: >-
            The directory (absolute path) to search in. Defaults to the current
            working directory.
        include:
          anyOf:
            - type: string
            - type: 'null'
          title: Include
          description: >-
            Optional file pattern to filter which files to search (e.g., "*.js",
            "*.{ts,tsx}")
        kind:
          type: string
          const: GrepAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - pattern
        - kind
      title: GrepAction
      description: Schema for grep content search operations.
    PlanningFileEditorAction:
      properties:
        command:
          type: string
          enum:
            - view
            - create
            - str_replace
            - insert
            - undo_edit
          title: Command
          description: >-
            The commands to run. Allowed options are: `view`, `create`,
            `str_replace`, `insert`, `undo_edit`.
        path:
          type: string
          title: Path
          description: Absolute path to file or directory.
        file_text:
          anyOf:
            - type: string
            - type: 'null'
          title: File Text
          description: >-
            Required parameter of `create` command, with the content of the file
            to be created.
        old_str:
          anyOf:
            - type: string
            - type: 'null'
          title: Old Str
          description: >-
            Required parameter of `str_replace` command containing the string in
            `path` to replace.
        new_str:
          anyOf:
            - type: string
            - type: 'null'
          title: New Str
          description: >-
            Optional parameter of `str_replace` command containing the new
            string (if not given, no string will be added). Required parameter
            of `insert` command containing the string to insert.
        insert_line:
          anyOf:
            - type: integer
              minimum: 0
            - type: 'null'
          title: Insert Line
          description: >-
            Required parameter of `insert` command. The `new_str` will be
            inserted AFTER the line `insert_line` of `path`.
        view_range:
          anyOf:
            - items:
                type: integer
              type: array
            - type: 'null'
          title: View Range
          description: >-
            Optional parameter of `view` command when `path` points to a file.
            If none is given, the full file is shown. If provided, the file will
            be shown in the indicated line number range, e.g. [11, 12] will show
            lines 11 and 12. Indexing at 1 to start. Setting `[start_line, -1]`
            shows all lines from `start_line` to the end of the file.
        kind:
          type: string
          const: PlanningFileEditorAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - command
        - path
        - kind
      title: PlanningFileEditorAction
      description: |-
        Schema for planning file editor operations.

        Inherits from FileEditorAction but restricts editing to PLAN.md only.
        Allows viewing any file but only editing PLAN.md.
    TaskTrackerAction:
      properties:
        command:
          type: string
          enum:
            - view
            - plan
          title: Command
          description: >-
            The command to execute. `view` shows the current task list. `plan`
            creates or updates the task list based on provided requirements and
            progress. Always `view` the current list before making changes.
          default: view
        task_list:
          items:
            $ref: '#/components/schemas/TaskItem'
          type: array
          title: Task List
          description: The full task list. Required parameter of `plan` command.
        kind:
          type: string
          const: TaskTrackerAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: TaskTrackerAction
      description: >-
        An action where the agent writes or updates a task list for task
        management.
    TerminalAction:
      properties:
        command:
          type: string
          title: Command
          description: >-
            The bash command to execute. Can be empty string to view additional
            logs when previous exit code is `-1`. Can be `C-c` (Ctrl+C) to
            interrupt the currently running process. Note: You can only execute
            one bash command at a time. If you need to run multiple commands
            sequentially, you can use `&&` or `;` to chain them together.
        is_input:
          type: boolean
          title: Is Input
          description: >-
            If True, the command is an input to the running process. If False,
            the command is a bash command to be executed in the terminal.
            Default is False.
          default: false
        timeout:
          anyOf:
            - type: number
              minimum: 0
            - type: 'null'
          title: Timeout
          description: >-
            Optional. Sets a maximum time limit (in seconds) for running the
            command. If the command takes longer than this limit, you’ll be
            asked whether to continue or stop it. If you don’t set a value, the
            command will instead pause and ask for confirmation when it produces
            no new output for 30 seconds. Use a higher value if the command is
            expected to take a long time (like installation or testing), or if
            it has a known fixed duration (like sleep).
        reset:
          type: boolean
          title: Reset
          description: >-
            If True, reset the terminal by creating a new session. Use this only
            when the terminal becomes unresponsive. Note that all previously set
            environment variables and session state will be lost after reset.
            Cannot be used with is_input=True.
          default: false
        kind:
          type: string
          const: TerminalAction
          title: Kind
      additionalProperties: false
      type: object
      required:
        - command
        - kind
      title: TerminalAction
      description: Schema for bash command execution.
    ImageContent:
      properties:
        cache_prompt:
          type: boolean
          title: Cache Prompt
          default: false
        type:
          type: string
          const: image
          title: Type
          default: image
        image_urls:
          items:
            type: string
          type: array
          title: Image Urls
      type: object
      required:
        - image_urls
      title: ImageContent
    MCPToolObservation:
      properties:
        content:
          items:
            anyOf:
              - $ref: '#/components/schemas/TextContent'
              - $ref: '#/components/schemas/ImageContent'
          type: array
          title: Content
          description: >-
            Content returned from the tool as a list of TextContent/ImageContent
            objects. When there is an error, it should be written in this field.
        is_error:
          type: boolean
          title: Is Error
          description: Whether the observation indicates an error
          default: false
        tool_name:
          type: string
          title: Tool Name
          description: Name of the tool that was called
        kind:
          type: string
          const: MCPToolObservation
          title: Kind
      additionalProperties: false
      type: object
      required:
        - tool_name
        - kind
      title: MCPToolObservation
      description: Observation from MCP tool execution.
    FinishObservation:
      properties:
        content:
          items:
            anyOf:
              - $ref: '#/components/schemas/TextContent'
              - $ref: '#/components/schemas/ImageContent'
          type: array
          title: Content
          description: >-
            Content returned from the tool as a list of TextContent/ImageContent
            objects. When there is an error, it should be written in this field.
        is_error:
          type: boolean
          title: Is Error
          description: Whether the observation indicates an error
          default: false
        kind:
          type: string
          const: FinishObservation
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: FinishObservation
      description: |-
        Observation returned after finishing a task.
        The FinishAction itself contains the message sent to the user so no
        extra fields are needed here.
    ThinkObservation:
      properties:
        content:
          items:
            anyOf:
              - $ref: '#/components/schemas/TextContent'
              - $ref: '#/components/schemas/ImageContent'
          type: array
          title: Content
          description: >-
            Content returned from the tool as a list of TextContent/ImageContent
            objects. When there is an error, it should be written in this field.
        is_error:
          type: boolean
          title: Is Error
          description: Whether the observation indicates an error
          default: false
        kind:
          type: string
          const: ThinkObservation
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: ThinkObservation
      description: |-
        Observation returned after logging a thought.
        The ThinkAction itself contains the thought logged so no extra
        fields are needed here.
    BrowserObservation:
      properties:
        content:
          items:
            anyOf:
              - $ref: '#/components/schemas/TextContent'
              - $ref: '#/components/schemas/ImageContent'
          type: array
          title: Content
          description: >-
            Content returned from the tool as a list of TextContent/ImageContent
            objects. When there is an error, it should be written in this field.
        is_error:
          type: boolean
          title: Is Error
          description: Whether the observation indicates an error
          default: false
        screenshot_data:
          anyOf:
            - type: string
            - type: 'null'
          title: Screenshot Data
          description: Base64 screenshot data if available
        full_output_save_dir:
          anyOf:
            - type: string
            - type: 'null'
          title: Full Output Save Dir
          description: Directory where full output files are saved
        kind:
          type: string
          const: BrowserObservation
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: BrowserObservation
      description: Base observation for browser operations.
    FileEditorObservation:
      properties:
        content:
          items:
            anyOf:
              - $ref: '#/components/schemas/TextContent'
              - $ref: '#/components/schemas/ImageContent'
          type: array
          title: Content
          description: >-
            Content returned from the tool as a list of TextContent/ImageContent
            objects. When there is an error, it should be written in this field.
        is_error:
          type: boolean
          title: Is Error
          description: Whether the observation indicates an error
          default: false
        command:
          type: string
          enum:
            - view
            - create
            - str_replace
            - insert
            - undo_edit
          title: Command
          description: >-
            The command that was run: `view`, `create`, `str_replace`, `insert`,
            or `undo_edit`.
        path:
          anyOf:
            - type: string
            - type: 'null'
          title: Path
          description: The file path that was edited.
        prev_exist:
          type: boolean
          title: Prev Exist
          description: Indicates if the file previously existed. If not, it was created.
          default: true
        old_content:
          anyOf:
            - type: string
            - type: 'null'
          title: Old Content
          description: The content of the file before the edit.
        new_content:
          anyOf:
            - type: string
            - type: 'null'
          title: New Content
          description: The content of the file after the edit.
        kind:
          type: string
          const: FileEditorObservation
          title: Kind
      additionalProperties: false
      type: object
      required:
        - command
        - kind
      title: FileEditorObservation
      description: A ToolResult that can be rendered as a CLI output.
    EditObservation:
      properties:
        content:
          items:
            anyOf:
              - $ref: '#/components/schemas/TextContent'
              - $ref: '#/components/schemas/ImageContent'
          type: array
          title: Content
          description: >-
            Content returned from the tool as a list of TextContent/ImageContent
            objects. When there is an error, it should be written in this field.
        is_error:
          type: boolean
          title: Is Error
          description: Whether the observation indicates an error
          default: false
        file_path:
          anyOf:
            - type: string
            - type: 'null'
          title: File Path
          description: The file path that was edited.
        is_new_file:
          type: boolean
          title: Is New File
          description: Whether a new file was created.
          default: false
        replacements_made:
          type: integer
          title: Replacements Made
          description: Number of replacements actually made.
          default: 0
        old_content:
          anyOf:
            - type: string
            - type: 'null'
          title: Old Content
          description: The content before the edit.
        new_content:
          anyOf:
            - type: string
            - type: 'null'
          title: New Content
          description: The content after the edit.
        kind:
          type: string
          const: EditObservation
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: EditObservation
      description: Observation from editing a file.
    ListDirectoryObservation:
      properties:
        content:
          items:
            anyOf:
              - $ref: '#/components/schemas/TextContent'
              - $ref: '#/components/schemas/ImageContent'
          type: array
          title: Content
          description: >-
            Content returned from the tool as a list of TextContent/ImageContent
            objects. When there is an error, it should be written in this field.
        is_error:
          type: boolean
          title: Is Error
          description: Whether the observation indicates an error
          default: false
        dir_path:
          anyOf:
            - type: string
            - type: 'null'
          title: Dir Path
          description: The directory path that was listed.
        entries:
          items:
            $ref: '#/components/schemas/FileEntry'
          type: array
          title: Entries
          description: List of files and directories found.
        total_count:
          type: integer
          title: Total Count
          description: Total number of entries found.
          default: 0
        is_truncated:
          type: boolean
          title: Is Truncated
          description: Whether the listing was truncated due to too many entries.
          default: false
        kind:
          type: string
          const: ListDirectoryObservation
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: ListDirectoryObservation
      description: Observation from listing a directory.
    ReadFileObservation:
      properties:
        content:
          items:
            anyOf:
              - $ref: '#/components/schemas/TextContent'
              - $ref: '#/components/schemas/ImageContent'
          type: array
          title: Content
          description: >-
            Content returned from the tool as a list of TextContent/ImageContent
            objects. When there is an error, it should be written in this field.
        is_error:
          type: boolean
          title: Is Error
          description: Whether the observation indicates an error
          default: false
        file_path:
          type: string
          title: File Path
          description: The file path that was read.
        file_content:
          type: string
          title: File Content
          description: The content read from the file.
          default: ''
        is_truncated:
          type: boolean
          title: Is Truncated
          description: Whether the content was truncated due to size limits.
          default: false
        lines_shown:
          anyOf:
            - prefixItems:
                - type: integer
                - type: integer
              type: array
              maxItems: 2
              minItems: 2
            - type: 'null'
          title: Lines Shown
          description: If truncated, the range of lines shown (start, end) - 1-indexed.
        total_lines:
          anyOf:
            - type: integer
            - type: 'null'
          title: Total Lines
          description: Total number of lines in the file.
        kind:
          type: string
          const: ReadFileObservation
          title: Kind
      additionalProperties: false
      type: object
      required:
        - file_path
        - kind
      title: ReadFileObservation
      description: Observation from reading a file.
    WriteFileObservation:
      properties:
        content:
          items:
            anyOf:
              - $ref: '#/components/schemas/TextContent'
              - $ref: '#/components/schemas/ImageContent'
          type: array
          title: Content
          description: >-
            Content returned from the tool as a list of TextContent/ImageContent
            objects. When there is an error, it should be written in this field.
        is_error:
          type: boolean
          title: Is Error
          description: Whether the observation indicates an error
          default: false
        file_path:
          anyOf:
            - type: string
            - type: 'null'
          title: File Path
          description: The file path that was written.
        is_new_file:
          type: boolean
          title: Is New File
          description: Whether a new file was created.
          default: false
        old_content:
          anyOf:
            - type: string
            - type: 'null'
          title: Old Content
          description: The previous content of the file (if it existed).
        new_content:
          anyOf:
            - type: string
            - type: 'null'
          title: New Content
          description: The new content written to the file.
        kind:
          type: string
          const: WriteFileObservation
          title: Kind
      additionalProperties: false
      type: object
      required:
        - kind
      title: WriteFileObservation
      description: Observation from writing a file.
    GlobObservation:
      properties:
        content:
          items:
            anyOf:
              - $ref: '#/components/schemas/TextContent'
              - $ref: '#/components/schemas/ImageContent'
          type: array
          title: Content
          description: >-
            Content returned from the tool as a list of TextContent/ImageContent
            objects. When there is an error, it should be written in this field.
        is_error:
          type: boolean
          title: Is Error
          description: Whether the observation indicates an error
          default: false
        files:
          items:
            type: string
          type: array
          title: Files
          description: List of matching file paths sorted by modification time
        pattern:
          type: string
          title: Pattern
          description: The glob pattern that was used
        search_path:
          type: string
          title: Search Path
          description: The directory that was searched
        truncated:
          type: boolean
          title: Truncated
          description: Whether results were truncated to 100 files
          default: false
        kind:
          type: string
          const: GlobObservation
          title: Kind
      additionalProperties: false
      type: object
      required:
        - files
        - pattern
        - search_path
        - kind
      title: GlobObservation
      description: Observation from glob pattern matching operations.
    GrepObservation:
      properties:
        content:
          items:
            anyOf:
              - $ref: '#/components/schemas/TextContent'
              - $ref: '#/components/schemas/ImageContent'
          type: array
          title: Content
          description: >-
            Content returned from the tool as a list of TextContent/ImageContent
            objects. When there is an error, it should be written in this field.
        is_error:
          type: boolean
          title: Is Error
          description: Whether the observation indicates an error
          default: false
        matches:
          items:
            type: string
          type: array
          title: Matches
          description: List of file paths containing the pattern
        pattern:
          type: string
          title: Pattern
          description: The regex pattern that was used
        search_path:
          type: string
          title: Search Path
          description: The directory that was searched
        include_pattern:
          anyOf:
            - type: string
            - type: 'null'
          title: Include Pattern
          description: The file pattern filter that was used
        truncated:
          type: boolean
          title: Truncated
          description: Whether results were truncated to 100 files
          default: false
        kind:
          type: string
          const: GrepObservation
          title: Kind
      additionalProperties: false
      type: object
      required:
        - matches
        - pattern
        - search_path
        - kind
      title: GrepObservation
      description: Observation from grep content search operations.
    PlanningFileEditorObservation:
      properties:
        content:
          items:
            anyOf:
              - $ref: '#/components/schemas/TextContent'
              - $ref: '#/components/schemas/ImageContent'
          type: array
          title: Content
          description: >-
            Content returned from the tool as a list of TextContent/ImageContent
            objects. When there is an error, it should be written in this field.
        is_error:
          type: boolean
          title: Is Error
          description: Whether the observation indicates an error
          default: false
        command:
          type: string
          enum:
            - view
            - create
            - str_replace
            - insert
            - undo_edit
          title: Command
          description: >-
            The command that was run: `view`, `create`, `str_replace`, `insert`,
            or `undo_edit`.
        path:
          anyOf:
            - type: string
            - type: 'null'
          title: Path
          description: The file path that was edited.
        prev_exist:
          type: boolean
          title: Prev Exist
          description: Indicates if the file previously existed. If not, it was created.
          default: true
        old_content:
          anyOf:
            - type: string
            - type: 'null'
          title: Old Content
          description: The content of the file before the edit.
        new_content:
          anyOf:
            - type: string
            - type: 'null'
          title: New Content
          description: The content of the file after the edit.
        kind:
          type: string
          const: PlanningFileEditorObservation
          title: Kind
      additionalProperties: false
      type: object
      required:
        - command
        - kind
      title: PlanningFileEditorObservation
      description: >-
        Observation from planning file editor operations.


        Inherits from FileEditorObservation - same structure, just different
        type.
    TaskTrackerObservation:
      properties:
        content:
          items:
            anyOf:
              - $ref: '#/components/schemas/TextContent'
              - $ref: '#/components/schemas/ImageContent'
          type: array
          title: Content
          description: >-
            Content returned from the tool as a list of TextContent/ImageContent
            objects. When there is an error, it should be written in this field.
        is_error:
          type: boolean
          title: Is Error
          description: Whether the observation indicates an error
          default: false
        command:
          type: string
          enum:
            - view
            - plan
          title: Command
          description: 'The command that was executed: "view" or "plan".'
        task_list:
          items:
            $ref: '#/components/schemas/TaskItem'
          type: array
          title: Task List
          description: The current task list
        kind:
          type: string
          const: TaskTrackerObservation
          title: Kind
      additionalProperties: false
      type: object
      required:
        - command
        - kind
      title: TaskTrackerObservation
      description: This data class represents the result of a task tracking operation.
    TerminalObservation:
      properties:
        content:
          items:
            anyOf:
              - $ref: '#/components/schemas/TextContent'
              - $ref: '#/components/schemas/ImageContent'
          type: array
          title: Content
          description: >-
            Content returned from the tool as a list of TextContent/ImageContent
            objects. When there is an error, it should be written in this field.
        is_error:
          type: boolean
          title: Is Error
          description: Whether the observation indicates an error
          default: false
        command:
          anyOf:
            - type: string
            - type: 'null'
          title: Command
          description: >-
            The bash command that was executed. Can be empty string if the
            observation is from a previous command that hit soft timeout and is
            not yet finished.
        exit_code:
          anyOf:
            - type: integer
            - type: 'null'
          title: Exit Code
          description: >-
            The exit code of the command. -1 indicates the process hit the soft
            timeout and is not yet finished.
        timeout:
          type: boolean
          title: Timeout
          description: Whether the command execution timed out.
          default: false
        metadata:
          $ref: '#/components/schemas/CmdOutputMetadata'
          description: Additional metadata captured from PS1 after command execution.
        full_output_save_dir:
          anyOf:
            - type: string
            - type: 'null'
          title: Full Output Save Dir
          description: Directory where full output files are saved
        kind:
          type: string
          const: TerminalObservation
          title: Kind
      additionalProperties: false
      type: object
      required:
        - command
        - kind
      title: TerminalObservation
      description: A ToolResult that can be rendered as a CLI output.
    MCPToolDefinition:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        mcp_tool:
          $ref: '#/components/schemas/mcp__types__Tool'
          description: The MCP tool definition.
        kind:
          type: string
          const: MCPToolDefinition
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - mcp_tool
        - kind
        - title
      title: MCPToolDefinition
      description: MCP Tool that wraps an MCP client and provides tool functionality.
    FinishTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: FinishTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: FinishTool
      description: Tool for signaling the completion of a task or conversation.
    ThinkTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: ThinkTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: ThinkTool
      description: Tool for logging thoughts without making changes.
    BrowserClickTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: BrowserClickTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: BrowserClickTool
      description: Tool for clicking browser elements.
    BrowserCloseTabTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: BrowserCloseTabTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: BrowserCloseTabTool
      description: Tool for closing browser tabs.
    BrowserGetContentTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: BrowserGetContentTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: BrowserGetContentTool
      description: Tool for getting page content in markdown.
    BrowserGetStateTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: BrowserGetStateTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: BrowserGetStateTool
      description: Tool for getting browser state.
    BrowserGetStorageTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: BrowserGetStorageTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: BrowserGetStorageTool
      description: Tool for getting browser storage.
    BrowserGoBackTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: BrowserGoBackTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: BrowserGoBackTool
      description: Tool for going back in browser history.
    BrowserListTabsTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: BrowserListTabsTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: BrowserListTabsTool
      description: Tool for listing browser tabs.
    BrowserNavigateTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: BrowserNavigateTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: BrowserNavigateTool
      description: Tool for browser navigation.
    BrowserScrollTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: BrowserScrollTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: BrowserScrollTool
      description: Tool for scrolling the browser page.
    BrowserSetStorageTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: BrowserSetStorageTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: BrowserSetStorageTool
      description: Tool for setting browser storage.
    BrowserSwitchTabTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: BrowserSwitchTabTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: BrowserSwitchTabTool
      description: Tool for switching browser tabs.
    BrowserToolSet:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: BrowserToolSet
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: BrowserToolSet
      description: |-
        A set of all browser tools.

        This tool set includes all available browser-related tools
          for interacting with web pages.

        The toolset automatically checks for Chromium availability
        when created and automatically installs it if missing.
    BrowserTypeTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: BrowserTypeTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: BrowserTypeTool
      description: Tool for typing text into browser elements.
    FileEditorTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: FileEditorTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: FileEditorTool
      description: >-
        A ToolDefinition subclass that automatically initializes a
        FileEditorExecutor.
    EditTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: EditTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: EditTool
      description: Tool for editing files via find/replace.
    ListDirectoryTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: ListDirectoryTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: ListDirectoryTool
      description: Tool for listing directory contents with metadata.
    ReadFileTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: ReadFileTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: ReadFileTool
      description: Tool for reading file contents with pagination support.
    WriteFileTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: WriteFileTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: WriteFileTool
      description: Tool for writing complete file contents.
    GlobTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: GlobTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: GlobTool
      description: A ToolDefinition subclass that automatically initializes a GlobExecutor.
    GrepTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: GrepTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: GrepTool
      description: A ToolDefinition subclass that automatically initializes a GrepExecutor.
    PlanningFileEditorTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: PlanningFileEditorTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: PlanningFileEditorTool
      description: A planning file editor tool with read-all, edit-PLAN.md-only access.
    TaskTrackerTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: TaskTrackerTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: TaskTrackerTool
      description: >-
        A ToolDefinition subclass that automatically initializes a
        TaskTrackerExecutor.
    TerminalTool:
      properties:
        description:
          type: string
          title: Description
        action_type:
          type: string
          title: Action Type
        observation_type:
          anyOf:
            - type: string
            - type: 'null'
          title: Observation Type
        annotations:
          anyOf:
            - $ref: '#/components/schemas/openhands__sdk__tool__tool__ToolAnnotations'
            - type: 'null'
        meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
        kind:
          type: string
          const: TerminalTool
          title: Kind
        title:
          type: string
          title: Title
          readOnly: true
      type: object
      required:
        - description
        - action_type
        - kind
        - title
      title: TerminalTool
      description: >-
        A ToolDefinition subclass that automatically initializes a
        TerminalExecutor with auto-detection.
    TaskItem:
      properties:
        title:
          type: string
          title: Title
          description: A brief title for the task.
        notes:
          type: string
          title: Notes
          description: Additional details or notes about the task.
          default: ''
        status:
          type: string
          enum:
            - todo
            - in_progress
            - done
          title: Status
          description: >-
            The current status of the task. One of 'todo', 'in_progress', or
            'done'.
          default: todo
      type: object
      required:
        - title
      title: TaskItem
    FileEntry:
      properties:
        name:
          type: string
          title: Name
          description: Name of the file or directory
        path:
          type: string
          title: Path
          description: Absolute path to the file or directory
        is_directory:
          type: boolean
          title: Is Directory
          description: Whether this entry is a directory
        size:
          type: integer
          title: Size
          description: Size of the file in bytes (0 for directories)
        modified_time:
          type: string
          format: date-time
          title: Modified Time
          description: Last modified timestamp
      type: object
      required:
        - name
        - path
        - is_directory
        - size
        - modified_time
      title: FileEntry
      description: Information about a file or directory.
    CmdOutputMetadata:
      properties:
        exit_code:
          type: integer
          title: Exit Code
          description: The exit code of the last executed command.
          default: -1
        pid:
          type: integer
          title: Pid
          description: The process ID of the last executed command.
          default: -1
        username:
          anyOf:
            - type: string
            - type: 'null'
          title: Username
          description: The username of the current user.
        hostname:
          anyOf:
            - type: string
            - type: 'null'
          title: Hostname
          description: The hostname of the machine.
        working_dir:
          anyOf:
            - type: string
            - type: 'null'
          title: Working Dir
          description: The current working directory.
        py_interpreter_path:
          anyOf:
            - type: string
            - type: 'null'
          title: Py Interpreter Path
          description: The path to the current Python interpreter, if any.
        prefix:
          type: string
          title: Prefix
          description: Prefix to add to command output
          default: ''
        suffix:
          type: string
          title: Suffix
          description: Suffix to add to command output
          default: ''
      type: object
      title: CmdOutputMetadata
      description: Additional metadata captured from PS1
    openhands__sdk__tool__tool__ToolAnnotations:
      properties:
        title:
          anyOf:
            - type: string
            - type: 'null'
          title: Title
          description: A human-readable title for the tool.
        readOnlyHint:
          type: boolean
          title: Readonlyhint
          description: 'If true, the tool does not modify its environment. Default: false'
          default: false
        destructiveHint:
          type: boolean
          title: Destructivehint
          description: >-
            If true, the tool may perform destructive updates to its
            environment. If false, the tool performs only additive updates.
            (This property is meaningful only when `readOnlyHint == false`)
            Default: true
          default: true
        idempotentHint:
          type: boolean
          title: Idempotenthint
          description: >-
            If true, calling the tool repeatedly with the same arguments will
            have no additional effect on the its environment. (This property is
            meaningful only when `readOnlyHint == false`) Default: false
          default: false
        openWorldHint:
          type: boolean
          title: Openworldhint
          description: >-
            If true, this tool may interact with an 'open world' of external
            entities. If false, the tool's domain of interaction is closed. For
            example, the world of a web search tool is open, whereas that of a
            memory tool is not. Default: true
          default: true
      type: object
      title: openhands.sdk.tool.tool.ToolAnnotations
      description: >-
        Annotations to provide hints about the tool's behavior.


        Based on Model Context Protocol (MCP) spec:

        https://github.com/modelcontextprotocol/modelcontextprotocol/blob/caf3424488b10b4a7b1f8cb634244a450a1f4400/schema/2025-06-18/schema.ts#L838
    mcp__types__Tool:
      properties:
        name:
          type: string
          title: Name
        title:
          anyOf:
            - type: string
            - type: 'null'
          title: Title
        description:
          anyOf:
            - type: string
            - type: 'null'
          title: Description
        inputSchema:
          additionalProperties: true
          type: object
          title: Inputschema
        outputSchema:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Outputschema
        icons:
          anyOf:
            - items:
                $ref: '#/components/schemas/Icon'
              type: array
            - type: 'null'
          title: Icons
        annotations:
          anyOf:
            - $ref: '#/components/schemas/mcp__types__ToolAnnotations'
            - type: 'null'
        _meta:
          anyOf:
            - additionalProperties: true
              type: object
            - type: 'null'
          title: Meta
      additionalProperties: true
      type: object
      required:
        - name
        - inputSchema
      title: Tool
      description: Definition for a tool the client can call.
    Icon:
      properties:
        src:
          type: string
          title: Src
        mimeType:
          anyOf:
            - type: string
            - type: 'null'
          title: Mimetype
        sizes:
          anyOf:
            - items:
                type: string
              type: array
            - type: 'null'
          title: Sizes
      additionalProperties: true
      type: object
      required:
        - src
      title: Icon
      description: An icon for display in user interfaces.
    mcp__types__ToolAnnotations:
      properties:
        title:
          anyOf:
            - type: string
            - type: 'null'
          title: Title
        readOnlyHint:
          anyOf:
            - type: boolean
            - type: 'null'
          title: Readonlyhint
        destructiveHint:
          anyOf:
            - type: boolean
            - type: 'null'
          title: Destructivehint
        idempotentHint:
          anyOf:
            - type: boolean
            - type: 'null'
          title: Idempotenthint
        openWorldHint:
          anyOf:
            - type: boolean
            - type: 'null'
          title: Openworldhint
      additionalProperties: true
      type: object
      title: ToolAnnotations
      description: |-
        Additional properties describing a Tool to clients.

        NOTE: all properties in ToolAnnotations are **hints**.
        They are not guaranteed to provide a faithful description of
        tool behavior (including descriptive properties like `title`).

        Clients should never make tool use decisions based on ToolAnnotations
        received from untrusted servers.

````

---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt