//! Pure theme palette helpers shared by the splash application.

pub const DEFAULT_BACKGROUND: &str = "#101218";
pub const DEFAULT_FOREGROUND: &str = "#f4f4f4";
pub const DEFAULT_ACCENT: &str = "#7aa2f7";

pub fn theme_colors(theme: &str) -> Option<(&'static str, &'static str, &'static str)> {
  let normalized = theme.trim().to_ascii_lowercase().replace(['_', ' '], "-");
  let normalized = normalized.strip_prefix("argvus-").unwrap_or(&normalized);
  let normalized = normalized.strip_suffix("-float").unwrap_or(normalized);
  match normalized {
    "onedark" | "one-dark" => Some(("#282C34", "#ABB2BF", "#61AFEF")),
    "dracula" => Some(("#282A36", "#F8F8F2", "#BD93F9")),
    "dark-aether" => Some(("#191b27", "#3590bd", DEFAULT_ACCENT)),
    "dark-silver" => Some(("#595959", "#121518", "#121518")),
    "rosepine" => Some(("#191724", "#E0DEF4", "#C4A7E7")),
    "dark-slate" => Some(("#3b4352", "#7391a5", DEFAULT_ACCENT)),
    "dark-universe" => Some(("#000000", "#ffffff", DEFAULT_ACCENT)),
    "gruvbox-dark-medium" => Some(("#282828", "#EBDBB2", "#D79921")),
    "light-veil" => Some(("#ffffff", "#000000", DEFAULT_ACCENT)),
    "frost" => Some(("#f6f8fa", "#24292f", "#0969da")),
    "tokyo-night" => Some(("#1A1B26", "#C0CAF5", "#7AA2F7")),
    "catppuccin-latte" => Some(("#EFF1F5", "#4C4F69", "#1E66F5")),
    _ => None,
  }
}

pub fn valid_color(value: &str, fallback: &str) -> String {
  let valid = matches!(value.len(), 4 | 7 | 9)
    && value.starts_with('#')
    && value[1..]
      .chars()
      .all(|character| character.is_ascii_hexdigit());
  if valid {
    value.to_owned()
  } else {
    fallback.to_owned()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn accepts_supported_hex_colors() {
    assert_eq!(valid_color("#abc", DEFAULT_BACKGROUND), "#abc");
    assert_eq!(valid_color("#AABBCC", DEFAULT_BACKGROUND), "#AABBCC");
    assert_eq!(valid_color("#AABBCCDD", DEFAULT_BACKGROUND), "#AABBCCDD");
  }

  #[test]
  fn rejects_css_injection_and_bad_lengths() {
    assert_eq!(valid_color("red", DEFAULT_BACKGROUND), DEFAULT_BACKGROUND);
    assert_eq!(
      valid_color("#123456;", DEFAULT_BACKGROUND),
      DEFAULT_BACKGROUND
    );
    assert_eq!(
      valid_color("#12345", DEFAULT_BACKGROUND),
      DEFAULT_BACKGROUND
    );
  }

  #[test]
  fn maps_supported_theme_names() {
    assert_eq!(
      theme_colors("ARGVUS Dark Aether"),
      Some(("#191b27", "#3590bd", DEFAULT_ACCENT))
    );
    assert_eq!(
      theme_colors("argvus-dark-silver-float"),
      Some(("#595959", "#121518", "#121518"))
    );
    assert_eq!(
      theme_colors("ARGVUS Light Veil"),
      Some(("#ffffff", "#000000", DEFAULT_ACCENT))
    );
    assert_eq!(
      theme_colors("argvus-frost-float"),
      Some(("#f6f8fa", "#24292f", "#0969da"))
    );
    assert_eq!(
      theme_colors("ARGVUS Catppuccin Latte"),
      Some(("#EFF1F5", "#4C4F69", "#1E66F5"))
    );
    assert_eq!(
      theme_colors("ARGVUS One Dark"),
      Some(("#282C34", "#ABB2BF", "#61AFEF"))
    );
    assert_eq!(
      theme_colors("ARGVUS Tokyo Night"),
      Some(("#1A1B26", "#C0CAF5", "#7AA2F7"))
    );
  }
}
