use crate::error::AppResult;
use crate::get_connected_user_username;
use crate::oauth::Oauth2User;
use crate::view::repository::{get_repository_branches, BranchDto, Tab};
use crate::view::HtmlTemplate;
use anyhow::anyhow;
use askama::Template;
use axum::extract::Path;
use axum::Extension;

use crate::domain::repository::stats::RepositoryStats;
use crate::domain::repository::Repository;
use sqlx::PgPool;

#[derive(Template, Debug)]
#[template(path = "repository/pulls/compare.html")]
pub struct CompareTemplate {
    user: Option<String>,
    owner: String,
    repository: String,
    stats: RepositoryStats,
    branches: Vec<BranchDto>,
    current_branch: Option<String>,
    tab: Tab,
}

pub async fn compare(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository)): Path<(String, String)>,
) -> AppResult<HtmlTemplate<CompareTemplate>> {
    let connected_username = get_connected_user_username(&db, user).await;
    let stats = RepositoryStats::get(&owner, &repository, &db).await?;
    let repo = Repository::by_namespace(&owner, &repository, &db).await?;
    let current_branch = repo
        .get_default_branch(&db)
        .await
        .ok_or_else(|| anyhow!("No default branch"))?;

    let current_branch = current_branch.name;
    let branches = get_repository_branches(&owner, &repository, &current_branch, &db).await?;

    Ok(HtmlTemplate(CompareTemplate {
        user: connected_username,
        owner,
        repository,
        stats,
        branches,
        current_branch: Some(current_branch),
        tab: Tab::PullRequests,
    }))
}
