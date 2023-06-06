# beggar-my-neighbour

attemps in solving:
- Is there a non-terminating game of beggar-my-neighbour?
- If not, what is the longest game of beggar-my-neighbour?

The general rules of the game can be found in [wikipedia](https://en.wikipedia.org/wiki/Beggar-my-neighbour).

## Running Game Simulations

```sh
RUSTFLAGS="--emit=asm -C target-cpu=native" cargo run --release -- longest
```

## Game implementation

Despite beggar my neighbour using a 52 card deck, we can represent it as 5 different distinct card types:
- Aces
- Kings
- Queens
- Jacks
- Other

Even with this simplification, there are still around [$6.54 * 10^{20}$ games](https://math.stackexchange.com/questions/2688331/beggar-my-neighbour-possible-games)

### Game state

An entire game can be described by its initial deck, as the game is deterministic.

However, in order to step through the game, we need to keep track of the following:
- Three decks (player 1, player 2, and the middle deck)
- Current player
- Penalty count

As winning can be determined by this state, we don't need to keep track of the winner.

The decks should be represented as a FIFO queue, as we only ever need to add to the bottom and remove from the top.

## Multi-game approach

In order to solve multiple games, it runs games based on a random deck with rayon.

## Perf testing

```sh
# make sure to uncomment the part in Cargo.toml
cargo build --release
perf record -g target/release/beggar-my-neighbour longest
perf report
```

For a simple visual with flamegraphs:

```sh
cargo flamegraph -- longest
```

## Running Machines

This simulation is running on two machines. If you're running this on a server, do let me know!

- @LeoDog896's dedicated server machine on https://bloom.host
- @CoasterFan5's server, dedicated pterodactyl node

Longest report:

```
-------------------
p1: -K---A--A--------Q--JQ----
p2: K-Q----J-KJ---JQA--A-K----
stringified: -K---A--A--------Q--JQ----/K-Q----J-KJ---JQA--A-K----
winner: P2
turns: 5287
tricks: 750
-------------------
```

There is still room for optimization (most likely algorithm-based, not machine-based)
