extern crate csv;
extern crate multimap;
extern crate serde;
extern crate serde_derive;
extern crate tokio;

use multimap::MultiMap;
use rosu::{model::*, Osu};
use serde::Deserialize;
use std::{env, error::Error, ffi::OsString, io};

// Record struct to store relevant API-fetched score data.
#[derive(Debug, Deserialize, PartialEq, PartialOrd, Clone)]
struct Record {
    #[serde(rename = "osuUserId")]
    user_id: u32,
    pp: f32,
    #[serde(rename = "beatmapId")]
    beatmap_id: u32,
}

// User struct to store processed results to write to csv later
#[derive(Debug, Clone)]
struct User {
    user_id: u32,
    pp_weighted: f32,
    top_scores: Vec<f32>,
}

// main function
#[tokio::main]
async fn main() {
    // synchronously asks user to select if they want to default to 416.6 weighted pp or fetch the value from the osu api via STDIN input.
    let weighted_default = ask_weighted();

    // feches data for both score(raw_data) and bonuspp (bonuspp_data) from the osu! API asynchronously
    let fetched_data = fetch_data().await;
    let raw_data = fetched_data.0;
    let bonuspp_data = fetched_data.1;

    //synchronously processes the data to get new weighted pp and writes result to csv file in osu_true_rank.exe's directory
    let processed_data = process_data(raw_data, weighted_default, bonuspp_data);
    write_result(processed_data).expect("Error writing results to file");
}

// Asks User to select via STDIN if they want to default to 416.6 bonus pp for fetch it from the osu! API, returns answer as a bool (true for default, false for API)
fn ask_weighted() -> bool {
    println!("Do you wish to normalize bonus pp to 416.6, or fetch it from the API? For normalization type Y, for API fetch type N");
    let mut input_string = String::new();
    let weighted_default;
    loop {
        match input_string.trim() {
            "N" | "n" => {
                println!("User has selected API Fetch");
                weighted_default = false;
                break;
            }
            "Y" | "y" => {
                println!("User has selected 416.6 Default");
                weighted_default = true;
                break;
            }
            _ => {
                input_string.clear();
                io::stdin().read_line(&mut input_string).unwrap();
                println!("please enter either y or n");
            }
        }
    }
    return weighted_default;
}

// asynchronously fetches data from osu! api and synchronously fetches data from csv passed in cli arg
async fn fetch_data() -> (MultiMap<u32, f32>, MultiMap<u32, f32>) {
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
    let bonuspp_data = get_bpp(api_data.clone()).await;

    for n in api_data {
        score_data.insert(n.user_id, n.pp)
    }

    return (score_data, bonuspp_data);
}

// imports passed csv file after grabbing it from cli arg via get_first_arg() and returns it as a Vec of Record
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

// gets first cli arg, the csv file path, and returns it as OsString
fn get_first_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from(
            "please provide the path to the csv file (in quotes) as the first argument",
        )),
        Some(file_path) => Ok(file_path),
    }
}

// asynchronously imports osu! top scores from api (by calling get_top_scores using get_second_arg to grab the API key from cli args) and returns them as vec of record
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
        println!("ID {id} top scores fetched, {remaining} users remaining");
        if remaining > 0 {
            remaining = remaining - 1;
        }
    }
    return result;
}

// gets 2nd cli arg, the API key, and returns it as string
fn get_second_arg() -> Result<String, Box<dyn Error>> {
    match env::args().nth(2) {
        None => Err(From::from(
            "please provide your api key in the second argument",
        )),
        Some(apikey) => Ok(apikey),
    }
}

// gets user top scores from api after taking osu object and user uid as arguments and returns them as vec of record
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

// gets bpp from api after taking previously fetched data as argument, uses get_second_arg to grab the api key and returns a MultiMap (hashmap) containing respective uid(k) and bpp (v)
async fn get_bpp(api_data: Vec<Record>) -> MultiMap<u32, f32> {
    let mut user_scores = MultiMap::new();
    let mut user_bpp = MultiMap::new();
    for n in api_data {
        user_scores.insert(n.user_id, n.pp);
    }
    const WEIGHT: f32 = 0.95;
    for e in user_scores.clone() {
        let total_pp = get_user_totalpp(
            Osu::new(
                get_second_arg()
                    .expect("please provide your api key as the 2nd command line argument"),
            ),
            e.0,
        )
        .await;
        let mut weighted_pp = 0.0;
        let mut scores = user_scores.get_vec(&e.0).unwrap().to_owned();
        scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
        scores.reverse();
        for n in 1..100 {
            let pp_weighted = scores[n - 1] * WEIGHT.powi((n - 1).try_into().unwrap());
            weighted_pp = weighted_pp + pp_weighted;
        }
        let result_pp = total_pp - weighted_pp;
        user_bpp.insert(e.0, result_pp);
    }
    return user_bpp;
}

// asyncrhonously gets a user's total pp and returns it as f32 from the osu! api by taking an osu object and the user's uid as an argument
async fn get_user_totalpp(osu: Osu, uid: u32) -> f32 {
    println!("fetching user {uid} bonuspp from api");
    let request = osu.user(uid).await;
    let output = request.ok().unwrap();
    return match output {
        Some(x) => x.pp_raw,
        None => 416.6,
    };
}

// processes all data for writing to csv, returns a Vec of User
fn process_data(
    raw_data: MultiMap<u32, f32>,
    weighted_default: bool,
    bonuspp_data: MultiMap<u32, f32>,
) -> Vec<User> {
    println!("Processing Data");
    let data = raw_data.clone();
    let mut processed_data: Vec<User> = Vec::new();
    const WEIGHT: f32 = 0.95;
    let mut weighted_pp;
    let fetch_weighted = weighted_default;
    for key in data.clone() {
        if fetch_weighted == true {
            weighted_pp = 416.6;
        } else {
            weighted_pp = bonuspp_data.get(&key.0).unwrap_or(&416.6).to_owned();
        }

        let mut scores = data.get_vec(&key.0).unwrap().to_owned();
        scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
        scores.reverse();
        for n in 1..100 {
            let pp_weighted = scores[n - 1] * WEIGHT.powi((n - 1).try_into().unwrap());
            weighted_pp = weighted_pp + pp_weighted;
        }
        let temp_user = User {
            user_id: key.0,
            pp_weighted: weighted_pp,
            top_scores: scores,
        };
        processed_data.push(temp_user);
    }

    return processed_data;
}

// writes processed data to "output.csv" in executable directory
fn write_result(processed_data: Vec<User>) -> Result<(), Box<dyn Error>> {
    let mut dir = env::current_exe().expect("error fetching executable path");
    dir.pop();
    dir.push("output.csv");
    print!("Writing to: {}", dir.display());
    let mut wtr = csv::Writer::from_path(dir).expect("error creating writer");
    wtr.write_record(&["user_id", "weighted_pp", "vec of scores"])?;
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
