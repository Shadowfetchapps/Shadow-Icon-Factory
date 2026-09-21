#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Ios,
    Macos,
    Linux,
    Web,
    Android,
    All,
}

impl Target {
    pub fn from_index(i: u32) -> Self {
        match i {
            0 => Self::Ios,
            1 => Self::Macos,
            2 => Self::Linux,
            3 => Self::Web,
            4 => Self::Android,
            _ => Self::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Ios => "iOS",
            Self::Macos => "macOS",
            Self::Linux => "Linux",
            Self::Web => "Web",
            Self::Android => "Android",
            Self::All => "All",
        }
    }

    #[allow(dead_code)]
    pub fn folder(self) -> &'static str {
        match self {
            Self::Ios => "ios",
            Self::Macos => "macos",
            Self::Linux => "linux",
            Self::Web => "web",
            Self::Android => "android",
            Self::All => "all",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RasterSlot {
    pub rel_path: String,
    pub size: u32,
    pub idiom: Option<&'static str>,
    pub scale: Option<&'static str>,
    #[allow(dead_code)]
    pub role: Option<&'static str>,
}

pub fn slots_for(target: Target) -> Vec<RasterSlot> {
    match target {
        Target::Ios => ios_slots(),
        Target::Macos => macos_slots(),
        Target::Linux => linux_slots(),
        Target::Web => web_slots(),
        Target::Android => android_slots(),
        Target::All => {
            let mut v = ios_slots();
            v.extend(macos_slots());
            v.extend(linux_slots());
            v.extend(web_slots());
            v.extend(android_slots());
            v
        }
    }
}

fn ios_slots() -> Vec<RasterSlot> {
    // Current App Store / Xcode App Icon set (points × scale).
    vec![
        slot("ios/AppIcon.appiconset/icon-20@2x.png", 40, Some("iphone"), Some("2x"), Some("notification")),
        slot("ios/AppIcon.appiconset/icon-20@3x.png", 60, Some("iphone"), Some("3x"), Some("notification")),
        slot("ios/AppIcon.appiconset/icon-29@2x.png", 58, Some("iphone"), Some("2x"), Some("settings")),
        slot("ios/AppIcon.appiconset/icon-29@3x.png", 87, Some("iphone"), Some("3x"), Some("settings")),
        slot("ios/AppIcon.appiconset/icon-40@2x.png", 80, Some("iphone"), Some("2x"), Some("spotlight")),
        slot("ios/AppIcon.appiconset/icon-40@3x.png", 120, Some("iphone"), Some("3x"), Some("spotlight")),
        slot("ios/AppIcon.appiconset/icon-60@2x.png", 120, Some("iphone"), Some("2x"), Some("app")),
        slot("ios/AppIcon.appiconset/icon-60@3x.png", 180, Some("iphone"), Some("3x"), Some("app")),
        slot("ios/AppIcon.appiconset/icon-20-ipad.png", 20, Some("ipad"), Some("1x"), Some("notification")),
        slot("ios/AppIcon.appiconset/icon-20@2x-ipad.png", 40, Some("ipad"), Some("2x"), Some("notification")),
        slot("ios/AppIcon.appiconset/icon-29-ipad.png", 29, Some("ipad"), Some("1x"), Some("settings")),
        slot("ios/AppIcon.appiconset/icon-29@2x-ipad.png", 58, Some("ipad"), Some("2x"), Some("settings")),
        slot("ios/AppIcon.appiconset/icon-40-ipad.png", 40, Some("ipad"), Some("1x"), Some("spotlight")),
        slot("ios/AppIcon.appiconset/icon-40@2x-ipad.png", 80, Some("ipad"), Some("2x"), Some("spotlight")),
        slot("ios/AppIcon.appiconset/icon-76.png", 76, Some("ipad"), Some("1x"), Some("app")),
        slot("ios/AppIcon.appiconset/icon-76@2x.png", 152, Some("ipad"), Some("2x"), Some("app")),
        slot("ios/AppIcon.appiconset/icon-83.5@2x.png", 167, Some("ipad"), Some("2x"), Some("app")),
        slot("ios/AppIcon.appiconset/icon-1024.png", 1024, Some("ios-marketing"), Some("1x"), Some("marketing")),
    ]
}

fn macos_slots() -> Vec<RasterSlot> {
    vec![
        slot("macos/AppIcon.iconset/icon_16x16.png", 16, None, None, None),
        slot("macos/AppIcon.iconset/icon_16x16@2x.png", 32, None, None, None),
        slot("macos/AppIcon.iconset/icon_32x32.png", 32, None, None, None),
        slot("macos/AppIcon.iconset/icon_32x32@2x.png", 64, None, None, None),
        slot("macos/AppIcon.iconset/icon_128x128.png", 128, None, None, None),
        slot("macos/AppIcon.iconset/icon_128x128@2x.png", 256, None, None, None),
        slot("macos/AppIcon.iconset/icon_256x256.png", 256, None, None, None),
        slot("macos/AppIcon.iconset/icon_256x256@2x.png", 512, None, None, None),
        slot("macos/AppIcon.iconset/icon_512x512.png", 512, None, None, None),
        slot("macos/AppIcon.iconset/icon_512x512@2x.png", 1024, None, None, None),
    ]
}

fn linux_slots() -> Vec<RasterSlot> {
    [16, 24, 32, 48, 64, 128, 256, 512, 1024]
        .into_iter()
        .map(|s| {
            slot(
                &format!("linux/hicolor/{s}x{s}/apps/icon.png"),
                s,
                None,
                None,
                None,
            )
        })
        .collect()
}

fn web_slots() -> Vec<RasterSlot> {
    vec![
        slot("web/favicon-16.png", 16, None, None, None),
        slot("web/favicon-32.png", 32, None, None, None),
        slot("web/favicon-48.png", 48, None, None, None),
        slot("web/favicon-192.png", 192, None, None, None),
        slot("web/favicon-512.png", 512, None, None, None),
        slot("web/apple-touch-icon.png", 180, None, None, None),
    ]
}

fn android_slots() -> Vec<RasterSlot> {
    vec![
        slot("android/mipmap-mdpi/ic_launcher.png", 48, None, None, None),
        slot("android/mipmap-hdpi/ic_launcher.png", 72, None, None, None),
        slot("android/mipmap-xhdpi/ic_launcher.png", 96, None, None, None),
        slot("android/mipmap-xxhdpi/ic_launcher.png", 144, None, None, None),
        slot("android/mipmap-xxxhdpi/ic_launcher.png", 192, None, None, None),
    ]
}

fn slot(
    path: &str,
    size: u32,
    idiom: Option<&'static str>,
    scale: Option<&'static str>,
    role: Option<&'static str>,
) -> RasterSlot {
    RasterSlot {
        rel_path: path.to_string(),
        size,
        idiom,
        scale,
        role,
    }
}

pub fn max_required(target: Target) -> u32 {
    slots_for(target).into_iter().map(|s| s.size).max().unwrap_or(1024)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_covers_each_platform() {
        let all = slots_for(Target::All).len();
        let sum = slots_for(Target::Ios).len()
            + slots_for(Target::Macos).len()
            + slots_for(Target::Linux).len()
            + slots_for(Target::Web).len()
            + slots_for(Target::Android).len();
        assert_eq!(all, sum);
        assert_eq!(max_required(Target::Ios), 1024);
        assert_eq!(max_required(Target::Android), 192);
        assert_eq!(Target::from_index(5), Target::All);
        assert_eq!(Target::Ios.label(), "iOS");
    }
}
