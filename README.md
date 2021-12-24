# server_meshing
Attempt at building a dynamic server meshing system

Currently only supports one server, will expand to more to create a full mesh.

Currently uses the Bevy game engine to simulate a game, and the Rocket web framework to run the server.
Major updates (switching servers, getting players on a server) are done over REST APIs, while position updates are done over UDP.
