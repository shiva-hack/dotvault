use serde::{Deserialize, Serialize};

/// A parsed key-value pair from a .env file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedVar {
    pub key: String,
    pub value: String,
    pub comment: Option<String>,
    pub line_number: u32,
}

/// Parsed tier information from a .env filename
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierInfo {
    pub tier: String,
    pub depth: u8,
    pub sub_variant: Option<String>,
}

/// Parse a .env file's contents into key-value pairs.
/// Handles: KEY=value, KEY="quoted value", KEY='single quoted',
/// export KEY=value, multiline values, and comments.
pub fn parse_env_contents(contents: &str) -> Vec<ParsedVar> {
    let mut vars = Vec::new();
    let mut lines = contents.lines().enumerate().peekable();
    let mut pending_comment: Option<String> = None;

    while let Some((line_idx, line)) = lines.next() {
        let trimmed = line.trim();

        // Empty line resets pending comment
        if trimmed.is_empty() {
            pending_comment = None;
            continue;
        }

        // Comment line
        if trimmed.starts_with('#') {
            pending_comment = Some(trimmed.to_string());
            continue;
        }

        // Strip optional `export ` prefix
        let effective = if trimmed.starts_with("export ") {
            &trimmed[7..]
        } else {
            trimmed
        };

        // Find the = separator
        let Some(eq_pos) = effective.find('=') else {
            pending_comment = None;
            continue;
        };

        let key = effective[..eq_pos].trim().to_string();
        let raw_value = effective[eq_pos + 1..].trim();

        if key.is_empty() {
            pending_comment = None;
            continue;
        }

        // Parse value (handle quotes and multiline)
        let value = if raw_value.starts_with('"') {
            parse_double_quoted(raw_value, &mut lines)
        } else if raw_value.starts_with('\'') {
            parse_single_quoted(raw_value)
        } else {
            // Unquoted: strip inline comments
            raw_value.split('#').next().unwrap_or("").trim().to_string()
        };

        vars.push(ParsedVar {
            key,
            value,
            comment: pending_comment.take(),
            line_number: (line_idx + 1) as u32,
        });
    }

    vars
}

fn parse_double_quoted(
    raw: &str,
    lines: &mut std::iter::Peekable<std::iter::Enumerate<std::str::Lines<'_>>>,
) -> String {
    // Remove opening quote
    let mut content = raw[1..].to_string();

    // Check if closing quote is on this line
    if let Some(end) = content.find('"') {
        return content[..end].replace("\\n", "\n").replace("\\\"", "\"");
    }

    // Multiline: consume until closing quote
    while let Some((_, next_line)) = lines.next() {
        content.push('\n');
        if let Some(end) = next_line.find('"') {
            content.push_str(&next_line[..end]);
            break;
        } else {
            content.push_str(next_line);
        }
    }

    content.replace("\\n", "\n").replace("\\\"", "\"")
}

fn parse_single_quoted(raw: &str) -> String {
    let content = &raw[1..];
    if let Some(end) = content.find('\'') {
        content[..end].to_string()
    } else {
        content.to_string()
    }
}

/// Parse the filename to determine the environment tier and depth.
/// Rules from PRD Section 4:
///   .env                  → depth 0 → "base"
///   .env.local            → depth 1 → "local"
///   .env.development      → depth 1 → "development"
///   .env.production.local → depth 2 → "production.local"
pub fn parse_tier(filename: &str) -> TierInfo {
    // Strip the leading ".env" prefix
    let suffix = if filename == ".env" {
        ""
    } else if let Some(s) = filename.strip_prefix(".env.") {
        s
    } else if let Some(s) = filename.strip_prefix(".env") {
        s
    } else {
        return TierInfo {
            tier: "unknown".to_string(),
            depth: 0,
            sub_variant: None,
        };
    };

    if suffix.is_empty() {
        return TierInfo {
            tier: "base".to_string(),
            depth: 0,
            sub_variant: None,
        };
    }

    let parts: Vec<&str> = suffix.split('.').collect();
    let depth = parts.len() as u8;

    if depth == 1 {
        TierInfo {
            tier: parts[0].to_string(),
            depth: 1,
            sub_variant: None,
        }
    } else {
        // depth 2+: tier is the first part, sub_variant is the rest
        TierInfo {
            tier: parts[0].to_string(),
            depth,
            sub_variant: Some(parts[1..].join(".")),
        }
    }
}

