use amane::Color;

pub struct Theme {
    pub cinder: Color,
    pub ember: Color,
    pub brass: Color,
    pub sap: Color,
    pub frost: Color,
    pub fg: Color,
    pub fg_dim: Color,
    pub comment: Color,
    pub gutter: Color,
    pub bg_deep: Color,
    pub bg0: Color,
    pub bg1: Color,
    pub bg2: Color,
    pub bg3: Color,
    pub bg4: Color,
    pub bg5: Color,
}

pub fn theme() -> Theme {
    Theme {
        cinder: Color::from("#d1766e"),
        ember: Color::from("#ea9875"),
        brass: Color::from("#fcba81"),
        sap: Color::from("#99af6b"),
        frost: Color::from("#4e89a2"),
        fg: Color::from("#e6d5c2"),
        fg_dim: Color::from("#a09384"),
        comment: Color::from("#73665b"),
        gutter: Color::from("#4e4641"),
        bg_deep: Color::from("#0f0c0a"),
        bg0: Color::from("#171311"),
        bg1: Color::from("#201b19"),
        bg2: Color::from("#2a2422"),
        bg3: Color::from("#362f2c"),
        bg4: Color::from("#463e3a"),
        bg5: Color::from("#5a504c"),
    }
}
