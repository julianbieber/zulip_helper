#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate structopt;

use structopt::StructOpt;

use chrono::prelude::*;

use crate::github::{PullRequest, ReviewAnalysis, ReviewStates};
use failure::Error;
use std::thread;
use std::time::Duration;

mod config;
mod github;
mod zulip;

#[derive(StructOpt, Debug)]
#[structopt(name = "zulip-helper")]
struct Options {
    #[structopt(short = "f")]
    feature_id: String,
}

fn main() -> Result<(), Error> {
    let options: Options = Options::from_args();

    let mut pull_requests: Vec<PullRequest> = github::get_pull_requests()?
        .into_iter()
        .filter(|request| request.branch.contains(options.feature_id.as_str()))
        .collect();

    while pull_requests.iter().any(|pr| !pr.reviews.approved()) {
        for pull_request in pull_requests.iter_mut() {
            if !pull_request.posted && pull_request.reviews.is_empty() {
                zulip::post_message(
                    "Pull Requests",
                    options.feature_id.as_str(),
                    format!("Can someone review {}", pull_request.html_url.as_str()).as_str(),
                )?;
                pull_request.was_posted();
            }
            let current_reviews = github::get_reviews(pull_request.url.as_str())?;
            if vec_inequality(&current_reviews, &pull_request.reviews) {
                if current_reviews.half_approved() {
                    zulip::post_message(
                        "Pull Requests",
                        options.feature_id.as_str(),
                        format!(
                            "Can someone do the second review for {}",
                            pull_request.html_url.as_str()
                        )
                        .as_str(),
                    )?;
                }
                pull_request.update(current_reviews);
            }

        }
        pull_requests = pull_requests.into_iter().filter(|pr| !pr.reviews.approved()).collect();
        thread::sleep(Duration::from_millis(1000));
    }

    println!("{:?}", pull_requests);
    Ok(())
}

fn vec_inequality<A: PartialEq>(va: &[A], vb: &[A]) -> bool {
    va.len()!=vb.len() ||
        va.iter().zip(vb).any(|(a, b)| a != b)
}