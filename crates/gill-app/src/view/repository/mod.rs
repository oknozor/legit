use crate::domain::user::User;
use crate::error::AppResult;
use crate::state::AppState;

use axum::routing::{get, post};
use axum::Router;
use sqlx::PgPool;
use std::fmt;
use std::fmt::Formatter;

pub mod activity;
pub mod blob;
pub mod commits;
pub mod create;
pub mod diff;
pub mod issues;
pub mod pull_request;
pub mod tree;
pub mod user_content;

#[derive(Debug)]
pub enum Tab {
    Code,
    Issues,
    PullRequests,
    History,
}

pub fn routes() -> Router<AppState> {
    let router = Router::new()
        .route("/new", get(create::view))
        .route("/create-repository", get(create::submit))
        .route("/:owner/:repository", get(tree::root))
        .route("/:owner/:repository/tree/:branch", get(tree::tree_root))
        .route("/:owner/:repository/tree/:branch/*tree", get(tree::tree))
        .route("/:owner/:repository/blob/:branch/*blob", get(blob::blob))
        .route(
            "/:owner/:repository/commits/:branch/",
            get(commits::git_log),
        )
        .route("/:owner/:repository/commits/:branch", get(commits::git_log))
        .route("/:owner/:repository/commit/:sha", get(commits::commit_diff))
        .route("/:owner/:repository/diff", get(diff::view))
        .route("/:owner/:repository/get_diff", get(diff::get_diff))
        .route("/:owner/:repository/star", post(activity::star))
        .route("/:owner/:repository/watch", post(activity::watch))
        .route("/:owner/:repository/*path", get(user_content::image));

    router.merge(pull_request::router()).merge(issues::router())
}

#[derive(Debug)]
pub struct BranchDto {
    name: String,
    is_default: bool,
    is_current: bool,
}

async fn get_repository_branches(
    owner: &str,
    repository: &str,
    current_branch: &str,
    db: &PgPool,
) -> AppResult<Vec<BranchDto>> {
    let user = User::by_name(owner, db).await.unwrap();
    let repository = user.get_local_repository_by_name(repository, db).await?;
    let branches = repository.list_branches(20, 0, db).await?;
    let branches = branches
        .into_iter()
        .map(|branch| {
            let is_current = branch.name == current_branch;

            BranchDto {
                name: branch.name,
                is_default: branch.is_default,
                is_current,
            }
        })
        .collect();

    Ok(branches)
}

pub fn tree_and_blob_from_query(path: &str) -> (Option<&str>, &str) {
    match path.rsplit_once('/') {
        None => (None, path),
        Some((tree, blob_name)) => {
            if !tree.is_empty() {
                (Some(tree), blob_name)
            } else {
                (None, blob_name)
            }
        }
    }
}

impl fmt::Display for BranchDto {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{self:?}")
    }
}
