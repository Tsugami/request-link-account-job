use std::{collections::HashMap, env};

use dotenv::dotenv;
use reqwest::StatusCode;
use sqlx::{postgres::PgPoolOptions, Row};

#[tokio::main]
async fn main() {
    dotenv().unwrap();

    let database_uri = env::var("DATABASE_URL").unwrap();
    let api_uri = env::var("API_URI").unwrap();
    let access_token = env::var("ACCESS_TOKEN").unwrap();

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

    println!("{}", query);
    let rows = sqlx::query(&query)
        .fetch_all(&pool) // -> Vec<{ country: String, count: i64 }>
        .await
        .expect("query for get discord_id of current char's failed");

    let linked_users = rows.iter().map(|row| row.get("id")).collect::<Vec<i32>>();
    let unlink_users = online_char_ids
        .into_iter()
        .filter(|id| !linked_users.contains(id))
        .collect::<Vec<i32>>();

    let mut body = HashMap::new();
    body.insert("content", "você sabia que pode vincular a sua conta com o Discord? Vá para o servidor do Discord em INSTALAÇÃO E GUIAS -> CONECTAR-FRONTIER -> LOGIN COM DISCORD");
    body.insert("sender_name", "Mirim");

    for id in unlink_users {
        match client
            .post(format!("{}/send-message/{}", api_uri, id))
            .header("access_token", &access_token)
            .json(&body)
            .send()
            .await
        {
            Ok(res) => match res.status() {
                StatusCode::OK => println!("Enviado para {}.", id),
                StatusCode::NOT_FOUND => (),
                _ => println!("Err {:?}", res),
            },
            Err(er) => println!("Err {}", er),
        }
    }
}
