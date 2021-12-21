# server_meshing
Attempt at building a dynamic server meshing system

Currently uses the Bevy game engine to simulate a game, and the Rocket web framework to run the server.
Everything is communicated through REST APIs, though hot endpoints such as syncing positions, should be done through TCP in the future.
