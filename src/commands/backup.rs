use clap::error::Error;
use std::path::{Path, PathBuf};

pub fn backup(globs: &[String], output: &Option<PathBuf>) -> Result<(), Error> {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_foo() {
        assert!(false);
    }
}
