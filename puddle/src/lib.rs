use std::marker::PhantomData;

use crossbeam::channel::{Sender, Receiver, IntoIter as RecvIter, bounded};
use threadpool::ThreadPool;



pub struct Puddle<E: Executor> {
  threadpool: ThreadPool,
  sender: Sender<E::Output>,
  receiver: Receiver<E::Output>,
  executor: E
}

impl<E: Executor> Puddle<E> {
  pub fn new(num_threads: usize, executor: E) -> Self {
    let threadpool = ThreadPool::new(num_threads);
    let (sender, receiver) = bounded(threadpool.max_count());
    Puddle {
      threadpool,
      sender,
      receiver,
      executor
    }
  }

  pub fn with_name(name: String, num_threads: usize, executor: E) -> Self {
    let threadpool = ThreadPool::with_name(name, num_threads);
    let (sender, receiver) = bounded(num_threads);
    Puddle {
      threadpool,
      sender,
      receiver,
      executor
    }
  }

  #[inline]
  pub fn queued_count(&self) -> usize {
    self.threadpool.queued_count()
  }

  #[inline]
  pub fn active_count(&self) -> usize {
    self.threadpool.active_count()
  }

  #[inline]
  pub fn max_count(&self) -> usize {
    self.threadpool.max_count()
  }

  pub fn execute(&self, input: E::Input) {
    let sender = self.sender.clone();
    let executor = self.executor.clone();
    self.threadpool.execute(move || {
      let output = executor.execute(input);
      sender.send(output).unwrap();
    });
  }

  #[inline]
  pub fn finish(self) -> RecvIter<E::Output> {
    self.receiver.into_iter()
  }
}

impl<E: Executor> IntoIterator for Puddle<E> {
  type Item = E::Output;
  type IntoIter = RecvIter<E::Output>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.receiver.into_iter()
  }
}



pub trait Executor: Send + Sync + Clone + 'static {
  type Input: Send + 'static;
  type Output: Send + 'static;

  fn execute(self, input: Self::Input) -> Self::Output;
}

impl<I, O> Executor for fn(I) -> O
where I: Send + 'static, O: Send + 'static {
  type Input = I;
  type Output = O;

  #[inline]
  fn execute(self, input: I) -> O {
    (self)(input)
  }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Thunk<F>(PhantomData<F>);

impl<F> Clone for Thunk<F> {
  #[inline]
  fn clone(&self) -> Self {
    Self(PhantomData)
  }
}

impl<F> Copy for Thunk<F> {}

impl<F> Default for Thunk<F> {
  #[inline]
  fn default() -> Self {
    Self(PhantomData)
  }
}

unsafe impl<F> Send for Thunk<F> {}
unsafe impl<F> Sync for Thunk<F> {}

impl<F, O> Executor for Thunk<F>
where F: FnOnce() -> O + Send + 'static, O: Send + 'static {
  type Input = F;
  type Output = O;

  #[inline]
  fn execute(self, input: F) -> O {
    (input)()
  }
}
