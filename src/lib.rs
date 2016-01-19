extern crate iron;
extern crate router;
extern crate image;

use std::fs;
use std::io;
use std::path::{PathBuf, Path};

use iron::status;
use router::{Router};
use iron::{Request, Response, IronResult, IronError, Handler};
use iron::mime::Mime;
use iron::headers::{CacheControl, CacheDirective};
// use hyper::mime::{Mime, TopLevel, SubLevel};

// TODO
// - Add a constructor that takes a config and returns the tuple of before/after middleware

pub struct ImageProcessor {
    /// The path this handler is serving files from.
    pub root: PathBuf,
    // cache: Option<Cache>,
}

impl ImageProcessor {
    pub fn new<P: AsRef<Path>>(root: P) -> ImageProcessor {
        ImageProcessor {
            root: root.as_ref().to_path_buf(),
            // cache: None
        }
    }

    pub fn from_toml<P: AsRef<Path>>(config_path: P) -> ImageProcessor {

    }
}

impl Handler for ImageProcessor {

    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let mut filepath = self.root.clone();

        let ref variant = req.extensions.get::<Router>().unwrap().find("variant").unwrap();
        let ref subdir = req.extensions.get::<Router>().unwrap().find("subdir").unwrap();
        let ref file = req.extensions.get::<Router>().unwrap().find("file").unwrap();

        // TODO: Sanitize components before constructing a path from them

        if *variant != "thumb" {
            return Ok(Response::with((iron::status::NotFound, "Not Found: Unknown variant")));
        }

        filepath.push(subdir);
        filepath.push(file);

        let metadata = match fs::metadata(&filepath) {
            Ok(meta) => meta,
            Err(e) => {
                let status = match e.kind() {
                    io::ErrorKind::NotFound => status::NotFound,
                    io::ErrorKind::PermissionDenied => status::Forbidden,
                    _ => status::InternalServerError,
                };

                return Err(IronError::new(e, status))
            },
        };

        let img = match image::open(&filepath) {
            Ok(img) => img,
            Err(e) => return Err(IronError::new(e, status::InternalServerError))
        };

        let thumb = img.resize(128, 128, image::FilterType::CatmullRom);
        let mut buffer = vec![];

        match thumb.save(&mut buffer, image::PNG) {
            Ok(_) => {
                // TODO: Replace with type safe Mime
                let content_type = "image/png".parse::<Mime>().unwrap();
                let mut response = Response::with((iron::status::Ok, content_type,  buffer));
                let cache = vec![CacheDirective::Public, CacheDirective::MaxAge(60*50*24*365)];
                response.headers.set(CacheControl(cache));

                Ok(response)
            },
            Err(e) => Err(IronError::new(e, status::InternalServerError))
        }
    }

}
