# server_meshing
Attempt at building a dynamic server meshing system.

## Running
You must have Rust installed on your machine to run (https://www.rust-lang.org/tools/install)
- First start each server by opening two terminals (or more, currently setup for 2 servers) in the `server` crate, and running `cargo run -- --send=SEND_PORT --receive=RECEIVE_PORT --main=MAIN_PORT` where each port can be any (unique!) open UDP-accessible port on your machine (check code for the ones it's already setup for)
- Next start the coordination server by navigating a third terminal to the `coord_server` crate and running `cargo run -- --port=COORD_PORT` where COORD_PORT can be any open port on your machine.
- Finally, start up some clients by navigating another terminal to `client` and running `cargo run -- --send=SEND_PORT --receive=RECEIVE_PORT` where each port is a UDP-accessible open (unique!) port on your machine. Again, see the client code to check which ports are already set up to work with.

You should see a game window pop up for each client ran, and a set of cubes. The game area is quite small for now (will expand soon). The blue cube represents the player for that window, while other red cubes represent other players. If over 100 players gather in an area, the area will be split between servers. This will continue to happen until no more free servers are availiable, or each server has less than 100 players on it.

I will be adding a configuration file soon so that these ports don't need to be specified.

Currently uses a fairly simple (but efficient) system where the server does no game state tracking, only distributes updates to clients. One coordination server handles allocation and distribution of servers, and each server then distributes updates to players in it's area.

Client uses the Bevy game engine to simulate a game, and the Rocket web framework to run both the servers and the coordination server. Major updates (switching servers, getting players on a server) are done over REST APIs, while position updates are done over UDP.
