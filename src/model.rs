use {
    rocket_db_pools::sqlx::{
        FromRow,
        Type,
    },

    chrono::{
        NaiveDateTime
    },

    serde::{ Serialize, Deserialize },

};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
pub struct ForumId(i64);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
pub struct UserId(i64);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
pub struct PostId(i64);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
pub struct CommentId(i64);

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Forum {
    pub id: ForumId,
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
    pub parent: CommentId,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ForumRequest {
    id: ForumId,
    author_id: UserId,

    forum_name: String,
    description: Option<String>,

    motivation: String,
    submission: NaiveDateTime,
}

