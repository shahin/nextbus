extern crate reqwest;
extern crate env_logger;
#[macro_use] extern crate log;
extern crate serde;
extern crate serde_xml_rs;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate error_chain;

extern crate clap;

use clap::{Arg, App, SubCommand};

pub trait Contents {
    fn is_empty(&self) -> bool;
}

mod errors {
	error_chain! {
		foreign_links {
			ReqError(::reqwest::Error);
			IoError(::std::io::Error);
			SerdeError(::serde_xml_rs::Error);
		}
	}
}

mod client;
mod location;
mod schedule;
mod prediction;
mod routes;
mod stops;

fn main() {
    env_logger::init();

    let version_string: &str = &format!(
        "{} {} {}",
        env!("VERGEN_SEMVER"), env!("VERGEN_SHA"), env!("VERGEN_BUILD_TIMESTAMP"),
    );

    let cli = App::new("Nextbus Client")
        .version(version_string)
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
        .subcommand(SubCommand::with_name("stops")
            .about("Get the published stops for a route")
            .args(&[
                Arg::with_name("agency")
                    .help("Agency of the route to stops for (ex: sf-muni)")
                    .index(1)
                    .required(true),
                Arg::with_name("route")
                    .help("Route to retrieve stops for (ex: N)")
                    .index(2)
                    .required(true)
                    .multiple(true),
            ])
        )
        .subcommand(SubCommand::with_name("routes")
            .about("Get the published routes for an agency")
            .args(&[
                Arg::with_name("agency")
                    .help("Agency of the route to stops for (ex: sf-muni)")
                    .index(1)
                    .required(true),
            ])
        )
        .get_matches();

    match cli.subcommand() {
        ("locations", Some(subc)) => {
            let route = String::from(subc.value_of("route").unwrap_or(""));
            let agency = String::from(subc.value_of("agency").unwrap());
            location::get_locations(agency, route)
        },
        ("predictions", Some(subc)) => {
            let route = String::from(subc.value_of("route").unwrap_or(""));
            let agency = String::from(subc.value_of("agency").unwrap());
            let stops: Vec<String> = match subc.values_of("stops") {
                Some(stops) => stops.collect::<Vec<_>>().into_iter().map(|s| String::from(s)).collect(),
                None => Vec::new(),
            };
            prediction::get_predictions(agency, route, stops)
        },
        ("schedule", Some(subc)) => {
            let route = String::from(subc.value_of("route").unwrap_or(""));
            let agency = String::from(subc.value_of("agency").unwrap());
            schedule::get_schedule(agency, route)
        },
        ("stops", Some(subc)) => {
            let route = String::from(subc.value_of("route").unwrap_or(""));
            let agency = String::from(subc.value_of("agency").unwrap());
            stops::get_stops(agency, route)
        },
        ("routes", Some(subc)) => {
            let agency = String::from(subc.value_of("agency").unwrap());
            routes::get_routes(agency)
        },
        (c, Some(_)) => panic!("Unimplemented subcommand '{}'", c),
        _ => panic!("Missing or invalid subcommand"),
    };

}
