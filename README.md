# Rusty Battle Tanks (RBT)

As the name suggests, `Rust Battle Tank` is a game about tanks fighting on a battlefield. The game is played by `bots` which control rusty old tanks in a fight for life or death until there is only a single survivor or the time runs out.

## Purpose

The purpose of the game is to help with learning the Rust programming language, while at the same time having a little bit of fun, by unleashing your inner competetitive daemons 🙂.

The end-goal of the game is to explore the map and to find and destroy all the other tanks.

## General description

The game is turn-based. For every turn, each player is given the chance to make tactical decisions, and to provide back to the game engine the desired action. The engine is first asking all players for their desired actions, and then it will perform all the players' actions `simultaneously`.

Once the game has computed and performed all the actions, the turn is over and a new turn will begin. Therefore, the order in which the game engine asks each players for their actions, doesn't affect the end result of the turn.

For example, if two different players shoot at each other without missing, they both record a 'hit' on the other player (scoring the associated points), and it's even possible that they mutually kill each other in the same round if they were both previously wounded.

### Tank capabilities

Being old and rusty, the tanks have limited capabilities. Moreover, they can only perform one single action at a time. For example tanks cannot shoot while moving, or move while scanning, etc. The only exception is the GPS and compass data, which is offered in 'real-time' for free.

- Locomotion: tanks are able to move on the world map
- Scan: tanks have a directional radar which allows them to scan their immediate surroundings
- Shooting: in order to destroy an enemy tank, the tank may shoot at it
- GPS unit: each tank is equipped with a GPS unit to read their current position
- Compass unit: each tank is equipped with a compass to read their current cardinal orientation

## Implementation details

Each player needs to implement the following traits:

- `Player` - mandatory
- `MapReader` - if the players want to use the built-in A* pathfinder

### Player trait

```Rust
/// Public trait that players need to implement, in order for the game engine
/// to be able to interact with the player.
pub trait Player {
    /// Implement this method if and only if you need to perform expensive and
    /// potentially failing initialization.
    ///
    /// The return value should indicate the initialization success.
    fn initialized(&mut self) -> bool {
        true
    }

    /// This is the player's turn to fight.
    ///
    /// The changes performed by the game engine are provided in the `context`.
    fn act(&mut self, context: Context) -> Action;

    /// Returns the player's name
    fn name(&self) -> String;

    /// This indicates whether the player is ready for battle or not.
    fn is_ready(&self) -> bool {
        false
    }
}
```

In addition, each player needs to implement the `new()` method which delegates proper construction of their data structure. It is mandatory in this project (as well as a good coding practice) that constructors don't fail.

The `act()` function is invoked by the game engine once per turn. It is within this function that the players should execute their own bot tactic logic.

Players are free to organize their code as they see fit, as long as all their code resides within their own module directory.

#### Delayed (expensive) player initialization

In case that some players require expensive initialization which may fail, there is the `initialized()` function which should contain the expensive initialization code.

### MapReader trait

If a player wants to use the builtin pathfinding A* implementation, via the `PathFinder` type, one needs to create a new type which implements the `MapReader` trait below:

```Rust
/// MapReader is a trait that provides necessary map data for the PathFinder.
pub trait MapReader: Clone {
    /// Returns the MapCell found at `position`, as known by the player.
    fn read_at(&self, position: &Position) -> MapCell;
}
```

Alternatively, players may implement their own custom pathfinding algorithms.

### World map

As mentioned, the game world is a bi-dimensional map. The world map is represented by an array with the dimensions specified by the `MAX_WORLD_SIZE` constant. In practice, the actual map is smaller than the maximum world size, and the actual size is provided in the `context` parameter of the `act()` function.

The constant `MAX_WORLD_SIZE` is useful if players chose to cache their own world-view in arrays. The alternative is to represent the map (or portions of the map) using dynamic data structures such as Vectors.

### Terrain

