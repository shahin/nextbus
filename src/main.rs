extern crate reqwest;
extern crate env_logger;
extern crate serde;
extern crate serde_xml_rs;
extern crate serde_json;
extern crate clap;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;

use serde_xml_rs::deserialize;
use std::fmt::Display;
use std::str::FromStr;
use serde::de::{self, Deserializer, Deserialize};
use std::result::Result as StdResult;
use std::collections::HashMap;
use std::thread;
use clap::App;


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
    }
}

fn get_api_url(agency: &String, route: &String, epoch: &u64) -> String {
    format!(
        "http://webservices.nextbus.com/service/publicXMLFeed?command=vehicleLocations&a={agency}&r={route}&t={epoch:?}",
        agency = agency,
        epoch = epoch,
        route = route,
    )
}

fn run(agency: String, route: String) -> Result<()> {
    env_logger::init();
    let mut times = HashMap::new();

    loop {

        thread::sleep(std::time::Duration::from_millis(1000));

        let epoch = match times.get(&route) {
            Some(&i) => i,
            None => 0,
        };

        let downloaded = download_locations(&agency, &route, &epoch).unwrap_or_else(|e| {
            warn!("Error downloading locations: {}", e);
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

fn download_locations(agency: &String, route: &String, epoch: &u64) -> Result<Option<Locations>> {
    let url = get_api_url(agency, route, epoch);

    let mut response = reqwest::get(&url[..])?;
    let body = response.text()?;
    let date = response.headers().get::<reqwest::header::Date>()
        .map(|d| **d)
        .unwrap();
    let status = response.status();
    match status {
        reqwest::StatusCode::Ok => {
            debug!(r#"request="{}" response="{}" response_date="{}""#, url, status, date);
            println!("{:?}", body);
            let locations: Option<Locations> = deserialize(body.as_bytes()).ok().and_then(|any_locs: Locations| {
                if any_locs.vehicles.len() == 0 {
                    return Some(any_locs)
                }
                None
            });
            return Ok(locations);
        },
        _ => {
            warn!(r#"request="{}" response="{}" response_date="{}""#, url, status, date);
            Err(format!("Bad response: {}", status).into())
        },
    }

}

fn main() {
    let cli = App::new("Nextbus Client")
        .author("Shahin Saneinejad")
        .about("Get real-time locations of transit vehicles as JSON")
        .args_from_usage("<agency> 'Agency of the route to retrieve locations for (ex: sf-muni)'")
        .args_from_usage("[route] 'Optional name of the route to retrieve locations for (default: all routes)'")
        .get_matches();

    let route = String::from(cli.value_of("route").unwrap_or(""));
    let agency = String::from(cli.value_of("agency").unwrap());

    run(agency, route);
}
