// Canonical API-key/token provider registry for Vesper Settings → AI.
// Keep credential metadata here so UI/control-plane consumers do not duplicate it.
pub const PROVIDERS: &[(&str, &str, &str)] = &[
    ("openai", "OpenAI", "OPENAI_API_KEY"),
    ("anthropic", "Anthropic", "ANTHROPIC_API_KEY"),
    ("xai", "xAI", "XAI_API_KEY"),
    ("openrouter", "OpenRouter", "OPENROUTER_API_KEY"),
    ("google", "Google AI", "GEMINI_API_KEY"),
    ("github", "GitHub MCP", "GITHUB_PERSONAL_ACCESS_TOKEN"),
];
