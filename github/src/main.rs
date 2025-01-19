mod internal;

use crate::internal::{matrix, JobExt, Matrix, MatrixDimension, StepExt};
use gh_workflow::toolchain::Toolchain;
use gh_workflow::{
    Cargo, Concurrency, Event, Expression, Job, Level, Permissions, PullRequest, Push, Step,
    Workflow,
};
use gh_workflow::generate::Generate;

fn main() {
    let toolchain = Toolchain::default();

    let ci = Workflow::new("CI")
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
                .add_step(toolchain.clone())
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
        );

    let release_plz = Workflow::new("release-plz")
        .name("release-plz")
        .permissions(Permissions::default().pull_requests(Level::Write).contents(Level::Write))
        .on(Event::default().push(Push::default().add_branch("master")))
        .add_job(
            "release-plz",
            Job::new("Release plz")
                .runs_on_("ubuntu-latest")
                .concurrency(Concurrency::default().group("release-plz-${{ github.ref }}").cancel_in_progress(true))
                .add_step(Step::checkout().add_with(("fetch-depth", "0")))
                .add_step(toolchain)
                .add_step(
                    Step::run(r#"
                          # List all opened PRs which head branch starts with "release-plz-"
                          release_pr=$(gh pr list --state='open' --json number,headRefName --jq '.[] | select(.headRefName | startswith("release-plz-")) | .number')
                          # Close the release PR if there is one
                          if [[ -n "$release_pr" ]]; then
                            echo "Closing old release PR $release_pr"
                            gh pr close $release_pr
                          else
                            echo "No open release PR"
                          fi"#)
                        .name("Close old release PR")
                        .add_env(("GITHUB_TOKEN", "${{ secrets.GITHUB_TOKEN }}")),
                )
                .add_step(Step::uses("MarcoIeni", "release-plz-action", "v0.5")
                    .add_env(("GITHUB_TOKEN", "${{ secrets.GITHUB_TOKEN }}"))
                    .add_env(("CARGO_REGISTRY_TOKEN", "${{ secrets.CARGO_REGISTRY_TOKEN }}"))
                )
        );
    
    Generate::new(ci).name("ci.yml").generate().unwrap();
    Generate::new(release_plz).name("release-plz.yml").generate().unwrap();
}
