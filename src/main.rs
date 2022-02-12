//use std::collections::HashMap;
use url::{Url};
use serde::{Deserialize, Serialize};

extern crate dotenv;

//use dotenv::dotenv;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use std::fs;
use std::net::{IpAddr};
use std::collections::HashMap;

use actix_files::NamedFile;
use actix_web::{HttpRequest, Result};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
struct Currency {
    code: String,
    name: String,
    symbol: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Timezone {
    name: String,
    offset: i16,
    current_time: String,
    current_time_unix: f64,
    is_dst: bool,
    dst_savings: i64
}
#[derive(Serialize, Deserialize, Debug)]
struct IPRequest {
    ip: String,
    continent_code: String,
    continent_name: String,
    country_code2: String,
    country_code3: String,
    country_name: String,
    country_capital: String,
    state_prov: String,
    district: String,
    city: String,
    zipcode: String,
    latitude: String,
    longitude: String,
    is_eu: bool,
    calling_code: String,
    country_tld: String,
    languages: String,
    country_flag: String,
    geoname_id: String,
    isp: String,
    connection_type: String,
    organization: String,
    currency: Currency,
    time_zone: Timezone

}


#[derive(Serialize, Deserialize, Debug)]
struct IPGeolocation {
    ip: String,
    country_name : String,
}

/**
 * todo: muokkaa käyttämään ip tyyppiä
 */
#[tokio::main]
async fn ip_geolookup_api(ip : &String) -> Result<IPRequest, Box<dyn std::error::Error>> {
    let apikey = std::env::var("GEOAPIKEY").expect("NOT FOUND");
    
    let url = Url::parse_with_params("https://api.ipgeolocation.io/ipgeo",
    &[("apiKey", apikey), ("ip", ip.to_owned())])?;
    //println!("{:#?}", url);
    println!("{}",url.as_str());
    let parsed_url = format!("{}",url.as_str());
    let resp = reqwest::get(parsed_url)
        .await?
        .json::<IPRequest>()
        .await?;
    // let resp = reqwest::get(parsed_url).await?;
    println!("{:#?}", resp);
    Ok(resp)
}

/**
 * todo: muokkaa käyttämään ip tyyppiä
 */
fn resolve_iplocation_by_country(ip_list : Vec<IpAddr>) -> Vec<IPGeolocation> {
    let mut country_list = Vec::new();

    for ip in &ip_list {
        let ip = format!("{:?}",ip ); // todo muokkaa käyttämään ip
        let res = ip_geolookup_api(&ip);
        match res {
            Ok(api_response) => {
                let elem = IPGeolocation {
                    ip: api_response.ip,
                    country_name: api_response.country_name
                };
                country_list.push(elem);
            },
            Err(error) => {
                eprintln!("Error resolving ip {:?}", error);
                continue
            }

        }
    } 

    return country_list;
}

fn read_ips_from_file() -> Vec<IpAddr> {
    
    let contents = fs::read_to_string("ip_list.txt")
    .expect("Something went wrong reading the file");

    let ip_list_str : Vec<&str> = contents.split(" ").collect();
    
    let mut ip_list = Vec::new();

    for elem in ip_list_str {
        let res = elem.parse::<IpAddr>();
        match res {
            Ok(ip) => ip_list.push(ip),
            Err(error) => eprintln!("Error parsing ip addressess {:?}", error)
        }
    }

    return ip_list;
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{

    dotenv::dotenv().ok();

    HttpServer::new(|| {
        App::new()
            .service(return_dummy_data)
            .service(get_all_data_handler)
            .service(get_country_statistics)
            .route("/{index.html:.*}", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}


fn get_requests_count_by_country(ip_list: Vec<IPGeolocation>) -> HashMap<String, i64> {
    let mut map = HashMap::new();

    for i in ip_list.iter() {
        if let Some(x) =  map.get_mut(&i.country_name) {
            *x = *x + 1;
        } else {
            map.insert(i.country_name.to_owned(), 1);
        }
        
    }

    return map;
    
}

#[get("/data/countrystats")]
async fn get_country_statistics() -> impl Responder {
    
    let ip_list = read_ips_from_file();
    let res = resolve_iplocation_by_country(ip_list);
    let geo_locations = get_requests_count_by_country(res);
    let j = serde_json::to_string(&geo_locations);

    match j {
        Ok(json) => {
            return HttpResponse::Ok().body(json)
        }
        Err(_) => {
            return HttpResponse::Ok().body("virhe")
        }
    }
    

}

async fn index(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("index.html").parse().unwrap();
    Ok(NamedFile::open(path)?)
}



#[get("/data/all")]
async fn get_all_data_handler() -> impl Responder {
    let ip_list = read_ips_from_file();
    let res = resolve_iplocation_by_country(ip_list);
    let j = serde_json::to_string(&res);

    match j {
        Ok(json) => {
            return HttpResponse::Ok().body(json)
        }
        Err(_) => {
            return HttpResponse::Ok().body("virhe")
        }
    }
    
    
    // let json = format!("{}", j);

    // HttpResponse::Ok().body(j)
}

#[get("/data/dummy")]
async fn return_dummy_data() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}


