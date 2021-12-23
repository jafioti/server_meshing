use uuid::Uuid;
use rocket::{
    get, post,
    serde::json::Json, State,
};
use game_structs::{
    Player, Vec3,
    operations::PlayerRegister
};
use crate::Session;

#[post("/register_player", format = "json", data = "<player_register>")]
pub fn register_player(session: &State<Session>, player_register: Json<PlayerRegister>) -> String {
    let mut session = session.write().unwrap();
    let mut player = player_register.player.clone();
    player.id = Uuid::new_v4();
    session.players.insert(player.id, player.clone());
    session.addresses.insert(player.id, player_register.address.clone());
    serde_json::to_string(&player.id).unwrap()
}

#[post("/unregister_player", format = "json", data = "<player_id>")]
pub fn unregister_player(session: &State<Session>, player_id: Json<Uuid>) {
    let mut session = session.write().unwrap();
    session.players.remove(&player_id);
    session.addresses.remove(&player_id);
}

#[get("/get_players")]
pub fn get_players(session: &State<Session>) -> String {
    serde_json::to_string(&session.read().unwrap()
        .players.clone()).unwrap()
}