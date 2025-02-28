use crate::domain::repository::stats::RepositoryStats;
use crate::error::AppResult;
use crate::oauth::Oauth2User;
use crate::view::component::MarkdownPreviewForm;

use crate::get_connected_user_username;

use crate::view::HtmlTemplate;

use askama::Template;
use axum::extract::Path;

use axum::Extension;

use crate::domain::issue::comment::digest::IssueCommentDigest;
use crate::domain::issue::digest::IssueDigest;
use crate::domain::issue::IssueState;
use crate::domain::repository::Repository;
use crate::view::repository::Tab;
use sqlx::PgPool;

#[derive(Template, Debug)]
#[template(path = "repository/issues/issue.html")]
pub struct IssueTemplate {
    user: Option<String>,
    owner: String,
    repository: String,
    issue: IssueDigest,
    stats: RepositoryStats,
    current_branch: Option<String>,
    comments: Vec<IssueCommentDigest>,
    markdown_preview_form: MarkdownPreviewForm,
    tab: Tab,
}

pub async fn view(
    user: Option<Oauth2User>,
    Extension(db): Extension<PgPool>,
    Path((owner, repository, issue_number)): Path<(String, String, i32)>,
) -> AppResult<HtmlTemplate<IssueTemplate>> {
    let connected_username = get_connected_user_username(&db, user).await;
    let stats = RepositoryStats::get(&owner, &repository, &db).await?;
    let repo = Repository::by_namespace(&owner, &repository, &db).await?;
    let issue = repo.get_issue_digest(issue_number, &db).await?;
    let comments = issue.get_comments(&db).await?;
    let current_branch = repo.get_default_branch(&db).await.map(|branch| branch.name);

    let action_href = format!("/{owner}/{repository}/issues/{issue_number}/comment");
    Ok(HtmlTemplate(IssueTemplate {
        user: connected_username,
        owner: owner.clone(),
        repository: repository.clone(),
        issue,
        stats,
        current_branch,
        comments,
        markdown_preview_form: MarkdownPreviewForm {
            with_title: false,
            action_href,
            submit_value: "Comment".to_string(),
            owner,
            repository,
        },
        tab: Tab::Issues,
    }))
}
