mod internal;

use crate::internal::{matrix, JobExt, Matrix, MatrixDimension, StepExt};
use gh_workflow::toolchain::Toolchain;
use gh_workflow::{
    Cargo, Event, Expression, Job, Level, Permissions, PullRequest, Push, Step, Workflow,
};

fn main() {
    let toolchain = Toolchain::default();

    Workflow::new("CI")
        .on(Event::default()
            .push(Push::default().add_branch("master"))
            .pull_request(PullRequest::default()))
        .permissions(Permissions::default().contents(Level::Write))
        .add_job(
            "build-and-test",
            Job::new("Build and test")
                .strategy(matrix(
                    Matrix::empty().add_dimension(
                        MatrixDimension::new("os")
                            .value("ubuntu-latest")
                            .value("windows-latest")
                            .value("macos-latest"),
                    ),
                ))
                .runs_on("${{ matrix.os }}")
                .add_step(Step::checkout())
                .add_step(toolchain.clone())
                .add_step(Cargo::new("test")),
        )
        .add_job(
            "checks",
            Job::new("Checks")
                .runs_on_("ubuntu-latest")
                .add_step(Step::checkout())
                .add_step(toolchain)
                .add_step(Step::install_action().add_tool("cargo-deny"))
                .add_step(Cargo::new("clippy").args("--no-deps --all-targets -- -Dwarnings"))
                .add_step(Cargo::new("fmt").args("--all -- --check"))
                .add_step(Cargo::new("deny").args("check")),
        )
        .add_job(
            "deploy-book",
            Job::new("Deploy book")
                .runs_on_("ubuntu-latest")
                .add_step(Step::checkout())
                .add_step(Step::setup_mdbook())
                .add_step(Step::run("mdbook build").working_directory("book"))
                .add_step(
                    Step::ghpages()
                        .if_condition(Expression::new("${{ github.ref == 'refs/heads/master' }}"))
                        .github_token("${{ secrets.GITHUB_TOKEN }}")
                        .publish_dir("./book/book")
                        .cname("desert-rust.vigoo.dev"),
                ),
        )
        .generate()
        .unwrap();
}
