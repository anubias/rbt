# Rusty Battle Tanks (RBT)

## About the game

The purpose of the game is to practice writing Rust code. And at the same time to have fun and feed our competitive beasts 😊. The basic idea is that the developers get to implement their own player "A.I.", which will interact with the game engine.

The game represents a tank battlefield, where each player represents a tank.

The map has certain types of terrain:

- Field: this is the normal terrain which allows a tank to safely cross it
- Forest: this terrain type is off limits for tanks, they cannot enter it
- Lake: this terrain allows tanks to drive into, but doing so results in instant death by drowning
- Swamp: this terrain allows tanks to drive into, but doing so renders the tank imobile

### Game engine

The game is turn-based. For every turn, each player has the change of calculating its own strategy, and to return a desired action to the game engine. The engine is first asking all players for their actions querying all the players to act, and then the enging is acting on all the players' actions. Once the game has computed and performed all the actions, the turn is over and a new turn will begin.

Therefore, it should not matter in which order the game engine is querying the actions
from the players.

### Tank capabilities

 Being old and rusty, the tanks have limited capabilities: they can scan their surroundings, rotate, move and fire an ordnance. They can only perform one action at a time. For example tanks cannot fire while moving, or move while scanning, etc.

These are their capabilities:

- Fire: tanks can fire one round of ordnance, but only forward. Their turrets cannot independently rotate compared to the chasis.
- Move: tanks can move in two different directions: forwards and backwards
- Rotate: tanks can change their orientation in any of the eight cardinal directions
- Scan: tanks can scan their surroundings in two different way: monodirectional and omnidirectional
- GPS and compass: each tanks has a GPS and compass unit that gives their own position and orientation

### Tank movement

Tank movement is done one "cell" at a time. There are only two directions of movement:

- forward movement
- backwards movement

The movement of the tank takes into account the above mentioned direction and the tank's own cardinal orientation.

### Tank rotation

As mentioned above, tanks can rotate to align with the desired orientation. Like movement, rotation is done in steps, each step rotating the tank either clockwise or counter-clockwise by 45°. Tank orientation is important for movement and firing.

### Surroundings scan

Tanks have an attached mini-radar unit which allows them to scan their vicinity. The radar has a limited range. There are two different types of supported scans:

- Mono-directional: sends the radar energy beam in a single direction
- Omni-directional: sends the radar energy beam in a swift pattern all around the tank

The area of the scanned map surface is always the same. However, that means that mono-directional scanning will give you data that is farther away from the tank, albeit towards one direction only.

### Firing (tbd)

## Implementation

Each player needs to implement the **Player** trait. Within that trait, the **act()** function is the one invoked by the game engine once per turn. It is within this function that the players should execute their AI logic.

Players are free to organize their code as they see fit, as long as all their code resides within their own module.
