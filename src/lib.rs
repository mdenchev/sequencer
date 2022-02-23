use std::collections::HashSet;

use slotmap::{new_key_type, SlotMap};

new_key_type! {
    pub struct SeqKey;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    Active,
    Inactive,
    Completed,
}

#[derive(Debug, Clone)]
pub struct SeqNode<I> {
    pub key: SeqKey,
    parents: Vec<SeqKey>,
    children: Vec<SeqKey>,
    status: NodeStatus,
    pub item: I,
}

#[derive(Debug)]
pub struct Sequencer<I> {
    /// All root nodes
    roots: Vec<SeqKey>,
    /// Storage of all nodes for the sequence graph
    nodes: SlotMap<SeqKey, SeqNode<I>>,
    /// List of all nodes ready for processing
    queued_nodes: Vec<SeqKey>,
    /// List of all nodes that are currently running
    active_nodes: HashSet<SeqKey>,
}

impl<T> Default for Sequencer<T> {
    fn default() -> Self {
        let nodes = SlotMap::<SeqKey, SeqNode<T>>::with_key();
        Self {
            roots: vec![],
            nodes,
            queued_nodes: vec![],
            active_nodes: HashSet::new(),
        }
    }
}

impl<I> Sequencer<I> {
    fn create_node(&mut self, item: I) -> SeqKey {
        self.nodes.insert_with_key(|key| SeqNode {
            key,
            parents: vec![],
            children: vec![],
            status: NodeStatus::Inactive,
            item,
        })
    }

    fn create_node_with_parents(&mut self, parents: Vec<SeqKey>, item: I) -> SeqKey {
        let key = self.create_node(item);
        for pkey in parents.iter().copied() {
            self.nodes[pkey].children.push(key);
        }
        self.nodes[key].parents = parents;
        key
    }

    /// Inserts a new node with no parents and immediately
    /// queues it for processing.
    /// Returns the nodes key.
    pub fn new_node(&mut self, item: I) -> SeqKey {
        let key = self.create_node(item);
        self.roots.push(key);
        self.queued_nodes.push(key);
        key
    }

    /// Inserts items to be executed linearly, one after the other.
    /// The first item is immediately queued for processing.
    /// Returns the key of the last node in the sequence.
    pub fn new_seq(&mut self, mut items: Vec<I>) -> SeqKey {
        // Make the root element be last so we can pop it
        // TODO there's probably an O(1) way to do this
        items.rotate_left(1);

        // Create the root node
        let root_item = items.pop().unwrap();
        let mut prev_key = self.new_node(root_item);

        // Create the rest of the sequence
        for item in items {
            prev_key = self.create_node_with_parents(vec![prev_key], item);
        }
        prev_key
    }

    /// Inserts a vector of items to be executed linearly, one after the other.
    /// The first item is run once the parent is done.
    /// Returns the key of the last node in the sequence.
    pub fn new_child_seq(&mut self, parent: SeqKey, items: Vec<I>) -> SeqKey {
        let mut prev_key = parent;
        for item in items {
            prev_key = self.create_node_with_parents(vec![prev_key], item);
        }
        prev_key
    }

    /// Inserts a vector of items to be executed linearly, one after the other.
    /// The first item is run once the parent is done.
    /// The parents children are transfered to the last node of the new sequence.
    /// Returns the key of the last node in the sequence.
    pub fn inject_child_seq(&mut self, parent: SeqKey, items: Vec<I>) -> SeqKey {
        // Detach children from parent
        let mut parent_children = std::mem::take(&mut self.nodes[parent].children);

        // Inject the new sequence
        let last_key = self.new_child_seq(parent, items);

        // Insert the parent's ex-children to the last node in the new seq
        self.nodes[last_key].children.append(&mut parent_children);

        last_key
    }

