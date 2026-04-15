use std::sync::OnceLock;

static SYMBOL_RENDERER: OnceLock<fn(Symbol) -> &'static str> = OnceLock::new();

pub enum Symbol {
    Up,
    Down,
    Left,
    Right,
    Tab,
    Enter,
    Spacebar,
    SignalStrength(u8),
    Esc,
}

pub fn render_unicode(sym: Symbol) -> &'static str {
    match sym {
        Symbol::Up => "↑",
        Symbol::Esc => "󱊷",
    }
}

pub fn render_ascii(sym: Symbol) -> &'static str {
    match sym {
        Symbol::Up => "ARROW_UP",
        Symbol::Esc => "ESC",
    }
}

pub fn render(sym: Symbol) -> &'static str {
    SYMBOL_RENDERER
        .get()
        .expect("symbol renderer not initialized")(sym)
}

pub fn init_renderer(use_unicode: bool) {
    let renderer = if use_unicode {
        render_unicode
    } else {
        render_ascii
    };
    SYMBOL_RENDERER
        .set(renderer)
        .expect("symbol renderer already initialized");
}
