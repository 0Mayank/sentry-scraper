use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use sentry_scraper::*;
use sentry_scraper::{api::TOKEN_ENV_NAME, records::*};
use std::io;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod constants {
    pub const OUTPUT: &str = "result.txt";
    pub const FAILED_USERS: &str = "failed_users.txt";
    pub const RAW_DATA: &str = "raw_data.txt";
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "SENTRY API TOKEN", env = TOKEN_ENV_NAME)]
    token: String,

    #[arg(short, long, default_value_t = false)]
    use_raw: bool,

    #[arg(short, long, default_value_t = false)]
    tree: bool,

    #[arg(short, long, value_name = "Output path", default_value_os_t = PathBuf::from(constants::OUTPUT))]
    output: PathBuf,

    #[arg(long, value_name = "Failed users list path", default_value_os_t = PathBuf::from(constants::FAILED_USERS))]
    failed_users: PathBuf,

    #[arg(long, value_name = "Raw data output path", default_value_os_t = PathBuf::from(constants::RAW_DATA))]
    raw_data: PathBuf,

    #[arg(
        long,
        value_name = "Raw data output path",
        default_value_os_t = String::from("7d"),
        help = "Values like 1h, 24h, 7d, 90d, ..."
    )]
    period: String,

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
        #[arg(help = "Read from this path instead of stdin")]
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
    let token = &cli.token;
    let period = &cli.period;

    let failed_users = std::sync::Mutex::new(vec![]);
    let count = std::sync::atomic::AtomicUsize::new(0);

    let errs = if cli.use_raw {
        serde_json::from_str(&std::fs::read_to_string(&cli.raw_data).unwrap()).unwrap()
    } else {
        match cli.command {
            Commands::Search { to_search } => match to_search {
                SearchCommands::Query { list } => {
                    let issues = list
                        .into_iter()
                        .filter_map(|q| api::get_issues_with_query(q, token, period).ok())
                        .flatten();

                    let events = issues
                        .filter_map(|issue| {
                            api::get_issue_events(issue.id, token, period)
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
                            api::get_user_issues(e, token, period).map(|x| {
                                x.into_iter()
                                    .map(|issue| {
                                        api::get_issue_events_for_user(issue.id, e, token, period)
                                            .map(|x| {
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
    std::fs::write(&cli.raw_data, raw_data).unwrap();

    if cli.tree {
        let mut error_tree = ErrorTree::new();
        tracing::info!("inserting records in error_tree");
        for (email, errors) in errs {
            error_tree.insert_many(email, errors);
        }

        error_tree.prune(|e| matches!(e, ErrorReason::OtherError));

        tracing::info!("Writing res to file");
        std::fs::write(&cli.failed_users, format!("{:?}", failed_users)).unwrap();
        std::fs::write(&cli.output, error_tree.to_string()).unwrap();
    } else {
        let mut error_graph = error_graph::Graph::default();

        tracing::info!("inserting records in error_tree");
        for (email, errors) in errs {
            error_graph.insert_many(email, errors);
        }

        tracing::info!("Writing res to file");
        std::fs::write(&cli.failed_users, format!("{:?}", failed_users)).unwrap();
        std::fs::write(&cli.output, error_graph.to_string()).unwrap();
    }
}

fn subscriber_init() {
    let subs = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subs).expect("setting default subs failed");
}
