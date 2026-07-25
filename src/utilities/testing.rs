use std::sync::atomic::{AtomicBool, Ordering};
use std::path::PathBuf;

pub mod binary_path;
pub mod logging_test_internal;
pub mod logging_test_subscriber;

static IN_TEST_PROCESS: AtomicBool = AtomicBool::new(false);

tokio::task_local! {
    static CURRENT_ASYNC_TEST_NAME: String;
    pub static TEST_STORAGE_DIR: PathBuf;
}

thread_local! {
    pub static CURRENT_TEST_NAME: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
    pub static TEST_STORAGE_DIR_SYNC: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

pub fn try_get_test_storage_dir() -> Option<PathBuf> {
    TEST_STORAGE_DIR
        .try_with(Clone::clone)
        .ok()
        .or_else(|| {
            TEST_STORAGE_DIR_SYNC.with(|c| c.borrow().clone())
        })
}

pub fn set_current_test_name(p: String) {
    IN_TEST_PROCESS.store(true, Ordering::Relaxed);
    
    let unique_id = rand::random::<u64>();
    let temp_dir = std::env::temp_dir().join(format!("collectivetoolbox_ctb_test_{}_{}", p, unique_id));
    let _ = std::fs::create_dir_all(&temp_dir);
    
    TEST_STORAGE_DIR_SYNC.with(|c| *c.borrow_mut() = Some(temp_dir));
    CURRENT_TEST_NAME.with(|c| *c.borrow_mut() = Some(p));
}

pub struct TestNameGuard {
    previous: Option<String>,
}

impl Drop for TestNameGuard {
    fn drop(&mut self) {
        CURRENT_TEST_NAME.with(|c| *c.borrow_mut() = self.previous.take());
    }
}

pub fn push_current_test_name(name: Option<String>) -> TestNameGuard {
    if name.is_some() {
        IN_TEST_PROCESS.store(true, Ordering::Relaxed);
    }
    let previous = CURRENT_TEST_NAME.with(|c| c.replace(name));
    TestNameGuard { previous }
}

pub async fn scope_current_test_name<F>(name: String, future: F) -> F::Output
where
    F: std::future::Future,
{
    IN_TEST_PROCESS.store(true, Ordering::Relaxed);

    let unique_id = rand::random::<u64>();
    let temp_dir = std::env::temp_dir().join(format!("collectivetoolbox_ctb_test_{}_{}", name, unique_id));
    let _ = std::fs::create_dir_all(&temp_dir);

    struct TempDirGuard(std::path::PathBuf);
    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _guard = TempDirGuard(temp_dir.clone());

    let temp_dir_clone = temp_dir.clone();
    CURRENT_ASYNC_TEST_NAME
        .scope(name.clone(), async move {
            TEST_STORAGE_DIR.scope(temp_dir_clone, async move {
                let _guard = push_current_test_name(Some(name));
                future.await
            }).await
        })
        .await
}

pub fn spawn_blocking_with_current_test_name<F, R>(
    f: F,
) -> tokio::task::JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let test_name = try_get_current_test_name();
    let test_storage_dir = try_get_test_storage_dir();
    tokio::task::spawn_blocking(move || {
        let _guard = push_current_test_name(test_name);
        
        let previous_dir = if let Some(ref dir) = test_storage_dir {
            TEST_STORAGE_DIR_SYNC.with(|c| c.replace(Some(dir.clone())))
        } else {
            None
        };
        
        struct BlockingGuard {
            previous_dir: Option<PathBuf>,
        }
        impl Drop for BlockingGuard {
            fn drop(&mut self) {
                let _ = TEST_STORAGE_DIR_SYNC.with(|c| c.replace(self.previous_dir.take()));
            }
        }
        let _blocking_guard = BlockingGuard { previous_dir };
        
        f()
    })
}

pub fn get_current_test_name() -> String {
    if let Some(p) = try_get_current_test_name() {
        p
    } else {
        panic!(
            "Test name not set. This function should only be called within tests."
        );
    }
}

pub fn try_get_current_test_name() -> Option<String> {
    CURRENT_TEST_NAME
        .with(|c| c.borrow().clone())
        .or_else(|| CURRENT_ASYNC_TEST_NAME.try_with(Clone::clone).ok())
}

pub fn is_in_test() -> bool {
    IN_TEST_PROCESS.load(Ordering::Relaxed)
        || try_get_current_test_name().is_some()
}

