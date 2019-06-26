use crate::config::*;
use chrono::{DateTime, Utc};
use failure::Error;
use reqwest::Client;

#[derive(Deserialize)]
struct PullRequestResponse {
    head: PullRequestHead,
    html_url: String,
    url: String,
}

impl PullRequestResponse {
    fn to_pull_request(self, reviews: Vec<Review>) -> PullRequest {
        PullRequest::new(self.html_url, self.url, self.head.branch, reviews)
    }
}

#[derive(Deserialize)]
struct PullRequestHead {
    #[serde(rename = "ref")]
    branch: String,
}

#[derive(Debug)]
pub struct PullRequest {
    pub html_url: String,
    pub url: String,
    pub branch: String,
    pub reviews: Vec<Review>,
    pub update_timestamp: DateTime<Utc>,
    pub posted: bool,
}

impl PullRequest {
    pub fn update(&mut self, reviews: Vec<Review>) {
        self.update_timestamp = Utc::now();
        self.reviews = reviews;
    }

    pub fn new(html_url: String, url: String, branch: String, reviews: Vec<Review>) -> PullRequest {
        PullRequest {
            html_url,
            url,
            branch,
            reviews,
            update_timestamp: Utc::now(),
            posted: false,
        }
    }

    pub fn was_posted(&mut self) {
        self.posted = true;
    }
}

pub trait ReviewAnalysis {
    fn approved(&self) -> bool;
    fn half_approved(&self) -> bool;
}

impl ReviewAnalysis for Vec<Review> {
    fn approved(&self) -> bool {
        let two_approvals = self
            .iter()
            .filter(|review| review.state == ReviewStates::APPROVED)
            .count()
            >= 2;
        let no_changes_requested = self
            .iter()
            .filter(|review| review.state == ReviewStates::CHANGES_REQUESTED)
            .count()
            == 0;
        two_approvals && no_changes_requested
    }

    fn half_approved(&self) -> bool {
        let one_approval = self
            .iter()
            .filter(|review| review.state == ReviewStates::APPROVED)
            .count()
            >= 1;
        let no_changes_requested = self
            .iter()
            .filter(|review| review.state == ReviewStates::CHANGES_REQUESTED)
            .count()
            == 0;

        one_approval && no_changes_requested
    }
}

#[derive(Debug, PartialEq)]
pub struct Review {
    pub state: ReviewStates,
    pub user: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum ReviewStates {
    APPROVED,
    CHANGES_REQUESTED,
    COMMENTED,
    DISMISSED,
}

pub fn get_pull_requests() -> Result<Vec<PullRequest>, Error> {
    let mut pull_requests = Vec::new();
    for url in get_pull_request_url_per_project()? {
        for pull_request_response in get_project_pull_requests(url.as_str())? {
            let reviews = get_reviews(pull_request_response.url.as_str())?;

            pull_requests.push(pull_request_response.to_pull_request(reviews))
        }
    }

    Ok(pull_requests)
}

#[derive(Deserialize, Debug)]
struct RepoResponse {
    pulls_url: String,
}

fn get_pull_request_url_per_project() -> Result<Vec<String>, Error> {
    let client = Client::new();

    let repo_responses: Vec<RepoResponse> = client
        .get(format!("{}/orgs/{}/repos", &GITHUB_URL, &CONFIG.organisation).as_str())
        .basic_auth(&CONFIG.github_user, Some(&CONFIG.github_password))
        .send()?
        .json()?;

    Ok(repo_responses
        .into_iter()
        .map(|r| r.pulls_url.replace("{/number}", ""))
        .collect())
}

#[derive(Deserialize, Debug)]
struct ReviewResponse {
    state: ReviewStates,
    user: UserResponse,
}

impl ReviewResponse {
    fn to_review(self) -> Review {
        Review {
            state: self.state,
            user: self.user.login,
        }
    }
}

#[derive(Deserialize, Debug)]
struct UserResponse {
    login: String,
}

pub fn get_reviews(pull_request_url: &str) -> Result<Vec<Review>, Error> {
    let client = Client::new();

    let review_responses: Vec<ReviewResponse> = client
        .get(format!("{}/reviews", pull_request_url).as_str())
        .basic_auth(&CONFIG.github_user, Some(&CONFIG.github_password))
        .send()?
        .json()?;

    Ok(review_responses
        .into_iter()
        .map(|review_response| review_response.to_review())
        .collect())
}

fn get_project_pull_requests(url: &str) -> Result<Vec<PullRequestResponse>, Error> {
    let client = Client::new();

    let requests: Vec<PullRequestResponse> = client
        .get(url)
        .basic_auth(&CONFIG.github_user, Some(&CONFIG.github_password))
        .send()?
        .json()?;

    Ok(requests)
}
