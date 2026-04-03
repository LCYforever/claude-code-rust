//! Locale Loader - Loads locale data from embedded resources

use super::{Language, Locale};
use rust_embed::Embed;

/// Embedded locale files
#[derive(Embed)]
#[folder = "locales/"]
struct LocaleAssets;

/// Loader for locale data
pub struct LocaleLoader;

impl LocaleLoader {
    /// Load locale data for a language
    pub fn load(language: Language) -> anyhow::Result<Locale> {
        let mut locale = Locale::new(language);
        
        // Load embedded locale data
        let file_name = format!("{}.ftl", language.code());
        
        if let Some(content) = LocaleAssets::get(&file_name) {
            let content = std::str::from_utf8(&content.data)?;
            Self::parse_ftl(content, &mut locale)?;
        } else {
            // Use built-in fallback if no file found
            Self::load_builtin(language, &mut locale)?;
        }
        
        Ok(locale)
    }

    /// Parse Fluent FTL format
    fn parse_ftl(content: &str, locale: &mut Locale) -> anyhow::Result<()> {
        // Simple FTL parser (in production, use fluent-bundle crate)
        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Parse key = value
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim();
                let value = line[pos + 1..].trim();
                locale.add_message(key, value);
            }
        }
        
        Ok(())
    }

    /// Load built-in locale data
    fn load_builtin(language: Language, locale: &mut Locale) -> anyhow::Result<()> {
        match language {
            Language::English => Self::load_english(locale),
            Language::Chinese => Self::load_chinese(locale),
            Language::Japanese => Self::load_japanese(locale),
            Language::Spanish => Self::load_spanish(locale),
            Language::French => Self::load_french(locale),
            Language::German => Self::load_german(locale),
            Language::Russian => Self::load_russian(locale),
            Language::Portuguese => Self::load_portuguese(locale),
            Language::Italian => Self::load_italian(locale),
            Language::Korean => Self::load_korean(locale),
        }
        Ok(())
    }

    fn load_english(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude Code"),
            ("app.welcome", "Welcome to Claude Code"),
            ("app.exit", "Exit"),
        ];
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }

    fn load_chinese(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude Code"),
            ("app.welcome", "欢迎使用 Claude Code"),
            ("app.exit", "退出"),
        ];
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }

    fn load_japanese(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude Code"),
            ("app.welcome", "Claude Codeへようこそ"),
            ("app.exit", "終了"),
        ];
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }

    fn load_spanish(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude Code"),
            ("app.welcome", "Bienvenido a Claude Code"),
            ("app.exit", "Salir"),
        ];
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }

    fn load_french(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude Code"),
            ("app.welcome", "Bienvenue dans Claude Code"),
            ("app.exit", "Quitter"),
        ];
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }

    fn load_german(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude Code"),
            ("app.welcome", "Willkommen bei Claude Code"),
            ("app.exit", "Beenden"),
        ];
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }

    fn load_russian(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude Code"),
            ("app.welcome", "Добро пожаловать в Claude Code"),
            ("app.exit", "Выход"),
        ];
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }

    fn load_portuguese(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude Code"),
            ("app.welcome", "Bem-vindo ao Claude Code"),
            ("app.exit", "Sair"),
        ];
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }

    fn load_italian(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude Code"),
            ("app.welcome", "Benvenuto in Claude Code"),
            ("app.exit", "Esci"),
        ];
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }

    fn load_korean(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude Code"),
            ("app.welcome", "Claude Code에 오신 것을 환영합니다"),
            ("app.exit", "종료"),
        ];
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }
}