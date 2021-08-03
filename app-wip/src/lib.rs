use chargrid::{align::*, control_flow::*, core::*, text::*};

pub fn app() -> impl Component<Output = app::Output, State = ()> {
    cf(Align::centre(StyledString {
        string: "Hello, World!".to_string(),
        style: Default::default(),
    }))
    .press_any_key()
    .map(|()| app::Exit)
}
