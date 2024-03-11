use chrono::prelude::*;
use clap::Parser;
use serde_json::Value;
use std::thread::sleep;
use std::time::Duration;

// Argument defintion
#[derive(Parser, Debug)]
#[command(
    name = "redditor",
    version,
    about = "Simple command line reddit browser"
)]
struct Args {
    /// Name of the subreddit to browse
    #[arg(short, long, required = true)]
    name: String,

    /// Sorting type for the subreddit, can be 'hot', 'new', 'top'
    #[arg(short, long, default_value = "hot")]
    sort: String,

    /// Wait time in seconds between checking for new posts
    #[arg(short, long, default_value = "10")]
    wait: u64,
}

// Function to show posts based on subreddit name and sorting type
fn show_posts(
    subreddit_name: &str,
    sort_type: &str,
    wait_time: u64,
) -> Result<(), Box<dyn std::error::Error>> {

    println!(
        "Showing posts from r/{} sorted by {} ...\n",
        subreddit_name, sort_type
    );

    let mut current_posts: Vec<String> = Vec::new(); // Vector to store current posts
    let link_prefix = "https://www.reddit.com".to_string();
    let mut link: String;
    let mut path: String = "https://www.reddit.com/r/".to_string();
    let mut found_new: bool;
    let mut first_iteration: bool = true; // Variable to check if it is the first iteration, used for printing messages

    path.push_str(subreddit_name);
    path.push('/');
    path.push_str(sort_type);
    path.push_str(".json?limit=10"); // Limiting the number of posts to 10

    loop {
        found_new = false;
        let message: Value = ureq::get(&path).call()?.into_json()?;
        let duration = Duration::from_secs(wait_time);

        for i in message["data"]["children"].as_array().unwrap() {
            if current_posts.contains(&i["data"]["title"].to_string().trim_matches('"').to_string())
                && !first_iteration
            {
                continue;
            } else if !first_iteration {
                found_new = true;
            }

            // Printing the title of the post
            println!("{}", i["data"]["title"].to_string().trim_matches('"'));

            // Extracting link to post - "url" field was not used as it reffered to the content and not the post itself
            link = link_prefix.clone() + i["data"]["permalink"].to_string().trim_matches('"');
            println!("{}", link); // Printing the link to the post, can be opened in browser

            // Extracting the date and time of the post - the raw timestamp is in epoch time, so it is converted to human readable format, local time
            let timestamp = i["data"]["created_utc"]
                .to_string()
                .trim_end_matches(".0")
                .parse::<i64>()
                .unwrap();
            let naive: Option<NaiveDateTime> = NaiveDateTime::from_timestamp_opt(timestamp, 0);
            let timestamp: DateTime<Local> = DateTime::<Local>::from_naive_utc_and_offset(
                naive.unwrap(),
                *Local::now().offset(),
            );
            let newdate: chrono::format::DelayedFormat<chrono::format::StrftimeItems<'_>> =
                timestamp.format("%d %B, %Y %H:%M:%S");
            println!("{}\n", newdate); // Printing the date and time of the post

            // Adding the post to the vector of current posts
            current_posts.push(i["data"]["title"].to_string().trim_matches('"').to_string());
        }


        // Check relevant flags for displaying messages.
        if !found_new && !first_iteration {
            println!(
                "No new posts found, checking again in {} seconds...",
                wait_time
            );
        } else if found_new && !first_iteration {
            println!(
                "Found the above new posts, checking again in {} seconds...",
                wait_time
            );
        }
        if first_iteration {
            println!("Checking for new posts every {} seconds...\n", wait_time)
        }
        first_iteration = false;
        sleep(duration);
    }
}

// Main function
fn main() {
    let args = Args::parse();

    let subreddit_name = args.name;
    let sort_type = args.sort;
    let wait_time = args.wait;

    if let Err(e) = show_posts(&subreddit_name, &sort_type, wait_time) {
        println!("Error: {}", e);
    }
}
