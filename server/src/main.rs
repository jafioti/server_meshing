use std::{sync::Mutex, collections::HashMap};
use uuid::Uuid;
use rocket::{
    get, routes, post,
    serde::json::Json, State,
};
use game_structs::{
    Player, Vec3,
    operations::PositionUpdate
};

#[derive(Default, Debug)]
pub struct SessionStruct {
    pub positions: HashMap<Uuid, Vec3>,
    pub players: HashMap<Uuid, Player>
}

#[get("/get_all_positions")]
fn get_all_positions(session: &State<Session>) -> String {
    serde_json::to_string(&session.lock().unwrap().positions).unwrap()
}

#[post("/register_player", format = "json", data = "<player>")]
fn register_player(session: &State<Session>, mut player: Json<Player>) -> String {
    let mut session = session.lock().unwrap();
    player.id = Uuid::new_v4();
    session.players.insert(player.id, player.clone());
    session.positions.insert(player.id, Vec3::new(0., 0., 0.));
    serde_json::to_string(&player.id).unwrap()
}

#[post("/unregister_player", format = "json", data = "<player_id>")]
fn unregister_player(session: &State<Session>, player_id: Json<Uuid>) {
    let mut session = session.lock().unwrap();
    session.players.remove(&player_id);
    session.positions.remove(&player_id);
}

#[post("/update_position", format="json", data="<position_update>")]
fn update_position(session: &State<Session>, position_update: Json<PositionUpdate>) {
    if let Some(p) = session.lock().unwrap().positions.get_mut(&position_update.player_id) {
        *p = position_update.position;
    }
}

pub type Session<'a> = Mutex<SessionStruct>;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    rocket::build()
        .mount("/", routes![get_all_positions, register_player, unregister_player, update_position])
        .manage(Mutex::new(SessionStruct::default()))
        .launch().await
}