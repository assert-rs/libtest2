use libtest2_harness::Case;
use libtest2_harness::Source;
use libtest2_harness::TestKind;

use crate::RunResult;
use crate::TestContext;

#[derive(Copy, Clone)]
pub struct DynCase(pub &'static dyn Case);

impl Case for DynCase {
    fn name(&self) -> &str {
        self.0.name()
    }
    fn kind(&self) -> TestKind {
        self.0.kind()
    }
    fn source(&self) -> Option<&Source> {
        self.0.source()
    }
    fn exclusive(&self, context: &TestContext) -> bool {
        self.0.exclusive(context)
    }

    fn run(&self, context: &TestContext) -> RunResult {
        self.0.run(context)
    }
}

pub struct FnCase<R> {
    name: String,
    runner: R,
}

impl<R> FnCase<R>
where
    R: Fn(&TestContext) -> RunResult + Send + Sync + 'static,
{
    pub fn test(name: impl Into<String>, runner: R) -> Self {
        Self {
            name: name.into(),
            runner,
        }
    }
}

impl<R> Case for FnCase<R>
where
    R: Fn(&TestContext) -> RunResult + Send + Sync + 'static,
{
    fn name(&self) -> &str {
        &self.name
    }
    fn kind(&self) -> TestKind {
        Default::default()
    }
    fn source(&self) -> Option<&Source> {
        None
    }
    fn exclusive(&self, _: &TestContext) -> bool {
        false
    }

    fn run(&self, context: &TestContext) -> RunResult {
        (self.runner)(context)
    }
}

pub fn main(cases: impl IntoIterator<Item = impl Case + 'static>) {
    let harness = libtest2_harness::Harness::new();
    let harness = match harness.with_env() {
        Ok(harness) => harness,
        Err(err) => {
            eprintln!("{err}");
            ::std::process::exit(1);
        }
    };
    let harness = match harness.parse() {
        Ok(harness) => harness,
        Err(err) => {
            eprintln!("{err}");
            ::std::process::exit(1);
        }
    };
    let mut cases = cases.into_iter().collect::<Vec<_>>();
    cases.sort_by_key(|c| c.name().to_owned());
    let harness = match harness.discover(cases) {
        Ok(harness) => harness,
        Err(err) => {
            eprintln!("{err}");
            ::std::process::exit(libtest2_harness::ERROR_EXIT_CODE)
        }
    };
    match harness.run() {
        Ok(true) => ::std::process::exit(0),
        Ok(false) => ::std::process::exit(libtest2_harness::ERROR_EXIT_CODE),
        Err(err) => {
            eprintln!("{err}");
            ::std::process::exit(libtest2_harness::ERROR_EXIT_CODE)
        }
    }
}
