use error_chain::ChainedError;
use serde_json;
use client;
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
    pub tag: String,
    pub title: String,
}

impl client::Contents for Routes {
    fn is_empty(&self) -> bool {
        self.routes.len() == 0
    }
}

fn get_routes_url(agency: &String) -> String {
    format!(
        "https://retro.umoiq.com/service/publicXMLFeed?command=routeList&a={agency}",
        agency = agency,
    )
}

fn _get_routes(agency: &String) -> Result<Routes> {
    let url = get_routes_url(agency);
    let downloaded: Option<Routes> = client::download(&url).unwrap_or_else(|e| {
        warn!("Download error: {} from URL={}", e.display_chain().to_string(), url);
        None
    });
    let routes = downloaded.unwrap();
    Ok(routes)
}

pub fn get_routes(agency: String) -> Result<()> {
    let routes = _get_routes(&agency)?;
    let routes_json = serde_json::to_string(&routes).unwrap();
    println!("{}", routes_json);
    Ok(())
}

pub fn get_route_tags(agency: &String) -> Result<Vec<String>> {
    let route_list = _get_routes(agency)?;
    let routes: Vec<String> = route_list.routes.into_iter().map(|r| r.tag).collect();
    Ok(routes)
}