    /// Queue all children of node with completed parents
    fn queue_ready_children(&mut self, key: SeqKey) {
        let node = &self.nodes[key];
        'child: for ckey in node.children.iter().copied() {
            let cnode = &self.nodes[ckey];
            // Check that all parents are completed
            for pkey in cnode.parents.iter().copied() {
                let pnode = &self.nodes[pkey];
                if pnode.status != NodeStatus::Completed {
                    continue 'child;
                }
            }
            // If so queue the child node
            self.queued_nodes.push(ckey);
        }
    }

    /// Mark that a node is finished executing.
    /// Queues any children which have all of their parents marked as completed.
    pub fn node_finished(&mut self, key: SeqKey) {
        self.set_node_status(key, NodeStatus::Completed);
        self.queue_ready_children(key);
    }

    fn set_node_status(&mut self, key: SeqKey, new_status: NodeStatus) {
        let node = &mut self.nodes[key];
        match (node.status, new_status) {
            (NodeStatus::Active, NodeStatus::Completed) => {
                self.active_nodes.remove(&key);
            }
            (NodeStatus::Inactive, NodeStatus::Active) => {
                self.active_nodes.insert(key);
            }
            _ => {}
        }
        node.status = new_status;
    }

    /// Drains the queue of nodes to process, applying the provided function, and
    /// marking all of them with the status of "Active".
    pub fn drain_queue<F>(&mut self, mut f: F)
    where
        F: FnMut(SeqKey, &I),
    {
        let mut queued_nodes = std::mem::take(&mut self.queued_nodes);
        queued_nodes.drain(..).for_each(|key| {
            self.set_node_status(key, NodeStatus::Active);
            let node = &self.nodes[key];
            f(node.key, &node.item)
        });
    }

    /// Iterator for all nodes that are currently active.
    pub fn iter_active(&self) -> impl Iterator<Item = &SeqNode<I>> {
        self.active_nodes.iter().map(|key| &self.nodes[*key])
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{NodeStatus, SeqKey};

    use super::Sequencer;

    #[derive(PartialEq, Debug)]
    enum SeqItem {
        Walk,
        Wait,
        Say,
    }

    #[test]
    fn test_new_node() {
        let mut sequencer = Sequencer::default();
        let key = sequencer.new_node(SeqItem::Walk);
        assert_eq!(1, sequencer.nodes.len());
        assert_eq!(1, sequencer.roots.len());
        assert_eq!(key, sequencer.roots[0]);
        assert_eq!(1, sequencer.queued_nodes.len());

        let node = &sequencer.nodes[key];
        assert_eq!(SeqItem::Walk, node.item);
    }

    #[test]
    fn test_new_seq() {
        let mut sequencer = Sequencer::default();
        sequencer.new_seq(vec![SeqItem::Walk, SeqItem::Wait, SeqItem::Say]);
        assert_eq!(3, sequencer.nodes.len());
        assert_eq!(1, sequencer.queued_nodes.len());
        let queued_node = &sequencer.nodes[sequencer.queued_nodes[0]];
        assert_eq!(SeqItem::Walk, queued_node.item);
    }

    #[test]
    fn test_new_child_seq() {
        let mut sequencer = Sequencer::default();
        let parent = sequencer.new_node(SeqItem::Walk);
        sequencer.new_child_seq(parent, vec![SeqItem::Say, SeqItem::Say]);
        assert_eq!(3, sequencer.nodes.len());
        assert_eq!(1, sequencer.queued_nodes.len());
        let queued_node = &sequencer.nodes[sequencer.queued_nodes[0]];
        assert_eq!(SeqItem::Walk, queued_node.item);
    }

    #[test]
    fn test_inject_seq() {
        let mut sequencer = Sequencer::default();
        let s1 = sequencer.new_seq(vec![SeqItem::Walk, SeqItem::Walk]);
        sequencer.new_child_seq(s1, vec![SeqItem::Say, SeqItem::Say]);
        sequencer.inject_child_seq(s1, vec![SeqItem::Wait, SeqItem::Wait]);
        assert_eq!(6, sequencer.nodes.len());
        assert_eq!(1, sequencer.queued_nodes.len());
        let queued_node = &sequencer.nodes[sequencer.queued_nodes[0]];
        assert_eq!(SeqItem::Walk, queued_node.item);
        let mut key = s1;
        sequencer.drain_queue(|drain_key, _| key = drain_key);
        sequencer.node_finished(key);
        sequencer.drain_queue(|drain_key, _| key = drain_key);
        sequencer.node_finished(key);
        let queued_node = &sequencer.nodes[sequencer.queued_nodes[0]];
        assert_eq!(SeqItem::Wait, queued_node.item);
        sequencer.drain_queue(|drain_key, _| key = drain_key);
        sequencer.node_finished(key);
        sequencer.drain_queue(|drain_key, _| key = drain_key);
        sequencer.node_finished(key);
        let queued_node = &sequencer.nodes[sequencer.queued_nodes[0]];
        assert_eq!(SeqItem::Say, queued_node.item);
    }

    #[test]
    fn test_drain_queue() {
        let mut sequencer = Sequencer::default();
        let key1 = sequencer.new_node(SeqItem::Walk);
        let key2 = sequencer.new_node(SeqItem::Wait);
        assert_eq!(2, sequencer.queued_nodes.len());
        let keys = vec![key1, key2];
        let mut i = 0;
        sequencer.drain_queue(|key, _item| {
            assert!(keys[i] == key);
            i += 1;
        });
        assert_eq!(0, sequencer.queued_nodes.len());
        assert_eq!(2, i);
        // Check the nodes are set to Status: Running
        assert_eq!(NodeStatus::Active, sequencer.nodes[key1].status);
        assert_eq!(NodeStatus::Active, sequencer.nodes[key2].status);
    }

    #[test]
    fn test_node_finished_singular() {
        // Test case: One node exists, it's marked as finished, nothing new is queued
        let mut sequencer = Sequencer::default();
        let key = sequencer.new_node(SeqItem::Walk);
        sequencer.drain_queue(|_key, _item| {});
        sequencer.node_finished(key);
        assert_eq!(0, sequencer.queued_nodes.len());
        assert_eq!(NodeStatus::Completed, sequencer.nodes[key].status);
    }

    #[test]
    fn test_node_finished_seq() {
        // Test case: A seq of nodes exists. Finishing a node queues up the next.
        let mut sequencer = Sequencer::default();
        let mut key = sequencer.new_seq(vec![SeqItem::Walk, SeqItem::Wait, SeqItem::Say]);

        sequencer.drain_queue(|drain_key, item| {
            key = drain_key;
            assert_eq!(SeqItem::Walk, *item);
        });
        sequencer.node_finished(key);
        sequencer.drain_queue(|drain_key, item| {
            key = drain_key;
            assert_eq!(SeqItem::Wait, *item);
        });
        sequencer.node_finished(key);
        sequencer.drain_queue(|drain_key, item| {
            key = drain_key;
            assert_eq!(SeqItem::Say, *item);
        });
        sequencer.node_finished(key);
        sequencer.drain_queue(|_, _| unreachable!());
    }

    #[test]
    fn test_iter_active() {
        let mut sequencer = Sequencer::default();
        let key = sequencer.new_node(SeqItem::Walk);
        let key2 = sequencer.new_node(SeqItem::Wait);
        sequencer.drain_queue(|_key, _item| {});
        let expected_active: HashSet<SeqKey> = vec![key, key2].into_iter().collect();
        let actual_active: HashSet<SeqKey> = sequencer.iter_active().map(|node| node.key).collect();
        assert_eq!(expected_active, actual_active);
        sequencer.node_finished(key);
        sequencer.node_finished(key2);
        assert_eq!(0, sequencer.iter_active().count())
    }
}
