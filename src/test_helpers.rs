use std::sync::Mutex;

static ENV_VAR_TEST_MUTEX: Mutex<()> = Mutex::new(());

pub fn run_env_var_test<T, F>(f: F) -> T
where
    F: Fn() -> T,
{
    let _guard = ENV_VAR_TEST_MUTEX.lock().unwrap();
    f()
}
