mod backups_folder;
mod option_open;

pub use backups_folder::*;
pub use option_open::option_open;

#[macro_export]
macro_rules! try_option {
    ($option: expr) => {
        match $option? {
            Some(v) => v,
            None => return Ok(None),
        }
    };

    (no_try, $option: expr) => {
        match $option {
            Some(v) => v,
            None => return Ok(None),
        }
    };

    (no_result, $option: expr) => {
        match $option {
            Some(v) => v,
            None => return None,
        }
    };
}
