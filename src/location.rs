use error_chain::ChainedError;

use serde_json;
use std::thread;
use std::time::Duration;

use client;
use errors::*;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Vehicle {
    pub id: String,
    pub route_tag: String,
    // some vehicles seem to be missing the dirTag attribute, e.g. occasionally the C route
    #[serde(default)]
    pub dir_tag: String,
    #[serde(deserialize_with = "client::from_string")]
    pub lat: f32,
    #[serde(deserialize_with = "client::from_string")]
    pub lon: f32,
    #[serde(deserialize_with = "client::from_string")]
    pub secs_since_report: u32,
    #[serde(deserialize_with = "client::from_string")]
    pub predictable: bool,
    #[serde(deserialize_with = "client::from_string")]
    pub heading: i16,
    #[serde(deserialize_with = "client::from_string")]
    pub speed_km_hr: u32,
    #[serde(default)]
    pub leading_vehicle_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LastTime {
    #[serde(deserialize_with = "client::from_string")]
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
    pub leading_vehicle_id: String,
}

impl client::Contents for Locations {
    fn is_empty(&self) -> bool {
        self.vehicles.len() == 0
    }
}

fn get_locations_url(agency: &String, route: &String, epoch: &u64) -> String {
    format!(
        "https://retro.umoiq.com/service/publicXMLFeed?command=vehicleLocations&a={agency}&r={route}&t={epoch:?}",
        agency = agency,
        epoch = epoch,
        route = route,
    )
}

pub fn get_locations(agency: String, route: String, pause_seconds: Option<u64>) -> Result<()> {
    let mut epoch = 0;

    loop {
        let url = get_locations_url(&agency, &route, &epoch);
        let downloaded: Option<Locations> = client::download(&url).unwrap_or_else(|e| {
            warn!(
                "Download error: {} from URL={}",
                e.display_chain().to_string(),
                url
            );
            None
        });

        match downloaded {
            Some(locations) => {
                let locations_json;
                ((locations_json, epoch) = parse_locations(locations));
                println!("{}", locations_json);
            }
            None => (),
        };

        match pause_seconds {
            None => return Ok(()),
            Some(s) => thread::sleep(Duration::from_millis(s * 1000)),
        }
    }
}

fn parse_locations(locations: Locations) -> (String, u64) {
    let updated_time = locations.updated_time.time;

    let location_times: Vec<VehicleTime> = locations
        .vehicles
        .into_iter()
        .map(|v| VehicleTime {
            id: v.id,
            route_tag: v.route_tag,
            dir_tag: v.dir_tag,
            lat: v.lat,
            lon: v.lon,
            predictable: v.predictable,
            heading: v.heading,
            speed_km_hr: v.speed_km_hr,
            leading_vehicle_id: v.leading_vehicle_id,
            epoch: updated_time - ((v.secs_since_report * 1000) as u64),
        })
        .collect();

    let locations_json = serde_json::to_string(&location_times).unwrap();

    return (locations_json, updated_time);
}
