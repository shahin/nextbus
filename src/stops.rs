use error_chain::ChainedError;
use std::collections::HashMap;

use serde_json;

use client;
use client::from_string;
use errors::*;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RouteConfig {
    #[serde(rename = "route")]
    pub routes: Vec<Route>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Route {
    pub tag: String,
    pub title: String,
    pub latMin: String,
    pub latMax: String,
    pub lonMin: String,
    pub lonMax: String,
    #[serde(rename = "stop")]
    pub stops: Vec<Stop>,
    #[serde(rename = "direction")]
    pub directions: Vec<Direction>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[derive(Clone)]
struct Stop {
    pub tag: String,
    pub title: String,
    pub lat: String,
    pub lon: String,
    pub stop_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Direction {
    pub tag: String,
    pub title: String,
    pub name: String,
    #[serde(rename = "useForUI")]
    pub use_for_ui: bool,
    #[serde(rename = "stop")]
    pub stop_tags: Vec<StopTag>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct StopTag {
    pub tag: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FlatRoute {
    pub tag: String,
    pub title: String,
    pub latMin: String,
    pub latMax: String,
    pub lonMin: String,
    pub lonMax: String,
    pub directions: Vec<FlatDirection>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FlatDirection {
    pub tag: String,
    pub title: String,
    pub name: String,
    #[serde(rename = "useForUI")]
    pub use_for_ui: bool,
    pub stops: Vec<Stop>
}

impl client::Contents for RouteConfig {
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

fn _get_stops(agency: &String, route: &String) -> Result<Vec<FlatRoute>> {
    let url = get_stops_url(agency, route);
    let downloaded: Option<RouteConfig> = client::download(&url).unwrap_or_else(|e| {
        warn!("Download error: {} from URL={}", e.display_chain().to_string(), url);
        None
    });
    let route_config = downloaded.unwrap();

    let stops_for_tags: HashMap<String, Stop> = route_config.routes.iter().flat_map(|r: &Route| {
        r.stops.iter().map(|s: &Stop| (s.tag.clone(), s.clone())).collect::<Vec<(String, Stop)>>()
    }).collect();

    let flats: Vec<FlatRoute> = route_config.routes.iter().map(|r| {
        FlatRoute {
            tag: r.tag.clone(),
            title: r.title.clone(),
            latMin: r.latMin.clone(),
            latMax: r.latMax.clone(),
            lonMin: r.lonMin.clone(),
            lonMax: r.lonMax.clone(),
            directions: r.directions.iter().map(|d| {
                FlatDirection{
                    tag: d.tag.clone(),
                    title: d.title.clone(),
                    name: d.name.clone(),
                    use_for_ui: d.use_for_ui.clone(),
                    stops: d.stop_tags.iter().map(|st| {
                        Stop {
                            tag: st.tag.clone(),
                            title: stops_for_tags[&st.tag].title.clone(),
                            lat: stops_for_tags[&st.tag].lat.clone(),
                            lon: stops_for_tags[&st.tag].lon.clone(),
                            stop_id: stops_for_tags[&st.tag].stop_id.clone(),
                        }
                    }).collect(),
                }
            }).collect(),
        }
    }).collect();

    // TODO: for each route_config.directions, turn it into a FlatDirection
    // create a map from stopTag: Stop, and for each route.directions[i].stop_tags loop up the Stop
    Ok(flats)
}

pub fn get_stops(agency: String, route: String) -> Result<()> {
    let stops = _get_stops(&agency, &route)?;
    let stops_json = serde_json::to_string(&stops).unwrap();
    println!("{}", stops_json);
    Ok(())
}

pub fn get_stop_tags(agency: &String, route: &String) -> Result<Vec<String>> {
    let route_list = _get_stops(agency, route)?;
    let mut stop_tags: Vec<String> = route_list.into_iter().flat_map(|r: FlatRoute| {
        r.directions.into_iter().flat_map(|d: FlatDirection| {
            d.stops.into_iter().map(|s: Stop| s.tag).collect::<Vec<String>>()
        }).collect::<Vec<String>>()
    }).collect();
    println!("{:?}", stop_tags);
    stop_tags.sort_unstable();
    stop_tags.dedup();
    let ret: Vec<String> = vec![String::from("a")];
    Ok(ret)
}
