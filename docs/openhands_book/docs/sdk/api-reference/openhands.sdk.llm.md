# openhands.sdk.llm

> API reference for openhands.sdk.llm module

### class ImageContent

Bases: `BaseContent`

#### Properties

* `cache_prompt`: bool
* `image_urls`: list\[str]
* `model_config`: ClassVar\[ConfigDict] = (configuration object)
  Configuration for the model, should be a dictionary conforming to \[ConfigDict]\[pydantic.config.ConfigDict].
* `type`: Literal\['image']

#### Methods

#### to\_llm\_dict()

Convert to LLM API format.

### class LLM

Bases: `BaseModel`, `RetryMixin`, `NonNativeToolCallingMixin`

Language model interface for OpenHands agents.

The LLM class provides a unified interface for interacting with various
language models through the litellm library. It handles model configuration,
API authentication,
retry logic, and tool calling capabilities.

#### Example

```pycon  theme={null}
>>> from openhands.sdk import LLM
>>> from pydantic import SecretStr
>>> llm = LLM(
...     model="claude-sonnet-4-20250514",
...     api_key=SecretStr("your-api-key"),
...     usage_id="my-agent"
... )
>>> # Use with agent or conversation
```

#### Properties

* `api_key`: str | SecretStr | None
* `api_version`: str | None
* `aws_access_key_id`: str | SecretStr | None
* `aws_region_name`: str | None
* `aws_secret_access_key`: str | SecretStr | None
* `base_url`: str | None
* `caching_prompt`: bool
* `custom_tokenizer`: str | None
* `disable_stop_word`: bool | None
* `disable_vision`: bool | None
* `drop_params`: bool
* `enable_encrypted_reasoning`: bool
* `extended_thinking_budget`: int | None
* `extra_headers`: dict\[str, str] | None
* `force_string_serializer`: bool | None
* `input_cost_per_token`: float | None
* `litellm_extra_body`: dict\[str, Any]
* `log_completions`: bool
* `log_completions_folder`: str
* `max_input_tokens`: int | None
* `max_message_chars`: int
* `max_output_tokens`: int | None
* `metrics`: [Metrics](#class-metrics)
  Get usage metrics for this LLM instance.
  * Returns:
    Metrics object containing token usage, costs, and other statistics.
* `model`: str
* `model_canonical_name`: str | None
* `model_config`: ClassVar\[ConfigDict] = (configuration object)
  Configuration for the model, should be a dictionary conforming to \[ConfigDict]\[pydantic.config.ConfigDict].
* `model_info`: dict | None
  Returns the model info dictionary.
* `modify_params`: bool
* `native_tool_calling`: bool
* `num_retries`: int
* `ollama_base_url`: str | None
* `openrouter_app_name`: str
* `openrouter_site_url`: str
* `output_cost_per_token`: float | None
* `prompt_cache_retention`: str | None
* `reasoning_effort`: Literal\['low', 'medium', 'high', 'xhigh', 'none'] | None
* `reasoning_summary`: Literal\['auto', 'concise', 'detailed'] | None
* `retry_listener`: SkipJsonSchema\[Callable\[\[int, int, BaseException | None], None] | None]
* `retry_max_wait`: int
* `retry_min_wait`: int
* `retry_multiplier`: float
* `safety_settings`: list\[dict\[str, str]] | None
* `seed`: int | None
* `stream`: bool
* `telemetry`: Telemetry
  Get telemetry handler for this LLM instance.
  * Returns:
    Telemetry object for managing logging and metrics callbacks.
* `temperature`: float | None
* `timeout`: int | None
* `top_k`: float | None
* `top_p`: float | None
* `usage_id`: str

#### Methods

#### completion()

Generate a completion from the language model.

This is the method for getting responses from the model via Completion API.
It handles message formatting, tool calling, and response processing.

* Parameters:
  * `messages` – List of conversation messages
  * `tools` – Optional list of tools available to the model
  * `_return_metrics` – Whether to return usage metrics
  * `add_security_risk_prediction` – Add security\_risk field to tool schemas
  * `on_token` – Optional callback for streaming tokens
    kwargs\* – Additional arguments passed to the LLM API
* Returns:
  LLMResponse containing the model’s response and metadata.

#### NOTE

Summary field is always added to tool schemas for transparency and
explainability of agent actions.

* Raises:
  `ValueError` – If streaming is requested (not supported).

#### format\_messages\_for\_llm()

Formats Message objects for LLM consumption.

#### format\_messages\_for\_responses()

Prepare (instructions, input\[]) for the OpenAI Responses API.

* Skips prompt caching flags and string serializer concerns
* Uses Message.to\_responses\_value to get either instructions (system)

  or input items (others)
* Concatenates system instructions into a single instructions string

#### get\_token\_count()

#### is\_caching\_prompt\_active()

Check if prompt caching is supported and enabled for current model.

* Returns:
  True if prompt caching is supported and enabled for the given
  : model.
* Return type:
  boolean

#### classmethod load\_from\_env()

#### classmethod load\_from\_json()

#### model\_post\_init()

This function is meant to behave like a BaseModel method to initialise private attributes.

It takes context as an argument since that’s what pydantic-core passes when calling it.

* Parameters:
  * `self` – The BaseModel instance.
  * `context` – The context.

#### responses()

Alternative invocation path using OpenAI Responses API via LiteLLM.

Maps Message\[] -> (instructions, input\[]) and returns LLMResponse.

* Parameters:
  * `messages` – List of conversation messages
  * `tools` – Optional list of tools available to the model
  * `include` – Optional list of fields to include in response
  * `store` – Whether to store the conversation
  * `_return_metrics` – Whether to return usage metrics
  * `add_security_risk_prediction` – Add security\_risk field to tool schemas
  * `on_token` – Optional callback for streaming tokens (not yet supported)
    kwargs\* – Additional arguments passed to the API

#### NOTE

Summary field is always added to tool schemas for transparency and
explainability of agent actions.

#### restore\_metrics()

#### uses\_responses\_api()

Whether this model uses the OpenAI Responses API path.

#### vision\_is\_active()

### class LLMRegistry

Bases: `object`

A minimal LLM registry for managing LLM instances by usage ID.

This registry provides a simple way to manage multiple LLM instances,
avoiding the need to recreate LLMs with the same configuration.

#### Properties

* `registry_id`: str
* `retry_listener`: Callable\[\[int, int], None] | None
* `subscriber`: Callable\[\[[RegistryEvent](#class-registryevent)], None] | None
* `usage_to_llm`: dict\[str, [LLM](#class-llm)]
  Access the internal usage-ID-to-LLM mapping.

#### Methods

#### **init**()

Initialize the LLM registry.

* Parameters:
  `retry_listener` – Optional callback for retry events.

#### add()

Add an LLM instance to the registry.

* Parameters:
  `llm` – The LLM instance to register.
* Raises:
  `ValueError` – If llm.usage\_id already exists in the registry.

#### get()

Get an LLM instance from the registry.

* Parameters:
  `usage_id` – Unique identifier for the LLM usage slot.
* Returns:
  The LLM instance.
* Raises:
  `KeyError` – If usage\_id is not found in the registry.

#### list\_usage\_ids()

List all registered usage IDs.

#### notify()

Notify subscribers of registry events.

* Parameters:
  `event` – The registry event to notify about.

#### subscribe()

Subscribe to registry events.

* Parameters:
  `callback` – Function to call when LLMs are created or updated.

### class LLMResponse

Bases: `BaseModel`

Result of an LLM completion request.

This type provides a clean interface for LLM completion results, exposing
only OpenHands-native types to consumers while preserving access to the
raw LiteLLM response for internal use.

#### Properties

* `id`: str
  Get the response ID from the underlying LLM response.
  This property provides a clean interface to access the response ID,
  supporting both completion mode (ModelResponse) and response API modes
  (ResponsesAPIResponse).
  * Returns:
    The response ID from the LLM response
* `message`: [Message](#class-message)
* `metrics`: [MetricsSnapshot](#class-metricssnapshot)
* `model_config`: ClassVar\[ConfigDict] = (configuration object)
  Configuration for the model, should be a dictionary conforming to \[ConfigDict]\[pydantic.config.ConfigDict].
* `raw_response`: ModelResponse | ResponsesAPIResponse

#### Methods

#### message

The completion message converted to OpenHands Message type

* Type:
  [openhands.sdk.llm.message.Message](#class-message)

#### metrics

Snapshot of metrics from the completion request

* Type:
  [openhands.sdk.llm.utils.metrics.MetricsSnapshot](#class-metricssnapshot)

#### raw\_response

The original LiteLLM response (ModelResponse or
ResponsesAPIResponse) for internal use

* Type:
  litellm.types.utils.ModelResponse | litellm.types.llms.openai.ResponsesAPIResponse

### class Message

Bases: `BaseModel`

#### Properties

* `cache_enabled`: bool
* `contains_image`: bool
* `content`: Sequence\[[TextContent](#class-textcontent) | [ImageContent](#class-imagecontent)]
* `force_string_serializer`: bool
* `function_calling_enabled`: bool
* `model_config`: ClassVar\[ConfigDict] = (configuration object)
  Configuration for the model, should be a dictionary conforming to \[ConfigDict]\[pydantic.config.ConfigDict].
* `name`: str | None
* `reasoning_content`: str | None
* `responses_reasoning_item`: [ReasoningItemModel](#class-reasoningitemmodel) | None
* `role`: Literal\['user', 'system', 'assistant', 'tool']
* `send_reasoning_content`: bool
* `thinking_blocks`: Sequence\[[ThinkingBlock](#class-thinkingblock) | [RedactedThinkingBlock](#class-redactedthinkingblock)]
* `tool_call_id`: str | None
* `tool_calls`: list\[[MessageToolCall](#class-messagetoolcall)] | None
* `vision_enabled`: bool

#### Methods

#### classmethod from\_llm\_chat\_message()

Convert a LiteLLMMessage (Chat Completions) to our Message class.

Provider-agnostic mapping for reasoning:

* Prefer message.reasoning\_content if present (LiteLLM normalized field)
* Extract thinking\_blocks from content array (Anthropic-specific)

#### classmethod from\_llm\_responses\_output()

Convert OpenAI Responses API output items into a single assistant Message.

Policy (non-stream):

* Collect assistant text by concatenating output\_text parts from message items
* Normalize function\_call items to MessageToolCall list

#### to\_chat\_dict()

Serialize message for OpenAI Chat Completions.

Chooses the appropriate content serializer and then injects threading keys:

* Assistant tool call turn: role == “assistant” and self.tool\_calls
* Tool result turn: role == “tool” and self.tool\_call\_id (with name)

#### to\_responses\_dict()

Serialize message for OpenAI Responses (input parameter).

Produces a list of “input” items for the Responses API:

* system: returns \[], system content is expected in ‘instructions’
* user: one ‘message’ item with content parts -> input\_text / input\_image
  (when vision enabled)
* assistant: emits prior assistant content as input\_text,
  and function\_call items for tool\_calls
* tool: emits function\_call\_output items (one per TextContent)
  with matching call\_id

#### to\_responses\_value()

Return serialized form.

Either an instructions string (for system) or input items (for other roles).

### class MessageToolCall

Bases: `BaseModel`

Transport-agnostic tool call representation.

One canonical id is used for linking across actions/observations and
for Responses function\_call\_output call\_id.

#### Properties

* `arguments`: str
* `id`: str
* `model_config`: ClassVar\[ConfigDict] = (configuration object)
  Configuration for the model, should be a dictionary conforming to \[ConfigDict]\[pydantic.config.ConfigDict].
* `name`: str
* `origin`: Literal\['completion', 'responses']

#### Methods

#### classmethod from\_chat\_tool\_call()

Create a MessageToolCall from a Chat Completions tool call.

#### classmethod from\_responses\_function\_call()

Create a MessageToolCall from a typed OpenAI Responses function\_call item.

Note: OpenAI Responses function\_call.arguments is already a JSON string.

#### to\_chat\_dict()

Serialize to OpenAI Chat Completions tool\_calls format.

#### to\_responses\_dict()

Serialize to OpenAI Responses ‘function\_call’ input item format.

### class Metrics

Bases: [`MetricsSnapshot`](#class-metricssnapshot)

Metrics class can record various metrics during running and evaluation.
We track:

* accumulated\_cost and costs
* max\_budget\_per\_task (budget limit)
* A list of ResponseLatency
* A list of TokenUsage (one per call).

#### Properties

* `costs`: list\[Cost]
* `model_config`: ClassVar\[ConfigDict] = (configuration object)
  Configuration for the model, should be a dictionary conforming to \[ConfigDict]\[pydantic.config.ConfigDict].
* `response_latencies`: list\[ResponseLatency]
* `token_usages`: list\[TokenUsage]

#### Methods

#### add\_cost()

#### add\_response\_latency()

#### add\_token\_usage()

Add a single usage record.

#### deep\_copy()

Create a deep copy of the Metrics object.

#### diff()

Calculate the difference between current metrics and a baseline.

This is useful for tracking metrics for specific operations like delegates.

* Parameters:
  `baseline` – A metrics object representing the baseline state
* Returns:
  A new Metrics object containing only the differences since the baseline

#### get()

Return the metrics in a dictionary.

#### get\_snapshot()

Get a snapshot of the current metrics without the detailed lists.

#### initialize\_accumulated\_token\_usage()

#### log()

Log the metrics.

#### merge()

Merge ‘other’ metrics into this one.

#### classmethod validate\_accumulated\_cost()

### class MetricsSnapshot

Bases: `BaseModel`

A snapshot of metrics at a point in time.

Does not include lists of individual costs, latencies, or token usages.

#### Properties

* `accumulated_cost`: float
* `accumulated_token_usage`: TokenUsage | None
* `max_budget_per_task`: float | None
* `model_config`: ClassVar\[ConfigDict] = (configuration object)
  Configuration for the model, should be a dictionary conforming to \[ConfigDict]\[pydantic.config.ConfigDict].
* `model_name`: str

### class ReasoningItemModel

Bases: `BaseModel`

OpenAI Responses reasoning item (non-stream, subset we consume).

Do not log or render encrypted\_content.

#### Properties

* `content`: list\[str] | None
* `encrypted_content`: str | None
* `id`: str | None
* `model_config`: ClassVar\[ConfigDict] = (configuration object)
  Configuration for the model, should be a dictionary conforming to \[ConfigDict]\[pydantic.config.ConfigDict].
* `status`: str | None
* `summary`: list\[str]

### class RedactedThinkingBlock

Bases: `BaseModel`

Redacted thinking block for previous responses without extended thinking.

This is used as a placeholder for assistant messages that were generated
before extended thinking was enabled.

#### Properties

* `data`: str
* `model_config`: ClassVar\[ConfigDict] = (configuration object)
  Configuration for the model, should be a dictionary conforming to \[ConfigDict]\[pydantic.config.ConfigDict].
* `type`: Literal\['redacted\_thinking']

### class RegistryEvent

Bases: `BaseModel`

#### Properties

* `llm`: [LLM](#class-llm)
* `model_config`: ClassVar\[ConfigDict] = (configuration object)
  Configuration for the model, should be a dictionary conforming to \[ConfigDict]\[pydantic.config.ConfigDict].

### class RouterLLM

Bases: [`LLM`](#class-llm)

Base class for multiple LLM acting as a unified LLM.
This class provides a foundation for implementing model routing by
inheriting from LLM, allowing routers to work with multiple underlying
LLM models while presenting a unified LLM interface to consumers.
Key features:

* Works with multiple LLMs configured via llms\_for\_routing
* Delegates all other operations/properties to the selected LLM
* Provides routing interface through select\_llm() method

#### Properties

* `active_llm`: [LLM](#class-llm) | None
* `llms_for_routing`: dict\[str, [LLM](#class-llm)]
* `model_config`: ClassVar\[ConfigDict] = (configuration object)
  Configuration for the model, should be a dictionary conforming to \[ConfigDict]\[pydantic.config.ConfigDict].
* `router_name`: str

#### Methods

#### completion()

This method intercepts completion calls and routes them to the appropriate
underlying LLM based on the routing logic implemented in select\_llm().

* Parameters:
  * `messages` – List of conversation messages
  * `tools` – Optional list of tools available to the model
  * `return_metrics` – Whether to return usage metrics
  * `add_security_risk_prediction` – Add security\_risk field to tool schemas
  * `on_token` – Optional callback for streaming tokens
    kwargs\* – Additional arguments passed to the LLM API

#### NOTE

Summary field is always added to tool schemas for transparency and
explainability of agent actions.

#### model\_post\_init()

This function is meant to behave like a BaseModel method to initialise private attributes.

It takes context as an argument since that’s what pydantic-core passes when calling it.

* Parameters:
  * `self` – The BaseModel instance.
  * `context` – The context.

#### abstractmethod select\_llm()

Select which LLM to use based on messages and events.

This method implements the core routing logic for the RouterLLM.
Subclasses should analyze the provided messages to determine which
LLM from llms\_for\_routing is most appropriate for handling the request.

* Parameters:
  `messages` – List of messages in the conversation that can be used
  to inform the routing decision.
* Returns:
  The key/name of the LLM to use from llms\_for\_routing dictionary.

#### classmethod set\_placeholder\_model()

Guarantee model exists before LLM base validation runs.

#### classmethod validate\_llms\_not\_empty()

### class TextContent

Bases: `BaseContent`

#### Properties

* `cache_prompt`: bool
* `enable_truncation`: bool
* `model_config`: ClassVar\[ConfigDict] = (configuration object)
  Configuration for the model, should be a dictionary conforming to \[ConfigDict]\[pydantic.config.ConfigDict].
* `text`: str
* `type`: Literal\['text']

#### Methods

#### to\_llm\_dict()

Convert to LLM API format.

### class ThinkingBlock

Bases: `BaseModel`

Anthropic thinking block for extended thinking feature.

This represents the raw thinking blocks returned by Anthropic models
when extended thinking is enabled. These blocks must be preserved
and passed back to the API for tool use scenarios.

#### Properties

* `model_config`: ClassVar\[ConfigDict] = (configuration object)
  Configuration for the model, should be a dictionary conforming to \[ConfigDict]\[pydantic.config.ConfigDict].
* `signature`: str | None
* `thinking`: str
* `type`: Literal\['thinking']


---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt