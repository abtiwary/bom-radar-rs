use std::str;
use std::io::{Write, BufReader, Read, Cursor, Seek, SeekFrom};

use anyhow::{Result, Context};
use image::{io::Reader, DynamicImage, codecs::gif::{GifEncoder, Repeat}, Frame, Delay};
use suppaftp::{FtpStream, list::File};

pub struct BomRadarGifEncoder {
    prod_id: String,
    prod_id_2: String,
    gif_data: Cursor<Vec<u8>>,
    temp_files_dir: String,
    ftp_stream: FtpStream
}

impl BomRadarGifEncoder {
    pub fn new(prod_id: String, prod_id_2: String, temp_files_dir: String) -> Result<Self> {
        Ok(
            BomRadarGifEncoder { 
                prod_id: prod_id.clone(), 
                prod_id_2: prod_id_2.clone(),
                gif_data: Cursor::new(Vec::new()),
                temp_files_dir: temp_files_dir,
                ftp_stream: FtpStream::connect("ftp.bom.gov.au:21")?
            }
        )
    }

    pub fn make_gif(&mut self) -> Result<()> {
        self.ftp_stream.login("anonymous", "guest")?;
        println!("Current directory: {}", self.ftp_stream.pwd().unwrap());
      
        self.ftp_stream.cwd("anon/gen/radar_transparencies")?;   
        println!("Current directory: {}", self.ftp_stream.pwd().unwrap());

        //let files2: Vec<String> = self.ftp_stream.nlst(None)?.iter().map(|x| x.clone()).collect();
        //println!("{:?}", files2);
        
        let extra_layers: Vec<&str> = vec!["catchments", "topography", "waterways", "locations"];
      
        let base_image_path = String::from(format!("{0}.background.png", self.prod_id));
        let base_image = self.ftp_stream.retr_as_buffer(&base_image_path)?;
        let base_image_reader = Reader::new(base_image)
                                                          .with_guessed_format()?;
      
        let base_img = base_image_reader.decode()?;
        let mut base_img_converted = DynamicImage::to_rgba8(&base_img);

        // now download all the "transparencies" to layer atop the base image
        for layer in extra_layers {                         
            let layer_image_path = String::from(format!("{0}.{1}.png", self.prod_id, layer));
            let layer_image = self.ftp_stream.retr_as_buffer(&layer_image_path)?;
                     
            let layer_image_reader = Reader::new(layer_image)
                                                                     .with_guessed_format()?;
                 
            let layer_img = layer_image_reader.decode()?;
            let layer_img_converted = DynamicImage::to_rgba8(&layer_img);
                 
            image::imageops::overlay(&mut base_img_converted, &layer_img_converted, 0, 0);
        }
                 
        //base_img_converted.save(format!("{0}base_bom.png", self.temp_files_dir))?;

        self.ftp_stream.cwd("/")?;
        self.ftp_stream.cwd("anon/gen/radar")?;
      
        let files3: Vec<String> = self.ftp_stream.nlst(None)?.iter().map(|x| x.clone()).collect();
        //println!("{:?}", files3);
  
        let relevant_prod_files: Vec<&String> = files3
                                                .iter()
                                                .filter(|x| x.contains(&self.prod_id_2) && x.contains(".png"))
                                                .collect();
        //println!("{:?}", relevant_prod_files);
      
        let last_seven_relevant: Vec<&&String> = relevant_prod_files.iter().rev().take(7).collect();
        //println!("{:?}", last_seven_relevant);

        let mut vec_frames = Vec::with_capacity(7);
        let mut idx = (last_seven_relevant.len() - 1) as isize;
        let mut width = 0;
        let mut height = 0;
      
        while idx > -1 {
            let mut base_img_copy = base_img_converted.clone();
  
            let radar_img_name = last_seven_relevant[idx as usize].to_string();
  
            let layer_image = self.ftp_stream.retr_as_buffer(&radar_img_name)?;
  
            let layer_image_reader = Reader::new(layer_image)
                                                                .with_guessed_format()?;
  
            let layer_img = layer_image_reader.decode()?;
            let layer_img_converted = DynamicImage::to_rgba8(&layer_img);
            image::imageops::overlay(&mut base_img_copy, &layer_img_converted, 0, 0);
  
            //base_img_copy.save(format!("{0}base_bom_{1}.png", self.temp_files_dir, idx))?;

            if width == 0 && height == 0 {
                width = layer_img_converted.width();
                height = layer_img_converted.height();
            }
 
            vec_frames.push(base_img_copy);
 
            idx -= 1;
        }
 
        //let mut gif_image = GifEncoder::new(&gif_image_file);
        let mut gif_image = GifEncoder::new(&mut self.gif_data);

        gif_image.set_repeat(Repeat::Infinite)?;
     
        for gif_f in vec_frames {
            let delay_400ms = Delay::from_numer_denom_ms(400, 1);
            let frame = Frame::from_parts(gif_f, 0, 0, delay_400ms);
 
            gif_image.encode_frame(frame)?;                                                                                            
        }

        Ok(())
    }

    pub fn write_radar_gif(&mut self) -> Result<Vec<u8>> {
        let mut gif_image_file = std::fs::File::create(format!("{0}radar.gif", self.temp_files_dir))?;
        
        let mut gif_data_buf: Vec<u8> = Vec::new();
        self.gif_data.seek(SeekFrom::Start(0))?;
        self.gif_data.read_to_end(&mut gif_data_buf)?;

        gif_image_file.write_all(&gif_data_buf)?;

        Ok(gif_data_buf)
    }

}

impl Drop for BomRadarGifEncoder {
    fn drop(&mut self) {
        self.ftp_stream.quit().ok();
    }
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
