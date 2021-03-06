use font::Font;

static IOSEVKA_SFL: &'static str = include_str!("../../fonts/iosevka.sfl");
static IOSEVKA_PNG: &'static [u8] = include_bytes!("../../fonts/iosevka.png");

fn test_load_font() -> Font {
    Font::load("fonts/iosevka.sfl")
}

fn test_load_font_raw() -> Font {
    Font::load_raw(IOSEVKA_SFL, IOSEVKA_PNG)
}

#[test]
fn test_font_name() {
    assert_eq!(test_load_font().name, "Iosevka");
}

#[test]
fn test_load_font_raw_eq_regular_load() {
    assert_eq!(test_load_font(), test_load_font_raw());
}

#[test]
fn test_font_size() {
    let font = test_load_font();
    assert_eq!(font.width, 393);
    assert_eq!(font.height, 374);
}

#[test]
fn test_font_line_height_and_size() {
    let font = test_load_font();
    assert_eq!(font.line_height, 56);
    assert_eq!(font.size, 32);
}

#[test]
fn test_font_character_amount() {
    let font = test_load_font();
    assert_eq!(font.characters.keys().len(), 191);
}

#[test]
fn test_font_single_character_width() {
    let font = test_load_font();
    assert_eq!(font.get_character('a').unwrap().width, 18);
}

#[test]
fn test_font_single_character_height() {
    let font = test_load_font();
    assert_eq!(font.get_character('a').unwrap().height, 24);
}

#[test]
fn test_font_single_character_offsets() {
    let font = test_load_font();
    assert_eq!(font.get_character('a').unwrap().x_off, 2);
    assert_eq!(font.get_character('a').unwrap().y_off, 32);
}

#[test]
fn test_font_single_character_id() {
    let font = test_load_font();
    assert_eq!(font.get_character('a').unwrap().id, 97);
}