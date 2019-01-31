//! Provides utilities for modifying shims for 3rd-party executables

use std::{fs, io};

use error::ErrorDetails;
use notion_fail::{FailExt, Fallible};
use path;

fn from_io_error(error: &io::Error) -> ErrorDetails {
    if let Some(inner_err) = error.get_ref() {
        ErrorDetails::SymlinkError {
            error: inner_err.to_string(),
        }
    } else {
        ErrorDetails::SymlinkError {
            error: error.to_string(),
        }
    }
}

#[derive(PartialEq)]
pub enum ShimResult {
    Created,
    AlreadyExists,
    Deleted,
    DoesntExist,
}

fn is_3p_shim(name: &str) -> bool {
    match name {
        "node" | "yarn" | "npm" | "npx" => false,
        _ => true,
    }
}

pub fn create(shim_name: &str) -> Fallible<ShimResult> {
    let launchbin = path::launchbin_file()?;
    let shim = path::shim_file(shim_name)?;
    match path::create_file_symlink(launchbin, shim) {
        Ok(_) => Ok(ShimResult::Created),
        Err(err) => {
            if err.kind() == io::ErrorKind::AlreadyExists {
                Ok(ShimResult::AlreadyExists)
            } else {
                throw!(err.with_context(from_io_error));
            }
        }
    }
}

pub fn delete(shim_name: &str) -> Fallible<ShimResult> {
    if !is_3p_shim(shim_name) {
        throw!(ErrorDetails::SymlinkError {
            error: format!("cannot delete `{}`, not a 3rd-party executable", shim_name),
        });
    }
    let shim = path::shim_file(shim_name)?;
    match fs::remove_file(shim) {
        Ok(_) => Ok(ShimResult::Deleted),
        Err(err) => {
            if err.kind() == io::ErrorKind::NotFound {
                Ok(ShimResult::DoesntExist)
            } else {
                throw!(err.with_context(from_io_error));
            }
        }
    }
}
