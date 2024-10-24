use std::{future::Future, sync::Arc};

use crate::{
    effect::{Effect, EffectAction},
    reducer::Reducer,
    store::Store,
};

pub struct Scheduler<S> {
    spawner: S,
}

impl<Sp: Spawner> Scheduler<Sp> {
    pub fn new(spawner: Sp) -> Self {
        Self { spawner }
    }

    pub fn run_effect<'a, S, A, R>(&self, store: Arc<Store<Sp, S, A, R>>, effect: Effect<A>)
    where
        R: Reducer<State = S, Action = A>,
        Sp: Send + Sync + 'static,
        S: Send + Sync + 'static,
        A: Send + Sync + 'static,
    {
        match effect.action {
            EffectAction::None => {}
            EffectAction::Sync(sync_action) => {
                self.spawner.spawn_blocking(move || {
                    let store = store.clone();
                    sync_action(Box::new(move |action| {
                        store.clone().next(action);
                    }))
                });
            }
            EffectAction::Async(async_action) => {
                self.spawner.spawn(async_action(Box::new(move |action| {
                    store.clone().next(action);
                })));
            }
        }
    }
}

pub trait Spawner {
    fn spawn_blocking(&self, f: impl Fn() + Send + 'static);
    fn spawn(&self, f: impl Future<Output = ()> + Send + Sync + 'static);
}

impl Spawner for tokio::runtime::Runtime {
    fn spawn(&self, f: impl Future<Output = ()> + Send + Sync + 'static) {
        self.spawn(f);
    }
    fn spawn_blocking(&self, f: impl Fn() + Send + 'static) {
        self.spawn_blocking(move || f());
    }
}

impl Spawner for tokio::runtime::Handle {
    fn spawn(&self, f: impl Future<Output = ()> + Send + Sync + 'static) {
        self.spawn(f);
    }
    fn spawn_blocking(&self, f: impl Fn() + Send + 'static) {
        self.spawn_blocking(move || f());
    }
}
