# sequencer

A way to create a dependency graph of items to be executed.

## Data and Execution model

This crate allows you to create a Directed Acyclic Graph of items
and track which items are currently "active". Once an item is marked
as completed, any child nodes of that item that have all their parents
completed are marked as active.

## Intended uses

This crate was made for sequencing/scripting events and actions for a game
though it could potentially be used for any other case that can be
represented as a dependency graph.

## Examples

TBA

## Features to add:
- More ergonomic methods for creating graphs, especially ones with parallel
execution
- Optional serde support
- Visual debugging of the graphs
- Visual editting of the graphs
- Methods for garbage collecting completed nodes
