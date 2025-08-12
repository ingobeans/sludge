# SLUDGE

<img width="860" height="257" alt="banner" src="https://github.com/user-attachments/assets/d23c948d-a082-41ee-ac8b-a0913a761f29" />

sludge is a tower defense game written in rust with "deck building" mechanics like Noita's wand building. 

you have some towers that you can move around, but they wont do anything if you dont give them any cards. you get cards from a shop that opens between rounds and at the start of the game. the actual behaviour of how cards are cast is basically as close to Noita's wand building as i could get it, so theres a lot of complexity and you'll have to learn through experimentation how make good builds.

theres over 35 cards in the game as of right now, of types projectile, modifier and multidraw

## install

either download from releases if there is one, or use this command to install via cargo:
```sh
cargo install --git https://github.com/ingobeans/sludge.git --features bundled
```