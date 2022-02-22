use log::error;
use slotmap::{new_key_type, SlotMap};

new_key_type! {
    pub struct SeqKey;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeqStatus {
    NotRun,
    Running,
    Finished,
}

#[derive(Debug, Clone)]
pub struct SeqNode<I> {
    pub key: SeqKey,
    pub parents: Vec<SeqKey>,
    pub children: Vec<SeqKey>,
    pub status: SeqStatus,
    pub item: I,
}

#[derive(Debug)]
pub struct Sequencer<I> {
    /// All root nodes
    pub roots: Vec<SeqKey>,
    /// Storage of all nodes for the sequence graph
    pub nodes: SlotMap<SeqKey, SeqNode<I>>,
    /// List of all nodes ready for processing
    pub queued_nodes: Vec<SeqKey>,
}

impl<T> Default for Sequencer<T> {
    fn default() -> Self {
        let nodes = SlotMap::<SeqKey, SeqNode<T>>::with_key();
        Self {
            roots: vec![],
            nodes,
            queued_nodes: vec![],
        }
    }
}

impl<I> Sequencer<I> {
    fn create_node(&mut self, item: I) -> SeqKey {
        self.nodes.insert_with_key(|key| SeqNode {
            key,
            parents: vec![],
            children: vec![],
            status: SeqStatus::NotRun,
            item,
        })
    }

    /// Inserts a new node with no parents and immediately
    /// queues it for processing.
    /// Returns the nodes key.
    pub fn insert_node(&mut self, item: I) -> SeqKey {
        let key = self.create_node(item);
        self.roots.push(key);
        self.queued_nodes.push(key);
        key
    }

    /// Inserts a vector of items to be executed linearly, one of the other.
    /// The first item is immediately queued for processing.
    /// Returns the key of the last node in the sequence.
    pub fn insert_seq(&mut self, mut items: Vec<I>) -> SeqKey {
        // Make the root element be last so we can pop it
        items.rotate_left(1);

        // Create the root node
        let root_item = items.pop().unwrap();
        let mut prev_key = self.insert_node(root_item);

        // Create the rest of the sequence
        items.drain(..).for_each(|item| {
            let cur_key = self.create_node(item);
            let node = &mut self.nodes[cur_key];
            node.parents.push(prev_key);
            let pnode = &mut self.nodes[prev_key];
            pnode.children.push(cur_key);
            prev_key = cur_key
        });
        prev_key
    }

    /// Queue all children of node with finished parents
    fn queue_ready_children(&mut self, key: SeqKey) {
        let node = &self.nodes[key];
        'child: for ckey in node.children.iter().copied() {
            let cnode = &self.nodes[ckey];
            // Check that all parents are Finished
            for pkey in cnode.parents.iter().copied() {
                let pnode = &self.nodes[pkey];
                if pnode.status != SeqStatus::Finished {
                    continue 'child;
                }
            }
            // If so queue the child node
            self.queued_nodes.push(ckey);
        }
    }

    /// Mark that a node is finished executing.
    /// Queues any children which have all of their parents marked as finished.
    pub fn node_finished(&mut self, key: SeqKey) {
        if let Some(node) = self.nodes.get_mut(key) {
            node.status = SeqStatus::Finished;
            self.queue_ready_children(key);
        } else {
            error!("Tried to mark node as finished with key which doesn't exist");
        }
    }

    /// Drains the queue of nodes to process, applying the provided function, and
    /// marking all of them with the status of "Running".
    pub fn drain_queue<F>(&mut self, mut f: F)
    where
        F: FnMut(SeqKey, &I),
    {
        self.queued_nodes.drain(..).for_each(|key| {
            let node = &mut self.nodes[key];
            node.status = SeqStatus::Running;
            f(node.key, &node.item)
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::SeqStatus;

    use super::Sequencer;

    #[derive(PartialEq, Debug)]
    enum SeqItem {
        Walk,
        Wait,
        Say,
    }

    #[test]
    fn test_insert_root_node() {
        let mut sequencer = Sequencer::default();
        let key = sequencer.insert_node(SeqItem::Walk);
        assert_eq!(1, sequencer.nodes.len());
        assert_eq!(1, sequencer.roots.len());
        assert_eq!(key, sequencer.roots[0]);
        assert_eq!(1, sequencer.queued_nodes.len());

        let node = &sequencer.nodes[key];
        assert_eq!(SeqItem::Walk, node.item);
    }

    #[test]
    fn test_insert_root_seq() {
        let mut sequencer = Sequencer::default();
        sequencer.insert_seq(vec![SeqItem::Walk, SeqItem::Wait, SeqItem::Say]);
        assert_eq!(3, sequencer.nodes.len());
        assert_eq!(1, sequencer.queued_nodes.len());
        let queued_node = &sequencer.nodes[sequencer.queued_nodes[0]];
        assert_eq!(SeqItem::Walk, queued_node.item);
    }

    #[test]
    fn test_drain_queue() {
        let mut sequencer = Sequencer::default();
        let key1 = sequencer.insert_node(SeqItem::Walk);
        let key2 = sequencer.insert_node(SeqItem::Wait);
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
        assert_eq!(SeqStatus::Running, sequencer.nodes[key1].status);
        assert_eq!(SeqStatus::Running, sequencer.nodes[key2].status);
    }

    #[test]
    fn test_node_finished_singular() {
        // Test case: One node exists, it's marked as finished, nothing new is queued
        let mut sequencer = Sequencer::default();
        let key = sequencer.insert_node(SeqItem::Walk);
        sequencer.drain_queue(|_key, _item| {});
        sequencer.node_finished(key);
        assert_eq!(0, sequencer.queued_nodes.len());
        assert_eq!(SeqStatus::Finished, sequencer.nodes[key].status);
    }

    #[test]
    fn test_node_finished_seq() {
        // Test case: A seq of nodes exists. Finishing a node queues up the next.
        let mut sequencer = Sequencer::default();
        let mut key = sequencer.insert_seq(vec![SeqItem::Walk, SeqItem::Wait, SeqItem::Say]);

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
}
