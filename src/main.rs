extern crate reqwest;
extern crate env_logger;
extern crate serde;
extern crate serde_xml_rs;
extern crate serde_json;
extern crate clap;
#[macro_use] extern crate error_chain;
use error_chain::ChainedError;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;

use serde_xml_rs::deserialize;
use std::fmt::Display;
use std::str::FromStr;
use serde::de::{self, Deserializer, Deserialize};
use std::result::Result as StdResult;
use std::collections::HashMap;
use std::thread;
use clap::{Arg, App, SubCommand};


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Vehicle {
    pub id: String,
    pub route_tag: String,
    // some vehicles seem to be missing the dirTag attribute, e.g. occasionally the C route
    #[serde(default)]
    pub dir_tag: String,
    #[serde(deserialize_with = "from_string")]
    pub lat: f32,
    #[serde(deserialize_with = "from_string")]
    pub lon: f32,
    #[serde(deserialize_with = "from_string")]
    pub secs_since_report: u32,
    #[serde(deserialize_with = "from_string")]
    pub predictable: bool,
    #[serde(deserialize_with = "from_string")]
    pub heading: i16,
    #[serde(deserialize_with = "from_string")]
    pub speed_km_hr: u32,
    #[serde(default)]
    pub leading_vehicle_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LastTime {
    #[serde(deserialize_with = "from_string")]
    pub time: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Locations {
    #[serde(rename = "vehicle", default)]
    pub vehicles: Vec<Vehicle>,

    #[serde(rename = "lastTime")]
    pub updated_time: LastTime,
}

#[derive(Serialize, Debug)]
struct VehicleTime {
    pub id: String,
    pub route_tag: String,
    pub dir_tag: String,
    pub lat: f32,
    pub lon: f32,
    pub epoch: u64,
    pub predictable: bool,
    pub heading: i16,
    pub speed_km_hr: u32,
}

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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Schedule {
    #[serde(rename = "route")]
    pub routes: Vec<Route>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Route {
    pub tag: String,
    pub title: String,
    pub schedule_class: String,
    pub service_class: String,
    pub direction: String,
    #[serde(rename = "tr")]
    pub blocks: Vec<VehicleBlock>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct VehicleBlock {
    #[serde(rename = "blockID")]
    pub block_id: String,
    #[serde(rename = "stop")]
    pub stops: Vec<VehicleStop>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct VehicleStop {
    pub tag: String,
    #[serde(deserialize_with = "from_string", default)]
    pub epoch_time: i64,
}

pub trait Contents {
    fn is_empty(&self) -> bool;
}

impl Contents for Locations {
    fn is_empty(&self) -> bool {
        self.vehicles.len() == 0
    }
}

impl Contents for PredictionsList {
    fn is_empty(&self) -> bool {
        self.predictions.len() == 0
    }
}

impl Contents for Schedule {
    fn is_empty(&self) -> bool {
        self.routes.len() == 0
    }
}

// Explicit deserialization converter from a String to a FromStr-implementer
// https://github.com/serde-rs/json/issues/317
fn from_string<'de, T, D>(deserializer: D) -> StdResult<T, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}


error_chain! {
    foreign_links {
        ReqError(reqwest::Error);
        IoError(std::io::Error);
        SerdeError(serde_xml_rs::Error);
    }
}

fn get_schedule_url(agency: &String, route: &String) -> String {
    format!(
        "http://webservices.nextbus.com/service/publicXMLFeed?command=schedule&a={agency}&r={route}",
        agency = agency,
        route = route,
    )
}

fn get_predictions_url(agency: &String, route: &String, stops: &Vec<String>) -> String {
    let route_stops: Vec<String> = stops.into_iter().map(|s| route.to_string() + "|" + s).collect();
    format!(
        "http://webservices.nextbus.com/service/publicXMLFeed?command=predictionsForMultiStops&a={agency}&stops={stops}",
        agency = agency,
        stops = route_stops.join("&stops="),
    )
}

fn get_locations_url(agency: &String, route: &String, epoch: &u64) -> String {
    format!(
        "http://webservices.nextbus.com/service/publicXMLFeed?command=vehicleLocations&a={agency}&r={route}&t={epoch:?}",
        agency = agency,
        epoch = epoch,
        route = route,
    )
}

fn get_predictions(agency: String, route: String, stops: Vec<String>) -> Result<()> {
    let mut n_attempts = 0;

    let stops = match stops.len() {
        0 => get_stops(&agency, &route)?,
        _ => stops
    };

    loop {
        if n_attempts > 0 {
            thread::sleep(std::time::Duration::from_millis(20000));
        }
        n_attempts += 1;

        let url = get_predictions_url(&agency, &route, &stops);
        println!("{}", url);
        let downloaded: Option<PredictionsList> = download(&url).unwrap_or_else(|e| {
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

fn _get_schedule(agency: &String, route: &String) -> Result<Schedule> {
    let url = get_schedule_url(agency, route);
    let downloaded: Option<Schedule> = download(&url).unwrap_or_else(|e| {
        warn!("Download error: {} from URL={}", e.display_chain().to_string(), url);
        None
    });
    let schedule = downloaded.unwrap();
    Ok(schedule)
}

fn get_schedule(agency: String, route: String) -> Result<()> {
    let schedule = _get_schedule(&agency, &route)?;
    let schedule_json = serde_json::to_string(&schedule).unwrap();
    println!("{}", schedule_json);
    Ok(())
}

fn get_stops(agency: &String, route: &String) -> Result<Vec<String>> {
    let routes: Vec<Route> = _get_schedule(agency, route)?.routes;
    let blocks: Vec<VehicleBlock> = routes.into_iter().flat_map(|r| r.blocks).collect();
    let stops: Vec<VehicleStop> = blocks.into_iter().flat_map(|b| b.stops).collect();
    let mut stop_tags: Vec<String>  = stops.into_iter().map(|s| s.tag).collect();
    stop_tags.sort_unstable();
    stop_tags.dedup();
    Ok(stop_tags)
}

fn get_locations(agency: String, route: String) -> Result<()> {
    let mut times = HashMap::new();

    loop {

        thread::sleep(std::time::Duration::from_millis(1000));

        let epoch = match times.get(&route) {
            Some(&i) => i,
            None => 0,
        };

        let url = get_locations_url(&agency, &route, &epoch);
        let downloaded: Option<Locations> = download(&url).unwrap_or_else(|e| {
            warn!("Download error: {} from URL={}", e.display_chain().to_string(), url);
            None
        });
        let locations = match downloaded {
            Some(locations) => locations,
            None => { continue },
        };

        let updated_time = locations.updated_time.time;
        times.insert(route.clone(), updated_time);

        let location_times: Vec<VehicleTime> = locations.vehicles.into_iter().map(|v| VehicleTime {
            id: v.id,
            route_tag: v.route_tag,
            dir_tag: v.dir_tag,
            lat: v.lat,
            lon: v.lon,
            predictable: v.predictable,
            heading: v.heading,
            speed_km_hr: v.speed_km_hr,
            epoch: updated_time - ((v.secs_since_report * 1000) as u64),
        }).collect();

        let locations_json = serde_json::to_string(&location_times).unwrap();
        println!("{}", locations_json);
    }

}

fn download<'de, T>(url: &String) -> Result<Option<T>> where
    T: Deserialize<'de> + std::fmt::Debug + Contents {

    let mut response = reqwest::get(&url[..])?;
    let body = response.text()?;
    let date = response.headers().get::<reqwest::header::Date>()
        .map(|d| **d)
        .unwrap();
    let status = response.status();
    match status {
        reqwest::StatusCode::Ok => {
            debug!(r#"request="{}" response="{}" response_date="{}""#, url, status, date);
            deserialize(body.as_bytes())
                .and_then(|d: T| {
                    // TODO: add is_empty() trait to Locations, PredictionList, ... to return None here if we got nothing
                    if d.is_empty() {
                        Ok(None)
                    }
                    else {
                        Ok(Some(d))
                    }
                })
                .chain_err(|| "Deserialization failed.")
        },
        _ => {
            warn!(r#"request="{}" response="{}" response_date="{}""#, url, status, date);
            return Err(format!("Bad response: {}", status).into());
        },
    }

}

fn main() {
    env_logger::init();

    let cli = App::new("Nextbus Client")
        .author("Shahin Saneinejad")
        .about("Get real-time locations of transit vehicles as JSON")
        .subcommand(SubCommand::with_name("locations")
            .about("Get real-time locations for vehicles")
            .args_from_usage("<agency> 'Agency of the route to retrieve locations for (ex: sf-muni)'")
            .args_from_usage("[route] 'Optional name of the route to retrieve locations for (default: all routes)'")
        )
        .subcommand(SubCommand::with_name("predictions")
            .about("Get predictions for vehicle arrival times")
            .args(&[
                Arg::with_name("agency")
                    .help("Agency of the route to retrieve locations for (ex: sf-muni)")
                    .index(1)
                    .required(true),
                Arg::with_name("route")
                    .help("Route to get predictions for (ex: N)")
                    .index(2)
                    .required(true),
                Arg::with_name("stops")
                    .help("Stop tags to get predictions for (ex: 6997)")
                    .required(false)
                    .multiple(true)
                    .use_delimiter(true)
                    .value_delimiter(" ")
                    .last(true),
            ])
        )
        .subcommand(SubCommand::with_name("schedule")
            .about("Get the published schedule for a route")
            .args(&[
                Arg::with_name("agency")
                    .help("Agency of the route to schedules for (ex: sf-muni)")
                    .index(1)
                    .required(true),
                Arg::with_name("route")
                    .help("Route to retrieve schedules for (ex: N)")
                    .index(2)
                    .required(true)
                    .multiple(true),
            ])
        )
        .get_matches();

    match cli.subcommand() {
        ("locations", Some(subc)) => {
            let route = String::from(subc.value_of("route").unwrap_or(""));
            let agency = String::from(subc.value_of("agency").unwrap());
            get_locations(agency, route)
        },
        ("predictions", Some(subc)) => {
            let route = String::from(subc.value_of("route").unwrap_or(""));
            let agency = String::from(subc.value_of("agency").unwrap());
            let stops: Vec<String> = match subc.values_of("stops") {
                Some(stops) => stops.collect::<Vec<_>>().into_iter().map(|s| String::from(s)).collect(),
                None => Vec::new(),
            };
            get_predictions(agency, route, stops)
        },
        ("schedule", Some(subc)) => {
            let route = String::from(subc.value_of("route").unwrap_or(""));
            let agency = String::from(subc.value_of("agency").unwrap());
            get_schedule(agency, route)
        },
        (c, Some(_)) => panic!("Unimplemented subcommand '{}'", c),
        _ => panic!("Missing or invalid subcommand"),
    };

}
