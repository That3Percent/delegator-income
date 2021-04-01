use once_cell::sync::OnceCell;
use std::sync::Arc;

struct TaskInner<T> {
    lazy: OnceCell<T>,
    source: Box<dyn Sync + Send + Fn() -> T>,
}

pub struct Task<T> {
    inner: Arc<TaskInner<T>>,
}
impl<T> Clone for Task<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Task<T> {
    pub fn new<S>(source: S) -> Self
    where
        S: 'static + Sync + Send + TaskSource<Output = T>,
    {
        let inner = TaskInner {
            lazy: OnceCell::new(),
            source: Box::new(move || source.execute()),
        };
        Task {
            inner: Arc::new(inner),
        }
    }

    pub fn get(&self) -> &T {
        self.inner.lazy.get_or_init(|| (self.inner.source)())
    }
}

pub trait TaskSource {
    type Output;
    fn execute(&self) -> Self::Output;
}
