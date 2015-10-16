pub trait Event<C>: Send {
  fn is_failed(&self) -> bool;
  fn generate_response(&self) -> bool;
  fn process(&self);
}

pub trait NetworkEvent<C> : Event<C> {
    fn get_host(&self) -> String;
}
