//! Semantic colors and spacing shared by every framework control.
use gpui::{App, Global, Hsla, Pixels, SharedString, WindowAppearance, px, rgb};

/// Select an explicit theme, or follow the system appearance at startup.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemeMode {
    /// Light surfaces and dark text.
    Light,
    /// Dark surfaces and light text.
    #[default]
    Dark,
    /// Resolve the operating system appearance when applied.
    System,
}

/// Semantic color roles. Customize these through [`Theme::apply`].
#[derive(Clone, Copy)]
pub struct Colors {
    pub background: Hsla,
    pub surface: Hsla,
    pub text: Hsla,
    pub muted: Hsla,
    pub border: Hsla,
    pub accent: Hsla,
    pub accent_text: Hsla,
    pub focus: Hsla,
    pub selection: Hsla,
    pub danger: Hsla,
}

/// Density-independent logical pixels; customize once rather than per control.
#[derive(Clone, Copy)]
pub struct SpacingTokens {
    pub xs: Pixels,
    pub sm: Pixels,
    pub md: Pixels,
    pub lg: Pixels,
}

/// Shared spacing steps used by layout builders.
#[derive(Clone, Copy, Default)]
pub enum Spacing {
    None,
    Xs,
    Sm,
    #[default]
    Md,
    Lg,
}

impl Spacing {
    pub(crate) fn pixels(self, theme: &Theme) -> Pixels {
        match self {
            Self::None => px(0.),
            Self::Xs => theme.spacing.xs,
            Self::Sm => theme.spacing.sm,
            Self::Md => theme.spacing.md,
            Self::Lg => theme.spacing.lg,
        }
    }
}

/// Application-wide tokens. Fonts are system fonts; no font assets are bundled.
#[derive(Clone)]
pub struct Theme {
    pub colors: Colors,
    pub spacing: SpacingTokens,
    pub font_size: Pixels,
    pub font_family: SharedString,
    pub radius: Pixels,
    pub mode: ThemeMode,
}

impl Global for Theme {}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// A dark theme with semantic focus and selection colors.
    pub fn dark() -> Self {
        Self::palette(
            ThemeMode::Dark,
            Colors {
                background: rgb(0x15181e).into(),
                surface: rgb(0x202630).into(),
                text: rgb(0xeff2f7).into(),
                muted: rgb(0xafbacb).into(),
                border: rgb(0x465267).into(),
                accent: rgb(0x315cba).into(),
                accent_text: rgb(0xffffff).into(),
                focus: rgb(0x93baff).into(),
                selection: rgb(0x3e567b).into(),
                danger: rgb(0xffa4a4).into(),
            },
        )
    }

    /// A light theme with the same component roles.
    pub fn light() -> Self {
        Self::palette(
            ThemeMode::Light,
            Colors {
                background: rgb(0xf3f5f9).into(),
                surface: rgb(0xffffff).into(),
                text: rgb(0x1a2535).into(),
                muted: rgb(0x4c5d72).into(),
                border: rgb(0x8795a8).into(),
                accent: rgb(0x2454b2).into(),
                accent_text: rgb(0xffffff).into(),
                focus: rgb(0x164bbb).into(),
                selection: rgb(0xc6dcff).into(),
                danger: rgb(0xa42626).into(),
            },
        )
    }

    fn palette(mode: ThemeMode, colors: Colors) -> Self {
        Self {
            mode,
            colors,
            spacing: SpacingTokens {
                xs: px(4.),
                sm: px(8.),
                md: px(16.),
                lg: px(24.),
            },
            font_family: ".SystemUIFont".into(),
            font_size: px(15.),
            radius: px(6.),
        }
    }

    /// Read the initialized theme inside a view or event callback.
    pub fn get(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    /// Apply tokens and repaint existing windows.
    pub fn apply(self, cx: &mut App) {
        cx.set_window_appearance(Some(if self.mode == ThemeMode::Light {
            WindowAppearance::Light
        } else {
            WindowAppearance::Dark
        }));
        cx.set_global(self);
        cx.refresh_windows();
    }

    /// Change the theme. `System` resolves current appearance, not future OS changes.
    pub fn set_mode(mode: ThemeMode, cx: &mut App) {
        let light = mode == ThemeMode::Light
            || (mode == ThemeMode::System
                && matches!(
                    cx.window_appearance(),
                    WindowAppearance::Light | WindowAppearance::VibrantLight
                ));
        if light {
            Self::light().apply(cx);
        } else {
            Self::dark().apply(cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn themes_share_spacing_but_not_surfaces() {
        let dark = Theme::dark();
        let light = Theme::light();
        assert_eq!(dark.spacing.md, light.spacing.md);
        assert_ne!(dark.colors.background, light.colors.background);
        assert_ne!(dark.colors.text, dark.colors.surface);
        assert!(dark.spacing.xs < dark.spacing.sm && dark.spacing.sm < dark.spacing.md);
    }
}
