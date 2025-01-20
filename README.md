# Rusty Battle Tanks (RBT)

## About the game

The purpose of the game is to practice writing Rust code. And at the same time to have fun and feed our competitive beasts 😊. The basic idea is that the developers get to implement their own player "A.I.", which will interact with the game engine.

The game represents a tank battlefield, where each player represents a tank. The end-goal of the game is to locate and destroy all other players.

### Game engine

The game is turn-based. For every turn, each player has the change of calculating its own strategy, and to return a desired action to the game engine. The engine is first asking all players for their actions querying all the players to act, and then the enging is acting on all the players' actions. Once the game has computed and performed all the actions, the turn is over and a new turn will begin.

Therefore, it should not matter in which order the game engine is querying the actions
from the players.

### Tank capabilities

 Being old and rusty, the tanks have a limited number of capabilities. In addition, they can only perform one action at a time. For example tanks cannot fire while moving, or move while scanning, etc. The only exception is the GPS and compass data, which is offered in 'real-time' for free.

- Ordnance firing: in order to destroy an enemy tank, the tank may collide with it or firing at it
- Propulsion: tanks are able to move on the world map
- Scan: tanks have a directional radar which allows them to scan their immediate surroundings
- GPS unit: each tank is equipped with a GPSunit to read their current position
- Compass unit: each tank is equipped with a compass to read their current

## Implementation details

Each player needs to implement the **Player** trait. Within that trait, the **act()** function is the one invoked by the game engine once per turn. It is within this function that the players should execute their AI logic.

Players are free to organize their code as they see fit, as long as all their code resides within their own module.

### Constructors

Each player needs to implement the `new()` method which constructs an instance of their data structure. It is mandatory (as well as a good coding practice) that **constructors never fail**.

### Late initialization

In case that some players require expensive initialization which may fail, there is the `Player::initialized()` function which needs to be implemented.

### World map

As mentioned, the world is a bi-dimensional map. The map size is defined in the `world_size` field of the `Players::Context` object provided to players as a parameter of the `act()` function.

The actual map is represented as an array, with the dimensions specified by the `Players::MAX_WORLD_SIZE` constant. If players prefer caching data in arrays, this is the constant that can be used for the array size. The alternative is to represent the map (or portions of the map) using dynamic data structures such as Vectors.

### Terrain

The world map has different types of terrain:

- Field: this is the normal terrain which allows a tank to safely cross it
- Forest: this terrain type is off limits for tanks, they cannot enter it
- Lake: this terrain allows tanks to drive into, but doing so results in instant death by drowning
- Swamp: this terrain allows tanks to drive into, but doing so renders the tank imobile

### Positioning

The `Players::Position` structure defines a position that can be used to reference a cell in the world map. The position is relative to the top-left corner of the map.

The position of the origin (the top-left corner) of the map is `Position {x:0, y:0}`.

Taking this into consideration, moving one step to the right on the map increments the `x` coordinate, and moving one step lower on the map increments the `y` coordinate.

### Orientation

Orientation dictates the movement and firing vectors.

The game offers eight cardinal points as orientation. They are North, North-East, East, and so on. They are mapped so that North corresponds to moving up on the map, etc.

### Rotation

Rotation is the machanism by wich players change their own orientation.

It is done in 'steps' or 'increments'. Each increment changes the perceived angle of the player by 45°, corresponding with the eight defined orientations. Rotation can be done clockwise or counter-clockwise.

### Movement

Tank movement is done one "cell" at a time. There are only two directions of movement:

- forward movement
- backwards movement

The movement of the tank takes into account the above mentioned direction and the tank's own cardinal orientation.

### Scanning surroundings

Tanks have an attached mini-radar unit which allows them to scan their vicinity. The radar has a limited range. There are two different types of supported scans:

- Mono-directional: sends the radar energy beam in a single direction
- Omni-directional: sends the radar energy beam in a swift pattern all around the tank

The area of the scanned map surface is always the same. It is represented as an array of size `Players::SCANNING_DISTANCE`

It should be obvious that mono-directional scanns will give you data that is farther away from the tank, albeit towards one direction only.

### Firing (tbd)