/// Detect the ecosystem from project marker files
pub fn detect_ecosystem(project_path: &std::path::Path) -> Option<String> {
    let markers = [
        ("package.json", "node"),
        ("Cargo.toml", "rust"),
        ("pyproject.toml", "python"),
        ("setup.py", "python"),
        ("go.mod", "go"),
        ("Gemfile", "ruby"),
        ("composer.json", "php"),
    ];

    for (marker, ecosystem) in &markers {
        if project_path.join(marker).exists() {
            return Some(ecosystem.to_string());
        }
    }

    // Check for .NET markers
    if std::fs::read_dir(project_path)
        .ok()
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.ends_with(".sln") || name.ends_with(".csproj")
            })
        })
        .unwrap_or(false)
    {
        return Some("dotnet".to_string());
    }

    if project_path.join(".git").exists() {
        return Some("unknown".to_string());
    }

    None
}

/// Extract project name from manifest files
pub fn extract_project_name(project_path: &std::path::Path) -> String {
    // Try package.json
    if let Ok(contents) = std::fs::read_to_string(project_path.join("package.json")) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
            if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                return name.to_string();
            }
        }
    }

    // Try Cargo.toml (basic parse)
    if let Ok(contents) = std::fs::read_to_string(project_path.join("Cargo.toml")) {
        for line in contents.lines() {
            if let Some(rest) = line.strip_prefix("name") {
                let rest = rest.trim();
                if let Some(rest) = rest.strip_prefix('=') {
                    let name = rest.trim().trim_matches('"').trim_matches('\'');
                    if !name.is_empty() {
                        return name.to_string();
                    }
                }
            }
        }
    }

    // Try pyproject.toml
    if let Ok(contents) = std::fs::read_to_string(project_path.join("pyproject.toml")) {
        for line in contents.lines() {
            if let Some(rest) = line.strip_prefix("name") {
                let rest = rest.trim();
                if let Some(rest) = rest.strip_prefix('=') {
                    let name = rest.trim().trim_matches('"').trim_matches('\'');
                    if !name.is_empty() {
                        return name.to_string();
                    }
                }
            }
        }
    }

    // Fallback to directory name
    project_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown Project".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tier_base() {
        let t = parse_tier(".env");
        assert_eq!(t.tier, "base");
        assert_eq!(t.depth, 0);
    }

    #[test]
    fn test_parse_tier_depth1() {
        let t = parse_tier(".env.local");
        assert_eq!(t.tier, "local");
        assert_eq!(t.depth, 1);
    }

    #[test]
    fn test_parse_tier_depth2() {
        let t = parse_tier(".env.production.local");
        assert_eq!(t.tier, "production");
        assert_eq!(t.depth, 2);
        assert_eq!(t.sub_variant.as_deref(), Some("local"));
    }

    #[test]
    fn test_parse_basic_env() {
        let content = r#"
# Database config
DATABASE_URL=postgres://localhost:5432/mydb
API_KEY="my-secret-key"
DEBUG='true'
export PORT=3000
"#;
        let vars = parse_env_contents(content);
        assert_eq!(vars.len(), 4);
        assert_eq!(vars[0].key, "DATABASE_URL");
        assert_eq!(vars[0].value, "postgres://localhost:5432/mydb");
        assert!(vars[0].comment.is_some());
        assert_eq!(vars[1].key, "API_KEY");
        assert_eq!(vars[1].value, "my-secret-key");
        assert_eq!(vars[2].key, "DEBUG");
        assert_eq!(vars[2].value, "true");
        assert_eq!(vars[3].key, "PORT");
        assert_eq!(vars[3].value, "3000");
    }
}
