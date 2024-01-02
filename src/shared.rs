#[macro_export]
macro_rules! convert_error {
    ($err:expr $(,)?) => ({
        let error = match $err {
            error => eyre!(error.to_string()),
        };
        error
    });
}

#[macro_export]
macro_rules! exec_or_eyre {
    ($f: expr) => ({
        $f.map_err(|error| convert_error!(error))
    });
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Coords {
    pub x: f32,
    pub y: f32,
}