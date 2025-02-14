# Rusty Battle Tanks (RBT)

## About the game

The purpose of the game is to practice writing Rust code. And at the same time to have fun and feed our competitive beasts 😊. The basic idea is that the developers get to implement their own player "A.I.", which will interact with the game engine.

The game represents a tank battlefield, where each player represents a tank. The end-goal of the game is to locate and destroy all the other tanks.

### Game engine

The game is turn-based. For every turn, each player is given the chance to calculate their own strategy, and to return a desired action to the game engine. The engine is first asking all players for their actions, and then the engine is acting on all the players' actions. Once the game has computed and performed all the actions, the turn is over and a new turn will begin.

Therefore, it should not matter in which order the game engine is querying the actions from the players.

### Tank capabilities

Being old and rusty, the tanks have a limited number of capabilities. In addition, they can only perform one action at a time. For example tanks cannot shoot while moving, or move while scanning, etc. The only exception is the GPS and compass data, which is offered in 'real-time' for free.

- Shell shooting: in order to destroy an enemy tank, the tank may shoot at it
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

Orientation are used to describe the movement and shooting vectors (in case of `Cardinal` aiming).

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

The shape and size of the scanned map surface is always a square with side length of `Players::SCANNING_DISTANCE`.

It is important to note that the scan output will _always_ include the tank who requested it. The differences between the different types of scans affect the position of the requesting tank relative to the returned result. In the case of `Omni`-directional scanning, the scanning tank will be located in the middle of the scanned area. In case of `Mono`-directional scanning, the scanning tank will be located on the edge or corner of the scanned area. For example, the scanning tank will be located on the bottom-left side of the scanned area in case of a `Mono`-directional North-East scan.

It should be obvious that mono-directional scans will give you data that is farther away from the tank, albeit towards one direction only.

### Shooting

Tanks can shoot shells on each other. It is what defines tanks. Shooting depends on the aiming type, and there are two types of `Aiming`:

- `Positional`: this aiming defines the exact coordinate where the shell will hit on the map
- `Cardinal`: this aiming defines the cardinal orientation the shell path will follow

These two aiming types have different pros and cons:

- `Positional`:
  - Pro: shooting is precisely at the specified `Position`
  - Con: the range is limited to the area that is returned by an `Omni`-directional scan
- `Cardinal`:
  - Pro: the range is limited to the area that is returned by a `Mono`-directional scan
  - Con: the line of shooting must be one of the eight cardinal `Orientation`

The shell will impact in one of these conditions:

- when `Positional` aiming is used, at the requested position
- when `Cardinal` aiming is used, if it hits directly any tank along the way, otherwise when it reaches the end of its range

In all cases, regardless on where a shell lands (even if on a player, or any type of terrain -- including `Lake`), the shell will create a 3x3 `square` damage pattern. Anything located in the middle of that 3x3 square pattern will suffer a `direct hit` and anything located on the edges of that 3x3 square pattern will suffer an `indirect hit`. Damage is done exclusively to other tanks, the terrain will not suffer any changes upon a shell impact.

- `Direct hit` is when the shell lands exactly on the enemy or if the enemy is aligned perfectly along the same cardinal `Orientation` as the flying shell, with respect of the shooting player (in case of `Cardinal` aiming).
- `Indirect hit` is when the shell lands on any of the immediately adjacent cells to the enemy or if another player located on a immediately adjacent cell is directly hit.

As a corolary: if a tank shoots at an enemy or a position which is located `right next to it`, it will also suffer an indirect hit, because of the damage pattern explained above. It should not be possible to perform a `direct hit` on itself (anti-suicide rule).

Please note that shooting can be done in any direction, in other words the orientation of the tank and its turret are independent. Also, turret rotation is 'free', i.e. it is not performed by the game engine as an `Action`.

**Examples**:

In the next scenario, we look at `Positional` shooting. The hero is is located in the middle of the map, and the enemy is offset. Assuming that the enemy is close enough for `Positional` shooting, the hero is able to strike precisely, even if the enemy is not perfectly alligned on any cardinal `Orientation`.

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

In the next scenario, the hero shoots while the enemy is moving, and the shell lands right next to the enemy. This is an example of `indirect hit` during a `Positional` shooting.

    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩💥💥💥🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩💥💥💥🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩💥😈💥🟩
    🟩🟩🟩🟩🟩🟩🟩🙂🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩

In the next scenario, we look at `Cardinal` shooting. The hero is located on the left side of the map, an the enemy is straight to the right of our hero. We can observe that the hitting distance is larger.

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

In the following scenario of `Cardinal` shooting, the enemy is slightly offset from the orientation, therefore the shell is missing, and doesn't damage the enemy. You can observe that the shell still lands at the end of its range, but in this case it doesn't damage the enemy tank.

    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩😈💥💥
    🙂🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩💥💥
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩💥💥
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩

The next scenario is very similar to the one above, only that the enemy player is close enough to where the shell lands, and you can see how in this case the enemy suffers an `indirect hit`.

    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩😈💥
    🙂🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩💥💥
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩💥💥
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩

In the last scenario, we revisit a modified version of a previous scenario, where we add another enemy immediately near our main target. In this scenario, the main enemy (the one surrounded by the flames) will suffer a `direct hit`, while the enemy next to it will suffer a `indirect hit`.

    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩💥💥💥🟩
    🙂🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩💥😈💥🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩💥💥👽🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩
    🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩

Please note that the same damage pattern is created regardless if the shell has landed directly on another tank or on an unoccupied terrain, or if `Cardinal` or `Positional` aiming was used.

### Damage

The damage is expressed as a percentage of _full health_. Any damage inflicted by a player is not proportional with the current health level. So for example, if a certain damage is defined to be **25%**, then each such damage will inflict a decrease of health level corresponding to 25% of the full health. In ther words, maximum four consecutive such damages will kill any tank, depending on his current health level.

Tanks take damage in several scenarios:

- When entering `Lake` terrain, the damage is **100%** (instant death)
- When colliding with `Forest` terrain, the damage is **25%**
- When colliding with other tanks, the damage is **10%** _to both tanks_
- The damage is **75%** for direct hits, and **25%** for indirect hits
