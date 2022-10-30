use std::{collections::HashMap, env};

use dotenv::dotenv;
use reqwest::StatusCode;
use sqlx::{postgres::PgPoolOptions, Row};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_uri = env::var("DATABASE_URL").expect("DATABASE_URL not found.");
    let api_uri = env::var("API_URI").expect("API_URI not found.");
    let access_token = env::var("ACCESS_TOKEN").expect("ACCESS_TOKEN not found.");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_uri)
        .await
        .expect("connect database failed");

    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/current-chars", api_uri))
        .header("access_token", &access_token)
        .send()
        .await
        .expect("get current char's failed, check api.");

    let online_char_ids = resp
        .json::<Vec<i32>>()
        .await
        .expect("api returns unexpected current chats data");

    let ids = online_char_ids
        .clone()
        .into_iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let query = format!("
        SELECT c.id FROM characters c JOIN user_connections uc ON uc.user_id = c.id WHERE c.id IN ({})
        ", ids,
    );

    let rows = sqlx::query(&query)
        .fetch_all(&pool)
        .await
        .expect("query for get discord_id of current char's failed");

    let linked_users = rows.iter().map(|row| row.get("id")).collect::<Vec<i32>>();
    let unlink_users = online_char_ids
        .into_iter()
        .filter(|id| !linked_users.contains(id))
        .collect::<Vec<i32>>();

    let send = |id: i32, sender_name: &str, content: &str| {
        let mut body = HashMap::new();
        body.insert("content", content);
        body.insert("sender_name", sender_name);

        client
            .post(format!("{}/send-message/{}", api_uri, id))
            .header("access_token", &access_token)
            .json(&body)
            .send()
    };

    let messages = [
        "Sabia que pode vincular a sua conta com o Discord?",
        "No nosso servidor do Discord, entre no canal",
        "INSTALACAO E GUIAS > CONECTAR-FRONTIER > LOGIN COM DISCORD",
    ];

    for id in unlink_users {
        for m in messages {
            match send(id, "Mirim", m).await {
                Ok(res) => match res.status() {
                    StatusCode::OK => println!("Enviado para {}.", id),
                    StatusCode::NOT_FOUND => (),
                    _ => println!("Err {:?}", res),
                },
                Err(er) => println!("Err {}", er),
            }
        }
    }
}
