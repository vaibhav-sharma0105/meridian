# ai-chat Specification

## Purpose
TBD - created by archiving change document-existing-system. Update Purpose after archive.
## Requirements
### Requirement: Multi-Provider AI Support

The system SHALL support multiple AI providers: OpenAI, Anthropic, Google Gemini, Groq, LiteLLM, Ollama, and custom OpenAI-compatible endpoints.

#### Scenario: Configure OpenAI
- **WHEN** user selects OpenAI provider and enters API key
- **THEN** system validates connection and saves configuration

#### Scenario: Configure Anthropic
- **WHEN** user selects Anthropic provider and enters API key
- **THEN** system uses Anthropic API format for chat completions

#### Scenario: Configure Ollama
- **WHEN** user selects Ollama provider
- **THEN** system connects to local Ollama instance (no API key required)

#### Scenario: Configure custom endpoint
- **WHEN** user selects Custom provider and enters endpoint URL
- **THEN** system uses provided URL for OpenAI-compatible API calls

### Requirement: Dynamic Model Selection

The system SHALL fetch available models from the configured provider and allow user to select a model.

#### Scenario: Fetch available models
- **WHEN** user opens model selector
- **THEN** system queries provider API for available models

#### Scenario: Select model
- **WHEN** user selects a model from dropdown
- **THEN** system uses selected model for chat completions

#### Scenario: Default models
- **WHEN** no model is selected
- **THEN** system uses provider default (gpt-4o-mini for OpenAI, claude-sonnet-4-6 for Anthropic, etc.)

### Requirement: Project-Aware Chat

The system SHALL provide project context to AI during chat conversations. Context includes open tasks, recent completed tasks, recent meetings, and relevant documents.

#### Scenario: Include task context
- **WHEN** user sends chat message
- **THEN** system includes up to 50 open tasks and 20 recently completed tasks in context

#### Scenario: Include meeting context
- **WHEN** user sends chat message
- **THEN** system includes up to 3 recent meetings with AI summaries in context

#### Scenario: Include document context
- **WHEN** user sends chat message
- **THEN** system searches documents for relevant chunks and includes in context

### Requirement: Chat Message Flow

The system SHALL support conversational chat with message history persistence.

#### Scenario: Send message
- **WHEN** user sends message
- **THEN** system sends message with context to AI provider and displays response

#### Scenario: Conversation history
- **WHEN** chat continues
- **THEN** system includes last 10 messages in conversation context

#### Scenario: Persist chat history
- **WHEN** chat message exchange completes
- **THEN** system saves messages to chat_history table

### Requirement: Chat UI Features

The system SHALL provide chat panel UI with message display, input field, suggested prompts, and copy-to-clipboard.

#### Scenario: Display messages
- **WHEN** chat has messages
- **THEN** system renders user and assistant messages with markdown formatting

#### Scenario: Suggested prompts
- **WHEN** chat is empty
- **THEN** system displays suggested prompts: "Summarize open tasks", "What's blocking progress?", etc.

#### Scenario: Copy response
- **WHEN** user clicks copy button on assistant message
- **THEN** system copies message content to clipboard

#### Scenario: Message length limit
- **WHEN** user enters message over 4000 characters
- **THEN** system prevents submission and shows limit warning

### Requirement: Prompt Templates

The system SHALL provide pre-built prompt templates for common outputs: status updates, Jira feature requests, meeting agendas, project status.

#### Scenario: List templates
- **WHEN** user opens template selector
- **THEN** system displays available templates with descriptions

#### Scenario: Use template
- **WHEN** user selects template
- **THEN** system populates input with template prompt including project variables

#### Scenario: Generate from template
- **WHEN** template is submitted
- **THEN** AI generates output following template format

### Requirement: AI Connection Verification

The system SHALL allow users to verify AI provider connection before saving configuration.

#### Scenario: Verify connection
- **WHEN** user clicks "Test Connection"
- **THEN** system sends test request to provider and reports success/failure

#### Scenario: Connection failure
- **WHEN** connection test fails
- **THEN** system displays error message with details

### Requirement: AI Settings Persistence

The system SHALL persist AI settings including provider, API key, model, and endpoint URL.

#### Scenario: Save settings
- **WHEN** user saves AI settings
- **THEN** system stores in app_settings table (not OS keychain)

#### Scenario: Load settings
- **WHEN** app starts
- **THEN** system loads AI settings and configures provider

