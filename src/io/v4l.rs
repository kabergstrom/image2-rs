use std::io::Result;

use rscam;

pub struct Webcam {
    width: u32,
    height: u32,
    handle: rscam::Camera,
}

impl Webcam {
    fn config<'a>(width: u32, height: u32) -> rscam::Config<'a> {
        let mut cfg = rscam::Config::default();
        cfg.format = b"RGB3";
        cfg.resolution = (width, height);
        cfg.interval = (1, 30);
        cfg
    }

    pub fn new(device: &str, width: u32, height: u32) -> Result<Webcam> {
        let cam = rscam::Camera::new(device)?;

        Ok(Webcam {
            width,
            height,
            handle: cam,
        })
    }

    pub fn start(&mut self) -> rscam::Result<()> {
        let cfg = Self::config(self.width, self.height);
        self.handle.start(&cfg)
    }

    pub fn capture(&mut self) -> Result<crate::ImageBuf<u8, crate::Rgb>> {
        let frame = self.handle.capture()?;
        let (width, height) = frame.resolution;
        Ok(crate::ImageBuf::new_from(
            width as usize,
            height as usize,
            (*frame).to_vec(),
        ))
    }
}
