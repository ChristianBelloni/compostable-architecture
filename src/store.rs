use std::{marker::PhantomData, sync::Arc};

use futures_signals::signal::{Mutable, MutableSignalCloned, SignalExt, SignalStream};

use crate::{
    reducer::Reducer,
    scheduler::{Scheduler, Spawner},
};

pub struct Store<Spawner, State, Action, Reducer> {
    state: Mutable<State>,
    inner: Reducer,
    _marker: PhantomData<Action>,
    scheduler: Scheduler<Spawner>,
}

impl<Sp, S, A, R> Store<Sp, S, A, R>
where
    Sp: Spawner + Send + Sync + 'static,
    R: Reducer<State = S, Action = A>,
    S: Send + Sync + 'static,
    A: Send + Sync + 'static,
{
    pub fn new(state: S, _: R, scheduler: Scheduler<Sp>) -> Arc<Self> {
        Arc::new(Self {
            state: Mutable::new(state),
            inner: R::default(),
            _marker: PhantomData,
            scheduler,
        })
    }

    pub(crate) fn next(self: Arc<Self>, action: A) {
        let mut state = self.state.lock_mut();
        let effect = self.inner.reduce(&mut state, action);
        drop(state);

        self.scheduler.run_effect(self.clone(), effect);
    }

    pub fn send(self: &Arc<Self>, action: A) {
        self.clone().next(action);
    }
}

impl<Sp, S, A, R> Store<Sp, S, A, R>
where
    Sp: Spawner + Send + Sync + 'static,
    R: Reducer<State = S, Action = A>,
    S: Clone + Send + Sync + 'static,
    A: Send + Sync + 'static,
{
    pub fn subscribe(&self) -> SignalStream<MutableSignalCloned<S>> {
        self.state.signal_cloned().to_stream()
    }
}
