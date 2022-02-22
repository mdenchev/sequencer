## What

A way to specify a dependency graph. Created for sequencing game events though I guess could be used for other stuff.

## Examples

### Linear sequence: Taking a piece in chess

If you want to animate a piece taking another piece then you need to have the sequence:
1. P1 moves to position of P2
2. delete P2
3. Switch turn to opponent

This library makes it easy to create such sequence:

```rust
struct BoardLoc(u8, u8);


enum Events {
    MovePiece(BoardLoc, BoardLoc),
    RemovePiece(BoardLoc),
    EndTurn
}

// Create a sequencer
fn setup_sequencer() -> Sequencer<Events> {
    Sequencer::default()
}

fn handle_input(..., sequencer: &mut Sequencer<Events>) {
    ...

    // The player has selected a piece and location to move it to.
    // Now we insert the sequence.
    sequencer.insert_seq(vec![MovePiece(from, to), RemovePiece(to), EndTurn]);

    ...
}

fn handle_events(..., sequencer: &mut Sequencer) {
    sequencer.drain_queue(|key, event| {
        match event {

        }
    });
}
```

## TODO
- [] Add optional serde support
- [] Add graphical visualization of nodes
- [] Add methods for garbage collecting finished nodes