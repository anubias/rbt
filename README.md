# Rusty Battle Tanks (RBT)

## About the game

The purpose of the game is to practice writing Rust code. And at the same time to have fun and feed our competitive beasts 😊. The basic idea is that the developers get to implement their own player "A.I.", which will interact with the game engine.

The game represents a tank battlefield, where each player represents a tank. The end-goal of the game is to locate and destroy all the other tanks.

### Game engine

The game is turn-based. For every turn, each player is given the chance to calculate their own strategy, and to return a desired action to the game engine. The engine is first asking all players for their actions, and then the engine is acting on all the players' actions. Once the game has computed and performed all the actions, the turn is over and a new turn will begin.

Therefore, it should not matter in which order the game engine is querying the actions from the players.

### Tank capabilities

Being old and rusty, the tanks have a limited number of capabilities. In addition, they can only perform one action at a time. For example tanks cannot fire while moving, or move while scanning, etc. The only exception is the GPS and compass data, which is offered in 'real-time' for free.

- Ordnance firing: in order to destroy an enemy tank, the tank may fire at it or collide with it
- Propulsion: tanks are able to move on the world map
- Scan: tanks have a directional radar which allows them to scan their immediate surroundings
- GPS unit: each tank is equipped with a GPS unit to read their current position
- Compass unit: each tank is equipped with a compass to read their current cardinal orientation

## Implementation details

Each player needs to implement the `Player` trait. Within that trait, the `Player::act()` function is the one invoked by the game engine once per turn. It is within this function that the players should execute their own AI logic.

Players are free to organize their code as they see fit, as long as all their code resides within their own module directory.

### Constructors

Each player needs to implement the `Player::new()` method which constructs an instance of their data structure. It is mandatory in this project (as well as a good coding practice) that constructors don't fail.

### Late initialization

In case that some players require expensive initialization which may fail, there is the `Player::initialized()` function which needs to be implemented.

### World map

As mentioned, the world is a bi-dimensional map. The world map is represented as an array, with the dimensions specified by the `Players::MAX_WORLD_SIZE` constant. The actual map is smaller or equal to the world map, and its size is defined in the `world_size` field of the `Players::Context` object, which is provided to players as a parameter of the `Player::act()` function.

If players prefer caching data in arrays, the constant `Players::MAX_WORLD_SIZE` is what should be used for the array size. The alternative is to represent the map (or portions of the map) using dynamic data structures such as Vectors.

### Terrain

The world map has different types of `Terrain`:

- `Field`: this is the normal terrain which allows a tank to safely cross it
- `Forest`: this terrain type is off limits for tanks, they cannot enter it
- `Lake`: this terrain allows tanks to drive into, but doing so results in instant death by drowning
- `Swamp`: this terrain allows tanks to drive into, but doing so renders the tank imobile

### Positioning

The `Players::Position` structure defines a position that can be used to reference a cell in the world map. The position is relative to the top-left corner of the map.

The position of the origin (the top-left corner) of the map is `Position {x:0, y:0}`.

Taking this into consideration, moving one step to the right on the map increments the `x` coordinate, and moving one step lower on the map increments the `y` coordinate.

### Orientation

Orientation dictates the movement and firing vectors.

The game offers eight cardinal points as orientation. They are `North`, `North-East`, `East`, and so on. They are mapped so that `North` corresponds to moving up on the map, and `East` means moving right on the map, etc.

### Rotation

`Rotation` is the machanism by wich players change their own `Orientation`.

It is done in 'steps' or 'increments'. Each increment changes the perceived angle of the player by 45°, corresponding with the eight defined cardinal orientations. Rotation can be done `clockwise` or `counter-clockwise`.

### Movement

Tank movement is done one "cell" at a time. There are only two directions of movement:

- `Forward` movement
- `Backwards` movement

The movement of the tank takes into account the above mentioned `Direction` and the tank's own cardinal `Orientation`.

### Scanning surroundings

Tanks have an attached mini-radar unit which allows them to scan their vicinity. The radar has a limited range. There are two different types of supported scans:

- `Mono`-directional: sends the radar energy beam in a single direction
- `Omni`-directional: sends the radar energy beam in a swift pattern all around the tank

The area of the scanned map surface is always the same. It is represented as an array of size `Players::SCANNING_DISTANCE`. It should be obvious that mono-directional scanns will give you data that is farther away from the tank, albeit towards one direction only.

The scanned data created as the result of scanning the environment *always* contains the tank who has initiated the scan.

### Firing

Tanks can fire ordnance on each other. It is what defines tanks. Firing depends on aiming, and there are two types of Aiming:

- `Positional`: this aiming defines the exact coordinate where the ordnance will hit on the map
- `Cardinal`: this aiming defines the cardinal orientation to shoot the ordnance towards

These two aiming types have different pros and cons:

- `Positional`:
  - Pro: firing is precisely at the specified `Position`
  - Con: the range is limited to the area that is returned by an `Omni`-directional scan
- `Cardinal`:
  - Pro: the range is limited to the area that is returned by a `Mono`-directional scan
  - Con: the line of fire must be one of the eight cardinal `Orientation`

There are two types of hits: `direct` and `indirect`.

`Direct hit` is when the ordnance lands exactly on the enemy (in case of `Positional` aiming) or if the enemy is aligned perfectly along the same cardinal `Orientation` as the ordnance, with respect of the firing player (in case of `Cardinal` aiming).

`Indirect hit` is when the ordnance lands on any of the immediately adjacent cells to the enemy (in case of `Positional` aiming) or if another player located on a immediately adjacent cell is directly hit.

**Examples**:

In this first example, we look at `Positional` firing. The hero is is located in the of the map, and the enemy is offset. Assuming that the enemy is close enough for `Positional` firing, the hero is able to strike precisely, even if the enemy is not perfectly alligned on any cardinal `Orientation`.

    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩💥💥💥🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩💥😈💥🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩💥💥💥🟩
    🟩🟩🟩🟩🟩🟩🟩🙂🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩  
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    
In the second example, we look at `Cardinal` firing. In this scenario, the hero is located on the left side of the map, an the enemy is straight to the right of our hero. We can observe that the hitting distance is larger. However, if the enemy would have been just one cell higher or lower, a `Cardinal` fire would have missed it entirely.

    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩💥💥💥🟩
    🙂🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩💥😈💥🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩💥💥💥🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩

In the last example, we revisit a modified version of the previous example, where we add another enemy immediately near our main target. In this scenario, the main enemy (the one surrounded by the flames) will suffer a `direct hit`, while the enemy next to it will suffer a `indirect hit`.

    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩💥💥💥🟩
    🙂🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩💥😈💥🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩👽💥💥🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩

The direct or indirect hit logic applies similarly to both aiming techniques.

### Damage

The damage is expressed as a percentage of *full health*. Any damage inflicted by a player is not proportional with the current health level. So for example, if a certain damage is defined to be **25%**, then each such damage will inflict a decrease of health level corresponding to 25% of the full health. In ther words, maximum four consecutive such damages will kill any tank, depending on his current health level.

Tanks take damage in several scenarios:

- When entering `Lake` terrain, damage is **100%** (instant death)
- When colliding with `Forest` terrain, damage is **25%**
- When colliding with other tanks, damage is **10%** *to both tanks*
- When suffering an ordnance hit, damage is **75%** for direct hit, and **25%** for indirect hit
