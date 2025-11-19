use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use sentry_scraper::records::*;
use sentry_scraper::*;
use std::io;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "SENTRY API TOKEN")]
    token: Option<String>,

    #[arg(short, long, default_value_t = false)]
    use_raw: bool,

    #[arg(short, long, default_value_t = false)]
    graph: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Search {
        #[command(subcommand)]
        to_search: SearchCommands,
    },

    Users {
        csv: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum SearchCommands {
    Query { list: Vec<String> },
}

fn main() {
    subscriber_init();

    tracing::info!("Starting sentry scraper");

    let cli = Cli::parse();

    let failed_users = std::sync::Mutex::new(vec![]);
    let count = std::sync::atomic::AtomicUsize::new(0);

    let errs = if cli.use_raw {
        serde_json::from_str(&std::fs::read_to_string("raw_data.txt").unwrap()).unwrap()
    } else {
        match cli.command {
            Commands::Search { to_search } => match to_search {
                SearchCommands::Query { list } => {
                    let issues = list
                        .into_iter()
                        .filter_map(|q| api::get_issues_with_query(q).ok())
                        .flatten();

                    let events = issues
                        .filter_map(|issue| {
                            api::get_issue_events(issue.id)
                                .map(|events| {
                                    events.into_iter().filter_map(|event| {
                                        Some((
                                            event.tags.get("user_email").cloned()?,
                                            ErrorRepr::from_msg(&event.message).ok()?,
                                        ))
                                    })
                                })
                                .ok()
                        })
                        .flatten()
                        .collect::<Vec<_>>();

                    events
                }
            },
            Commands::Users { csv } => {
                tracing::info!("fetching records from csv");
                let records = if let Some(path) = csv {
                    let mut rdr = csv::Reader::from_path(path).unwrap();
                    Records::try_from(&mut rdr).unwrap()
                } else {
                    let mut rdr = csv::Reader::from_reader(io::stdin());
                    Records::try_from(&mut rdr).unwrap()
                };
                let by_users = records.by_users();
                tracing::info!("processing records for {} users", by_users.len());

                by_users
                    .par_iter()
                    .map(|e| {
                        count.fetch_add(1, std::sync::atomic::Ordering::Release);
                        tracing::info!(
                            "User Index: {}",
                            count.load(std::sync::atomic::Ordering::Acquire)
                        );
                        (
                            e,
                            api::get_user_issues(e).map(|x| {
                                x.into_iter()
                                    .map(|issue| {
                                        api::get_issue_events_for_user(issue.id, e).map(|x| {
                                            x.iter()
                                                .map(|event| {
                                                    (
                                                        e,
                                                        ErrorRepr::from_msg(&event.message)
                                                            .unwrap(),
                                                    )
                                                })
                                                .collect::<Vec<_>>()
                                        })
                                    })
                                    .collect::<Result<Vec<_>, _>>()
                                    .map(|x| x.into_iter().flatten().collect::<Vec<_>>())
                            }),
                        )
                    })
                    .filter_map(|(e, x)| {
                        if let Ok(Ok(x)) = x {
                            Some(x)
                        } else {
                            failed_users.lock().unwrap().push(e.clone());
                            None
                        }
                    })
                    .flatten()
                    .map(|(e, x)| (e.clone(), x))
                    .collect::<Vec<_>>()
            }
        }
    };

    let raw_data = serde_json::to_string_pretty(&errs).unwrap();
    std::fs::write("raw_data.txt", raw_data).unwrap();

    if cli.graph {
        let mut error_graph = error_graph::Graph::default();

        tracing::info!("inserting records in error_tree");
        for (email, errors) in errs {
            error_graph.insert_many(email, errors);
        }

        tracing::info!("Writing res to file");
        std::fs::write("failed_users.txt", format!("{:?}", failed_users)).unwrap();
        std::fs::write("result.txt", error_graph.to_string()).unwrap();
    } else {
        let mut error_tree = ErrorTree::new();
        tracing::info!("inserting records in error_tree");
        for (email, errors) in errs {
            error_tree.insert_many(email, errors);
        }

        error_tree.prune(|e| matches!(e, ErrorReason::OtherError));

        tracing::info!("Writing res to file");
        std::fs::write("failed_users.txt", format!("{:?}", failed_users)).unwrap();
        std::fs::write("result.txt", error_tree.to_string()).unwrap();
    }
}

fn subscriber_init() {
    let subs = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subs).expect("setting default subs failed");
}
