pub mod auth;
pub mod model;
pub mod forum;

#[macro_use]
extern crate rocket_include_static_resources;

use {
    crate::{
        forum::TopPosts,
        model::{
            Forum,
            Post,
            ForumRequest,
            RequestStatus,
        },
        auth::AuthenticatedUser,
    },

    rocket_db_pools::{
        Database,
        Connection,

        sqlx::{self, PgPool, Row}
    },

    rocket_dyn_templates::{Template, context},
    rocket::{
        State,

        // macros
        launch,
        get, post,
        catch,
        catchers,
        routes,
        uri,
        Request,

        form::{
            FromForm,
            Form,
        },

        http::{
            Status,
            Cookie,
            CookieJar,
        },

        response::{
            Redirect,
        },

        fs::{
            FileServer,
            relative,
        }
    },
};

#[derive(Database)]
#[database("forum")]
pub struct Db(PgPool);

#[get("/")]
async fn index(tp: &State<TopPosts>, mut db: Connection<Db>) -> Template {
    let posts = tp.fetch(&mut db).await.unwrap();
    Template::render("index", context! { posts })
}

#[derive(FromForm)]
pub struct Token(pub String);

#[get("/auth")]
async fn auth_site() -> Template {
    Template::render("auth", context! {})
}

#[post("/set-token", data = "<form>")]
async fn set_token(form: Form<Token>, cookies: &CookieJar<'_>) -> Redirect {
    cookies.add(Cookie::new("token", form.into_inner().0));

    Redirect::to(uri!("/forums"))
}

#[get("/forums")]
async fn site_forums(mut db: Connection<Db>) -> Template {
    let forums: Vec<Forum> = sqlx::query_as::<_, Forum>("SELECT * FROM forums LIMIT 10")
        .fetch_all(&mut **db)
        .await
        .unwrap();

    Template::render("forums", context! { forums })
}

#[derive(FromForm)]
pub struct NewPostForm {
    pub title: String,
    pub content: String,
}

#[post("/forum/<id>/create-post", data = "<form>")]
async fn create_post(user: AuthenticatedUser, id: i64, form: Form<NewPostForm>, mut db: Connection<Db>) -> Redirect {
    let sections: Vec<String> = form.content
        .replace("\r\n", "\n")
        .split("\n\n")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    sqlx::query("INSERT INTO posts (forum_id, author_uid, name, sections) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(user.user_id)
        .bind(form.title.clone())
        .bind(sections)
        .execute(&mut **db)
        .await
        .unwrap();

    Redirect::to(format!("/forum/{}", id))
}

#[get("/forum/<id>")]
async fn site_forum(user: Option<AuthenticatedUser>, id: i64, mut db: Connection<Db>) -> Template {
    let forum: Forum = sqlx::query_as("SELECT * FROM forums WHERE id = $1")
        .bind(id)
        .fetch_one(&mut **db)
        .await
        .unwrap();

    let posts: Vec<Post> = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE forum_id = $1 ORDER BY name LIMIT $2")
        .bind(id)
        .bind(10)
        .fetch_all(&mut **db)
        .await
        .unwrap();

    if let Some(user) = user {
        Template::render("forum", context! { forum, posts, user })
    } else {
        Template::render("forum", context! { forum, posts })
    }
}

#[derive(FromForm)]
pub struct ForumRequestForm {
    forum_name: String,
    description: Option<String>,

    motivation: String,
}

#[get("/create-forum")]
async fn forum_request_site(user: Option<AuthenticatedUser>) -> Template {
    Template::render("create_forum", context! { user })
}

#[post("/create-forum", data="<request>")]
async fn create_forum_request(user: AuthenticatedUser, request: Form<ForumRequestForm>, mut db: Connection<Db>) -> Redirect {
    let res = sqlx::query(r#"
        INSERT INTO forum_requests
        (author_id, forum_name, description, motivation)
        VALUES ($1, $2, $3, $4)
    "#)
        .bind(user.user_id)
        .bind(request.forum_name.clone())
        .bind(request.description.clone())
        .bind(request.motivation.clone())
        .execute(&mut **db)
        .await;

    res.unwrap();

    Redirect::to(uri!("/forum"))
}

#[get("/admin/panel")]
async fn admin_panel(user: AuthenticatedUser, mut db: Connection<Db>) -> Result<Template, Status> {
    if !user.admin {
        return Err(Status::Forbidden);
    }

    let requests: Vec<ForumRequest> = sqlx::query_as::<_, ForumRequest>(r#"
        SELECT * FROM forum_requests LIMIT 20;
    "#)
        .fetch_all(&mut **db)
        .await
        .unwrap();

    Ok(Template::render("admin_panel", context! { user, requests }))
}

#[post("/admin/forum_req/<id>/approve/<status>")]
async fn admin_forum_req_trial(id: i64, status: RequestStatus, user: AuthenticatedUser, mut db: Connection<Db>) -> Result<Redirect, Status> {
    if !user.admin {
        return Err(Status::Forbidden);
    }

    match status {
        RequestStatus::Accepted => {
            println!("Inserting forum in database...");
            let approved_request: ForumRequest = sqlx::query_as(
                r#"
                    UPDATE forum_requests
                    SET status = 'accepted'
                    WHERE id = $1
                "#)
                .bind(id)
                .fetch_one(&mut **db)
                .await
                .expect("Unable to update request to accepted, aborting.");

            let res_id: i64 = sqlx::query(r#"
                    INSERT INTO forums (name, description, author_id) VALUES ($1, $2, $3) RETURNING id
                "#)
                .bind(approved_request.forum_name)
                .bind(approved_request.description)
                .bind(approved_request.author_id)
                .fetch_one(&mut **db)
                .await
                .expect("Unable to insert new forum into database, aborting.")
                .get("id");

            return Ok(Redirect::to(format!("/forum/{}", res_id)));

            
        }
        RequestStatus::Denied => {
            println!("Updating database as follows...");
        }
        RequestStatus::Pending => {
            return Err(Status::BadRequest);
        }
    }

    Ok(Redirect::to(uri!("/")))
}

// #[catch(401)]
// async fn forbidden(request: &Request<'_>) -> &'static str {
//     "Nämen nu blev det allt lite lurt!, här är du inte välkommen"
// }

#[catch(401)]
async fn kattauth_redirect(request: &Request<'_>) -> Redirect {
    let host = request.host().unwrap();
    Redirect::to(format!(
        "https://kattmys.se/login?redirect=https://{}{}",
        host, request.uri()
    ))
}

#[catch(404)]
async fn not_found() -> Template {
    Template::render("404", context! { })
}

static_response_handler! {
    "/favicon.ico" => favicon => "favicon-png",
    "/favicon-16.png" => favicon_png => "favicon-png",
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::init())
        .attach(Template::fairing())
        .manage(TopPosts::new())

        .attach(static_resources_initializer!(
            "favicon-png" => "static/images/forum.png",
        ))

        .mount("/", routes![
            index,
            site_forums,
            site_forum,
            auth_site,
            set_token,
            create_post,
            forum_request_site,
            create_forum_request,
            admin_panel,
            admin_forum_req_trial,
        ])

        .mount("/", routes![favicon, favicon_png])


        .register("/", catchers![not_found, kattauth_redirect])

        .mount("/", FileServer::from(relative!("/static")))
}
