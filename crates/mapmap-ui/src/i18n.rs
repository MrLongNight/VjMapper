use fluent::{FluentBundle, FluentResource};
use fluent_langneg::{negotiate_languages, NegotiationStrategy};
use rust_embed::RustEmbed;
use unic_langid::LanguageIdentifier;

#[derive(RustEmbed)]
#[folder = "locales/"]
struct Locales;

pub struct LocaleManager {
    bundle: FluentBundle<FluentResource>,
    pub current_lang: LanguageIdentifier,
}

impl Default for LocaleManager {
    fn default() -> Self {
        Self::new("en-US")
    }
}

impl LocaleManager {
    pub fn new(lang_id: &str) -> Self {
        let lang: LanguageIdentifier = lang_id.parse().unwrap_or_else(|_| "en-US".parse().unwrap());
        let bundle = Self::load_bundle(&lang);
        Self {
            bundle,
            current_lang: lang,
        }
    }

    fn load_bundle(lang_id: &LanguageIdentifier) -> FluentBundle<FluentResource> {
        let mut bundle = FluentBundle::new(vec![lang_id.clone()]);

        // Determine available locales
        // Note: This must match folders in locales/
        let available_locales: Vec<LanguageIdentifier> =
            vec!["en".parse().unwrap(), "de".parse().unwrap()];

        // Negotiate
        let requested = vec![lang_id.clone()];
        let default = "en".parse().unwrap();
        let supported = negotiate_languages(
            &requested,
            &available_locales,
            Some(&default),
            NegotiationStrategy::Filtering,
        );

        // Load resources
        let active_lang = supported.first().unwrap();
        // Use just the language code ("en", "de") for folder names
        let lang_key = active_lang.language.as_str();

        let path = format!("{}/main.ftl", lang_key);

        // Function to load and add resource
        let load_res = |b: &mut FluentBundle<FluentResource>, p: &str| {
            if let Some(file) = Locales::get(p) {
                if let Ok(source) = String::from_utf8(file.data.into_owned()) {
                    if let Ok(resource) = FluentResource::try_new(source) {
                        let _ = b.add_resource(resource);
                        return true;
                    }
                }
            }
            false
        };

        if !load_res(&mut bundle, &path) {
            eprintln!("Locale file not found or invalid: {}", path);
            // Fallback to English if failed
            if lang_key != "en" {
                load_res(&mut bundle, "en/main.ftl");
            }
        }

        bundle
    }

    pub fn set_locale(&mut self, lang_id: &str) {
        let lang: LanguageIdentifier = lang_id.parse().unwrap_or_else(|_| "en-US".parse().unwrap());
        self.bundle = Self::load_bundle(&lang);
        self.current_lang = lang;
    }

    pub fn t(&self, key: &str) -> String {
        let pattern = match self.bundle.get_message(key) {
            Some(msg) => match msg.value() {
                Some(pattern) => pattern,
                None => return key.to_string(),
            },
            None => return key.to_string(), // Return key if missing
        };

        let mut errors = vec![];
        let value = self.bundle.format_pattern(pattern, None, &mut errors);
        value.to_string()
    }
}
