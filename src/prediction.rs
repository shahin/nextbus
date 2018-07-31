use error_chain::ChainedError;

use std::thread;
use std::time::{Duration};
use serde_json;

use client;
use client::from_string;
use errors::*;
use schedule::get_stops;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PredictionsList {
    pub predictions: Vec<Predictions>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Predictions {
    pub direction: Direction,
    pub agency_title: String,
    pub route_title: String,
    pub route_tag: String,
    pub stop_tag: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Direction {
    pub prediction: Vec<Prediction>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Prediction {
    #[serde(deserialize_with = "from_string")]
    #[serde(rename = "epochTime")]
    pub epoch: u64,
    #[serde(deserialize_with = "from_string")]
    pub seconds: u64,
    #[serde(deserialize_with = "from_string")]
    pub minutes: u64,
    #[serde(deserialize_with = "from_string")]
    pub is_departure: bool,
    pub dir_tag: String,
    #[serde(deserialize_with = "from_string", default)]
    pub affected_by_layover: bool,
    #[serde(deserialize_with = "from_string", default)]
    pub delayed: bool,
    #[serde(deserialize_with = "from_string", default)]
    pub slowness: f32,
    pub vehicle: String,
    #[serde(deserialize_with = "from_string", default)]
    pub vehicles_in_consist: u32,
    pub block: String,
    pub trip_tag: String,
}

impl client::Contents for PredictionsList {
    fn is_empty(&self) -> bool {
        self.predictions.len() == 0
    }
}

fn get_predictions_url(agency: &String, route: &String, stops: &Vec<String>) -> String {
    let route_stops: Vec<String> = stops.into_iter().map(|s| route.to_string() + "|" + s).collect();
    format!(
        "http://webservices.nextbus.com/service/publicXMLFeed?command=predictionsForMultiStops&a={agency}&stops={stops}",
        agency = agency,
        stops = route_stops.join("&stops="),
    )
}

pub fn get_predictions(agency: String, route: String, stops: Vec<String>) -> Result<()> {
    let mut n_attempts = 0;

    let stops = match stops.len() {
        0 => get_stops(&agency, &route)?,
        _ => stops
    };

    loop {
        if n_attempts > 0 {
            thread::sleep(Duration::from_millis(20000));
        }
        n_attempts += 1;

        let url = get_predictions_url(&agency, &route, &stops);
        println!("{}", url);
        let downloaded: Option<PredictionsList> = client::download(&url).unwrap_or_else(|e| {
            warn!("Download error: {} from URL={}", e.display_chain().to_string(), url);
            None
        });
        let predictions = match downloaded {
            Some(predictions) => predictions,
            None => continue,
        };
        let predictions_json = serde_json::to_string(&predictions).unwrap();
        println!("{}", predictions_json);
    }
}
