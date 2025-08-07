use derr::records::*;
use derr::*;
use std::io;

fn main() {
    println!("fetching records from csv");
    let mut rdr = csv::Reader::from_reader(io::stdin());
    let records = Records::try_from(&mut rdr).unwrap();

    let mut error_tree = ErrorTree::new();
    let mut failed_users = vec![];
    // let mut failed_issues = vec![];

    println!("processing records");
    let by_users = records.by_users();
    let errs = by_users
        .iter()
        .map(|(e, _)| {
            (
                e,
                api::get_user_issues(e).map(|x| {
                    x.into_iter()
                        .map(|issue| {
                            // (
                            // issue.id,
                            api::get_issue_events(issue.id, e).map(|x| {
                                x.iter()
                                    .map(|event| (e, ErrorRepr::from_msg(&event.message).unwrap()))
                                    .collect::<Vec<_>>()
                            })
                            // ,)
                        })
                        // .filter_map(|(id, x)| {
                        //     if let Ok(x) = x {
                        //         Some(x)
                        //     } else {
                        //         failed_issues.push((id, e));
                        //         None
                        //     }
                        // })
                        // .flatten()
                        .collect::<Result<Vec<_>, _>>()
                        .map(|x| x.into_iter().flatten().collect::<Vec<_>>())
                }),
            )
        })
        .filter_map(|(e, x)| {
            if let Ok(Ok(x)) = x {
                Some(x)
            } else {
                failed_users.push(e);
                None
            }
        })
        .flatten()
        .collect::<Vec<_>>();

    let raw_data = serde_json::to_string_pretty(&errs).unwrap();
    std::fs::write("raw_data.txt", raw_data).unwrap();

    println!("inserting records in error_tree");
    for (email, errors) in errs {
        error_tree.insert_many(email, errors);
    }

    println!("Writing res to file");
    std::fs::write("failed_users.txt", format!("{:?}", failed_users)).unwrap();
    // std::fs::write("failed_issues.txt", format!("{:?}", failed_issues)).unwrap();
    std::fs::write("result.txt", error_tree.to_string()).unwrap();
}
