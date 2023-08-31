use {
    crate::config::Db,
    rsql::{Connection, Error},
    tokio::sync::{
        mpsc::{self, Receiver, Sender},
        oneshot::{self, Sender as Response},
    },
};

pub fn make(conf: &Db) -> Result<(Access, Service), Error> {
    let conn = Connection::open(conf.path())?;
    let db = Database(conn);
    db.init()?;

    let (send, recv) = mpsc::channel(8);
    Ok((Access(send), Service { db, recv }))
}

#[derive(Clone)]
pub struct Access(Sender<Event>);

impl Access {
    pub async fn store(&self, key: String, link: String) {
        self.send(Event::Store { key, link }).await;
    }

    pub async fn load(&self, key: String) -> Option<String> {
        let (res, req) = oneshot::channel();
        self.send(Event::Load { key, res }).await;
        req.await.expect("request")
    }

    async fn send(&self, ev: Event) {
        self.0
            .send(ev)
            .await
            .expect("the database service should be available");
    }
}

pub struct Service {
    db: Database,
    recv: Receiver<Event>,
}

impl Service {
    pub async fn run(self) -> Result<(), Error> {
        let Self { db, mut recv } = self;
        while let Some(ev) = recv.recv().await {
            match ev {
                Event::Store { key, link } => db.store_link(&key, &link)?,
                Event::Load { key, res } => {
                    let out = db.load_link(&key)?;
                    res.send(out).expect("response");
                }
            }
        }

        Ok(())
    }
}

enum Event {
    Store {
        key: String,
        link: String,
    },
    Load {
        key: String,
        res: Response<Option<String>>,
    },
}

struct Database(Connection);

impl Database {
    fn init(&self) -> Result<(), Error> {
        const INIT_LINK: &str = "
            CREATE TABLE IF NOT EXISTS link ( \
                k TEXT UNIQUE NOT NULL, \
                v TEXT NOT NULL \
            )
        ";

        self.0.execute(INIT_LINK, ()).map(drop)
    }

    fn store_link(&self, key: &str, link: &str) -> Result<(), Error> {
        let rows = self.0.execute(
            "INSERT OR REPLACE INTO link (k, v) VALUES (:k, :v)",
            rsql::named_params! {
                ":k": key,
                ":v": link,
            },
        )?;

        if rows == 1 {
            println!("stored {link} link as {key}");
        }

        Ok(())
    }

    fn load_link(&self, key: &str) -> Result<Option<String>, Error> {
        let mut stat = self.0.prepare("SELECT v FROM link WHERE k = :k")?;
        let mut rows = stat.query(rsql::named_params! { ":k": key })?;
        let link = rows.next()?.map(|row| row.get("v").expect("value"));
        Ok(link)
    }
}
