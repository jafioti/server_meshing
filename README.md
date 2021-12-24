# server_meshing
Attempt at building a dynamic server meshing system.

Currently uses a fairly simple (but efficient) system where the server does no game state tracking, only distributes updates to clients.

One coordination server handles allocation and distribution of servers, and each server then distributes updates to players in it's area.

Client uses the Bevy game engine to simulate a game, and the Rocket web framework to run both the servers and the coordination server.

Major updates (switching servers, getting players on a server) are done over REST APIs, while position updates are done over UDP.
