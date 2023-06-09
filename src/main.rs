use actix_web::{get, App, HttpResponse, HttpServer, Responder};

use bom_radar_rs::bom_radar_gif_encoder;

#[get("/")]
async fn bom_radar_gif() -> impl Responder {
    let mut bom_radar = bom_radar_gif_encoder::BomRadarGifEncoder::new(
        "IDR713".to_string(),
        "IDR71B".to_string(), 
        "/home/pimeson/temp/".to_string()
    ).unwrap();

    bom_radar.make_gif().unwrap();
    let gif_data = bom_radar.write_radar_gif().unwrap();

    HttpResponse::Ok()
        .content_type("image/gif")
        .body(gif_data)    
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(bom_radar_gif)
    })
        .bind(("0.0.0.0", 9009))?
        .run()
        .await
}


// notes: 
// Terrey Hills 128km radar loop
// http://www.bom.gov.au/products/IDR713.loop.shtml
// 
// more information on the API: http://www.bom.gov.au/catalogue/data-feeds.shtml
// FTP access procedures: http://reg.bom.gov.au/other/Ftp.shtml
// 
// [ ..., "IDR713.background.png", "IDR713.catchments.png", "IDR713.locations.png", "IDR713.rail.png", 
// "IDR713.range.png", "IDR713.roads.png", "IDR713.topography.png", "IDR713.waterways.png", 
// "IDR713.wthrDistricts.png", ... ]
// 
// [ ..., "IDR713.T.202306081319.png", "IDR713.T.202306081324.png", "IDR713.T.202306081329.png", 
// "IDR713.T.202306081334.png", "IDR713.T.202306081339.png", "IDR713.T.202306081344.png", 
// "IDR713.T.202306081349.png", "IDR713.T.202306081354.png", "IDR713.T.202306081359.png", 
// "IDR713.T.202306081404.png", "IDR713.T.202306081409.png", "IDR713.T.202306081414.png", 
// "IDR713.T.202306081419.png", "IDR713.T.202306081424.png", "IDR713.T.202306081429.png", 
// "IDR713.T.202306081434.png", "IDR713.T.202306081439.png", "IDR713.T.202306081444.png", 
// "IDR713.T.202306081449.png", "IDR713.T.202306081454.png", ... ]
// 
