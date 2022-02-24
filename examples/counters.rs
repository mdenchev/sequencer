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

fn main() {
    let mut sequencer = Sequencer::default();
    sequencer.new_seq(vec![Actions::Count(1..6), Actions::Count(6..11), Actions::Done]);
    while sequencer.is_active() {
        sequencer.drain_queue(|key, action| {
            println!("Started: {action:?}");
        });
        sequencer.for_each_active(|key, item| {
            item.tick()
        });
    }
}
