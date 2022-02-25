# sequencer

A way to create a dependency graph of items to be executed.

## Data and Execution model

This crate allows you to create a Directed Acyclic Graph of items
and track which items are currently "active". Once an item is marked
as completed, any child nodes of that item that have all their parents
completed are marked as active.

## Intended uses

This crate was made for sequencing/scripting events for a game
though it could be used for anything that can be modeled as
a dependency graph.

## Cargo

Add the following to Cargo.toml:

```toml
sequencer = "0.1"
```

## Examples

A simple linear sequence such as wait for 5 ticks and then print
something can be expressed as:

```rust
enum Actions {
    Wait(usize)
    Print(String)
}

impl Actions {
    fn tick(&mut self) -> bool {
        match self {
            ...
        }
    }
}

fn main() {
    let mut sequencer = Sequencer::default();
    sequencer.new_seq(vec![Wait(5usize), Print("Done waiting".to_string())]);
    while sequencer.is_active() {
        // Activate next nodes ready for processing
        sequencer.drain_queue(|_key, action| {});
        // Process active nodes. Returning true means the node was finishes.
        sequencer.for_each_active(|_key, item| item.tick());
    }
}
```

Check examples dir for complete examples.

## Features to add:
- More ergonomic methods for creating graphs, especially ones with parallel
execution
- Optional serde support
- Visual debugging of the graphs
- Visual editting of the graphs
- Methods for garbage collecting completed nodes
