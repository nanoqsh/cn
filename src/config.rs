use {
    serde::Deserialize,
    std::{
        fmt, fs,
        net::{IpAddr, SocketAddr},
        path::{Path, PathBuf},
    },
    toml::de,
};

#[derive(Deserialize)]
pub struct Config {
    ip: IpAddr,
    port: u16,
}

impl Config {
    pub fn load<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|_| Error::Read(path.to_owned()))?;
        toml::from_str(&content).map_err(|err| Error::Parse(path.to_owned(), err))
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }
}

pub enum Error {
    Read(PathBuf),
    Parse(PathBuf, de::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Read(path) => write!(f, "failed to read {path} file", path = path.display()),
            Self::Parse(path, err) => write!(
                f,
                "failed to parse config {path}: {err}",
                path = path.display(),
            ),
        }
    }
}