The world map has different types of `Terrain`:

- `Field`: this is the normal terrain which allows a tank to safely cross it
- `Forest`: this terrain type is off limits for tanks, they cannot enter it
- `Lake`: this terrain allows tanks to drive into, but doing so results in instant death by drowning
- `Swamp`: this terrain allows tanks to drive into, but doing so renders the tank stuck and imobile. Tanks can still shoot while they are stuck in the swamp, but they cannot move again.

### Positioning

The `Position` structure defines a position that can be used to reference a cell in the world map. The position is relative to the top-left corner of the map.

The position of the origin (the top-left corner) of the map is `Position {x:0, y:0}`.

Taking this into consideration, moving one step to the right on the map increments the `x` coordinate, and moving one step lower on the map increments the `y` coordinate.

Pay special attention when manipulating `Position` data, taking into account the true meaning and direction of the horizontal (`X`) and vertical (`Y`) axis.

### Orientation

Orientations are used to describe the movement and shooting vectors (in case of `Cardinal` aiming).

The game offers eight cardinal points as orientation. They are `North`, `North-East`, `East`, `South-East`, and so on. They are mapped so that `North` corresponds to moving up on the map, `East` means moving right on the map, etc.

### Rotation

`Rotation` is the mechanism used by players to change their own `Orientation`.

It is done in 'steps' or 'increments'. Each step changes the perceived angle of the player by 45°, corresponding with the eight defined cardinal orientations. Rotation can be done `clockwise` or `counter-clockwise`.

### Movement

Tank movement is done one "cell" at a time. There are only two directions of movement:

- `Forward` movement
- `Backwards` movement

The movement of the tank takes into account the above mentioned `Direction` and the tank's own cardinal `Orientation`.

### Scanning surroundings

Tanks have an attached mini-radar unit which allows them to scan their vicinity. The radar has a limited range. There are two different types of supported scans:

- `Mono`-directional: sends the radar energy beam in a single direction
- `Omni`-directional: sends the radar energy beam in a swift pattern all around the tank

The shape and size of the scanned map surface is always a square with side length of `SCANNING_DISTANCE`.

It is important to note that the scan output will _always_ include the tank who requested it. The differences between the different types of scans affect the position of the requesting tank relative to the returned result. In the case of `Omni`-directional scanning, the scanning tank will be located in the center of the scanned area. In case of `Mono`-directional scanning, the scanning tank will be located either on the edge or corner of the scanned area (depending on `Orientation`). For example, the scanning tank will be located on the bottom-left side of the scanned area in case of a `Mono`-directional North-East scan.

It should be obvious that mono-directional scans will give you data that is farther away from the tank, albeit towards one direction only.

### Shooting

By definition, tanks can shoot shells on each other. Shooting depends on the aiming type, and there are two types of `Aiming`:

- `Aiming::Positional` : defines the exact coordinate where the shell will hit on the map
- `Aiming::Cardinal` : defines the cardinal orientation the shell path will follow

These two aiming types have different pros and cons:

- **Positional**:
  - Pro: shooting is precisely at the specified `Position`
  - Con: the range is limited to the area that is returned by an `Omni`-directional scan
- **Cardinal**:
  - Pro: the range is limited to the area that is returned by a `Mono`-directional scan
  - Con: the line of shooting must be one of the eight cardinal `Orientation`

The shell will impact in one of these conditions:

- for **Positional** aiming, at the indicated position
- for **Cardinal** aiming, if it hits directly any tank along the way, otherwise at the end of its range

In all cases, regardless on where a shell lands (even if on a player, or any type of terrain), the shell will create a **3x3 square damage pattern**. Anything located in the middle of that 3x3 square pattern will suffer a **direct hit** and anything located on the edges of that 3x3 square pattern will suffer an **indirect hit**.

Damage is done exclusively to other tanks, the terrain will not suffer any changes upon a shell impact. In other words, one cannot clear the forest by shooting at it. In fact the shots are flying over the forest.

