use std::{future::Future, pin::Pin};

use busybody::ServiceContainer;

pub struct Effect<Action> {
    pub(crate) action: EffectAction<Action>,
}

pub enum EffectAction<Action> {
    None,
    Sync(BoxedSyncAction<Action>),
    Async(BoxedAsyncAction<Action>),
}

pub type BoxedFn<T> = Box<dyn Fn(T) + Send + Sync + 'static>;
pub type BoxedAsyncFn<T> = Box<
    dyn Fn(T) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>> + Send + Sync + 'static,
>;

pub type BoxedSyncAction<T> = Box<dyn Fn(BoxedFn<T>) + Send + Sync + 'static>;
pub type BoxedAsyncAction<T> =
    Box<dyn FnOnce(BoxedFn<T>) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>>;

impl<A: 'static> Effect<A> {
    pub fn none() -> Self {
        Self {
            action: EffectAction::None,
        }
    }
    pub fn run_sync(f: impl Fn(BoxedFn<A>) + Send + Sync + 'static) -> Self {
        let action = Box::new(move |send| f(send));
        let action = EffectAction::Sync(action);
        Self { action }
    }

    pub fn run_async<F: Future<Output = ()> + Send + Sync + 'static>(
        f: impl Fn(BoxedFn<A>) -> F + Send + Sync + 'static,
    ) -> Self {
        let action = Box::new(move |send| Box::pin(async move { f(send).await }) as _);
        let action = EffectAction::Async(action);
        Self { action }
    }
}
