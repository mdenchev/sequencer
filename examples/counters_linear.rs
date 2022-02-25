use sequencer::Sequencer;

#[derive(Debug)]
enum Actions {
    Count(std::ops::Range<i32>),
    Done,
}

impl Actions {
    pub fn tick(&mut self) -> bool {
        match self {
            Actions::Count(range) => match range.next() {
                Some(i) => {
                    println!("{i}");
                    false
                }
                None => {
                    println!("Finished counting");
                    true
                }
            },
            Actions::Done => true,
        }
    }
}

/// In this example we create two nodes that run concurrently counting
/// up to a number and then once everything is finished we call the
/// last node, Done.
/// Output:
/// Started: Count(1..6)
/// 1
/// 2
/// 3
/// 4
/// 5
/// Finished counting    
/// Started: Count(6..11)
/// 6
/// 7
/// 8
/// 9
/// 10
/// Finished counting
/// Started: Done
fn main() {
    let mut sequencer = Sequencer::default();
    sequencer.new_seq(vec![
        Actions::Count(1..6),
        Actions::Count(6..11),
        Actions::Done,
    ]);
    while sequencer.is_active() {
        sequencer.drain_queue(|_key, action| {
            println!("Started: {action:?}");
        });
        sequencer.for_each_active(|_key, item| item.tick());
    }
}
