- client and game_client
- put input listener code in client, and distribute events to listeners in ui and game_client

- work on map(s)
  - chain of maps, just like on pacman
- ui is last
- improve interpolation by increasing update period, or by adding some latency compensation
- optimize rendering via instancing
- cleanup client
  - break up start_client, consider adopting struct/impl
  - Good error handling, get rid of unwraps

- random pack member counts, synced between client and server
