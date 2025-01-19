use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::{routing::get, Router};

use bom_radar_rs::bom_radar_gif_encoder;

fn init_router() -> Router {
    Router::new().route("/", get(bom_radar_gif))
}

async fn bom_radar_gif() -> impl IntoResponse {
    let mut bom_radar = bom_radar_gif_encoder::BomRadarGifEncoder::new(
        "IDR713".to_string(),
        "IDR71B".to_string(),
        "/home/pimeson/temp/".to_string(),
    )
    .unwrap();

    bom_radar.make_gif().unwrap();
    let gif_data = bom_radar.write_radar_gif().unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("content-type", "image/gif".parse().unwrap());

    (headers, gif_data)
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = init_router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9009").await.unwrap();
    axum::serve(listener, app).await
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
