use gtk::glib::MainContext;
use std::process::Command;
use std::thread;

pub fn internet_check_loop<F>(closure: F)
where
    F: FnOnce(bool) + 'static + Clone, // Closure takes `rx` as an argument
{
    let (sender, receiver) = async_channel::unbounded();

    thread::spawn(move || {
        let mut last_result = false;
        loop {
            if last_result == true {
                std::thread::sleep(std::time::Duration::from_secs(60));
            }

            let check_internet_connection_cli = Command::new("nmcli")
                .args(&["networking", "connectivity", "check"])
                .output()
                .expect("failed to execute process");

            let connectivity_status = if check_internet_connection_cli.status.success() {
                String::from_utf8_lossy(&check_internet_connection_cli.stdout)
                    .trim()
                    .to_string()
            } else {
                String::from("unknown")
            };

            let is_connected =
                matches!(connectivity_status.as_str(), "full" | "limited" | "portal");

            if is_connected {
                sender
                    .send_blocking(true)
                    .expect("The channel needs to be open.");
                last_result = true
            } else {
                sender
                    .send_blocking(false)
                    .expect("The channel needs to be open.");
                last_result = false
            }
        }
    });

    let main_context = MainContext::default();

    main_context.spawn_local(async move {
        while let Ok(state) = receiver.recv().await {
            let closure = closure.clone();
            closure(state);
        }
    });
}
