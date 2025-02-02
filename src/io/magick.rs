use std::io::{Read, Write};
use std::num::ParseIntError;
use std::path::Path;
use std::process::{Command, Stdio};
use std::usize;

use crate::color::Color;
use crate::image::Image;
use crate::image_buf::ImageBuf;
use crate::ty::Type;

#[derive(Debug)]
pub enum Error {
    InvalidImageShape,
    InvalidColor,
    FileDoesNotExist,
    InvalidImageData,
    UnableToExecuteCommand,
    ErrorWritingImage,
}

pub struct Magick {
    identify: &'static [&'static str],
    convert: &'static [&'static str],
}

pub fn kind<C: Color>() -> String {
    format!("{}:-", C::name())
}

fn depth<T: Type, C: Color>(cmd: &mut Command) {
    let depth = std::mem::size_of::<T>() * 8;
    cmd.arg("-depth");
    cmd.arg(format!("{}", depth));

    if T::is_float() {
        cmd.args(&["-define", "quantum:format=floating-point"]);
    }
}

pub const IM: Magick = Magick {
    identify: &["identify"],
    convert: &["convert"],
};

pub const GM: Magick = Magick {
    identify: &["gm", "identify"],
    convert: &["gm", "convert"],
};

pub static mut DEFAULT: Magick = IM;

/// Change default command
pub fn set_default(magick: Magick) {
    unsafe {
        DEFAULT = magick;
    }
}

impl Magick {
    /// Get size of image using identify command
    pub fn get_image_shape<P: AsRef<Path>>(&self, path: P) -> Result<(usize, usize), Error> {
        let identify = Command::new(self.identify[0])
            .args(self.identify[1..].iter())
            .args(&["-format", "%w %h"])
            .arg(path.as_ref())
            .output();

        let shape = match identify {
            Ok(shape) => shape,
            Err(_) => return Err(Error::InvalidImageShape),
        };

        let shape = match String::from_utf8(shape.stdout) {
            Ok(shape) => shape,
            Err(_) => return Err(Error::InvalidImageShape),
        };

        let t = shape
            .split(' ')
            .map(|a| a.trim().parse::<usize>())
            .collect::<Vec<Result<usize, ParseIntError>>>();

        if t.len() < 2 {
            return Err(Error::InvalidImageShape);
        }

        match (&t[0], &t[1]) {
            (Ok(a), Ok(b)) => Ok((a.clone(), b.clone())),
            (Err(_), _) | (_, Err(_)) => Err(Error::InvalidImageShape),
        }
    }

    /// Read image from disk using ImageMagick/GraphicsMagick
    pub fn read<P: AsRef<Path>, T: Type, C: Color>(
        &self,
        path: P,
    ) -> Result<ImageBuf<T, C>, Error> {
        let (width, height) = match self.get_image_shape(&path) {
            Ok((width, height)) => (width, height),
            Err(e) => return Err(e),
        };

        let kind = kind::<C>();
        let mut cmd = Command::new(self.convert[0]);
        cmd.args(self.convert[1..].iter()).arg(path.as_ref());
        depth::<T, C>(&mut cmd);
        cmd.arg(kind);

        let mut cmd = match cmd.output() {
            Ok(cmd) => cmd,
            Err(_) => return Err(Error::InvalidImageData),
        };

        let data = unsafe {
            Vec::from_raw_parts(
                cmd.stdout.as_mut_ptr() as *mut T,
                cmd.stdout.len() / std::mem::size_of::<T>(),
                cmd.stdout.capacity() / std::mem::size_of::<T>(),
            )
        };

        std::mem::forget(cmd);

        Ok(ImageBuf::new_from(width, height, data))
    }

    /// Write image to disk using ImageMagick/GraphicsMagick
    pub fn write<P: AsRef<Path>, T: Type, C: Color, I: Image<T, C>>(
        &self,
        path: P,
        image: &I,
    ) -> Result<(), Error> {
        let kind = kind::<C>();
        let (width, height, _) = image.shape();
        let size = format!("{}x{}", width, height);
        let mut cmd = Command::new(self.convert[0]);
        cmd.args(self.convert[1..].iter()).stdin(Stdio::piped());
        depth::<T, C>(&mut cmd);
        cmd.args(&["-size", size.as_str()])
            .arg(kind)
            .arg(path.as_ref());

        let mut proc = match cmd.spawn() {
            Ok(c) => c,
            Err(_) => return Err(Error::UnableToExecuteCommand),
        };

        {
            let mut stdin = proc.stdin.take().unwrap();
            match stdin.write_all(image.buffer()) {
                Ok(()) => (),
                Err(_) => return Err(Error::ErrorWritingImage),
            }
            let _ = stdin.flush();
        }

        match proc.wait() {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::UnableToExecuteCommand),
        }
    }

    /// Encode image to an im-memory buffer using ImageMagick/GraphicsMagick
    pub fn encode<T: Type, C: Color, I: Image<T, C>>(
        &self,
        format: &str,
        image: &I,
    ) -> Result<Vec<u8>, Error> {
        let kind = kind::<C>();
        let (width, height, _) = image.shape();
        let size = format!("{}x{}", width, height);
        let mut cmd = Command::new(self.convert[0]);
        cmd.args(self.convert[1..].iter())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());
        depth::<T, C>(&mut cmd);
        cmd.args(&["-size", size.as_str()])
            .arg(&kind)
            .arg(format!("{}:-", format));

        let mut proc = match cmd.spawn() {
            Ok(c) => c,
            Err(_) => return Err(Error::UnableToExecuteCommand),
        };

        {
            let mut stdin = proc.stdin.take().unwrap();
            match stdin.write_all(image.buffer()) {
                Ok(()) => (),
                Err(_) => return Err(Error::ErrorWritingImage),
            }
            let _ = stdin.flush();
        }

        match proc.wait() {
            Ok(_) => {
                let mut buffer: Vec<u8> = Vec::new();
                match proc.stdout.unwrap().read_to_end(&mut buffer) {
                    Ok(_) => (),
                    Err(_) => return Err(Error::InvalidImageData),
                }
                Ok(buffer)
            }
            Err(_) => Err(Error::UnableToExecuteCommand),
        }
    }
}

/// Read image from disk using default command-line tool
pub fn read<P: AsRef<Path>, T: Type, C: Color>(path: P) -> Result<ImageBuf<T, C>, Error> {
    unsafe { DEFAULT.read(path) }
}

/// Write image to disk using default command-line tool
pub fn write<P: AsRef<Path>, T: Type, C: Color, I: Image<T, C>>(
    path: P,
    image: &I,
) -> Result<(), Error> {
    unsafe { DEFAULT.write(path, image) }
}