- **Direct hit** is when the shell lands exactly on the enemy or if the enemy is aligned perfectly along the same cardinal `Orientation` as the flying shell, with respect of the shooting player (in case of `Aiming::Cardinal`).
- **Indirect hit** is when the shell lands on any of the immediately adjacent cells to the enemy or if another player located on a immediately adjacent cell is directly hit.

**As a corolary**, if a tank shoots at an enemy or a position which is located right next to it, it will also suffer an indirect hit, because of the damage pattern explained above. It should not be possible to perform a direct hit on itself (anti-suicide rule).

Please note that shooting can be done in any direction, in other words the orientation of the tank and its turret are independent. Also, turret rotation is 'free', i.e. it is not performed by the game engine as an `Action`.

**Examples**:

In the next scenario, we look at Positional shooting. The hero is is located in the middle of the map, and the enemy is offset. Assuming that the enemy is close enough for Positional shooting, the hero is able to strike precisely, even if the enemy is not perfectly alligned on any cardinal `Orientation`.

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

In the next scenario, the hero shoots while the enemy is moving, and the shell lands right next to the enemy. This is an example of **indirect hit** during a Positional shooting.

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

In the next scenario, we look at Cardinal shooting. The hero is located on the left side of the map, an the enemy is straight to the right of our hero. We can observe that the hitting distance is larger.

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

In the following scenario of Cardinal shooting, the enemy is slightly offset from the orientation, therefore the shell is missing, and doesn't damage the enemy. You can observe that the shell still lands at the end of its range, but in this case it doesn't damage the enemy tank.

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

The next scenario is very similar to the one above, only that the enemy player is close enough to where the shell lands, and you can see how in this case the enemy suffers an indirect hit.

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

In the last scenario, we revisit a modified version of a previous scenario, where we add another enemy immediately near our main target. In this scenario, the main enemy (the one surrounded by the flames) will suffer a direct hit, while the enemy next to it will suffer a indirect hit.

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

Please note that the same damage pattern is created regardless if the shell has landed directly on another tank or on an unoccupied terrain, or if Cardinal or Positional aiming was used.

### Damage

The damage is expressed as a percentage of _full health_. Any damage inflicted by a player is not proportional with the current health level. So for example, if a certain damage is defined to be **25%**, then each such damage will inflict a decrease of health level corresponding to 25% of the full health. In ther words, maximum four consecutive such damages will kill any tank, depending on his current health level.

Tanks take damage in several scenarios:

- When entering `Terrain::Lake`, the damage is **100%** (instant death, by drowning)
- The damage is **75%** for direct hits, and **25%** for indirect hits
- When colliding with other tanks, the damage is **25%** _to both tanks_
- When colliding with `Terrain::Forest`, the damage to the tank is **10%**

## Strategy

There are multiple strategies that can be employed.

Generally speaking, exploring and navigating the map safely by avoiding terrain obstacles, locating and killing the enemies, lurking in a corner and ambushing enemies - are all valid strategies. However, one needs to take into account the scoring system when optimizing for one strategy or another.

Use of neural networks or even LLMs to take the next tactical decision is up for the player, however certain penalties will be applied when unreasonably high amount of time is consumed a player.

(more details to follow)

## Scoring

Scoring is based by accumulating points for certain "achievements". The total score of a player is the sum of the collected points.

Achievement points are awarded to players as follows:

- **1 point** - for inflicting an indirect hit on another player
- **2 points** - for inflicting a direct hit on another player
- **3 points** - for giving the final blow to another player. This is _in addition_ to the points awarded for the direct/indirect hit causing the death of the receiving player.
- **5 points** - for surviving the game. If the game ends in a stalemate, the game will end after a certain amount of rounds, and in that case it usually means that there are multiple 'survivors'. In these cases, each survivor receives 5 points.
