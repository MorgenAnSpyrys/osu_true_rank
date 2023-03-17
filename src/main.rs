extern crate csv;
extern crate multimap;
extern crate serde;
extern crate serde_derive;
extern crate tokio;

use multimap::MultiMap;
use rosu::{model::*, Osu};
use serde::Deserialize;
use std::{env, error::Error, ffi::OsString};

#[tokio::main]
async fn main() {
    let raw_data = fetch_data().await;
    write_result(process_data(raw_data));
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd, Clone)]
struct Record {
    #[serde(rename = "osuUserId")]
    user_id: u32,
    pp: f32,
    #[serde(rename = "beatmapId")]
    beatmap_id: u32,
}

#[derive(Debug, Clone)]
struct User {
    user_id: u32,
    pp_weighted: f32,
    top_scores: Vec<f32>,
}
fn write_result(processed_data: Vec<User>) -> Result<(), Box<dyn Error>> {
    let mut dir = env::current_exe().expect("error fetching executable path");
    dir.pop();
    dir.push("output.csv");
    print!("Writing to: {}", dir.display());
    let mut wtr = csv::Writer::from_path(dir).expect("error creating writer");
    wtr.write_record(&["user_id", "weighted_pp(+416 bonus)", "vec of scores"])?;
    for i in 0..processed_data.len() {
        let mut top_scores_string = String::from("");
        for score in &processed_data[i].top_scores {
            top_scores_string.push_str(&score.to_string());
            top_scores_string.push_str(";")
        }

        wtr.write_record([
            &processed_data[i].user_id.to_string(),
            &processed_data[i].pp_weighted.to_string(),
            &top_scores_string,
        ])?;
        wtr.flush()?;
    }
    Ok(())
}

async fn fetch_data() -> MultiMap<u32, f32> {
    let mut score_data = MultiMap::new();
    let csv_data = import_csv().expect("something went wrong during csv parsing");
    for n in csv_data {
        score_data.insert(n.user_id, n.pp)
    }

    let mut uids: Vec<u32> = Vec::new();
    for i in score_data.clone() {
        uids.push(i.0)
    }

    let api_data = import_api(uids).await;

    for n in api_data {
        score_data.insert(n.user_id, n.pp)
    }

    return score_data;
}

fn process_data(raw_data: MultiMap<u32, f32>) -> Vec<User> {
    let data = raw_data.clone();
    let mut processed_data: Vec<User> = Vec::new();
    const WEIGHT: f32 = 0.95;
    let mut weighted_pp = 0.0;
    for key in &data {
        weighted_pp = 416.6;
        let mut scores = data.get_vec(&key.0).unwrap().to_owned();
        scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
        scores.reverse();
        for n in 1..100 {
            let pp_weighted = scores[n - 1] * WEIGHT.powi((n - 1).try_into().unwrap());
            weighted_pp = weighted_pp + pp_weighted;
        }
        let temp_user = User {
            user_id: *key.0,
            pp_weighted: weighted_pp,
            top_scores: scores,
        };
        processed_data.push(temp_user);
    }

    return processed_data;
}

async fn import_api(uids: Vec<u32>) -> Vec<Record> {
    println!("Starting API Import");
    let mut result: Vec<Record> = Vec::new();
    let mut remaining = uids.len() - 1;
    for id in uids {
        result.append(
            &mut get_top_scores(
                Osu::new(
                    get_second_arg()
                        .expect("please provide your api key as the 2nd command line argument"),
                ),
                id,
            )
            .await
            .unwrap_or(Vec::new()),
        );
        println!("ID {id} processed, {remaining} users remaining");
        if remaining > 0 {
            remaining = remaining - 1;
        }
    }
    return result;
}

async fn get_top_scores(osu: Osu, uid: u32) -> Option<Vec<Record>> {
    let request = osu.top_scores(uid).mode(GameMode::STD).limit(100);
    let scores: Vec<Score> = request.await.ok()?;
    let mut conv_scores: Vec<Record> = Vec::new();
    for n in scores.iter() {
        let temp = Record {
            user_id: n.user_id,
            pp: n.pp.unwrap(),
            beatmap_id: n.beatmap_id.unwrap(),
        };
        conv_scores.push(temp);
    }
    Some(conv_scores)
}

fn import_csv() -> Result<Vec<Record>, Box<dyn Error>> {
    // get csv file and open reader
    let file_path =
        get_first_arg().expect("please provide the path of your csv file encapsulated in quotes");
    let mut reader = csv::Reader::from_path(file_path)?;

    // read csv file
    let mut raw_csv_data = Vec::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        raw_csv_data.push(record)
    }
    Ok(raw_csv_data)
}

fn get_first_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from(
            "please provide the path to the csv file (in quotes) as the first argument",
        )),
        Some(file_path) => Ok(file_path),
    }
}

fn get_second_arg() -> Result<String, Box<dyn Error>> {
    match env::args().nth(2) {
        None => Err(From::from(
            "please provide your api key in the second argument",
        )),
        Some(apikey) => Ok(apikey),
    }
}
