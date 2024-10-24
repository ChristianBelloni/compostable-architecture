pub mod effect;
pub mod reducer;
pub mod scheduler;
pub mod state;
pub mod store;

pub use busybody::helpers as di;
pub use futures_signals;
pub use uniffi;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::StreamExt;

    use crate::{effect::Effect, reducer::Reducer, scheduler::Scheduler, store::Store};

    #[derive(Default)]
    struct CounterReducer;
    #[derive(Debug, Default, Clone)]
    struct CounterState {
        pub count: u32,
    }
    enum CounterActions {
        Add,
        Sub,
    }
    impl Reducer for CounterReducer {
        type State = CounterState;
        type Action = CounterActions;
        fn reduce(&self, state: &mut Self::State, action: Self::Action) -> Effect<Self::Action> {
            match action {
                CounterActions::Add => {
                    println!("add");
                    state.count += 1;
                    Effect::none()
                }
                CounterActions::Sub => {
                    println!("sub");
                    state.count -= 1;
                    Effect::none()
                }
            }
        }
    }
    #[tokio::test]
    async fn basic_reducer() {
        let handle = tokio::runtime::Handle::current();
        let scheduler = Scheduler::new(handle.clone());
        let store = Store::new(
            CounterState::default(),
            CounterReducer::default(),
            scheduler,
        );

        let mut stream = store.subscribe();

        let j_handle = handle.spawn(async move {
            while let Some(nxt) = stream.next().await {
                println!("{nxt:?}")
            }
        });

        store.send(CounterActions::Add);
        tokio::time::sleep(Duration::from_micros(1)).await;
        store.send(CounterActions::Sub);

        drop(store);

        j_handle.await.unwrap();
    }
}
