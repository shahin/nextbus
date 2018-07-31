use error_chain::ChainedError;

use serde_json;

use client;
use client::from_string;
use errors::*;

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

impl client::Contents for Schedule {
    fn is_empty(&self) -> bool {
        self.routes.len() == 0
    }
}

fn get_schedule_url(agency: &String, route: &String) -> String {
    format!(
        "http://webservices.nextbus.com/service/publicXMLFeed?command=schedule&a={agency}&r={route}",
        agency = agency,
        route = route,
    )
}

fn _get_schedule(agency: &String, route: &String) -> Result<Schedule> {
    let url = get_schedule_url(agency, route);
    let downloaded: Option<Schedule> = client::download(&url).unwrap_or_else(|e| {
        warn!("Download error: {} from URL={}", e.display_chain().to_string(), url);
        None
    });
    let schedule = downloaded.unwrap();
    Ok(schedule)
}

pub fn get_schedule(agency: String, route: String) -> Result<()> {
    let schedule = _get_schedule(&agency, &route)?;
    let schedule_json = serde_json::to_string(&schedule).unwrap();
    println!("{}", schedule_json);
    Ok(())
}

pub fn get_stops(agency: &String, route: &String) -> Result<Vec<String>> {
    let routes: Vec<Route> = _get_schedule(agency, route)?.routes;
    let blocks: Vec<VehicleBlock> = routes.into_iter().flat_map(|r| r.blocks).collect();
    let stops: Vec<VehicleStop> = blocks.into_iter().flat_map(|b| b.stops).collect();
    let mut stop_tags: Vec<String>  = stops.into_iter().map(|s| s.tag).collect();
    stop_tags.sort_unstable();
    stop_tags.dedup();
    Ok(stop_tags)
}
