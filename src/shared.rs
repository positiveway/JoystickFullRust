use serde::{Deserialize, Serialize};
use strum_macros::Display;
#[macro_export]
macro_rules! convert_error {
    ($err:expr $(,)?) => ({
        let error = match $err {
            error => color_eyre::eyre::eyre!(error.to_string()),
        };
        error
    });
}

#[macro_export]
macro_rules! exec_or_eyre {
    ($f: expr) => ({
        $f.map_err(|error| crate::convert_error!(error))
    });
}

