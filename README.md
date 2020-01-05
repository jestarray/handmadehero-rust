# HandmadeHero-Rust
https://handmadehero.org/watch

HanmadeHero is a game made from scratch as much as reasonably possible*. I'm trying to port it to rust to make learning more interactive, engaging, and to have a 1:1 program comparison of rust vs c++.

## experience thus far:
Porting HmH in safe rust has been an uphill battle with the borrow checker because mutable aliasing is allowed in C++, and casey uses a lot of it.
I advise for those who want to learn rust and lessons from HmH to use SDL and start off writing things the rust way.

## For future gamedev reference:

* e18: 1:13:00 debunks why you shouldn't add previous frame time to compensate for missed frame
* ep35: 1:20:14 why not consider teleporting as a means to substitute 3D positioning for moving up floors
* ep38 1:29:00 linear alpha blend, one of the most important math equations in game
* ep42-44: why linear algebra is very useful and makes programming math more simpler
## todo:

* follow rust naming conventions
* clean null types
* avoid unsafe as much as possible, e.g use refernces and avoid null with option<T>
* make vulkan version to learn?
* handmade.exe currently copies the handmade.dll functions entirely, opposed to having lightweight stub functions 
* put skipped assert assert tests
* turn overflow safety checks back on

# translation notes:
* handmade.rs = handmade.h(contains interface, structs etc both used by the platform layer and the game) & handmade.cpp
* win32_handmade.rs = win32_handmade.cpp
* assign calls to cstring!("") to a variable first, then .as_ptr(), otherwise they return empty: https://stackoverflow.com/questions/52174925/cstringnew-unwrap-as-ptr-gives-empty-const-c-char
* try to port to linux after opengl(ep 200+)
* b"string" is better than "string".as_bytes() because it retains information about the length
* When an unsigned int and an int are added together, the int is first converted to unsigned int before the addition takes place (and the result is also an unsigned int).

# bugs:
* replay is not playing back from recorded game state (seems to be fixed now)
* fullscreen does not work, see line 1047 in platform layer

# disclaimer!

I DO NOT want people cloning this repo and not buy the game, it is not my intention to sabatoge Handmade Hero, therefore art assets will not be provided though I will try to find some dummy replacements. This repo is up so I can bug rust people for help when the borrow checker kicks my ass.