use error_chain::ChainedError;

use serde_json;

use client;
use client::from_string;
use errors::*;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Routes {
    #[serde(rename = "route")]
    pub routes: Vec<Route>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Route {
    #[serde(rename = "stop")]
    pub stops: Vec<Stop>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Stop {
    pub tag: String,
    pub title: String,
    pub lat: String,
    pub lon: String,
    pub stop_id: String,
}

impl client::Contents for Routes {
    fn is_empty(&self) -> bool {
        self.routes.len() == 0
    }
}

fn get_stops_url(agency: &String, route: &String) -> String {
    format!(
        "http://webservices.nextbus.com/service/publicXMLFeed?command=routeConfig&a={agency}&r={route}",
        agency = agency,
        route = route,
    )
}

fn _get_stops(agency: &String, route: &String) -> Result<Routes> {
    let url = get_stops_url(agency, route);
    let downloaded: Option<Routes> = client::download(&url).unwrap_or_else(|e| {
        warn!("Download error: {} from URL={}", e.display_chain().to_string(), url);
        None
    });
    let stops = downloaded.unwrap();
    Ok(stops)
}

pub fn get_stops(agency: String, route: String) -> Result<()> {
    let stops = _get_stops(&agency, &route)?;
    let stops_json = serde_json::to_string(&stops).unwrap();
    println!("{}", stops_json);
    Ok(())
}

pub fn get_stop_ids(agency: &String, route: &String) -> Result<Vec<String>> {
    let route_list = _get_stops(agency, route)?;
    let stops: Vec<Stop> = route_list.routes.into_iter().flat_map(|r| r.stops).collect();
    let mut stop_ids: Vec<String> = stops.into_iter().map(|s| s.tag).collect();
    stop_ids.sort_unstable();
    stop_ids.dedup();
    Ok(stop_ids)
}
