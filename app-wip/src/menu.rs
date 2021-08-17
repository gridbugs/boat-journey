#[derive(Clone, Copy, Debug)]
pub struct Cancel;
pub type OrCancel<T> = Result<T, Cancel>;
