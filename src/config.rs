use {
    serde::Deserialize,
    std::{
        fmt, fs,
        net::{IpAddr, Ipv4Addr, SocketAddr},
        path::{Path, PathBuf},
    },
    toml::de,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub net: Net,
    #[serde(default)]
    pub db: Db,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Error> {
        let content = fs::read_to_string(path).map_err(|_| Error::Read(path.to_owned()))?;
        toml::from_str(&content).map_err(|err| Error::Parse(path.to_owned(), err))
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Net {
    ip: IpAddr,
    port: u16,
}

impl Net {
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }
}

impl Default for Net {
    fn default() -> Self {
        Self {
            ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 3000,
        }
    }
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Db {
    path: Option<PathBuf>,
}

impl Db {
    const DEFAULT_PATH: &str = "db.sqlite";

    pub fn path(&self) -> &Path {
        self.path.as_deref().unwrap_or(Self::DEFAULT_PATH.as_ref())
    }
}
