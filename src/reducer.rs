use crate::effect::Effect;

pub trait Reducer: Default + Send + Sync + 'static {
    type State;
    type Action;

    fn reduce(&self, state: &mut Self::State, action: Self::Action) -> Effect<Self::Action>;
}
