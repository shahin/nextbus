use error_chain::ChainedError;

use std::collections::HashMap;
use std::thread;
use std::time::{Duration};
use serde_json;

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
        "http://webservices.nextbus.com/service/publicXMLFeed?command=vehicleLocations&a={agency}&r={route}&t={epoch:?}",
        agency = agency,
        epoch = epoch,
        route = route,
    )
}

pub fn get_locations(agency: String, route: String) -> Result<()> {
    let mut times = HashMap::new();

    loop {

        thread::sleep(Duration::from_millis(5000));

        let epoch = match times.get(&route) {
            Some(&i) => i,
            None => 0,
        };

        let url = get_locations_url(&agency, &route, &epoch);
        let downloaded: Option<Locations> = client::download(&url).unwrap_or_else(|e| {
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
            leading_vehicle_id: v.leading_vehicle_id,
            epoch: updated_time - ((v.secs_since_report * 1000) as u64),
        }).collect();

        let locations_json = serde_json::to_string(&location_times).unwrap();
        println!("{}", locations_json);
    }

}
