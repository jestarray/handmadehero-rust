# HandmadeHero-Rust
https://handmadehero.org/watch

HanmadeHero is a game made from scratch as much as reasonably possible*. I'm trying to port it to rust to make learning more interactive, engaging, and to have a 1:1 program comparison of rust vs c++.

## For future gamedev reference:

e18: 1:13:00 debunks why you shouldn't add previous frame time to compensate for missed frame

## todo:

* follow rust naming conventions
* clean null types
* avoid unsafe as much as possible, e.g use refernces and avoid null with option<T>
* make vulkan version to learn?
* handmade.exe currently copies the handmade.dll functions entirely, opposed to having lightweight stub functions 

# translation notes:
* handmade.rs = handmade.h(contains interface, structs etc both used by the platform layer and the game) & handmade.cpp
* win32_handmade.rs = win32_handmade.cpp