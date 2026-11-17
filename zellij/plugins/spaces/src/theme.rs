use crate::model::Status;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub fg: Rgb,
    pub bg: Rgb,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Style {
    pub const fn new(fg: Rgb, bg: Rgb) -> Self {
        Style {
            fg,
            bg,
            bold: false,
            dim: false,
            italic: false,
            underline: false,
        }
    }

    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub const fn dim(mut self) -> Self {
        self.dim = true;
        self
    }
}

// Catppuccin Mocha, matching the Zellij theme used by this setup.
pub const CRUST: Rgb = Rgb(0x11, 0x11, 0x1b);
pub const MANTLE: Rgb = Rgb(0x18, 0x18, 0x25);
pub const BASE: Rgb = Rgb(0x1e, 0x1e, 0x2e);
pub const SURFACE0: Rgb = Rgb(0x31, 0x32, 0x44);
pub const SURFACE1: Rgb = Rgb(0x45, 0x47, 0x5a);
pub const OVERLAY0: Rgb = Rgb(0x6c, 0x70, 0x86);
pub const SUBTEXT0: Rgb = Rgb(0xa6, 0xad, 0xc8);
pub const TEXT: Rgb = Rgb(0xcd, 0xd6, 0xf4);
pub const LAVENDER: Rgb = Rgb(0xb4, 0xbe, 0xfe);
pub const BLUE: Rgb = Rgb(0x89, 0xb4, 0xfa);
pub const GREEN: Rgb = Rgb(0xa6, 0xe3, 0xa1);
pub const YELLOW: Rgb = Rgb(0xf9, 0xe2, 0xaf);
pub const RED: Rgb = Rgb(0xf3, 0x8b, 0xa8);

pub fn status_color(status: Status) -> Rgb {
    match status {
        Status::NeedsYou | Status::Failed => RED,
        Status::Done => GREEN,
        Status::Working => YELLOW,
        Status::Quiet | Status::NewOutput => BLUE,
        Status::Idle => OVERLAY0,
    }
}

pub struct Palette;

impl Palette {
    pub const PANEL: Rgb = crate::theme::BASE;
    pub const HEADER: Rgb = crate::theme::MANTLE;
    pub const FOOTER: Rgb = crate::theme::MANTLE;
    pub const BORDER: Rgb = crate::theme::SURFACE1;
    pub const SELECTION: Rgb = crate::theme::SURFACE0;
    pub const ACCENT: Rgb = crate::theme::LAVENDER;
    pub const MUTED: Rgb = crate::theme::OVERLAY0;
    pub const TEXT: Rgb = crate::theme::TEXT;
    pub const SUBTEXT: Rgb = crate::theme::SUBTEXT0;
    pub const BASE: Rgb = crate::theme::BASE;
    pub const MANTLE: Rgb = crate::theme::MANTLE;
    pub const SURFACE0: Rgb = crate::theme::SURFACE0;
    pub const CRUST: Rgb = crate::theme::CRUST;
    pub const SURFACE1: Rgb = crate::theme::SURFACE1;
    pub const LAVENDER: Rgb = crate::theme::LAVENDER;
    pub const GREEN: Rgb = crate::theme::GREEN;
    pub const RED: Rgb = crate::theme::RED;
}
