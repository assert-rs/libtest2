pub(crate) use crate::*;

pub trait Case: Send + Sync + 'static {
    /// The name of a test
    ///
    /// By convention this follows the rules for rust paths; i.e., it should be a series of
    /// identifiers separated by double colons. This way if some test runner wants to arrange the
    /// tests hierarchically it may.
    fn name(&self) -> &str;
    fn kind(&self) -> TestKind;
    fn source(&self) -> Option<&Source>;
    /// This case cannot run in parallel to other cases within this binary
    fn exclusive(&self, state: &TestContext) -> bool;

    fn run(&self, state: &TestContext) -> Result<(), RunError>;
}

impl Case for Box<dyn Case> {
    fn name(&self) -> &str {
        self.as_ref().name()
    }
    fn kind(&self) -> TestKind {
        self.as_ref().kind()
    }
    fn source(&self) -> Option<&Source> {
        self.as_ref().source()
    }
    fn exclusive(&self, state: &TestContext) -> bool {
        self.as_ref().exclusive(state)
    }

    fn run(&self, state: &TestContext) -> Result<(), RunError> {
        self.as_ref().run(state)
    }
}

impl Case for std::sync::Arc<dyn Case> {
    fn name(&self) -> &str {
        self.as_ref().name()
    }
    fn kind(&self) -> TestKind {
        self.as_ref().kind()
    }
    fn source(&self) -> Option<&Source> {
        self.as_ref().source()
    }
    fn exclusive(&self, state: &TestContext) -> bool {
        self.as_ref().exclusive(state)
    }

    fn run(&self, state: &TestContext) -> Result<(), RunError> {
        self.as_ref().run(state)
    }
}

/// Type of the test according to the [rust book](https://doc.rust-lang.org/cargo/guide/tests.html)
/// conventions.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum TestKind {
    /// Unit-tests are expected to be in the `src` folder of the crate.
    UnitTest,
    /// Integration-style tests are expected to be in the `tests` folder of the crate.
    IntegrationTest,
    /// Doctests are created by the `librustdoc` manually, so it's a different type of test.
    DocTest,
    /// Tests for the sources that don't follow the project layout convention
    /// (e.g. tests in raw `main.rs` compiled by calling `rustc --test` directly).
    #[default]
    Unknown,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Source {
    Rust {
        source_file: std::path::PathBuf,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    },
    Path(std::path::PathBuf),
}
