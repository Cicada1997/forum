use {
    rocket::{
        request::{self, FromRequest, Request},
        outcome::Outcome,
        http::Status,
        serde::{ Serialize, Deserialize },
    }
};

#[derive(Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub user_id:    i32,
    pub username:   String,
    pub admin:      bool,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, ()> {
        let cookie = match req.cookies().get("token") {
            Some(cookie) => cookie.value().to_string(),
            None => return Outcome::Error((Status::Unauthorized, ())),
        };

        let result = reqwest::Client::new()
            .post("https://auth.kattmys.se/token-login")
            .json(&cookie)
            .send()
            .await;

        let response = match result {
            Ok(res) => res,
            Err(_) => return Outcome::Forward(Status::InternalServerError)
        };
        
        if response.status() == reqwest::StatusCode::OK {
            let user = response.json::<AuthenticatedUser>().await.unwrap();
            return Outcome::Success(user)
        }

        Outcome::Forward(Status::Unauthorized)
    }
}
