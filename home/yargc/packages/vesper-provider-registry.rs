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

// AI endpoint metadata is shared by the Cargo control plane and the legacy
// credential frontend. Credential-only integrations such as GitHub MCP stay in
// PROVIDERS but intentionally do not appear here as model providers.
pub const PROVIDER_ENDPOINTS: &[(&str, &str, &str, &str)] = &[
    ("openai", "OpenAI", "https://api.openai.com/v1", "openai"),
    ("anthropic", "Anthropic", "https://api.anthropic.com", "anthropic"),
    ("xai", "xAI", "https://api.x.ai/v1", "xai"),
    ("openrouter", "OpenRouter", "https://openrouter.ai/api/v1", "openrouter"),
    ("google", "Google AI", "https://generativelanguage.googleapis.com", "google"),
];
