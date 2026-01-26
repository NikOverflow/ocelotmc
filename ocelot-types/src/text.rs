use serde::{Deserialize, Serialize};

use crate::{ResourceLocation, text::private::ComponentAccess};

mod private {
    use crate::text::TextComponent;

    pub trait ComponentAccess {
        fn access_component(&mut self) -> &mut TextComponent;
    }
}

pub trait GenericComponent: ComponentAccess + Sized {
    fn color(mut self, color: impl Into<String>) -> Self {
        self.access_component().color = Some(color.into());
        self
    }
    fn font(mut self, font: ResourceLocation) -> Self {
        self.access_component().font = Some(font);
        self
    }
    fn bold(mut self, bold: bool) -> Self {
        self.access_component().bold = Some(bold);
        self
    }
    fn italic(mut self, italic: bool) -> Self {
        self.access_component().italic = Some(italic);
        self
    }
    fn underlined(mut self, underlined: bool) -> Self {
        self.access_component().underlined = Some(underlined);
        self
    }
    fn strikethrough(mut self, strikethrough: bool) -> Self {
        self.access_component().strikethrough = Some(strikethrough);
        self
    }
    fn obfuscated(mut self, obfuscated: bool) -> Self {
        self.access_component().obfuscated = Some(obfuscated);
        self
    }
    fn shadow_color(mut self, shadow_color: ShadowColor) -> Self {
        self.access_component().shadow_color = Some(shadow_color);
        self
    }
    fn insertion(mut self, insertion: impl Into<String>) -> Self {
        self.access_component().insertion = Some(insertion.into());
        self
    }
    fn click_event(mut self, click_event: ClickEvent) -> Self {
        self.access_component().click_event = Some(click_event);
        self
    }
    fn hover_event(mut self, hover_event: HoverEvent) -> Self {
        self.access_component().hover_event = Some(hover_event);
        self
    }
}
#[derive(Default, Serialize, Deserialize)]
pub struct TextComponent {
    #[serde(flatten)]
    content: Content,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    extra: Vec<TextComponent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    font: Option<ResourceLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    underlined: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strikethrough: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    obfuscated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shadow_color: Option<ShadowColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    insertion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    click_event: Option<ClickEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hover_event: Option<HoverEvent>,
}
impl TextComponent {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: Content::Text { text: text.into() },
            ..Default::default()
        }
    }
    pub fn translate(key: impl Into<String>) -> TranslatableBuilder {
        TranslatableBuilder {
            key: key.into(),
            ..Default::default()
        }
    }
    pub fn keybind(keybind: impl Into<String>) -> Self {
        Self {
            content: Content::Keybind {
                keybind: keybind.into(),
            },
            ..Default::default()
        }
    }
}
impl ComponentAccess for TextComponent {
    fn access_component(&mut self) -> &mut TextComponent {
        self
    }
}
impl GenericComponent for TextComponent {}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    Text {
        text: String,
    },
    Translatable {
        translate: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        fallback: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        with: Option<Vec<TextComponent>>,
    },
    Keybind {
        keybind: String,
    },
}
impl Default for Content {
    fn default() -> Self {
        Self::Text {
            text: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ShadowColor {
    Int(i32),
    FloatArray([f32; 4]),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ClickEvent {
    OpenUrl {
        url: String,
    },
    OpenFile {
        path: String,
    },
    RunCommand {
        command: String,
    },
    SuggestCommand {
        command: String,
    },
    ChangePage {
        page: i32,
    },
    CopyToClipboard {
        value: String,
    },
    //ShowDialog {},
    Custom {
        id: ResourceLocation,
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<serde_json::Value>,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum HoverEvent {
    ShowText { value: Box<TextComponent> },
}

#[derive(Default)]
pub struct TranslatableBuilder {
    base: TextComponent,
    key: String,
    fallback: Option<String>,
    with: Option<Vec<TextComponent>>,
}
impl TranslatableBuilder {
    pub fn with_fallback(mut self, fallback: impl Into<String>) -> Self {
        self.fallback = Some(fallback.into());
        self
    }
    pub fn with_args(mut self, args: Vec<TextComponent>) -> Self {
        self.with = Some(args);
        self
    }
    pub fn build(mut self) -> TextComponent {
        self.base.content = Content::Translatable {
            translate: self.key,
            fallback: self.fallback,
            with: self.with,
        };
        self.base
    }
}
impl ComponentAccess for TranslatableBuilder {
    fn access_component(&mut self) -> &mut TextComponent {
        &mut self.base
    }
}
impl GenericComponent for TranslatableBuilder {}
