use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    typing_layout: String,
    buttons_layout: String,
    finger_rotation: Option<u8>,
}
// ButtonsLayout TypingLayout

#[derive(Deserialize)]
struct ButtonsLayout {

}