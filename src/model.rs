use {
    std::str::FromStr,

    chrono::{
        NaiveDateTime
    },

    serde::{ Serialize, Deserialize },

    rocket::request::FromParam,

    rocket_db_pools::sqlx::{
        FromRow,
        Type,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
pub struct ForumId(i64);
impl From<i64> for ForumId { fn from(v: i64) -> Self { ForumId(v) } }
impl From<ForumId> for i64 { fn from(f: ForumId) -> Self { f.0 } }

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
pub struct UserId(i64);
impl From<i64> for UserId { fn from(v: i64) -> Self { UserId(v) } }
impl From<UserId> for i64 { fn from(f: UserId) -> Self { f.0 } }

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
pub struct PostId(i64);
impl From<i64> for PostId { fn from(v: i64) -> Self { PostId(v) } }
impl From<PostId> for i64 { fn from(f: PostId) -> Self { f.0 } }

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
pub struct CommentId(i64);

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Forum {
    pub id: ForumId,
    pub author_id: UserId,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Post {
    pub id: PostId,
    pub forum_id: ForumId,
    pub author_uid: UserId,
    pub name: String,
    pub sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Comment {
    pub id: CommentId,
    pub post: PostId,
    pub author_id: UserId,
    pub parent: Option<CommentId>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "request_status", rename_all = "lowercase")]
pub enum RequestStatus {
    Accepted,
    Pending,
    Denied,
}

impl FromStr for RequestStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "accepted" => Ok(RequestStatus::Accepted),
            "pending"  => Ok(RequestStatus::Pending),
            "denied"   => Ok(RequestStatus::Denied),
            _ => Err(()),
        }
    }
}

impl<'r> FromParam<'r> for RequestStatus {
    type Error = ();

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        RequestStatus::from_str(param).map_err(|_| ())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ForumRequest {
    pub id: ForumId,
    pub author_id: UserId,

    pub forum_name: String,
    pub description: Option<String>,

    pub motivation: String,
    pub submission: NaiveDateTime,

    pub status: RequestStatus,
}

