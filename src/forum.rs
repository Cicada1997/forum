use {
    crate::{
        Db,

        model::{
            Forum,
            Post,
        },
    },

    std::{
        error::Error,

        time::{
            Duration,
            Instant,
        },

        sync::{
            Arc,
            RwLock,
        },
    },

    rocket_db_pools::{
        Connection,
        sqlx,
    },
};

#[derive(Debug, Clone)]
pub struct InnerTopPosts {
    posts: Option<Vec<Post>>,
    last_updated: Instant,
}

pub struct TopPosts {
    inner: Arc<RwLock<InnerTopPosts>>,
}

impl TopPosts {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(
                InnerTopPosts {
                    posts: None,
                    last_updated: Instant::now(),
                }
            ))
        }
    }

    pub async fn update(&self, db: &mut Connection<Db>) -> Result<(), Box<dyn Error + '_>> {
        let posts: Vec<Post> = sqlx::query_as::<_, Post>("SELECT * FROM posts ORDER BY RANDOM() LIMIT 10")
            .fetch_all(&mut ***db)
            .await?;

        dbg!(&posts);

        {
            let mut inner = self.inner.write()?;
            inner.posts = Some(posts);
            inner.last_updated = Instant::now();
        }

        Ok(())
    }

    pub async fn fetch(&self, db: &mut Connection<Db>) -> Result<Vec<Post>, Box<dyn Error + '_>> {
        let needs_refresh = {
            let inner = self.inner.read().unwrap();
            inner.posts.is_none() || inner.last_updated.elapsed() > Duration::from_secs(60)
        };

        if needs_refresh {
            self.update(db).await?;
        }

        let inner = self.inner.read().unwrap();
        Ok(inner.posts.clone().unwrap())
    }
}


